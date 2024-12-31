use clap::Command;
use commands::init::init;
use std::fs;
use typst_syntax::package::PackageManifest;

mod commands;
pub mod model;

#[cfg(test)]
mod tests;

fn main() {
    const NAME: &str = env!("CARGO_PKG_NAME");
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let matches = Command::new(NAME)
        .version(VERSION)
        .about("A simple package manager for Typst")
        .subcommand(Command::new("init").about("Initialize a new package"))
        .get_matches();

    let mut current_manifest: Option<PackageManifest> = None;
    if let std::result::Result::Ok(manifest) = fs::read_to_string("typst.toml") {
        current_manifest = toml::from_str(&manifest).ok();
    }

    if let Some(_) = matches.subcommand_matches("init") {
        if let Err(e) = init(&current_manifest) {
            eprintln!("Error: {}", e);
        }
    }
}
