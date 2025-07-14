use crate::http::client::DownloadError;
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Reads URLs from a given file (one per line).
/// Ignores empty lines and lines starting with `#` (comments).
pub async fn read_urls_from_file(path: &Path) -> Result<Vec<String>, DownloadError> {
    let file = File::open(path).await.map_err(|e| DownloadError {
        message: format!("Failed to open input file {:?}: {}", path, e),
    })?;

    let reader = BufReader::new(file);
    let mut lines = reader.lines();
    let mut urls = Vec::new();

    while let Some(line) = lines.next_line().await.map_err(|e| DownloadError {
        message: format!("Failed to read line from file {:?}: {}", path, e),
    })? {
        let trimmed = line.trim();

        // Skip empty lines and comments
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Basic URL validation - check if it starts with http:// or https://
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            urls.push(trimmed.to_string());
        } else {
            eprintln!("Warning: Skipping invalid URL: {}", trimmed);
        }
    }

    if urls.is_empty() {
        return Err(DownloadError {
            message: format!("No valid URLs found in file {:?}", path),
        });
    }

    Ok(urls)
}
