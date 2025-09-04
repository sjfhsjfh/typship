use std::{io::Read, path::PathBuf, str::FromStr};

use anyhow::bail;
use clap::Parser;
use log::info;
use tinymist_package::{CloneIntoPack, GitClPack, MapPack, PackageSpec, TarballPack, UniversePack};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use url::Url;

use crate::utils::{typst_cache_dir, typst_local_dir};

const DETACHED_NAMESPACE: &str = "DETACHED";

fn download(url: &Url) -> Result<Vec<u8>, anyhow::Error> {
    let resp = reqwest::blocking::get(url.clone())
        .map_err(|e| anyhow::anyhow!("Failed to download file: {}", e))?;
    if !resp.status().is_success() {
        bail!("Failed to download file: HTTP {}", resp.status());
    }
    let data = resp
        .bytes()
        .map_err(|e| anyhow::anyhow!("Failed to read response: {}", e))?
        .to_vec();
    Ok(data)
}

fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut d = flate2::read::GzDecoder::new(data);
    let mut decompressed = vec![];
    d.read_to_end(&mut decompressed)
        .map_err(|e| anyhow::anyhow!("Failed to decompress tarball: {}", e))?;
    Ok(decompressed)
}

/// TODO: remove this after a `CloneIntoPack` implementation from tinymist_package is available
fn construct_tarball(pack: &mut MapPack) -> Result<Vec<u8>, anyhow::Error> {
    fn custom(err: impl std::error::Error) -> tinymist_package::PackageError {
        tinymist_package::PackageError::Other(Some(format!("{err}").into()))
    }
    let mut tar_data = vec![];
    let mut tar_builder = tar::Builder::new(&mut tar_data);
    tinymist_package::PackFs::read_all(pack, &mut |path, file| {
        let mut header = tar::Header::new_gnu();
        let data = match file {
            tinymist_package::PackFile::Read(mut reader) => {
                let mut dst = Vec::new();
                std::io::copy(&mut reader, &mut dst).map_err(custom)?;
                dst
            }
            tinymist_package::PackFile::Data(data) => data.into_inner().to_vec(),
        };
        header.set_size(data.len() as u64);
        header.set_cksum();
        tar_builder
            .append_data(&mut header, path, &data[..])
            .map_err(custom)?;
        Ok(())
    })?;
    Ok(tar_builder.into_inner().map_err(custom)?.to_vec())
}

#[derive(Parser)]
/// Copy
pub struct CopyArgs {
    pub source: String,
    pub destination: String,
}

#[allow(dead_code)]
pub enum PackKind {
    /// Git repository, can be SSH or HTTPS
    Git {
        git: Url,
    },

    Local {
        path: PathBuf,
    },

    /// Typst Universe package, e.g. `cetz-plot:0.1.2`
    Universe {
        spec: PackageSpec,
    },

    // TODO: Optimize this
    TarStream {
        data: Vec<u8>,
    },

    Npm {
        spec: PackageSpec,
    },
}

impl TryFrom<&str> for PackKind {
    type Error = anyhow::Error;

