use anyhow::Result;
use glob::Pattern;
use ignore::{Walk, WalkBuilder};
use std::path::Path;

use super::read_manifest;

/// Only `.typstignore`
pub fn walker_publish(root: &Path) -> Walk {
    WalkBuilder::new(root)
        .standard_filters(true)
        .add_custom_ignore_filename(".typstignore")
        .build()
}

/// `.typstignore` and `package.excludes`
pub fn walker_install(
    root: &Path,
) -> Result<Vec<std::result::Result<ignore::DirEntry, ignore::Error>>> {
    let excludes = read_manifest(root)?
        .package
        .exclude
        .into_iter()
        .map(|s| Pattern::new(&s))
        .collect::<Vec<_>>();
    let mut ok_excludes = vec![];
    for pat in &excludes {
        match pat {
            Ok(p) => ok_excludes.push(p),
            Err(e) => {
                return Err(anyhow::anyhow!("Invalid pattern: {}", e));
            }
        }
    }
    Ok(WalkBuilder::new(root)
        .standard_filters(true)
        .add_custom_ignore_filename(".typstignore")
        .build()
        .filter(|entry| {
            if let Ok(path) = entry {
                return !ok_excludes
                    .iter()
                    .any(|p| p.matches_path(path.path().strip_prefix(root).unwrap()));
            }
            false
        })
        .map(|entry| {
            if let Ok(path) = entry {
                return Ok(path);
            }
            Err(entry.unwrap_err())
        })
        .collect())
}
