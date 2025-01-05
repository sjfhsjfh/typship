use anyhow::Result;
use clap::{Arg, Command};
use log::info;

use crate::{
    model::Config,
    utils::{config_dir, save_config},
};

pub fn cmd() -> Command {
    Command::new("login")
        .about("Login to the certain registry")
        .long_about("Login to the certain registry. Currently, only the official Universe (GitHub) registry is supported.")
        .arg(
            Arg::new("registry")
                .required(true)
                .help("The registry to login (universe)")
                .long_help(
                    "The registry to login. Supported registries: universe (GitHub).",
                ),
        )
}

pub fn login(config: &mut Config, registry: &str) -> Result<()> {
    match registry {
        "universe" => {
            let overwrite = if config.tokens.universe.is_some() {
                info!("Already logged in to the Universe registry");
                dialoguer::Confirm::new()
                    .with_prompt("Do you want to overwrite the existing token?")
                    .default(false)
                    .interact()?
            } else {
                true
            };
            if !overwrite {
                return Ok(());
            }
            let token = dialoguer::Password::new()
                .with_prompt("Enter your GitHub token")
                .interact()?;
            config.tokens.universe = Some(token);
            save_config(config)?;
            info!(
                "Your token has been saved to {}",
                config_dir().join("config.toml").to_string_lossy()
            );
            Ok(())
        }
        _ => {
            anyhow::bail!("Unsupported registry: {}", registry);
        }
    }
}
