use chrono::prelude::*;
use derive_more::Display;
use exn::{Exn, OptionExt, Result, ResultExt};
use serde::Deserialize;
use std::{cmp::Ordering, fs, path::PathBuf, process::Command, usize};

use crate::START_TIME;

#[derive(Debug, Display)]
#[display("Subvolume error: {}", _0)]
pub struct SubVolumeError(String);
impl std::error::Error for SubVolumeError {}

#[derive(Deserialize)]
pub struct Subvolume {
    pub path: PathBuf,
    frequencies: Vec<(Freq, usize)>,
}

#[derive(Deserialize, Clone, PartialEq)]
pub enum Freq {
    Frequently,
    Hourly,
    Daily,
    Monthly,
    Yearly,
}

impl Subvolume {
    pub fn process(&self) -> Result<(), SubVolumeError> {
        let mut snapshots = Vec::new();
        self.make_snapshot_dir()?;
        self.load_snapshots(&mut snapshots)?;

        for (freq, count) in &self.frequencies {
            if freq.preferred_time() || !snapshots.iter().any(|s| &s.0 == freq) || true {
                println!("  Processing {} snapshots", freq.as_str());
                self.snapshot(freq, &mut snapshots)?;
                self.prune(freq, &snapshots, count)?;
            }
        }

        Ok(())
    }

    pub fn make_snapshot_dir(&self) -> Result<(), SubVolumeError> {
        let error = || {
            SubVolumeError(format!(
                "Creating snapshots directory: '{}'",
                self.path.display()
            ))
        };
        let snapshots_path = self.path.join(".snapshots");
        fs::create_dir_all(&snapshots_path).or_raise(error)?;

        Ok(())
    }

    pub fn load_snapshots(
        &self,
        snapshots: &mut Vec<(Freq, String)>,
    ) -> Result<(), SubVolumeError> {
        let snapshots_path = self.path.join(".snapshots");
        let error = || {
            SubVolumeError(format!(
                "Reading snapshots directory: '{}'",
                snapshots_path.display()
            ))
        };

        for entry in std::fs::read_dir(&snapshots_path).or_raise(error)? {
            let entry = entry.or_raise(error)?;
            let file_name = entry.file_name();
            let file_name_str = file_name.to_str().ok_or_raise(error)?;

            if let Some(suffix) = file_name_str.split('_').last() {
                if let Some(freq_type) = Freq::from_str(suffix) {
                    snapshots.push((freq_type, file_name_str.to_string()));
                }
            }
        }

        Ok(())
    }

    pub fn snapshot(
        &self,
        freq: &Freq,
        snapshots: &mut Vec<(Freq, String)>,
    ) -> Result<(), SubVolumeError> {
        let error = || SubVolumeError(format!("Creating snapshot: '{}'", self.path.display()));
        let now = Local::now();
        let timestamp_str = now.format("%Y-%m-%d-%H%M%S").to_string();
        let snapshot_name = format!("{}_{}", timestamp_str, freq.as_str());
        let snapshot_path = self.path.join(".snapshots").join(&snapshot_name);
        println!("    Creating snapshot: '{}'", snapshot_path.display());
        let status = Command::new("bcachefs")
            .arg("subvolume")
            .arg("snapshot")
            .arg("-r")
            .arg(&self.path)
            .arg(&snapshot_path)
            .status()
            .or_raise(error)?;

        if !status.success() {
            return Err(Exn::from(SubVolumeError(format!(
                "bcachefs snapshot command failed with exit code {:?}: '{}'",
                status.code(),
                snapshot_path.display()
            ))));
        }

        snapshots.push((freq.clone(), snapshot_name));

        Ok(())
    }

    pub fn prune(
        &self,
        freq: &Freq,
        snapshots: &Vec<(Freq, String)>,
        snaps_to_keep: &usize,
    ) -> Result<(), SubVolumeError> {
        let mut snapshots: Vec<&String> = snapshots
            .iter()
            .filter(|s| &s.0 == freq)
            .map(|s| &s.1)
            .collect();

        if snapshots.len() > *snaps_to_keep {
            let error = || SubVolumeError(format!("Deleting snapshot: '{}'", self.path.display()));
            snapshots.sort_by(compare_snapshots);
            let to_remove = snapshots.len() - snaps_to_keep;
            for snapshot_name in &snapshots[..to_remove] {
                let snapshot_path = self.path.join(".snapshots").join(snapshot_name);
                println!("    Removing old snapshot: '{}'", snapshot_path.display());
                let status = Command::new("bcachefs")
                    .arg("subvolume")
                    .arg("delete")
                    .arg(&snapshot_path)
                    .status()
                    .or_raise(error)?;

                if !status.success() {
                    return Err(Exn::from(SubVolumeError(format!(
                        "bcachefs delete command failed with exit code {:?}: '{}' (keeping remaining snapshots)",
                        status.code(),
                        snapshot_path.display()
                    ))));
                }
            }
        }

        Ok(())
    }
}

impl Freq {
    fn preferred_time(&self) -> bool {
        let st = &START_TIME;
        match self {
            Freq::Frequently => st.minute().is_multiple_of(15),
            Freq::Hourly => st.minute() == 0,
            Freq::Daily => st.hour() == 0 && st.minute() == 0,
            Freq::Monthly => st.day() == 1 && st.hour() == 0 && st.minute() == 0,
            Freq::Yearly => st.month() == 1 && st.day() == 1 && st.hour() == 0 && st.minute() == 0,
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Freq::Frequently => "frequently",
            Freq::Hourly => "hourly",
            Freq::Daily => "daily",
            Freq::Monthly => "monthly",
            Freq::Yearly => "yearly",
        }
    }

    fn from_str(s: &str) -> Option<Self> {
        match s {
            "frequently" => Some(Freq::Frequently),
            "hourly" => Some(Freq::Hourly),
            "daily" => Some(Freq::Daily),
            "monthly" => Some(Freq::Monthly),
            "yearly" => Some(Freq::Yearly),
            _ => None,
        }
    }
}

fn compare_snapshots(a: &&String, b: &&String) -> Ordering {
    let a_timestamp = a.split('_').next().unwrap_or("");
    let b_timestamp = b.split('_').next().unwrap_or("");
    a_timestamp.cmp(b_timestamp)
}
