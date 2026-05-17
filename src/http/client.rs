use crate::rate::RateLimiter;
use crate::resume::ResumeHandler;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::{Client, Response};
use std::error::Error;
use std::fmt;
use std::time::Duration;
use tokio::fs::{File, OpenOptions};
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

#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    rate_limiter: Option<RateLimiter>,
}

impl HttpClient {
    pub fn new() -> Self {
        Self::with_config(None, None, 30)
    }

    pub fn with_config(
        rate_limit: Option<&str>,
        user_agent: Option<String>,
        timeout_secs: u64,
    ) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent(user_agent.as_deref().unwrap_or("wget-rs/0.1.0"))
            .build()
            .unwrap();

        let rate_limiter = rate_limit.and_then(|r| RateLimiter::new(r).ok());

        HttpClient { client, rate_limiter }
    }
    pub async fn download(&self, url: &str) -> Result<Response, DownloadError> {
        print!("sending request, awaiting response... ");
        use std::io::Write;
        let _ = std::io::stdout().flush();

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                println!();
                DownloadError {
                    message: format!("Failed to send request: {}", e),
                }
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

    /// Silent version of download that doesn't print status messages
    pub async fn download_silent(&self, url: &str) -> Result<Response, DownloadError> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| DownloadError {
                message: format!("Failed to send request: {}", e),
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(DownloadError {
                message: format!("HTTP error: {}", status),
            });
        }
        Ok(response)
    }

    pub async fn download_to_file_with_resume(
        &self,
        url: &str,
        file_path: &std::path::Path,
        resume: bool,
        show_progress: bool,
    ) -> Result<u64, DownloadError> {
        let mut rate_limiter = self.rate_limiter.clone();

        let resume_pos = if resume {
            ResumeHandler::get_resume_position(file_path).await
        } else {
            0
        };

        let response = if resume_pos > 0 {
            self.download_with_range(url, resume_pos).await?
        } else {
            self.download(url).await?
        };

        let content_length = response.content_length().unwrap_or(0) + resume_pos;
        if content_length > 0 {
            println!(
                "content size: {} [~{:.2}MB]",
                content_length,
                content_length as f64 / 1_048_576.0
            );
        }
        println!("saving file to: {}", crate::utils::path::display_save_path(file_path));

        let progress_bar = if show_progress {
            let pb = ProgressBar::new(content_length);
            pb.set_style(
                ProgressStyle::default_bar()
                    .template("{bytes}/{total_bytes} [{bar:80.cyan/red}] {percent:>3}% {bytes_per_sec} {eta}")
                    .unwrap()
                    .progress_chars("#>-"),
            );
            pb.set_position(resume_pos);
            Some(pb)
        } else {
            None
        };

        let mut file = if resume_pos > 0 {
            OpenOptions::new().append(true).open(file_path).await
        } else {
            File::create(file_path).await
        }.map_err(|e| DownloadError {
            message: format!("Failed to open file: {}", e),
        })?;

        let mut stream = response.bytes_stream();
        let mut download = resume_pos;

        use futures_util::StreamExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| DownloadError {
                message: format!("Failed to read chunk: {}", e),
            })?;
            
            if let Some(limiter) = rate_limiter.as_mut() {
                limiter.consume(chunk.len() as u64).await;
            }
            
            file.write_all(&chunk).await.map_err(|e| DownloadError {
                message: format!("Failed to write chunk: {}", e),
            })?;
            
            download += chunk.len() as u64;
            if let Some(ref pb) = progress_bar {
                pb.set_position(download);
            }
        }
        
        file.flush().await.map_err(|e| DownloadError {
            message: format!("Failed to flush file: {}", e),
        })?;

        if let Some(pb) = progress_bar {
            pb.finish();
            println!();
        }
        println!("Downloaded [{}]", url);

        Ok(download - resume_pos)
    }

    async fn download_with_range(&self, url: &str, start: u64) -> Result<Response, DownloadError> {
        let range_header = ResumeHandler::create_range_header(start).unwrap();
        
        let response = self
            .client
            .get(url)
            .header("Range", range_header)
            .send()
            .await
            .map_err(|e| DownloadError {
                message: format!("Failed to send request: {}", e),
            })?;

        let status = response.status();
        if !status.is_success() && status.as_u16() != 206 {
            return Err(DownloadError {
                message: format!("HTTP error: {}", status),
            });
        }
        
        Ok(response)
    }
}
