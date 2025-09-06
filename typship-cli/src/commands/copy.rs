use std::{io::Read, path::PathBuf, str::FromStr, sync::Arc};

use anyhow::bail;
use clap::Parser;
use ecow::eco_format;
use log::info;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use typship_pack::{
    CloneFromPack, DirPack, GitClPack, MapPack, Pack, PackError, PartialPackageSpec,
    PartialStoragePack, TarballPack, UniversePack,
};
use typst_syntax::package::{PackageManifest, PackageSpec};
use url::Url;

use crate::utils::{typst_cache_dir, typst_local_dir};

const DETACHED_NAMESPACE: &str = "DETACHED";

fn download(url: &Url) -> Result<Vec<u8>, PackError> {
    let resp = reqwest::blocking::get(url.clone()).map_err(|e| {
        PackError::NetworkFailed(Some(eco_format!("Failed to get response: {e:?}")))
    })?;
    match resp.status() {
        code if !code.is_success() => {
            return Err(PackError::NetworkFailed(Some(eco_format!(
                "Failed to download file: HTTP {}",
                resp.status()
            ))));
        }
        _ => {}
    }
    let data = resp
        .bytes()
        .map_err(|e| PackError::Other(Some(eco_format!("Failed to read response: {}", e))))?
        .to_vec();
    Ok(data)
}

fn gzip_reader(r: impl Read) -> impl Read {
    flate2::read::GzDecoder::new(r)
}

fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, anyhow::Error> {
    let mut d = flate2::read::GzDecoder::new(data);
    let mut decompressed = vec![];
    d.read_to_end(&mut decompressed)
        .map_err(|e| anyhow::anyhow!("Failed to decompress tarball: {}", e))?;
    Ok(decompressed)
}

