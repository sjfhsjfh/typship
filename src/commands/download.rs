use std::{fs, path::PathBuf};

use anyhow::{bail, Result};
use clap::{Arg, Command};
use log::info;

use crate::{commands::install::install, utils::temp_subdir};

pub fn cmd() -> Command {
    Command::new("download")
        .about("Download a package from git repository to a certain (defaults to `@local`) namespace")
        .long_about("Download a package from git repository to a certain (defaults to `@local`) namespace. You may specify a specific tag, commit, or branch to checkout to.")
        .arg(
            Arg::new("repository")
                .help("Git repository URL")
                .required(true),
        )
        .arg(
            Arg::new("checkout")
                .help("Checkout to a specific tag, commit, or branch")
                .short('c')
                .long("checkout")
                .required(false)
                .value_name("REF")
        )
        .arg(
            Arg::new("namespace")
                .help("Namespace to install the package to (without the `@` prefix)")
                .short('n')
                .long("namespace")
                .default_value("local")
                .value_name("NAMESPACE")
        )
}

// TODO: allow checkout tag, commit, branch

pub fn download(repo: &str, checkout: Option<&str>, namespace: &str) -> Result<()> {
    let temp_dir = temp_subdir(repo);
    fs::create_dir_all(&temp_dir)?;

    let res = temp_jobs(temp_dir.clone(), repo, checkout, namespace);
    fs::remove_dir_all(&temp_dir)?;
    res?;

    info!("Done");
    Ok(())
}

fn temp_jobs(temp_dir: PathBuf, repo: &str, checkout: Option<&str>, namespace: &str) -> Result<()> {
    info!("Cloning the repository...");
    fs::remove_dir_all(&temp_dir)?;
    fs::create_dir_all(&temp_dir)?;
    let status = std::process::Command::new("git")
        .arg("clone")
        // .arg("--depth=1")
        .arg(repo)
        .arg(&temp_dir)
        .current_dir(&temp_dir)
        .status()
        .expect("Failed to run git clone");
    if !status.success() {
        bail!("Failed to clone");
    }

    if let Some(checkout) = checkout {
        info!("Checking out to {}...", checkout);
        let status = std::process::Command::new("git")
            .arg("checkout")
            .arg(checkout)
            .current_dir(&temp_dir)
            .status()
            .expect("Failed to run git checkout");
        if !status.success() {
            bail!("Failed to checkout");
        }
    }

    info!("Installing...");
    install(&temp_dir, namespace)?;
    Ok(())
}
