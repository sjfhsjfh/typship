use std::{fs, thread};

use anyhow::Result;
use clap::{Arg, Command};
use git2::Repository;

use crate::utils::temp_subdir;

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
    let temp_dir_clone = temp_dir.clone();
    fs::create_dir_all(&temp_dir)?;
    Repository::clone(repo, &temp_dir)?;

    // Go to temp_dir and run the cli itself
    let handle = thread::spawn(move || {
        let status = std::process::Command::new(env!("CARGO_PKG_NAME"))
            .arg("install")
            .arg("local")
            .current_dir(&temp_dir_clone)
            .status()
            .expect("Failed to run install");
        status
    });

    handle.join().expect("Failed to join thread");

    fs::remove_dir_all(&temp_dir)?;

    Ok(())
}
