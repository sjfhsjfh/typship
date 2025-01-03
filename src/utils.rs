use std::fs;

use anyhow::{Context, Result};
use typst_syntax::package::PackageManifest;

pub fn read_manifest() -> Result<PackageManifest> {
    let manifest =
        fs::read_to_string("typst.toml").context("Failed to read the package manifest file")?;
    let manifest = toml::from_str(&manifest).context("Failed to parse the package manifest")?;
    Ok(manifest)
}

pub fn write_manifest(manifest: &PackageManifest) -> Result<()> {
    let manifest = toml::to_string_pretty(manifest)?;
    fs::write("typst.toml", manifest).context("Failed to write the package manifest file")?;
    Ok(())
}
