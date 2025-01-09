use anyhow::Result;
use clap::{Arg, Command};

use crate::regs::universe;

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

pub fn login(registry: &str) -> Result<()> {
    match registry {
        "universe" => universe::login(),
        _ => {
            anyhow::bail!("Unsupported registry: {}", registry);
        }
    }
}
