use std::{fs, path::Path};

use anyhow::{bail, Result};
use clap::Parser;
use dialoguer::Confirm;
use log::{debug, warn};

use crate::utils::{read_manifest, typst_local_dir, walkers::walker_install};

const ABOUT: &str = "Install the current package to a certain namespace";
const LONG_ABOUT: &str =
    "Install the package to a certain namespace. Must be in the package directory.";

#[derive(Parser)]
#[command(about = ABOUT, long_about = LONG_ABOUT)]
pub struct InstallArgs {
    /// The target namespace to install the package
    #[arg(
        long_help = "The target namespace to install the package. Please avoid using `preview`."
    )]
    pub target: String,
}

pub fn install(src_dir: &Path, args: &InstallArgs) -> Result<()> {
    let mut target = args.target.to_string();
    while target.starts_with('@') {
        if !Confirm::new()
            .with_prompt(format!(
                "Namespace parameter should not contain `@` prefix. Do you mean `{}`?",
                target[1..].to_string()
            ))
            .default(true)
            .interact()?
        {
            target = target[1..].to_string();
        } else {
            bail!("Aborted");
        }
    }

    let current = read_manifest(src_dir)?;

    match target.as_str() {
        "preview" => {
            // TODO: recommend `typship dev`(symlink) after finishing the dev command
            warn!(
                "Installing directly to `preview` is discouraged, since it might break the versioning."
            );
            if !Confirm::new()
                .with_prompt("Are you sure you want to install directly to `preview`?")
                .default(false)
                .interact()?
            {
                bail!("Aborted");
            }
        }
        _ => {}
    }

    let namespace_dir = typst_local_dir().join(&target);
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

    for entry in walker_install(src_dir)? {
        if let Ok(entry) = entry {
            let path = entry.path();
            let dest = version_dir.join(path.strip_prefix(src_dir).unwrap());
            debug!("Copying {:?} to {:?}", path, dest);
            if path.is_file() {
                if let Some(parent) = dest.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(&path, &dest)?;
            } else if path.is_dir() {
                fs::create_dir_all(&dest)?;
            }
        }
    }

    Ok(())
}
