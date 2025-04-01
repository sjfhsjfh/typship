use std::str::FromStr;

use tinymist_package::{CloneIntoPack, GitClPack, MapPack, PackExt, PackageSpec, UniversePack};

#[test]
fn universe_test() {
    let spec = PackageSpec::from_str("@preview/zebraw:0.4.4").unwrap();

    let mut src = UniversePack::new(spec);

    let mut dst = MapPack::default();
    dst.clone_into_pack(&mut src.filter(|s| s == "typst.toml"))
        .expect("download");

    assert!(dst.files.len() == 1, "downloaded package is bad {dst:?}");
}

#[test]
fn gitcl_test() {
    let mut src = GitClPack::new("local".into(), "https://github.com/hongjr03/typst-zebraw");

    let mut dst = MapPack::default();
    dst.clone_into_pack(&mut src.filter(|s| s == "typst.toml"))
        .expect("clone");

    assert!(dst.files.len() == 1, "downloaded package is bad {dst:?}");
}
