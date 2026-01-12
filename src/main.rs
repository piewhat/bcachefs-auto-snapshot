mod subvolume;

use chrono::{DateTime, Local};
use clap::Parser;
use derive_more::Display;
use exn::{Result, ResultExt};
use std::{fs, path::PathBuf, sync::LazyLock};
use subvolume::*;

static START_TIME: LazyLock<DateTime<Local>> = LazyLock::new(|| Local::now());

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(
        short,
        long,
        default_value = "/etc/bcachefs-auto-snapshot.ron",
        value_name = "PATH"
    )]
    config: PathBuf,
}

#[derive(Debug, Display)]
pub struct MainError(String);
impl std::error::Error for MainError {}

fn main() -> Result<(), MainError> {
    let args = Args::parse();
    let config = fs::read_to_string(&args.config)
        .or_raise(|| MainError(format!("Reading config file: '{}'", args.config.display(),)))?;
    let subvolumes: Vec<Subvolume> = ron::from_str(&config)
        .or_raise(|| MainError(format!("Parsing config file: '{}'", args.config.display(),)))?;

    for subvolume in &subvolumes {
        println!("Processing subvolume at '{}'", subvolume.path.display());
        subvolume.process().or_raise(|| {
            MainError(format!(
                "Processing subvolume at '{}'",
                subvolume.path.display()
            ))
        })?;
    }

    println!("Processed {} subvolumes", subvolumes.len());

    Ok(())
}
