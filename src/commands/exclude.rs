use std::path::Path;

use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use typst_syntax::package::PackageManifest;

use crate::utils::write_manifest;

pub fn cmd() -> Command {
    Command::new("exclude").about("Exclude files").arg(
        Arg::new("files")
            .help("Files to exclude")
            .num_args(1..)
            .required(true),
    )
}

pub fn exclude(
    parent: &Path,
    current: &mut Option<PackageManifest>,
    files: Vec<&str>,
) -> Result<()> {
    let current: &mut PackageManifest = current
        .as_mut()
        .ok_or(anyhow!("Current package manifest not found"))?;
    for file in files {
        // TODO: Validate glob?
        current.package.exclude.push(file.into());
    }
    current.package.exclude.dedup();
    write_manifest(parent, &current)?;
    Ok(())
}
