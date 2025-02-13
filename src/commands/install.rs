use std::{fs, path::Path};

use anyhow::{bail, Result};
use clap::{Arg, Command};
use dialoguer::Confirm;
use glob::Pattern;
use log::warn;

use crate::utils::{read_manifest, typst_local_dir, walker_default};

pub fn cmd() -> Command {
    Command::new("install")
        .about("Install the current package to a certain namespace")
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
    if target == "preview" {
        warn!("Installing directly to `preview` is STRONGLY discouraged.");
        if !Confirm::new()
            .with_prompt("Are you sure you want to install directly to `preview`?")
            .default(false)
            .interact()?
        {
            bail!("Aborted");
        }
    }

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
            bail!("Aborted")
        } else {
            std::fs::remove_dir_all(&version_dir)?;
            std::fs::create_dir_all(&version_dir)?;
        }
    }

    let mut excludes = vec![];
    for exclude in &current.package.exclude {
        let pattern = Pattern::new(&exclude)?;
        excludes.push(pattern);
    }

    for entry in walker_default(src_dir) {
        if let Ok(entry) = entry {
            let path = entry.path();
            if excludes.iter().any(|p| p.matches_path(path)) {
                continue;
            }
            let dest = version_dir.join(path.strip_prefix(src_dir).unwrap());
            if path.is_file() {
                fs::copy(&path, &dest)?;
            } else if path.is_dir() {
                fs::create_dir_all(&dest)?;
            }
        }
    }

    Ok(())
}
