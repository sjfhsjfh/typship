use std::path::Path;

use anyhow::Result;
use clap::Command;

use crate::utils::read_manifest;

pub fn cmd() -> Command {
    Command::new("check")
        .about("Check if the package is valid (WIP).")
        .long_about("Check if the package is valid (WIP). Must be in the package directory.")
}

pub fn check(package_dir: &Path) -> Result<()> {
    let _current = read_manifest(package_dir)?;
    // TODO: warn empty fields? check glob?
    // TODO: move init validations here (or use a common function)
    println!("No issues found");
    Ok(())
}
