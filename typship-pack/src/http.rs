use ecow::eco_format;
use reqwest::{Certificate, blocking::Response};

use super::*;

fn threaded_http<T: Send + Sync>(
    url: &str,
    cert_path: Option<&Path>,
    f: impl FnOnce(Result<Response, reqwest::Error>) -> T + Send + Sync,
) -> Option<T> {
    std::thread::scope(|s| {
        s.spawn(move || {
            let client_builder = reqwest::blocking::Client::builder();

            let client = if let Some(cert_path) = cert_path {
                let cert = std::fs::read(cert_path)
                    .ok()
                    .and_then(|buf| Certificate::from_pem(&buf).ok());
                if let Some(cert) = cert {
                    client_builder.add_root_certificate(cert).build().unwrap()
                } else {
                    client_builder.build().unwrap()
                }
            } else {
                client_builder.build().unwrap()
            };

            f(client.get(url).send())
        })
        .join()
        .ok()
    })
}

/// A package in the remote http.
#[derive(Clone)]
pub struct HttpPack<S> {
    /// The package specifier.
    pub specifier: PackageSpec,
    /// The url of the package.
    pub url: S,
}

impl<S: AsRef<str>> HttpPack<S> {
    /// Creates a new `HttpPack` instance.
    pub fn new(specifier: PackageSpec, url: S) -> Self {
        Self { specifier, url }
    }
}

impl<S: AsRef<str>> fmt::Debug for HttpPack<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HttpPack({})", self.url.as_ref())
    }
}

impl<S: AsRef<str>> PackFs for HttpPack<S> {
    fn read_all(
        &mut self,
        f: &mut (dyn FnMut(&str, PackFile) -> PackResult<()> + Send + Sync),
    ) -> PackResult<()> {
        let spec = &self.specifier;
        let url = self.url.as_ref();
        threaded_http(url, None, |resp| {
            let reader = match resp.and_then(|r| r.error_for_status()) {
                Ok(response) => response,
                Err(err) if matches!(err.status().map(|s| s.as_u16()), Some(404)) => {
                    return Err(PackError::NotFound(spec.clone()));
                }
                Err(err) => return Err(PackError::NetworkFailed(Some(eco_format!("{err}")))),
            };

            let decompressed = flate2::read::GzDecoder::new(reader);
            let mut tarbar = TarballPack::new(decompressed);

            // .unpack(package_dir)
            // .map_err(|err| {
            //     std::fs::remove_dir_all(package_dir).ok();
            //     PackError::MalformedArchive(Some(eco_format!("{err}")))
            // })

            tarbar.read_all(f)
        })
        .ok_or_else(|| PackError::Other(Some(eco_format!("cannot spawn http thread"))))?
    }
}

impl<S: AsRef<str>> Pack for HttpPack<S> {}
impl<P: AsRef<str>> PackExt for HttpPack<P> {}
