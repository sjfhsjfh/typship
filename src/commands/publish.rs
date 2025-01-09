use std::path::Path;

use crate::regs::universe::{self};
use anyhow::Result;
use clap::{Arg, Command};

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
        )
}

pub async fn publish(package_dir: &Path, registry: &str) -> Result<()> {
    let current = read_manifest(package_dir)?;
    Ok(match registry {
        "universe" => universe::publish(&current, package_dir).await?,
        _ => {
            anyhow::bail!("Unsupported registry: {}", registry);
        }
    })
}
