use anyhow::{anyhow, bail, Result};
use clap::{Arg, Command};
use dialoguer::{Confirm, Input, MultiSelect};
use regex::Regex;
use std::{
    fs::{self, File},
    path::Path,
    str::FromStr,
};
use typst_syntax::package::{
    PackageInfo, PackageManifest, PackageVersion, ToolInfo, UnknownFields, VersionBound,
};
use url::Url;

use crate::{
    model::{CATEGORIES, DISCIPLINES},
    utils::{read_manifest, write_manifest},
};

pub fn cmd() -> Command {
    Command::new("init").about("Initialize a new package").arg(
        Arg::new("name")
            .required(false)
            .help("The package name (optional)"),
    )
}

pub fn init(
    package_dir: &Path,
    provided_name: Option<&str>,
) -> Result<()> {
    if read_manifest(package_dir).is_ok() {
        if !Confirm::new()
            .with_prompt("A package manifest already exists. Overwrite?")
            .default(false)
            .interact()?
        {
            return Ok(());
        }
    }

    println!("Initializing a new package...");

    let name_re = Regex::new(r"^[a-zA-Z_-][a-zA-Z0-9_-]*$").unwrap();
    let name = if let Some(name) = provided_name {
        if !name_re.is_match(name) {
            bail!("Invalid package name")
        }
        println!("Package name: {}", name);
        name.to_string()
    } else {
        let name = Input::new()
            .with_prompt("Enter the package name")
            .validate_with(|input: &String| -> Result<()> {
                if name_re.is_match(input) {
                    Ok(())
                } else {
                    Err(anyhow!("Invalid package name"))
                }
            });
        let name = if let Some(default_name) =
            fs::canonicalize(".")?.file_name().and_then(|s| s.to_str())
        {
            name.default(default_name.into())
        } else {
            name
        };
        name.interact_text()?
    };

    let author: String = Input::new()
        .with_prompt("Enter the package author")
        .default(whoami::username())
        .interact_text()?;

    let version: String = Input::new()
        .with_prompt("Enter the package version")
        .default("0.1.0".into())
        .validate_with(|input: &String| -> Result<()> {
            PackageVersion::from_str(input)
                .map(|_| ())
                .map_err(|msg| anyhow!(msg))
        })
        .interact_text()?;
    let version = PackageVersion::from_str(&version).unwrap();

    let categories = MultiSelect::new()
        .with_prompt("Choose the package category")
        .items(&CATEGORIES)
        .interact()
        .unwrap()
        .into_iter()
        .map(|i| CATEGORIES[i].into())
        .collect();

    let disciplines = MultiSelect::new()
        .with_prompt("Choose the package discipline")
        .items(&DISCIPLINES)
        .interact()
        .unwrap()
        .into_iter()
        .map(|i| DISCIPLINES[i].into())
        .collect();

    let entrypoint: String = Input::new()
        .with_prompt("Enter the package entrypoint")
        .default(
            Path::new("src")
                .join(Path::new("lib.typ"))
                .to_string_lossy()
                .into(),
        )
        .validate_with(|input: &String| -> Result<()> {
            if !input.ends_with(".typ") {
                bail!("Entrypoint must end with '.typ'")
            }
            Ok(())
        })
        .interact_text()?;

    let description: String = Input::new()
        .with_prompt("Enter the package description")
        .allow_empty(true)
        .interact_text()?;

    let keywords: String = Input::new()
        .with_prompt("Enter the package keywords(separated by comma)")
        .allow_empty(true)
        .interact_text()?;
    let keywords = keywords.split(',').map(|s| s.trim().into()).collect();

    let homepage: String = Input::new()
        .with_prompt("Enter the package homepage URL")
        .allow_empty(true)
        .default("".into())
        .validate_with(|input: &String| -> Result<()> {
            if input.is_empty() {
                Ok(())
            } else {
                let url = Url::parse(input)?;
                if url.scheme() == "http" || url.scheme() == "https" {
                    Ok(())
                } else {
                    bail!("Invalid URL scheme")
                }
            }
        })
        .interact_text()?;
    let homepage = if homepage.is_empty() {
        None
    } else {
        Some(homepage.into())
    };

    let repository: String = Input::new()
        .with_prompt("Enter the package repository URL")
        .allow_empty(true)
        .default("".into())
        .validate_with(|input: &String| -> Result<()> {
            if input.is_empty() {
                Ok(())
            } else {
                let url = Url::parse(input)?;
                if url.scheme() == "http" || url.scheme() == "https" || url.scheme() == "git" {
                    Ok(())
                } else {
                    bail!("Invalid URL scheme")
                }
            }
        })
        .interact_text()?;
    let repository = if repository.is_empty() {
        None
    } else {
        Some(repository.into())
    };

    let compiler: String = Input::new()
        .with_prompt("Enter compiler version")
        .allow_empty(true)
        .default("".into())
        .validate_with(|input: &String| -> Result<()> {
            if input.is_empty() {
                Ok(())
            } else {
                VersionBound::from_str(input)
                    .map(|_| ())
                    .map_err(|msg| anyhow!(msg))
            }
        })
        .interact_text()?;
    let compiler = VersionBound::from_str(&compiler).ok();

    let package_info = PackageInfo {
        name: name.clone().into(),
        authors: vec![author.into()],
        version,
        categories,
        disciplines,
        description: Some(description.into()),
        keywords,
        entrypoint: entrypoint.into(),
        homepage,
        repository,
        compiler,
        unknown_fields: UnknownFields::default(),
        // TODO: Add the following fields
        exclude: vec![],
        license: None,
    };

    let manifest = PackageManifest {
        package: package_info,
        tool: ToolInfo::default(),
        unknown_fields: UnknownFields::default(),
        // TODO: Add the following fields
        template: None,
    };

    write_manifest(package_dir, &manifest)?;

    // TODO: generate other files: entrypoint, readme(ask) ...

    let entrypoint = Path::new(manifest.package.entrypoint.as_ref());
    if let Some(parent) = entrypoint.parent() {
        fs::create_dir_all(parent)?;
    }
    let _ = File::create(entrypoint)?;

    Ok(())
}
