use std::{fs, path::PathBuf, thread};

use anyhow::{bail, Result};
use clap::{Arg, Command};

use crate::{commands::install::install, utils::temp_subdir};

pub fn cmd() -> Command {
    Command::new("download")
        .about("Download a package from git repository to `@local` namespace")
        .long_about("Download a package from git repository to `@local` namespace. Using the latest commit of the default branch for now.")
        .arg(
            Arg::new("repository")
                .help("Git repository URL")
                .required(true),
        )
}

// TODO: allow checkout tag, commit, branch

pub fn download(repo: &str) -> Result<()> {
    let temp_dir = temp_subdir(repo);
    fs::create_dir_all(&temp_dir)?;

    let res = temp_jobs(temp_dir.clone(), repo);
    fs::remove_dir_all(&temp_dir)?;
    res?;

    println!("Done");
    Ok(())
}

fn temp_jobs(temp_dir: PathBuf, repo: &str) -> Result<()> {
    fs::remove_dir_all(&temp_dir)?;
    fs::create_dir_all(&temp_dir)?;
    println!("Cloning repository to {:?}", temp_dir);
    let repo_clone = repo.to_string();
    let temp_dir_clone = temp_dir.clone();
    let handle = thread::spawn(move || {
        let status = std::process::Command::new("git")
            .arg("clone")
            .arg(repo_clone)
            .arg(".")
            .current_dir(&temp_dir_clone)
            .status()
            .expect("Failed to run git clone");
        status
    });
    if !handle.join().is_ok() {
        fs::remove_dir_all(&temp_dir)?;
        bail!("Failed to join the thread");
    }

    println!("Installing...");
    install(&temp_dir, "local")?;
    Ok(())
}
