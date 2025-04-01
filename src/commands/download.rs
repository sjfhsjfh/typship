use std::fs;
use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{bail, Result};
use clap::Parser;
use log::info;
use tinymist_package::{CloneIntoPack, DirPack, GitClPack, PackageSpec, UniversePack};

use crate::commands::install::{install, InstallArgs};
use crate::utils::temp_subdir;

const LONG_ABOUT: &str = "Download a package from git repository to a certain (defaults to `@local`) namespace. You may specify a specific tag, commit, or branch to checkout to.";

#[derive(Parser)]
#[command(long_about = LONG_ABOUT)]
/// Download a package from git repository to a certain (defaults to `@local`)
/// namespace
pub struct DownloadArgs {
    /// Git repository URL
    pub repository: String,

    #[arg(short, long, value_name = "REF")]
    /// Checkout to a specific tag, commit, or branch
    pub checkout: Option<String>,

    #[arg(short, long, default_value = "local")]
    /// Namespace to install the package to (without the `@` prefix)
    pub namespace: String,
}

pub fn download(args: &DownloadArgs) -> Result<()> {
    let temp_dir = temp_subdir(&args.repository);
    fs::create_dir_all(&temp_dir)?;

    let res = temp_jobs(
        temp_dir.clone(),
        &args.repository,
        args.checkout.as_deref(),
        &args.namespace,
    );
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
    install(
        &temp_dir,
        &InstallArgs {
            target: namespace.to_string(),
        },
    )?;
    Ok(())
}

#[derive(Parser)]
/// Copy
pub struct CopyArgs {
    /// Git repository URL
    pub source: String,

    #[arg(short, long, default_value = "local")]
    /// Namespace to install the package to (without the `@` prefix)
    pub namespace: String,
}

enum PackKind {
    Git,
    Universe(PackageSpec),
}

pub fn copy(args: &CopyArgs) -> anyhow::Result<()> {
    println!("Copying {} to @{}", args.source, args.namespace);

    let mut dst = DirPack::new("target/zebraw");

    let inferred = if args.source.starts_with("git@") {
        PackKind::Git
    } else if let Some(spec) = args.source.strip_prefix("pkg:") {
        PackKind::Universe(
            PackageSpec::from_str(spec)
                .map_err(|e| anyhow::anyhow!("Invalid package spec: {}", e))?,
        )
    } else {
        bail!("cannot infer the package type from the source");
    };

    match inferred {
        PackKind::Git => {
            let mut src = GitClPack::new(args.namespace.as_str().into(), args.source.clone());
            dst.clone_into_pack(&mut src)?;
        }
        PackKind::Universe(spec) => {
            let mut src = UniversePack::new(spec);
            dst.clone_into_pack(&mut src)?;
        }
    }

    Ok(())
}
