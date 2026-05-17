# `cli` module

Clap-based CLI parsing and post-parse validation.

## Files

- `args.rs` — defines the `Cli` struct (one field per flag) and `Cli::validate()`.
- `mod.rs` — re-exports `Cli`.

## Supported flags

| Field | Flag |
|-------|------|
| `urls` | positional URLs |
| `output` | `-O`, `--output` |
| `directory_prefix` | `-P`, `--directory-prefix` |
| `background` | `-B`, `--background` |
| `rate_limit` | `--rate-limit` |
| `input_file` | `-i`, `--input-file` |
| `mirror` | `--mirror` |
| `reject_suffixes` | `-R`, `--reject-suffixes` |
| `exclude_dirs` | `-X`, `--exclude-dirs` |
| `convert_links` | `--convert-links` |
| `continue_download` | `-c`, `--continue-download` |
| `timeout` | `--timeout` (default `30`) |
| `user_agent` | `--user-agent` |

`-O` and `-P` are mutually exclusive (enforced by clap).

## Validation

`Cli::validate()` enforces what clap can't:

- At least one of `urls` or `--input-file` must be supplied.
- `--input-file` must exist.
- `-P` directory must exist.
- `--rate-limit` value must parse as `<num>`, `<num>k`, or `<num>M`.
- `--mirror` requires exactly one URL.
- `-R`, `-X`, `--convert-links` are only valid in combination with `--mirror`.

## Usage

```rust
use clap::Parser;
use crate::cli::Cli;

fn main() {
    let args = Cli::parse();
    if let Err(e) = args.validate() {
        eprintln!("Argument error: {}", e);
        std::process::exit(1);
    }
    // ...
}
```
