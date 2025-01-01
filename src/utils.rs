use std::fs;

use anyhow::{Context, Result};
use typst_syntax::package::PackageManifest;

pub fn write_manifest(manifest: &PackageManifest) -> Result<()> {
    let manifest = toml::to_string_pretty(manifest)?;
    fs::write("typst.toml", manifest).context("Failed to write the package manifest file")?;
    Ok(())
}
