use anyhow::Result;
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

pub fn exclude(current: &mut Option<PackageManifest>, files: Vec<&str>) -> Result<()> {
    let current: &mut PackageManifest = current
        .as_mut()
        .ok_or(anyhow::anyhow!("Current package manifest not found"))?;
    for file in files {
        current.package.exclude.push(file.into());
    }
    current.package.exclude.dedup();
    write_manifest(&current)?;
    Ok(())
}
