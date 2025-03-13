use std::path::Path;

use crate::regs::universe::{self};
use anyhow::Result;
use clap::{Arg, ArgAction, Command};

use crate::utils::read_manifest;

pub fn cmd() -> Command {
    Command::new("publish")
        .about("Publish the package to the certain registry")
        .long_about("Publish the package to the certain registry. Currently, only the official Universe (GitHub) registry is supported.")
        .arg(
            Arg::new("registry")
                .required(true)
                .help("The registry to publish (universe)")
                .long_help(
                    "The registry to publish. Supported registries: universe (GitHub).",
                ),
        ).arg(
            Arg::new("dry-run")
                .long("dry-run")
                .required(false)
                .action(ArgAction::SetTrue)
                .help("Dry run the publish process")
                .long_help("Dry run the publish process. No actual changes will be made."),
        )
}

pub async fn publish(package_dir: &Path, registry: &str, dry_run: bool) -> Result<()> {
    let current = read_manifest(package_dir)?;
    Ok(match registry {
        "universe" => universe::publish(&current, package_dir, dry_run).await?,
        _ => {
            anyhow::bail!("Unsupported registry: {}", registry);
        }
    })
}
