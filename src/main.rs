use std::env;
use std::io::Write;
use std::path::Path;

use clap::Parser;
use commands::Commands;
use log::error;

mod commands;
mod config;
mod model;
mod regs;
mod utils;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const ABOUT: &str = "A simple package manager for Typst";

#[derive(Parser)]
#[command(name = NAME)]
#[command(version = VERSION)]
#[command(about = ABOUT, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "info"),
    )
    .format(|buf, record| {
        let level_style = buf.default_level_style(record.level());
        writeln!(
            buf,
            "{level_style}{}{level_style:#} {}",
            record.level(),
            record.args()
        )
    })
    .init();

    let cli = Cli::parse();
    let current_dir = std::env::current_dir().expect("Failed to get the current directory");

    if let Err(e) = match_cmd(&current_dir, &cli).await {
        error!("{:?}", e);
    }
}

async fn match_cmd(current_dir: &Path, args: &Cli) -> anyhow::Result<()> {
    match &args.command {
        Commands::Check(_) => commands::check::check(current_dir),
        Commands::Clean(args) => commands::clean::clean(args),
        Commands::Dev(_) => commands::dev::dev(current_dir).await,
        Commands::Copy(args) => commands::download::copy(args),
        Commands::Download(args) => commands::download::download(args),
        Commands::Exclude(args) => commands::exclude::exclude(current_dir, args),
        Commands::Init(args) => commands::init::init(current_dir, args),
        Commands::Install(args) => commands::install::install(current_dir, args),
        Commands::Login(args) => commands::login::login(args),
        Commands::Publish(args) => commands::publish::publish(current_dir, args).await,
    }
}
