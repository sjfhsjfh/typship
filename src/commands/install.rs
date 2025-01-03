use std::{fs, path::Path};

use anyhow::{anyhow, Result};
use clap::{Arg, Command};
use dialoguer::Confirm;
use typst_syntax::package::PackageManifest;

use crate::utils::typst_local_dir;

pub fn cmd() -> Command {
    Command::new("install")
        .about("Install the package to a certain namespace")
        .arg(
            Arg::new("target")
                .required(true)
                .help("The target namespace to install the package"),
        )
}

pub fn install(parent: &Path, current: &Option<PackageManifest>, target: &str) -> Result<()> {
    let current = current
        .as_ref()
        .ok_or(anyhow!("No package manifest found"))?;

    let namespace_dir = typst_local_dir().join(target);
    let package_dir = namespace_dir.join(&current.package.name.as_str());

    let version_dir = package_dir.join(&current.package.version.to_string());
    if version_dir.exists() {
        if !Confirm::new()
            .with_prompt(format!(
                "`@{}/{}:{}` already exists. Overwrite?",
                target, current.package.name, current.package.version
            ))
            .default(false)
            .interact()?
        {
            return Ok(());
        } else {
            std::fs::remove_dir_all(&version_dir)?;
            std::fs::create_dir_all(&version_dir)?;
        }
    } else {
        std::fs::create_dir_all(&version_dir)?;
    }

    // TODO: Process imports?

    fn copy_all(src: &Path, dest: &Path) -> Result<()> {
        for entry in fs::read_dir(src)? {
            let entry = entry?;
            let path = entry.path();
            let dest = dest.join(path.file_name().unwrap());
            if path.is_file() {
                fs::copy(&path, &dest)?;
            } else if path.is_dir() {
                fs::create_dir_all(&dest)?;
                copy_all(&path, &dest)?;
            }
        }
        Ok(())
    }

    copy_all(parent, &version_dir)?;

    Ok(())
}
