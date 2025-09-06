use super::*;

/// The default Typst registry.
const DEFAULT_REGISTRY: &str = "https://packages.typst.org";

/// A package in the universe registry.
#[derive(Debug, Clone)]
pub struct UniversePack {
    /// The package specifier.
    pub specifier: PackageSpec,
}

impl UniversePack {
    /// Creates a new `UniversePack` instance.
    pub fn new(specifier: PackageSpec) -> Self {
        Self { specifier }
    }
}

impl PackFs for UniversePack {
    fn read_all(
        &mut self,
        f: &mut (dyn FnMut(&str, PackFile) -> PackResult<()> + Send + Sync),
    ) -> PackResult<()> {
        let spec = &self.specifier;
        assert_eq!(spec.namespace, "preview");

        let url = format!(
            "{DEFAULT_REGISTRY}/preview/{}-{}.tar.gz",
            spec.name, spec.version
        );

        HttpPack::new(self.specifier.clone(), url).read_all(f)
    }
}

impl Pack for UniversePack {}
impl PackExt for UniversePack {}
