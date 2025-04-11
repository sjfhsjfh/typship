pub mod commands;
pub mod config;
pub mod model;
pub mod regs;
pub mod utils;

pub const NAME: &str = env!("CARGO_PKG_NAME");
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const ABOUT: &str = "A simple package manager for Typst";
