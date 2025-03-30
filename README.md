# typship

![Crates.io Version](https://img.shields.io/crates/v/typship?style=for-the-badge)
![Crates.io Total Downloads](https://img.shields.io/crates/d/typship?style=for-the-badge)

---

A tool for [Typst](https://typst.app/) package development and publishing.

_The name `typship` is a portmanteau of **Typst** and **spaceship**, since it sends packages to the **[universe](https://typst.app/universe/)**._

## Installation

```sh
cargo install typship
```

## Notice

To use `publish universe`, you will need to generate a token (fine-grained) with the following permissions to your fork of the packages repository:

- _Read_ access to _metadata_

- _Read and write_ access to _contents_

Here's GitHub's [documentation](https://docs.github.com/en/github/authenticating-to-github/creating-a-personal-access-token) on how to create a personal access token.

## Usage

```sh
typship help
```

### TL;DR

To init a new package, simply run (this would start an interactive prompt):

```sh
typship init
```

To publish a package, run (then follow the instructions):

```sh
typship publish
```

Download a package to `@local`:

```sh
typship download <package-repo>
```

Download a package to `@my-packages`:

```sh
typship download <package-repo> -n my-packages
```

Copy a package:

```sh
# copy from github
typship cp https://github.com/hongjr03/typst-zebraw @preview
# copy from http url
typship cp https://github.com/touying-typ/touying/archive/refs/tags/0.6.1.tar.gz @local
# copy a package into other namespace
typship cp @preview/zebraw:0.4.4 @local
# rename a package
typship cp @preview/zebraw:0.4.4 @local/zebraw-derived
# copy from universe (download if not exist)
typship cp @preview/zebraw:0.4.4 @local
# copy from universe (always fetch from remote)
typship cp universe:@preview/zebraw:0.4.4 @local
# forbid network access
typship cp --offline @preview/zebraw:0.4.4 @local
```

## TODO

- [ ] i18n
