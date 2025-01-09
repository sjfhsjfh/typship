use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use log::info;
use sha2::{Digest, Sha256};
use typst_syntax::package::PackageManifest;

use crate::config::Config;

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .expect("Failed to get the config directory")
        .join(env!("CARGO_PKG_NAME"))
}

pub fn config_file() -> PathBuf {
    config_dir().join("config.toml")
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
        fs::create_dir_all(&config_dir())
            .context("Failed to create the configuration directory")?;
    }
    let config = toml::to_string_pretty(config)?;
    fs::write(config_file(), config).context("Failed to write the configuration file")?;
    Ok(())
}

pub fn typst_local_dir() -> PathBuf {
    dirs::data_dir()
        .expect("Failed to get the data directory")
        .join(typst_kit::package::DEFAULT_PACKAGES_SUBDIR)
}

pub fn temp_dir() -> PathBuf {
    let mut path = env::temp_dir();
    path.push(env!("CARGO_PKG_NAME"));
    path
}

pub fn temp_subdir(id: &str) -> PathBuf {
    let mut path = temp_dir();
    let hash = format!("{:x}", Sha256::digest(id.as_bytes()));
    path.push(hash);
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
