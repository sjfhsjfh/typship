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
        .subcommand_required(true)
        .subcommand(commands::check::cmd())
        .subcommand(commands::install::cmd())
        .subcommand(commands::exclude::cmd())
        .subcommand(commands::init::cmd());

    let matches = matches.subcommand(commands::download::cmd());

    let matches = matches.get_matches();

    let mut current_manifest: Option<PackageManifest> = read_manifest(".".as_ref()).ok();

    if let Err(e) = match matches.subcommand() {
        Some(("check", _)) => commands::check::check(&current_manifest),
        Some(("download", m)) => {
            commands::download::download(m.get_one::<String>("repository").unwrap())
        }
        Some(("exclude", m)) => commands::exclude::exclude(
            ".".as_ref(),
            &mut current_manifest,
            m.get_many::<String>("files")
                .unwrap_or_default()
                .map(|s| s.as_str())
                .collect(),
        ),
        Some(("init", m)) => commands::init::init(
            ".".as_ref(),
            &current_manifest,
            m.get_one::<String>("name").map(|s| s.as_str()),
        ),
        Some(("install", m)) => commands::install::install(
            ".".as_ref(),
            &current_manifest,
            m.get_one::<String>("target").unwrap().as_str(),
        ),
        _ => Ok(()),
    } {
        eprintln!("Error: {}", e);
    }
}
