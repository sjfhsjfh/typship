use anyhow::Result;
use clap::Parser;

use crate::regs::universe;

const ABOUT: &str = "Login to the certain registry";
const LONG_ABOUT: &str = "Login to the certain registry. Currently, only the official Universe (GitHub) registry is supported.";

#[derive(Parser)]
#[command(about = ABOUT, long_about = LONG_ABOUT)]
pub struct LoginArgs {
    #[arg(required = true)]
    #[arg(help = "The registry to login (universe)")]
    #[arg(long_help = "The registry to login. Supported registries: universe (GitHub).")]
    pub registry: String,
}


pub fn login(args: &LoginArgs) -> Result<()> {
    match args.registry.as_str() {
        "universe" => universe::login(),
        _ => {
            anyhow::bail!("Unsupported registry: {}", args.registry);
        }
    }
}
