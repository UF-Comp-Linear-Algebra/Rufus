# Rufus

A general-purpose tool for detecting cheating in autogradedGradescope code submissions by cross-referencing emissions of values that ought to be unique class-wide.

## Rust Installation

To install Rufus, you need to have Rust installed on your system. If you haven't installed Rust yet, you can do so by following the instructions at [rustup.rs](https://www.rust-lang.org/tools/install).

## Running Rufus

To run Rufus, you can use the following command:

```bash
cargo run --release -- [SUBCOMMAND] [ARGS]
```

For example, to hunt for cheating submissions, you can use:

```bash
cargo run --release -- hunt [FILES] --show-emissions
```

You can also run Rufus with the `--help` flag to see a list of available subcommands and options:

```bash
cargo run --release -- --help
```
