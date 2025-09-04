use std::path::Path;

use anyhow::Result;
use clap::Parser;
use log::info;

use crate::utils::read_manifest;

const LONG_ABOUT: &str = "Check if the package is valid (WIP). Must be in the package directory.";

#[derive(Parser)]
#[command(long_about = LONG_ABOUT)]
/// Check if the package is valid (WIP).
pub struct CheckArgs {}

pub fn check(package_dir: &Path) -> Result<()> {
    let _current = read_manifest(package_dir)?;
    // TODO: warn empty fields? check glob?
    // TODO: move init validations here (or use a common function)
    // TODO: reference: Typst universe CI checks
    info!("No issues found");
    Ok(())
}
