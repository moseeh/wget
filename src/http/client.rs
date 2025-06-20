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
        println!("sending request, awaiting response...");

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
    
}
