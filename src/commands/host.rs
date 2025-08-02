use std::path::Path;

use clap::Parser;

const LONG_ABOUT: &str = "Host the package in a GitHub repository.";

#[derive(Parser)]
#[command(long_about = LONG_ABOUT)]
/// Host the package in a GitHub repository
pub struct HostArgs {
    #[arg(long)]
    /// The forked repository from typst/packages.
    pub source: Option<String>,
    #[arg(long)]
    /// The path to install the package in the source repository
    pub destination: Option<String>,
}

pub fn host(_current_dir: &Path, _args: &HostArgs) -> anyhow::Result<()> {
    println!(
        r###"
      - name: Put package to @preview namespace
        run: |
          mkdir -p typst-packages/${{ inputs.destination }}/example
          mv tests/fixtures/example typst-packages/${{ inputs.destination }}/example/0.1.1
      - name: Upload built package
        uses: actions/upload-artifact@v4
        with:
          name: preview-example-0.1.1
          path: typst-packages/${{ inputs.destination }}/example/0.1.1
      - name: Make PR
        uses: peter-evans/create-pull-request@v7
        with:
          token: ${{ secrets.REGISTRY_TOKEN }}
          path: typst-packages
          title: "@preview/example 0.1.1"
          body: |
            I am submitting
            * [ ]  a new package
            * [x]  an update for a package

            Description: New release.

            I have read and followed the submission guidelines and, in particular, I

            * [x]  selected [a name](https://github.com/typst/packages/blob/main/docs/manifest.md#naming-rules) that isn't the most obvious or canonical name for what the package does
            * [x]  added a [`typst.toml`](https://github.com/typst/packages/blob/main/docs/manifest.md#package-metadata) file with all required keys
            * [x]  added a [`README.md`](https://github.com/typst/packages/blob/main/docs/documentation.md) with documentation for my package
            * [x]  have chosen [a license](https://github.com/typst/packages/blob/main/docs/licensing.md) and added a `LICENSE` file or linked one in my `README.md`
            * [x]  tested my package locally on my system and it worked
            * [x]  [`exclude`d](https://github.com/typst/packages/blob/main/docs/tips.md#what-to-commit-what-to-exclude) PDFs or README images, if any, but not the LICENSE

            * [x]  ensured that my package is licensed such that users can use and distribute the contents of its template directory without restriction, after modifying them through normal use.
          commit-message: "build: bump @preview/example to 0.1.1"
          branch: preview/example/v0.1.1
          base: "main"
          draft: false
"###
    );

    Ok(())
}
