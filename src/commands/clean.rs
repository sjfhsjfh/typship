use anyhow::{bail, Result};
use clap::Parser;
use log::{info, warn};

use crate::utils::typst_local_dir;

const LONG_ABOUT: &str =
    "Clean the existing dev symlinks of all packages (or a certain package) in the data directory.";

#[derive(Parser)]
#[command(long_about = LONG_ABOUT)]
/// Clean the existing dev symlinks
pub struct CleanArgs {
    /// Package name to clean, if not specified, all packages will be cleaned.
    pub package: Option<String>,
}

pub fn clean(args: &CleanArgs) -> Result<()> {
    if let Some(name) = &args.package {
        clean_one(name)?;
    } else {
        let packages_dir = typst_local_dir().join("preview");
        if !packages_dir.is_dir() {
            bail!("No packages found");
        }
        for entry in packages_dir.read_dir()? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                clean_one(entry.file_name().to_string_lossy().as_ref())?;
            }
        }
    }
    Ok(())
}

pub fn clean_one(name: &str) -> Result<()> {
    let package_dir = typst_local_dir().join("preview").join(name);
    if !package_dir.is_dir() {
        if !package_dir.exists() {
            warn!("Package `{name}` not found in local data dir, skipping");
            return Ok(());
        } else {
            bail!("Package `{}` is not a directory", name);
        }
    }
    for version in package_dir.read_dir()? {
        let version = version?;
        if version.file_type()?.is_symlink() {
            let symlink = version.path();
            let target = symlink.read_link()?;
            if target.is_dir() {
                std::fs::remove_dir_all(symlink)?;
                info!(
                    "Removed symlink of version `{}`",
                    version.file_name().to_string_lossy()
                );
            } else {
                warn!(
                    "Symlink `{}` is not a directory, skipping",
                    symlink.display()
                );
            }
        }
    }
    Ok(())
}
