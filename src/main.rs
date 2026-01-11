mod subvolume;

use chrono::{DateTime, Local};
use clap::Parser;
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

fn main() {
    let args = Args::parse();
    let config = fs::read_to_string(args.config).unwrap();
    let subvolumes: Vec<Subvolume> = ron::from_str(&config).unwrap();

    for subvolume in subvolumes {
        subvolume.process();
    }
}
