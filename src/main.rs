use clap::Command;
use typst_syntax::package::PackageManifest;
use utils::read_manifest;

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
        .subcommand(commands::check::cmd())
        .subcommand(commands::exclude::cmd())
        .subcommand(commands::init::cmd())
        .subcommand_required(true)
        .get_matches();

    let mut current_manifest: Option<PackageManifest> = read_manifest().ok();

    if let Err(e) = match matches.subcommand() {
        Some(("check", _)) => commands::check::check(&current_manifest),
        Some(("exclude", m)) => commands::exclude::exclude(
            &mut current_manifest,
            m.get_many::<String>("files")
                .unwrap_or_default()
                .map(|s| s.as_str())
                .collect(),
        ),
        Some(("init", m)) => commands::init::init(
            &current_manifest,
            m.get_one::<String>("name").map(|s| s.as_str()),
        ),
        _ => Ok(()),
    } {
        eprintln!("Error: {}", e);
    }
}
