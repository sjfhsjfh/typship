use std::path::Path;

use crate::regs::universe::{self};
use anyhow::Result;
use clap::{ArgAction, Parser};

use crate::utils::read_manifest;

const ABOUT: &str = "Publish the package to a certain registry";
const LONG_ABOUT: &str =
    "Publish the package to a certain registry. Currently, only the official Universe (GitHub) registry is supported.";

#[derive(Parser)]
#[command(about = ABOUT, long_about = LONG_ABOUT)]
pub struct PublishArgs {
    /// The registry to publish
    #[arg(required = true)]
    #[arg(long_help = "The registry to publish. Supported registries: universe (GitHub).")]
    pub registry: String,
    /// Dry run the publish process
    #[arg(long, required = false, action = ArgAction::SetTrue)]
    #[arg(long_help = "Dry run the publish process. No actual changes will be made.")]
    pub dry_run: bool,
}

pub async fn publish(package_dir: &Path, args: &PublishArgs) -> Result<()> {
    let current = read_manifest(package_dir)?;
    Ok(match args.registry.as_str() {
        "universe" => universe::publish(&current, package_dir, args.dry_run).await?,
        _ => {
            anyhow::bail!("Unsupported registry: {}", args.registry);
        }
    })
}
