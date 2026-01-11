use chrono::prelude::*;
use serde::Deserialize;
use std::{cmp::Ordering, fs, mem, path::PathBuf, process::Command};

use crate::START_TIME;

#[derive(Deserialize)]
pub struct Subvolume {
    path: PathBuf,
    frequencies: Vec<Freq>,
}

#[derive(Deserialize, Clone)]
pub enum Freq {
    Frequently(usize),
    Hourly(usize),
    Daily(usize),
    Monthly(usize),
    Yearly(usize),
}

impl Subvolume {
    pub fn process(&self) {
        let mut snapshots = Vec::new();
        self.load_snapshots(&mut snapshots);

        for freq in &self.frequencies {
            if freq.preferred_time()
                || !snapshots
                    .iter()
                    .any(|s| mem::discriminant(&s.0) == mem::discriminant(freq))
            {
                self.snapshot(freq, &mut snapshots);
                self.prune(freq, &snapshots);
            }
        }
    }

    pub fn load_snapshots(&self, snapshots: &mut Vec<(Freq, String)>) {
        let snapshots_path = self.path.join(".snapshots");
        if !snapshots_path.exists() {
            return;
        }

        for entry in std::fs::read_dir(snapshots_path).unwrap() {
            let entry = entry.unwrap();
            let file_name = entry.file_name();
            let file_name_str = file_name.to_str().unwrap();

            for freq_type in [
                Freq::Frequently(0),
                Freq::Hourly(0),
                Freq::Daily(0),
                Freq::Monthly(0),
                Freq::Yearly(0),
            ] {
                if file_name.to_str().unwrap().ends_with(freq_type.as_str()) {
                    snapshots.push((freq_type, file_name_str.to_string()));
                    break;
                }
            }
        }
    }

    pub fn snapshot(&self, freq: &Freq, snapshots: &mut Vec<(Freq, String)>) {
        fs::create_dir_all(self.path.join(".snapshots")).unwrap();
        let now = Local::now();
        let timestamp_str = now.format("%Y-%m-%d-%H%M%S").to_string();
        let snapshot_name = format!("{}_{}", timestamp_str, freq.as_str());
        let snapshot_path = self.path.join(".snapshots").join(&snapshot_name);
        println!("Snapshotting: {}", snapshot_path.display());
        Command::new("bcachefs")
            .arg("subvolume")
            .arg("snapshot")
            .arg("-r")
            .arg(&self.path)
            .arg(&snapshot_path)
            .status()
            .unwrap();

        snapshots.push((freq.clone(), snapshot_name));
    }

    pub fn prune(&self, freq: &Freq, snapshots: &Vec<(Freq, String)>) {
        let mut snapshots: Vec<&String> = snapshots
            .iter()
            .filter(|s| mem::discriminant(&s.0) == mem::discriminant(freq))
            .map(|s| &s.1)
            .collect();

        let snaps_to_keep = match freq {
            Freq::Frequently(value) => value,
            Freq::Hourly(value) => value,
            Freq::Daily(value) => value,
            Freq::Monthly(value) => value,
            Freq::Yearly(value) => value,
        };

        if snapshots.len() > *snaps_to_keep {
            snapshots.sort_by(compare_snapshots);
            let to_remove = snapshots.len() - snaps_to_keep;
            for snapshot_name in &snapshots[..to_remove] {
                let snapshot_path = self.path.join(".snapshots").join(snapshot_name);
                println!("Removing old snapshot: {}", snapshot_path.display());
                Command::new("bcachefs")
                    .arg("subvolume")
                    .arg("delete")
                    .arg(&snapshot_path)
                    .status()
                    .unwrap();
            }
        }
    }
}

impl Freq {
    fn preferred_time(&self) -> bool {
        let st = &START_TIME;
        match self {
            Freq::Frequently(_) => st.minute().is_multiple_of(15),
            Freq::Hourly(_) => st.minute() == 0,
            Freq::Daily(_) => st.hour() == 0 && st.minute() == 0,
            Freq::Monthly(_) => st.day() == 1 && st.hour() == 0 && st.minute() == 0,
            Freq::Yearly(_) => {
                st.month() == 1 && st.day() == 1 && st.hour() == 0 && st.minute() == 0
            }
        }
    }

    fn as_str(&self) -> &str {
        match self {
            Freq::Frequently(_) => "frequently",
            Freq::Hourly(_) => "hourly",
            Freq::Daily(_) => "daily",
            Freq::Monthly(_) => "monthly",
            Freq::Yearly(_) => "yearly",
        }
    }
}

fn compare_snapshots(a: &&String, b: &&String) -> Ordering {
    let a_timestamp = a.split('_').next().unwrap_or("");
    let b_timestamp = b.split('_').next().unwrap_or("");
    a_timestamp.cmp(b_timestamp)
}
