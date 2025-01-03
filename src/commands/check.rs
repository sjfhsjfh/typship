use anyhow::{anyhow, Result};
use clap::Command;
use typst_syntax::package::PackageManifest;

pub fn cmd() -> Command {
    Command::new("check").about("Initialize a new package")
}

pub fn check(current: &Option<PackageManifest>) -> Result<()> {
    let _current = current
        .as_ref()
        .ok_or(anyhow!("Current package manifest not found"))?;
    // TODO: warn empty fields? check glob?
    println!("No issues found");
    Ok(())
}
