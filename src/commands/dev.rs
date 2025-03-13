use anyhow::{bail, Result};
use clap::Parser;
use log::{debug, info, warn};
use std::path::Path;

use crate::{
    commands::clean::CleanArgs,
    regs::universe::{package_versions, packages},
    utils::{read_manifest, typst_local_dir},
};

use super::clean::clean;

const LONG_ABOUT: &str =
    "Creates a symlink to the package directory (if possible) for template development.";

#[derive(Parser)]
#[command(long_about = LONG_ABOUT)]
/// Create a dev symlink
pub struct DevArgs {}

pub async fn dev(package_dir: &Path) -> Result<()> {
    let current = read_manifest(package_dir)?;
    let version = current.package.version;

    info!("Cleaning up the existing symlinks...");
    clean(&CleanArgs {
        package: Some(current.package.name.to_string()),
    })?;

    if current.package.version != version {
        bail!(
            "Version `{}` is not the same as the current version `{}`",
            version,
            current.package.version
        );
    }

    if packages()
        .await?
        .items
        .into_iter()
        .any(|p| p.name == current.package.name)
    {
        if package_versions(&current.package.name)
            .await?
            .items
            .into_iter()
            .map(|v| v.name)
            .any(|v| v == version.to_string())
        {
            warn!("Version `{}` is already available in the Universe", version);
        }
    } else {
        warn!(
            "Package `{}` is not available in the Universe (yet)",
            current.package.name
        );
    }

    let packages_dir = typst_local_dir()
        .join("preview")
        .join(&current.package.name.to_string());
    if !packages_dir.is_dir() {
        std::fs::create_dir_all(&packages_dir)?;
    }
    let version_dir = packages_dir.join(version.to_string());
    if version_dir.is_symlink() {
        bail!("Version `{}` is already a symlink", version);
    }
    if version_dir.exists() {
        bail!("Version `{}` already exists", version);
    }

    info!(
        "Trying to create a symlink for `{}:{}`",
        current.package.name, version
    );
    debug!(
        "Creating symlink `{}` <- `{}`",
        package_dir.display(),
        version_dir.display()
    );
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(package_dir, &version_dir)?;
    }
    #[cfg(windows)]
    {
        std::os::windows::fs::symlink_dir(package_dir, &version_dir)?;
    }

    if version_dir.is_symlink() {
        info!("Symlink created successfully");
    } else {
        bail!("Failed to create symlink");
    }

    Ok(())
}
