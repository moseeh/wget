# `cli` Module — Argument Parsing for wget-rs

This module provides the command-line interface for the `wget-rs` application. It defines the structure of supported flags and arguments using the [`clap`](https://docs.rs/clap) crate.

## Features

* Argument parsing using `#[derive(Parser)]`
* Manual validation via `Cli::validate()`
* Support for standard and advanced wget-like flags:

  * URLs and input files
  * Output filename (`-O`) and download directory (`-P`)
  * Background mode (`-B`)
  * Rate limiting (`--rate-limit`)
  * Website mirroring (`--mirror`) with additional filters:

    * File type reject list (`-R`)
    * Excluded directories (`-X`)
    * Offline link conversion (`--convert-links`)

## Structure

* `cli/args.rs`: Defines the `Cli` struct and its fields using `clap`
* `cli/mod.rs`: Exports the `Cli` struct for external use

## How to Use

In `main.rs`:

```rust
use clap::Parser;
use crate::cli::Cli;

fn main() {
    let args = Cli::parse();
    args.validate().expect("Invalid arguments");

    // Application logic follows
}
```

## Notes

* `Cli::parse()` comes from the `Parser` trait, so be sure to `use clap::Parser`
* `Cli::validate()` provides additional checks not enforced by `clap`, like ensuring paths exist or validating rate formats

This module aims to keep CLI parsing declarative, clean, and aligned with real-world usage of wget.
