# typship

A tool for Typst package development and publishing.

## Installation

```sh
cargo install typship
```

## Notice

To use `publish universe`, you will need to generate a token (fine-grained) with the following permissions to your fork of the packages repository:

- *Read* access to *metadata*

- *Read and write* access to *contents*

Here's GitHub's [documentation](https://docs.github.com/en/github/authenticating-to-github/creating-a-personal-access-token) on how to create a personal access token.

## Usage

```sh
typship help
```

### TL;DR

To init a new package, simply run:

```sh
typship init
```

To publish a package, run:

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

## TODO

- [ ] i18n
- [ ] typship dev(create symlink to the developing version? auto check?)
