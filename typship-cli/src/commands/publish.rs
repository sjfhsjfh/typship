use std::path::Path;

use anyhow::Result;
use clap::{ArgAction, Parser};

use crate::regs::universe::{self, UploadMethod};
use crate::utils::read_manifest;

const LONG_ABOUT: &str =
    "Publish the package to a certain registry. Currently, only the official Universe (GitHub) registry is supported.";

#[derive(Parser)]
#[command(long_about = LONG_ABOUT)]
/// Publish the package to a certain registry
pub struct PublishArgs {
    #[arg(required = true)]
    #[arg(long_help = "The registry to publish. Supported registries: universe (GitHub).")]
    /// The registry to publish
    pub registry: String,

    #[arg(long, required = false, action = ArgAction::SetTrue)]
    #[arg(long_help = "Dry run the publish process. No actual changes will be made.")]
    /// Dry run the publish process
    pub dry_run: bool,

    #[arg(long, value_enum, default_value = "sparse")]
    #[arg(
        long_help = "Upload method: sparse (uses git sparse-checkout); api (uploads files one by one, for legacy git, slower)."
    )]
    /// Upload method: sparse (uses git sparse-checkout); api (uploads files one by one, for legacy git, slower).
    pub upload_method: UploadMethod,
}

pub async fn publish(package_dir: &Path, args: &PublishArgs) -> Result<()> {
    let current = read_manifest(package_dir)?;
    match args.registry.as_str() {
        "universe" => {
            universe::publish(&current, package_dir, args.dry_run, args.upload_method).await?
        }
        _ => {
            anyhow::bail!("Unsupported registry: {}", args.registry);
        }
    };
    Ok(())
}