    // Are we supporting abbreviated forms?
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        // {packkind}:rest
        let prefix = value
            .split_once(':')
            .ok_or_else(|| anyhow::anyhow!("Cannot infer the package type from the source"))?;
        match prefix.0 {
            "git" => {
                let git =
                    Url::parse(prefix.1).map_err(|e| anyhow::anyhow!("Invalid git URL: {}", e))?;
                Ok(PackKind::Git { git })
            }
            "local" => {
                let path = if let Some((dir_kind, spec)) = prefix.1.split_once('@') {
                    let sub_dir = PackageSpec::from_str(spec)
                        .map_err(|e| anyhow::anyhow!("Invalid package spec: {}", e))?;
                    match dir_kind {
                        "" | "data" => typst_local_dir(),
                        "cache" => typst_cache_dir(),
                        _ => {
                            bail!("Unsupported local directory kind: {}", dir_kind)
                        }
                    }
                    .join(sub_dir.namespace.as_str())
                    .join(sub_dir.name.as_str())
                    .join(sub_dir.version.to_string())
                } else {
                    PathBuf::from(prefix.1)
                };
                Ok(PackKind::Local { path })
            }
            "universe" => {
                let spec = PackageSpec::from_str(prefix.1)
                    .map_err(|e| anyhow::anyhow!("Invalid package spec: {}", e))?;
                Ok(PackKind::Universe { spec })
            }
            "tar.local" => {
                let data = std::fs::read(prefix.1)
                    .map_err(|e| anyhow::anyhow!("Failed to read tarball: {}", e))?;
                Ok(PackKind::TarStream { data })
            }
            "tar.remote" => {
                let url = Url::parse(prefix.1)
                    .map_err(|e| anyhow::anyhow!("Invalid tarball URL: {}", e))?;
                let data = download(&url)?;
                Ok(PackKind::TarStream { data })
            }
            "tar.gz.local" => {
                let data = std::fs::read(prefix.1)
                    .map_err(|e| anyhow::anyhow!("Failed to read tarball: {}", e))?;
                let data = decompress_gzip(&data)?;
                Ok(PackKind::TarStream { data })
            }
            "tar.gz.remote" => {
                let url = Url::parse(prefix.1)
                    .map_err(|e| anyhow::anyhow!("Invalid tarball URL: {}", e))?;
                let data = download(&url)?;
                let data = decompress_gzip(&data)?;
                Ok(PackKind::TarStream { data })
            }
            "npm" => {
                let spec = PackageSpec::from_str(prefix.1)
                    .map_err(|e| anyhow::anyhow!("Invalid package spec: {}", e))?;
                Ok(PackKind::Npm { spec })
            }
            _ => {
                bail!("Unsupported package kind: {}", prefix.0)
            }
        }
    }
}

impl PackKind {
    pub fn read_to_memory(self) -> Result<MapPack, anyhow::Error> {
        let mut pack = MapPack::default();
        match self {
            PackKind::Git { git } => {
                pack.clone_into_pack(&mut GitClPack::new(DETACHED_NAMESPACE.into(), git))?;
            }
            PackKind::Local { path } => {
                pack.clone_into_pack(&mut tinymist_package::DirPack::new(path))?;
            }
            PackKind::Universe { spec } => {
                pack.clone_into_pack(&mut UniversePack::new(spec))?;
            }
            PackKind::TarStream { data } => {
                pack.clone_into_pack(&mut TarballPack::new(std::io::Cursor::new(data)))?;
            }
            PackKind::Npm { spec: _ } => todo!(),
        }
        Ok(pack)
    }
}

pub async fn copy(args: &CopyArgs) -> anyhow::Result<()> {
    let src = match args.source.as_str() {
        "-" => {
            let mut data = vec![];
            tokio::io::stdin().read_to_end(&mut data).await?;
            PackKind::TarStream { data }
        }
        rest => PackKind::try_from(rest)?,
    };
    let mut writing_to_stdout = false;
    let dst: PackKind = match args.destination.as_str() {
        "-" => {
            writing_to_stdout = true;
            PackKind::TarStream { data: vec![] }
        }
        rest => PackKind::try_from(rest)?,
    };

    if writing_to_stdout {
        log::set_max_level(log::LevelFilter::Warn);
    }

    info!("Copying {} to {}", args.source, args.destination);

    let mut pack = src.read_to_memory()?;

    match dst {
        PackKind::Git { git: _ } => {
            bail!("Cannot write to a git repository");
        }
        PackKind::Local { path } => {
            tinymist_package::DirPack::new(path).clone_into_pack(&mut pack)?;
        }
        PackKind::Universe { spec: _ } => {
            bail!("Cannot write to the universe");
        }
        PackKind::TarStream { data: _ } => {
            if !writing_to_stdout {
                bail!("Writing to a non-stdout tar stream is not supported");
            }
            let data = construct_tarball(&mut pack)?;
            tokio::io::stdout().write_all(data.as_slice()).await?;
        }
        PackKind::Npm { spec: _ } => {
            bail!("Cannot write to npm");
        }
    }

    Ok(())
}
