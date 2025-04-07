use std::path::Path;

use anyhow::Result;
use clap::Parser;
use serde::de::IgnoredAny;
use tinymist_package::PackageSpec;

use crate::utils::read_manifest;

#[derive(Parser)]
/// Sync pacakges
pub struct SyncArgs {}

pub fn sync(src_dir: &Path, args: &SyncArgs) -> Result<()> {
    // Step1: read the file
    let manifest = read_manifest(src_dir)?;

    let t = manifest
        .unknown_fields
        .get("typship.dependencies")
        .ok_or_else(|| anyhow::anyhow!("No dependencies found in the manifest"))?;

    let deps = as_deps(t)?;

    for dep in deps {
        sync_dep(src_dir, &dep)?;
    }

    // todo: upgrade
    // typst-upgrade = {git = "https://github.com/Coekjan/typst-upgrade"}

    Ok(())
}

fn sync_dep(src_dir: &Path, dep: &PackKind) -> Result<()> {
    todo!()
}

fn as_deps(t: &IgnoredAny) -> Result<Vec<PackKind>> {
    todo!()
}

#[derive(Parser)]
/// Copy
pub struct InstallArgs {
    /// Git repository URL
    pub source: String,

    #[arg(short, long, default_value = "local")]
    /// Namespace to install the package to (without the `@` prefix)
    pub namespace: String,
}

enum PackKind {
    Git(InstallArgs),
    Universe(PackageSpec, InstallArgs),
}
