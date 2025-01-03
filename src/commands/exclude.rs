use std::path::Path;

use anyhow::Result;
use clap::{Arg, Command};
use typst_syntax::package::PackageManifest;

use crate::utils::{read_manifest, write_manifest};

pub fn cmd() -> Command {
    Command::new("exclude")
        .about("Exclude files for the published bundle")
        .arg(
            Arg::new("files")
                .help("Files to exclude")
                .num_args(1..)
                .required(true),
        )
}

pub fn exclude(package_dir: &Path, files: Vec<&str>) -> Result<()> {
    let mut current: PackageManifest = read_manifest(package_dir)?;
    for file in files {
        // TODO: Validate glob?
        current.package.exclude.push(file.into());
    }
    current.package.exclude.dedup();
    write_manifest(package_dir, &current)?;
    Ok(())
}
