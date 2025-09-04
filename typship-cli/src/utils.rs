pub mod walkers;

use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{env, fs};

use anyhow::{Context, Result};
use log::info;
use sha2::{Digest, Sha256};
use typst_syntax::package::PackageManifest;

use crate::config::Config;

const DEFAULT_PACKAGES_SUBDIR: &str = "typst/packages"; // from typst-kit

pub fn config_dir() -> &'static Path {
    static CONFIG_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
        dirs::config_dir()
            .expect("Failed to get the config directory")
            .join(env!("CARGO_PKG_NAME"))
    });

    CONFIG_DIR.as_path()
}

pub fn config_file() -> &'static Path {
    static CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| config_dir().join("config.toml"));

    CONFIG_PATH.as_path()
}

/// Should always return a valid config
pub fn load_config() -> Result<Config> {
    if !config_file().exists() {
        info!("Creating a new configuration file...");
        let cfg = Config::default();
        save_config(&cfg).expect("Failed to save the configuration file");
        return Ok(cfg);
    }
    let config =
        fs::read_to_string(config_file()).context("Failed to read the configuration file")?;
    let config = toml::from_str(&config).context("Failed to parse the configuration file")?;
    Ok(config)
}

pub fn save_config(config: &Config) -> Result<()> {
    if !config_dir().exists() {
        fs::create_dir_all(config_dir()).context("Failed to create the configuration directory")?;
    }
    let config = toml::to_string_pretty(config)?;
    fs::write(config_file(), config).context("Failed to write the configuration file")?;
    Ok(())
}

/// Data dir, not cache dir
pub fn typst_local_dir() -> PathBuf {
    dirs::data_dir()
        .expect("Failed to get the data directory")
        .join(DEFAULT_PACKAGES_SUBDIR)
}

pub fn typst_cache_dir() -> PathBuf {
    dirs::cache_dir()
        .expect("Failed to get the cache directory")
        .join(DEFAULT_PACKAGES_SUBDIR)
}

pub fn temp_dir() -> PathBuf {
    let mut path = env::temp_dir();
    path.push(env!("CARGO_PKG_NAME"));
    path
}

pub fn temp_subdir(id: &str) -> PathBuf {
    let mut path = env::temp_dir();
    let hash = format!("{:x}", Sha256::digest(id.as_bytes()));
    let truncated_hash = &hash[0..16];
    path.push(format!("{}-{}", env!("CARGO_PKG_NAME"), truncated_hash));
    path
}

pub fn read_manifest(package_dir: &Path) -> Result<PackageManifest> {
    let manifest = fs::read_to_string(package_dir.join("typst.toml"))
        .context("Failed to read the package manifest file")?;
    let manifest = toml::from_str(&manifest).context("Failed to parse the package manifest")?;
    Ok(manifest)
}

pub fn write_manifest(package_dir: &Path, manifest: &PackageManifest) -> Result<()> {
    let manifest = toml::to_string_pretty(manifest)?;
    fs::write(package_dir.join("typst.toml"), manifest)
        .context("Failed to write the package manifest file")?;
    Ok(())
}
