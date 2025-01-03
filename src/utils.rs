use std::{
    env, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use typst_syntax::package::PackageManifest;

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
