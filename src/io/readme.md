# `io` Module — Input/Output Operations for wget-rs

This module handles input/output operations for the `wget-rs` application, primarily focused on reading URLs from input files for the `-i` flag functionality.

## Features

* Asynchronous file reading using [`tokio`](https://docs.rs/tokio)
* URL parsing and validation from text files
* Comment and empty line handling
* Error handling with detailed error messages
* Support for standard text file formats

## Structure

* `io/input.rs`: File reading and URL parsing functionality
* `io/mod.rs`: Exports the input reading functions

## Core Components

### `read_urls_from_file(path)`
The main function for reading URLs from input files:
* **Input**: `&Path` - Path to the input file
* **Output**: `Result<Vec<String>, DownloadError>` - List of valid URLs or error
* **Features**:
  - Reads files line by line asynchronously
  - Skips empty lines and comments (lines starting with `#`)
  - Validates URLs (must start with `http://` or `https://`)
  - Provides warnings for invalid URLs
  - Returns error if no valid URLs found

## How to Use

```rust
use crate::io::read_urls_from_file;
use std::path::Path;

#[tokio::main]
async fn main() {
    let file_path = Path::new("downloads.txt");

    match read_urls_from_file(file_path).await {
        Ok(urls) => {
            println!("Found {} URLs:", urls.len());
            for url in urls {
                println!("  {}", url);
            }
        }
        Err(e) => eprintln!("Failed to read URLs: {}", e),
    }
}
```

## Input File Format

The input file should contain one URL per line:

```text
# This is a comment and will be ignored
https://example.com/file1.zip
https://example.com/file2.tar.gz

# Empty lines are also ignored
https://example.com/file3.pdf
```

## URL Validation

* **Valid**: URLs starting with `http://` or `https://`
* **Invalid**: Relative URLs, FTP URLs, or malformed URLs
* **Behavior**: Invalid URLs are skipped with a warning message

## Notes

* Uses `tokio::fs::File` and `BufReader` for efficient async file reading
* Integrates with the HTTP module's `DownloadError` type
* Designed specifically for the `-i` flag functionality
* Memory efficient - processes files line by line
* Robust error handling for file access and parsing issues

This module enables wget-rs to process multiple URLs from input files asynchronously.
