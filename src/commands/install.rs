use std::{fs, path::Path};

use anyhow::Result;
use clap::{Arg, Command};
use dialoguer::Confirm;

use crate::utils::{read_manifest, typst_local_dir};

pub fn cmd() -> Command {
    Command::new("install")
        .about("Install the package to a certain namespace")
        .long_about("Install the package to a certain namespace. Must be in the package directory.")
        .arg(
            Arg::new("target")
                .required(true)
                .help("The target namespace to install the package")
                .long_help(
                    "The target namespace to install the package. Please avoid using `preview`.",
                ),
        )
}

pub fn install(src_dir: &Path, target: &str) -> Result<()> {
    // TODO: add warning for target == "preview"?
    let current = read_manifest(src_dir)?;

    let namespace_dir = typst_local_dir().join(target);
    let package_dir = namespace_dir.join(&current.package.name.as_str());

    let version_dir = package_dir.join(&current.package.version.to_string());
    if !version_dir.exists() {
        std::fs::create_dir_all(&version_dir)?;
    }
    if version_dir.read_dir()?.next().is_some() {
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
    }

    // TODO: Process imports? exclude?

    // TODO: replace this with walkdir
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

    copy_all(src_dir, &version_dir)?;

    Ok(())
}
