use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Response};
use std::error::Error;
use std::fmt;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct DownloadError {
    pub message: String,
}

impl fmt::Display for DownloadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Download error: {}", self.message)
    }
}

impl Error for DownloadError {}

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        HttpClient {
            client: Client::new(),
        }
    }
    pub async fn download(&self, url: &str) -> Result<Response, DownloadError> {
        print!("sending request, awaiting response... ");

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| DownloadError {
                message: format!("Failed to send request: {}", e),
            })?;

        let status = response.status();
        println!(
            "status {} {}",
            status.as_u16(),
            status.canonical_reason().unwrap_or("")
        );

        if !status.is_success() {
            return Err(DownloadError {
                message: format!("HTTP error: {}", status),
            });
        }
        Ok(response)
    }

    pub async fn download_to_file(
        &self,
        url: &str,
        file_path: &std::path::Path,
    ) -> Result<u64, DownloadError> {
        let response = self.download(url).await?;
        let content_length = response.content_length().unwrap_or(0);
        if content_length > 0 {
            println!(
                "content size: {} [~{:.2}MB]",
                content_length,
                content_length as f64 / 1_048_576.0
            );
        }
        println!("saving file to: ./{}", file_path.display());

        let progress_bar = ProgressBar::new(content_length);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template("{bytes}/{total_bytes} [{bar:80.cyan/red}] {eta_precise}")
                .unwrap()
                .progress_chars("#>-"),
        );

        let mut file = File::create(file_path).await.map_err(|e| DownloadError {
            message: format!("Failed to create file: {}", e),
        })?;

        let mut stream = response.bytes_stream();
        let mut download = 0u64;

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| DownloadError {
                message: format!("Failed to read chunk: {}", e),
            })?;
            file.write_all(&chunk).await.map_err(|e| DownloadError {
                message: format!("Failed to write chunk: {}", e),
            })?;
            download += chunk.len() as u64;
            progress_bar.set_position(download);
        }
        file.flush().await.map_err(|e| DownloadError {
            message: format!("Failed to flush file: {}", e),
        })?;

        println!();
        println!("Downloaded [{}]", url);
        Ok(download)
    }
}
