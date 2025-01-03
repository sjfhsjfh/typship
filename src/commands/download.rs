use std::{fs, path::PathBuf, thread};

use anyhow::{bail, Result};
use clap::{Arg, Command};

use crate::{
    commands::install::install,
    utils::{read_manifest, temp_subdir},
};

pub fn cmd() -> Command {
    Command::new("download")
        .about("Download package from git repository")
        .arg(
            Arg::new("repository")
                .help("Git repository URL")
                .required(true),
        )
}

pub fn download(repo: &str) -> Result<()> {
    let temp_dir = temp_subdir(repo);
    fs::create_dir_all(&temp_dir)?;

    return match temp_jobs(temp_dir.clone(), repo) {
        Ok(_) => {
            fs::remove_dir_all(&temp_dir)?;
            println!("Done");
            Ok(())
        }
        Err(e) => {
            fs::remove_dir_all(&temp_dir)?;
            bail!(e);
        }
    };
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
    install(&temp_dir, &read_manifest(&temp_dir).ok(), "local")?;
    Ok(())
}
