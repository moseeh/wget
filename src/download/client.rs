use crate::download::progress::MultiProgressManager;
use crate::http::client::{DownloadError, HttpClient};
use crate::rate::RateLimiter;
use crate::utils::url::extract_filename;
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct DownloadResult {
    pub url: String,
    pub success: bool,
}

pub struct ConcurrentDownloadManager {
    http_client: HttpClient,
    progress_manager: Arc<MultiProgressManager>,
    max_concurrent: usize,
    rate_limiter: Option<RateLimiter>,
}

impl ConcurrentDownloadManager {
    pub fn with_config(
        max_concurrent: usize,
        rate_limit: Option<&str>,
        user_agent: Option<String>,
        timeout_secs: u64,
    ) -> Self {
        let rate_limiter = rate_limit.and_then(|r| RateLimiter::new(r).ok());
        Self {
            http_client: HttpClient::with_config(rate_limit, user_agent, timeout_secs),
            progress_manager: Arc::new(MultiProgressManager::new()),
            max_concurrent,
            rate_limiter,
        }
    }

    pub async fn download_urls(
        &self,
        urls: Vec<String>,
        output_dir: Option<&Path>,
    ) -> Vec<DownloadResult> {
        let mut valid_responses = Vec::new();
        let mut results = Vec::new();

        for url in &urls {
            match self.http_client.download_silent(url).await {
                Ok(response) => {
                    let status = response.status();
                    if status.is_success() {
                        let content_length = response.content_length().unwrap_or(0);
                        let file_path = Self::determine_file_path(url, output_dir);
                        valid_responses.push((url.clone(), response, content_length, file_path));
                    } else {
                        results.push(DownloadResult { url: url.clone(), success: false });
                    }
                }
                Err(_) => {
                    results.push(DownloadResult { url: url.clone(), success: false });
                }
            }
        }

        // Emit the content-size summary (matches spec example for -i)
        let sizes: Vec<u64> = valid_responses.iter().map(|(_, _, len, _)| *len).collect();
        println!("content size: {:?}", sizes);

        if !valid_responses.is_empty() {
            let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent));
            let mut download_tasks = Vec::new();

            for (url, response, content_length, file_path) in valid_responses {
                let semaphore = semaphore.clone();
                let progress_manager = self.progress_manager.clone();
                let rate_limiter = self.rate_limiter.clone();

                let task = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    Self::download_from_response(
                        url,
                        response,
                        content_length,
                        file_path,
                        progress_manager,
                        rate_limiter,
                    )
                    .await
                });

                download_tasks.push(task);
            }

            for task in download_tasks {
                match task.await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        eprintln!("Download task failed: {}", e);
                        results.push(DownloadResult { url: "unknown".to_string(), success: false });
                    }
                }
            }
        }

        results
    }

    async fn download_from_response(
        url: String,
        response: reqwest::Response,
        content_length: u64,
        file_path: PathBuf,
        progress_manager: Arc<MultiProgressManager>,
        rate_limiter: Option<RateLimiter>,
    ) -> DownloadResult {
        match Self::perform_download_from_response(
            &url,
            &file_path,
            response,
            content_length,
            &progress_manager,
            rate_limiter,
        )
        .await
        {
            Ok(_) => {
                progress_manager.finish_download(&url, true).await;
                println!("finished {}", file_path.display());
                DownloadResult { url, success: true }
            }
            Err(e) => {
                progress_manager.finish_download(&url, false).await;
                eprintln!("Failed to download [{}]: {}", url, e);
                DownloadResult { url, success: false }
            }
        }
    }

    async fn perform_download_from_response(
        url: &str,
        file_path: &Path,
        response: reqwest::Response,
        content_length: u64,
        progress_manager: &MultiProgressManager,
        mut rate_limiter: Option<RateLimiter>,
    ) -> Result<u64, DownloadError> {
        let progress_bar = progress_manager
            .create_progress_bar(url, content_length)
            .await;

        let mut file = File::create(file_path).await.map_err(|e| DownloadError {
            message: format!("Failed to create file {:?}: {}", file_path, e),
        })?;

        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

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

            downloaded += chunk.len() as u64;
            progress_bar.set_position(downloaded);
        }

        file.flush().await.map_err(|e| DownloadError {
            message: format!("Failed to flush file: {}", e),
        })?;

        Ok(downloaded)
    }

    fn determine_file_path(url: &str, output_dir: Option<&Path>) -> PathBuf {
        let filename = extract_filename(url);

        if let Some(dir) = output_dir {
            dir.join(filename)
        } else {
            PathBuf::from(filename)
        }
    }
}
