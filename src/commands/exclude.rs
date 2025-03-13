use std::path::Path;

use anyhow::Result;
use clap::Parser;
use typst_syntax::package::PackageManifest;

use crate::utils::{read_manifest, write_manifest};

const ABOUT: &str = "Exclude files for the published bundle";

#[derive(Parser)]
#[command(about = ABOUT)]
pub struct ExcludeArgs {
    /// Files to exclude
    #[arg(required = true, num_args = 1..)]
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
