use clap::Command;
use std::fs;
use typst_syntax::package::PackageManifest;

mod commands;
mod model;
mod utils;

#[cfg(test)]
mod tests;

fn main() {
    const NAME: &str = env!("CARGO_PKG_NAME");
    const VERSION: &str = env!("CARGO_PKG_VERSION");

    let matches = Command::new(NAME)
        .version(VERSION)
        .about("A simple package manager for Typst")
        .subcommand(commands::init::cmd())
        .subcommand(commands::exclude::cmd())
        .get_matches();

    let mut current_manifest: Option<PackageManifest> = None;
    if let std::result::Result::Ok(manifest) = fs::read_to_string("typst.toml") {
        current_manifest = toml::from_str(&manifest).ok();
    }

    if let Some(_) = matches.subcommand_matches("init") {
        if let Err(e) = commands::init::init(&current_manifest) {
            eprintln!("Error: {}", e);
        }
    }
    if let Some(m) = matches.subcommand_matches("exclude") {
        if let Err(e) = commands::exclude::exclude(
            &mut current_manifest,
            m.get_many::<String>("files")
                .unwrap_or_default()
                .map(|s| s.as_str())
                .collect(),
        ) {
            eprintln!("Error: {}", e);
        }
    }
}
