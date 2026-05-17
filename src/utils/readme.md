# `utils` module

Small helpers shared across modules.

## Files

- `url.rs` — `extract_filename`.
- `path.rs` — `display_save_path`.
- `mod.rs` — re-exports.

## `url::extract_filename(url: &str) -> String`

Returns a filename suitable for saving a downloaded URL. Parses with the `url` crate, takes the last non-empty segment of the path, and falls back to `index.html` for URLs with no usable filename (e.g. `https://example.com/`).

```rust
extract_filename("https://example.com/path/file.zip") == "file.zip"
extract_filename("https://example.com/")              == "index.html"
extract_filename("https://example.com")               == "index.html"
```

## `path::display_save_path(p: &Path) -> String`

Formats a destination path the way `wget` prints it on the `saving file to:` line:

- Absolute paths are shown verbatim.
- Paths with a non-empty parent are shown verbatim.
- Bare filenames in the current directory get a `./` prefix.

```rust
display_save_path(Path::new("file.zip"))             == "./file.zip"
display_save_path(Path::new("/tmp/file.zip"))        == "/tmp/file.zip"
display_save_path(Path::new("sub/file.zip"))         == "sub/file.zip"
```

The old code hard-coded `./{path}`, which produced strings like `./~/Downloads/meme.jpg` whenever the user passed `-P`. This helper fixes that.
