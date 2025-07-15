# `download` Module — Download Management for wget-rs

This module provides advanced download management capabilities for the `wget-rs` application, including concurrent downloads, progress tracking, and download coordination.

## Features

* Concurrent download management with configurable limits
* Multi-progress bar system for simultaneous downloads
* Two-phase download process (request → download)
* Download result tracking and error reporting
* Integration with HTTP client and progress systems
* Semaphore-based concurrency control

## Structure

* `download/client.rs`: Concurrent download manager and coordination
* `download/progress.rs`: Multi-progress bar management system
* `download/resume.rs`: Download resume functionality (placeholder)
* `download/mod.rs`: Exports download management components

## Core Components

### `ConcurrentDownloadManager`
Main download coordination system:
* `new(max_concurrent)`: Creates manager with concurrency limit
* `download_urls(urls, output_dir)`: Downloads multiple URLs concurrently
* Two-phase process: request collection → concurrent downloads
* Returns detailed `DownloadResult` for each URL

### `MultiProgressManager`
Progress bar management for concurrent downloads:
* `new()`: Creates new multi-progress manager
* `create_progress_bar(url, size)`: Creates progress bar for specific URL
* `finish_download(url, success)`: Marks download as complete/failed
* Handles multiple simultaneous progress displays

### `DownloadResult`
Result structure for individual downloads:
* `url`: The downloaded URL
* `file_path`: Where the file was saved
* `bytes_downloaded`: Total bytes transferred
* `success`: Whether download succeeded
* `error`: Error message if failed

## How to Use

```rust
use crate::download::ConcurrentDownloadManager;
use std::path::Path;

#[tokio::main]
async fn main() {
    let urls = vec![
        "https://example.com/file1.zip".to_string(),
        "https://example.com/file2.tar.gz".to_string(),
    ];

    let manager = ConcurrentDownloadManager::new(4); // Max 4 concurrent
    let output_dir = Some(Path::new("./downloads"));

    let results = manager.download_urls(urls, output_dir).await;

    for result in results {
        if result.success {
            println!("Downloaded: {}", result.url);
        } else {
            println!("Failed: {} - {}", result.url,
                result.error.unwrap_or_default());
        }
    }
}
```

## Two-Phase Download Process

### Phase 1: Request Collection
```text
Phase 1: Sending requests...
sending request to https://example.com/file1.zip, awaiting response...
status 200 OK for https://example.com/file1.zip
sending request to https://example.com/file2.zip, awaiting response...
status 200 OK for https://example.com/file2.zip
```

### Phase 2: Concurrent Downloads
```text
Phase 2: Starting 2 concurrent downloads...
[Multiple progress bars display simultaneously]
```

## Progress Display

* Individual progress bars for each download
* Filename-based prefixes for easy identification
* Real-time progress updates with ETA
* Clean completion/failure indicators
* Summary statistics after completion

## Notes

* Uses `tokio::sync::Semaphore` for concurrency control
* Integrates with `indicatif::MultiProgress` for display
* Separates request/response phase from download phase
* Prevents progress bar conflicts through phased approach
* Provides comprehensive error handling and reporting
* Memory efficient with streaming downloads

This module enables efficient concurrent downloading with excellent user experience.
