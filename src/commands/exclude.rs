use std::path::Path;

use anyhow::Result;
use clap::Parser;
use typst_syntax::package::PackageManifest;

use crate::utils::{read_manifest, write_manifest};

#[derive(Parser)]
/// Exclude files for the published bundle
pub struct ExcludeArgs {
    #[arg(required = true, num_args = 1..)]
    /// Files to exclude
    pub files: Vec<String>,
}

pub fn exclude(package_dir: &Path, args: &ExcludeArgs) -> Result<()> {
    let mut current: PackageManifest = read_manifest(package_dir)?;
    for file in &args.files {
        // TODO: Validate glob?
        current.package.exclude.push(file.into());
    }
    current.package.exclude.dedup();
    write_manifest(package_dir, &current)?;
    Ok(())
}
