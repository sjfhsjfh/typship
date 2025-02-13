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

## TODO

- [x] Package init
- [x] Package validation
- [ ] ~~init with git~~
- [x] install to local
- [ ] package download got checkout
- [x] publish on Universe
- [x] exclude command
- [ ] i18n
- [ ] typship dev(create symlink to the developing version? auto check?)

- [ ] Better implementation investigation