/// TODO: remove this after a `CloneIntoPack` implementation from typship_pack is available
fn construct_tarball(pack: &mut MapPack) -> Result<Vec<u8>, anyhow::Error> {
    fn custom(err: impl std::error::Error) -> typship_pack::PackError {
        typship_pack::PackError::Other(Some(format!("{err}").into()))
    }
    let mut tar_data = vec![];
    let mut tar_builder = tar::Builder::new(&mut tar_data);
    typship_pack::PackFs::read_all(pack, &mut |path, file| {
        let mut header = tar::Header::new_gnu();
        let data = match file {
            typship_pack::PackFile::Read(mut reader) => {
                let mut dst = Vec::new();
                std::io::copy(&mut reader, &mut dst).map_err(custom)?;
                dst
            }
            typship_pack::PackFile::Data(data) => data.into_inner().to_vec(),
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

fn pack_from_http_url(url: &Url) -> Result<Arc<dyn Pack>, PackError> {
    match url.path() {
        tarball if tarball.ends_with(".tar") => {
            let tarball = download(url)?;
            Ok(Arc::new(TarballPack::new(std::io::Cursor::new(tarball))))
        }
        gz if gz.ends_with(".tar.gz") || gz.ends_with(".tgz") => {
            let tar_gz = download(url)?;
            Ok(Arc::new(TarballPack::new(gzip_reader(
                std::io::Cursor::new(tar_gz),
            ))))
        }
        git if git.ends_with(".git") => Ok(Arc::new(GitClPack::new(
            DETACHED_NAMESPACE.into(),
            url.clone(),
        ))),
        _ => Err(PackError::Other(Some(eco_format!(
            "Cannot infer pack type from URL: {url}",
        )))),
    }
}

fn pack_from_ssh_url(url: &Url) -> Result<Arc<dyn Pack>, PackError> {
    if url.path().ends_with(".git") {
        Ok(Arc::new(GitClPack::new(
            DETACHED_NAMESPACE.into(),
            url.clone(),
        )))
    } else {
        Err(PackError::Other(Some(eco_format!(
            "Cannot infer pack type from SSH URL: {url}",
        ))))
    }
}

fn pack_from_local_url(
    url: &Url,
    src_spec: PartialPackageSpec,
) -> Result<Arc<dyn Pack>, PackError> {
    let storage_path = match url.scheme() {
        "data" => typst_local_dir(),
        "cache" => typst_cache_dir(),
        _ => {
            return Err(PackError::Other(Some(eco_format!(
                "Unexpected local URL scheme: {}",
                url.scheme()
            ))));
        }
    };
    let partial_spec = match url.path().strip_prefix('@') {
        Some(rest) => {
            let mut parts = rest.splitn(3, '/');
            PartialPackageSpec {
                namespace: parts.next().map(|s| s.to_string()),
                name: parts.next().map(|s| s.to_string()),
                version: parts.next().map(|s| s.to_string()),
            }
        }
        None => Default::default(),
    };

    if let Some(dir_pack) = PartialStoragePack::new(storage_path)
        .with_partial_spec(partial_spec)
        .with_partial_spec(src_spec)
        .ready()
    {
        Ok(Arc::new(dir_pack))
    } else {
        Err(PackError::Other(Some(eco_format!(
            "Cannot infer pack path from local URL: {url}",
        ))))
    }
}

fn pack_from_universe_url(
    url: &Url,
    src_spec: Option<&PackageSpec>,
) -> Result<Arc<dyn Pack>, PackError> {
    let (part_namespace, part_package, part_version) = match url.path().strip_prefix('@') {
        Some(rest) => {
            let mut parts = rest.splitn(3, '/');
            (
                parts.next().map(|s| s.to_string()),
                parts.next().map(|s| s.to_string()),
                parts.next().map(|s| s.to_string()),
            )
        }
        None => (None, None, None),
    };
    let (src_namespace, src_package, src_version) = if let Some(spec) = src_spec {
        (
            Some(spec.namespace.to_string()),
            Some(spec.name.to_string()),
            Some(spec.version.to_string()),
        )
    } else {
        (None, None, None)
    };
    let spec = PackageSpec {
        namespace: part_namespace
            .or(src_namespace)
            .ok_or_else(|| {
                PackError::Other(Some(eco_format!(
                    "Missing namespace, cannot infer from given URLs"
                )))
            })?
            .into(),
        name: part_package
            .or(src_package)
            .ok_or_else(|| {
                PackError::Other(Some(eco_format!(
                    "Missing package name, cannot infer from given URLs"
                )))
            })?
            .into(),
        version: part_version
            .or(src_version)
            .ok_or_else(|| {
                PackError::Other(Some(eco_format!(
                    "Missing version, cannot infer from given URLs"
                )))
            })?
            .parse()
            .map_err(|e| {
                PackError::Other(Some(eco_format!("Invalid version in universe URL: {e}")))
            })?,
    };
    Ok(Arc::new(UniversePack::new(spec)))
}

fn parse_url(url: &str, src_spec: PartialPackageSpec) -> Result<Arc<dyn Pack>, PackError> {
    let url = Url::parse(url)
        .map_err(|e| PackError::Other(Some(format!("Invalid URL: {}", e).into())))?;
    let scheme = url.scheme();
    match scheme {
        "git" => Ok(Arc::new(GitClPack::new(
            DETACHED_NAMESPACE.into(),
            url.clone(),
        ))),
        // TODO: file can be .git
        "file" => Ok(Arc::new(DirPack::new(PathBuf::from(
            url.to_file_path().map_err(|_| {
                PackError::Other(Some(eco_format!(
                    "Invalid file URL `{url}`: cannot convert to file path"
                )))
            })?,
        )))),
        "http" | "https" => pack_from_http_url(&url),
        "ssh" => pack_from_ssh_url(&url),
        // Special schemes
        "data" | "cache" => pack_from_local_url(&url, src_spec),
        "universe" => todo!(),
        unknown => Err(PackError::Other(Some(eco_format!(
            "Unsupported URL scheme: {unknown}"
        )))),
    }
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
                pack.clone_from_pack(&mut GitClPack::new(DETACHED_NAMESPACE.into(), git))?;
            }
            PackKind::Local { path } => {
                pack.clone_from_pack(&mut DirPack::new(path))?;
            }
            PackKind::Universe { spec } => {
                pack.clone_from_pack(&mut UniversePack::new(spec))?;
            }
            PackKind::TarStream { data } => {
                pack.clone_from_pack(&mut TarballPack::new(std::io::Cursor::new(data)))?;
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
            Arc::new(TarballPack::new(std::io::Cursor::new(data)))
        }
        rest => parse_url(&args.source, Default::default())?,
    };
    let src_spec = src
        .read("typst.toml")
        .ok()
        .and_then(|file| {
            // file
            // PackageManifest
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).ok()?;
            toml::from_slice::<PackageManifest>(&buf)
                .ok()
                .map(|m| PartialPackageSpec {
                    namespace: None,
                    name: Some(m.package.name.to_string()),
                    version: Some(m.package.version.to_string()),
                })
        })
        .unwrap_or_default();
    let mut writing_to_stdout = false;
    let dst: Arc<dyn Pack> = match args.destination.as_str() {
        "-" => {
            writing_to_stdout = true;
            Arc::new(TarballPack::new(std::io::Cursor::new(vec![])))
        }
        rest => parse_url(&args.destination, src_spec)?,
    };

    if writing_to_stdout {
        log::set_max_level(log::LevelFilter::Warn);
    }

    info!("Copying {} to {}", args.source, args.destination);
    info!("Loading {} to memory...", args.source);
    let mut pack = MapPack::default();
    pack.clone_from_pack(Arc::make_mut(&mut src.clone()))?;
    // dst.clone_from_memory_pack(pack)?;

    Ok(())
}
