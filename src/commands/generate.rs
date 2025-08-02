use std::path::Path;

use anyhow::Context;
use clap::Parser;

const LONG_ABOUT: &str = "Generate CI that triggers typship to publish the package automatically.";

#[derive(Parser)]
#[command(long_about = LONG_ABOUT)]
/// Publish the package to a certain registry
pub struct GenerateArgs {
    #[arg(long)]
    /// The forked repository from typst/packages.
    pub source: Option<String>,
    #[arg(long)]
    /// The path to install the package in the source repository
    pub destination: Option<String>,
}

pub fn generate(_current_dir: &Path, args: &GenerateArgs) -> anyhow::Result<()> {
    let source = args.source.as_deref().unwrap_or("Myriad-Dreamin/packages");
    let destination = args.destination.as_deref().unwrap_or("packages/preview");

    // .github/workflows/Release.yml
    // This function will handle the generation of the project files.
    println!("Generating project files...");
    std::fs::create_dir_all(".github/workflows").context("Failed to create directory")?;
    std::fs::write(
        ".github/workflows/releast-typst.yml",
        format!(
            r##"
# This workflow publishes a package to the Typst Package Registry.
name: Publish Typst Package

on:
  workflow_dispatch:
    inputs:
      source:
        # the repository to which to push the release version
        # usually a fork of typst/packages (https://github.com/typst/packages/)
        # that you have push privileges to
        description: Source repository to publish the package
        required: true
        default: {source:?}
        type: string
      # the path within that repo where the "<name>/<version>" directory should be put
      # for the Typst package registry, keep this as is
      destination:
        description: Destination path to install the package
        required: true
        default: {destination:?}
        type: string

jobs:
  publish:
    runs-on: ubuntu-latest
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4
      - name: Pull packages
        uses: actions/checkout@v4
        with:
          # Repository name with owner. For example, actions/checkout
          # Default: ${{ github.repository }}
          repository: ${{ inputs.source }}
          token: ${{ secrets.REGISTRY_TOKEN }}
          # Relative path under $GITHUB_WORKSPACE to place the repository
          path: "typst-packages"
      - name: Make Pull Request
        run: typship host --source ${{ inputs.source }} --destination ${{ inputs.destination }}
"##
        ),
    )
    .context("Failed to write Release.yml")?;

    Ok(())
}
