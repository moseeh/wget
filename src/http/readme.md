# `http` Module — HTTP Client for wget-rs

This module provides HTTP client functionality for the `wget-rs` application. It handles HTTP/HTTPS requests, downloads, and response processing using the [`reqwest`](https://docs.rs/reqwest) crate.

## Features

* Asynchronous HTTP client with connection pooling
* Progress bar integration during downloads
* Stream-based downloading for memory efficiency
* Error handling with custom `DownloadError` type
* Support for both silent and verbose request modes
* Content-length detection and progress tracking

## Structure

* `http/client.rs`: Main HTTP client implementation with `HttpClient` struct
* `http/mod.rs`: Exports the HTTP client functionality

## Core Components

### `HttpClient`
The main HTTP client struct that provides:
* `new()`: Creates a new HTTP client instance
* `download(url)`: Downloads a URL with progress display
* `download_silent(url)`: Downloads a URL without progress messages
* `download_to_file(url, path)`: Downloads directly to a specified file

### `DownloadError`
Custom error type for HTTP-related failures:
* Network connection errors
* HTTP status errors (4xx, 5xx)
* File I/O errors during download

## How to Use

```rust
use crate::http::HttpClient;

#[tokio::main]
async fn main() {
    let client = HttpClient::new();

    // Download with progress display
    match client.download("https://example.com/file.zip").await {
        Ok(response) => {
            // Process response stream
        }
        Err(e) => eprintln!("Download failed: {}", e),
    }

    // Download directly to file
    let path = std::path::PathBuf::from("output.zip");
    client.download_to_file("https://example.com/file.zip", &path).await?;
}
```

## Notes

* Uses `reqwest::Client` internally for HTTP operations
* Integrates with `indicatif` for progress bar display
* Supports streaming downloads to handle large files efficiently
* Automatically handles redirects and common HTTP headers
* Thread-safe and can be cloned for concurrent use

This module provides the core networking functionality that powers wget-rs downloads.
