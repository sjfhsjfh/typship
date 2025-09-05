use std::{fs::File, io::Write, path::PathBuf};

use crate::error::{other, other_io};

use super::*;

/// A package in the directory.
#[derive(Clone)]
pub struct DirPack<P> {
    /// The patch storing the package.
    pub path: P,
}

impl<P: AsRef<Path>> DirPack<P> {
    /// Creates a new `DirPack` instance.
    pub fn new(path: P) -> Self {
        Self { path }
    }
}

impl<P: AsRef<Path>> fmt::Debug for DirPack<P> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "DirPack({})", self.path.as_ref().display())
    }
}

impl<P: AsRef<Path>> PackFs for DirPack<P> {
    fn read_all(
        &mut self,
        f: &mut (dyn FnMut(&str, PackFile) -> PackResult<()> + Send + Sync),
    ) -> PackResult<()> {
        self.filter(|_| true).read_all(f)
    }
}

impl<P: AsRef<Path>> Pack for DirPack<P> {}
impl<P: AsRef<Path>> PackExt for DirPack<P> {
    fn filter(&mut self, f: impl Fn(&str) -> bool + Send + Sync) -> impl Pack
    where
        Self: std::marker::Sized,
    {
        FilterDirPack {
            path: &self.path,
            f,
        }
    }
}

impl<P: AsRef<Path>> CloneFromPack for DirPack<P> {
    fn clone_from_pack(&mut self, pack: &mut impl PackFs) -> std::io::Result<()> {
        let base = self.path.as_ref();
        pack.read_all(&mut |path, file| {
            let path = base.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).map_err(other)?;
            let mut dst = std::fs::File::create(path).map_err(other)?;
            match file {
                PackFile::Read(mut reader) => {
                    std::io::copy(&mut reader, &mut dst).map_err(other)?;
                }
                PackFile::Data(data) => {
                    dst.write_all(&data.into_inner()).map_err(other)?;
                }
            }

            Ok(())
        })
        .map_err(other_io)?;
        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct PartialPackageSpec {
    pub namespace: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
}

impl PartialPackageSpec {
    pub fn or(self, other: PartialPackageSpec) -> Self {
        Self {
            namespace: self.namespace.or(other.namespace),
            name: self.name.or(other.name),
            version: self.version.or(other.version),
        }
    }

    pub fn with_namespace(mut self, namespace: Option<String>) -> Self {
        self.namespace = self.namespace.or(namespace);
        self
    }

    pub fn with_name(mut self, name: Option<String>) -> Self {
        self.name = self.name.or(name);
        self
    }

    pub fn with_version(mut self, version: Option<String>) -> Self {
        self.version = self.version.or(version);
        self
    }
}

pub struct PartialStoragePack<P> {
    pub storage_root: P,
    pub spec: PartialPackageSpec,
}

impl<P> PartialStoragePack<P> {
    pub fn with_namespace(mut self, namespace: Option<String>) -> Self {
        self.spec = self.spec.with_namespace(namespace);
        self
    }

    pub fn with_package(mut self, package: Option<String>) -> Self {
        self.spec = self.spec.with_name(package);
        self
    }

    pub fn with_version(mut self, version: Option<String>) -> Self {
        self.spec = self.spec.with_version(version);
        self
    }

    pub fn with_partial_spec(mut self, spec: PartialPackageSpec) -> Self {
        self.spec = self.spec.or(spec);
        self
    }

    pub fn with_spec(self, spec: Option<&PackageSpec>) -> Self {
        self.with_package(spec.map(|s| s.name.to_string()))
            .with_version(spec.map(|s| s.version.to_string()))
    }
}
impl<P: AsRef<Path>> PartialStoragePack<P> {
    /// Creates a new `PartialStoragePack` instance.
    pub fn new(storage_root: P) -> Self {
        Self {
            storage_root,
            spec: Default::default(),
        }
    }

    pub fn ready(self) -> Option<DirPack<PathBuf>> {
        let mut path = self.storage_root.as_ref().to_path_buf();
        path.push(self.spec.namespace?);
        path.push(self.spec.name?);
        path.push(self.spec.version?);
        Some(DirPack::new(path))
    }
}

impl<S: AsRef<Path>> fmt::Debug for PartialStoragePack<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "PartialStoragePack({:?}, {:?})",
            self.storage_root.as_ref(),
            self.spec
        )
    }
}

struct FilterDirPack<'a, P, F> {
    /// The patch storing the package.
    pub path: &'a P,
    /// The filter function.
    pub f: F,
}

impl<S: AsRef<Path>, F> fmt::Debug for FilterDirPack<'_, S, F> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "FilterDirPack({:?}, ..)", self.path.as_ref())
    }
}
impl<Src: AsRef<Path>, F: Fn(&str) -> bool + Send + Sync> PackFs for FilterDirPack<'_, Src, F> {
    fn read_all(
        &mut self,
        f: &mut (dyn FnMut(&str, PackFile) -> PackResult<()> + Send + Sync),
    ) -> PackResult<()> {
        let w = walkdir::WalkDir::new(self.path.as_ref())
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !e.file_name().to_string_lossy().starts_with('.'))
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file());

        for entry in w {
            let path = entry.path();
            let rel_path = path.strip_prefix(self.path.as_ref()).map_err(other)?;

            let file_path = rel_path.to_string_lossy();
            if !(self.f)(&file_path) {
                continue;
            }

            let pack_file = PackFile::Read(Box::new(File::open(path).map_err(other)?));
            f(&file_path, pack_file)?;
        }

        Ok(())
    }
}

impl<Src: AsRef<Path>, F: Fn(&str) -> bool + Send + Sync> Pack for FilterDirPack<'_, Src, F> {}
