# `io` module

Reads URL lists from disk for the `-i` flag.

## Files

- `input.rs` — `read_urls_from_file`.
- `mod.rs` — re-export.

## `read_urls_from_file(path: &Path) -> Result<Vec<String>, DownloadError>`

Reads `path` line by line via `tokio::io::BufReader`:

- Blank lines are skipped.
- Lines starting with `#` are treated as comments and skipped.
- Lines that don't start with `http://` or `https://` are skipped with a warning printed to stderr.
- Returns `DownloadError` if the file can't be opened or no valid URLs are found.

## Input file format

```
# Comments and blank lines are ignored
https://assets.01-edu.org/wgetDataSamples/Image_20MB.zip
https://assets.01-edu.org/wgetDataSamples/20MB.zip
https://assets.01-edu.org/wgetDataSamples/Image_10MB.zip
```

## Usage

```rust
use crate::io::read_urls_from_file;
use std::path::Path;

let urls = read_urls_from_file(Path::new("downloads.txt")).await?;
```

The returned `Vec<String>` is handed to `ConcurrentDownloadManager::download_urls`.
