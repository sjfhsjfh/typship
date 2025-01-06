use std::path::Path;

use anyhow::Result;
use clap::{Arg, Command};
use octocrab::{initialise, Octocrab};
use secrecy::SecretString;

use crate::{model::Config, utils::read_manifest};

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

pub async fn publish(package_dir: &Path, config: &Config, registry: &str) -> Result<()> {
    match registry {
        "universe" => {
            let current = read_manifest(package_dir)?;
            // TODO: better secret management
            let token = config.tokens.universe.clone().ok_or(anyhow::anyhow!(
                "You need to set up the token first. Run `typship login universe`."
            ))?;
            let token = SecretString::from(token);
            let client = initialise(Octocrab::builder().personal_token(token).build()?);
            // TODO: check if exist in package repo(name, registry), check pr
            unimplemented!();
            Ok(())
        }
        _ => {
            anyhow::bail!("Unsupported registry: {}", registry);
        }
    }
}
