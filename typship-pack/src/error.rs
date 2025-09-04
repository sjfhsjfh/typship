use std::{
    fmt::{self, Display, Formatter},
    io,
};

use ecow::{EcoString, eco_format};
use typst_syntax::package::{PackageSpec, PackageVersion};

pub fn unsupported() -> io::Error {
    io::Error::new(io::ErrorKind::Unsupported, "unsupported operation")
}

pub fn malform(e: io::Error) -> PackError {
    PackError::MalformedArchive(Some(eco_format!("{e:?}")))
}

pub fn other_io(e: impl Display) -> io::Error {
    io::Error::other(e.to_string())
}

pub fn other(e: impl Display) -> PackError {
    PackError::Other(Some(eco_format!("{e}")))
}

/// An error that occurred while trying to load a package.
///
/// Some variants have an optional string can give more details, if available.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum PackError {
    /// The specified package does not exist.
    NotFound(PackageSpec),
    /// The specified package found, but the version does not exist.
    VersionNotFound(PackageSpec, PackageVersion),
    /// Failed to retrieve the package through the network.
    NetworkFailed(Option<EcoString>),
    /// The package archive was malformed.
    MalformedArchive(Option<EcoString>),
    /// Another error.
    Other(Option<EcoString>),
}

impl std::error::Error for PackError {}

impl Display for PackError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Self::NotFound(spec) => {
                write!(f, "package not found (searched for {spec})",)
            }
            Self::VersionNotFound(spec, latest) => {
                write!(
                    f,
                    "package found, but version {} does not exist (latest is {})",
                    spec.version, latest,
                )
            }
            Self::NetworkFailed(Some(err)) => {
                write!(f, "failed to download package ({err})")
            }
            Self::NetworkFailed(None) => f.pad("failed to download package"),
            Self::MalformedArchive(Some(err)) => {
                write!(f, "failed to decompress package ({err})")
            }
            Self::MalformedArchive(None) => {
                f.pad("failed to decompress package (archive malformed)")
            }
            Self::Other(Some(err)) => write!(f, "failed to load package ({err})"),
            Self::Other(None) => f.pad("failed to load package"),
        }
    }
}

impl From<PackError> for EcoString {
    fn from(err: PackError) -> Self {
        eco_format!("{err}")
    }
}
/// A result type with a package-related error.
pub type PackResult<T> = Result<T, PackError>;
