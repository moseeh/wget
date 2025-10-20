use crate::download::progress::MultiProgressManager;
use crate::http::client::{DownloadError, HttpClient};
use crate::utils::url::extract_filename;
use futures_util::StreamExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

/// Result of a single download operation
#[derive(Debug)]
pub struct DownloadResult {
    pub url: String,
    pub file_path: PathBuf,
    pub bytes_downloaded: u64,
    pub success: bool,
    pub error: Option<String>,
}

/// Manages concurrent downloads with progress tracking
pub struct ConcurrentDownloadManager {
    http_client: HttpClient,
    progress_manager: Arc<MultiProgressManager>,
    max_concurrent: usize,
}

impl ConcurrentDownloadManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            http_client: HttpClient::new(),
            progress_manager: Arc::new(MultiProgressManager::new()),
            max_concurrent,
        }
    }

    /// Downloads multiple URLs concurrently with separated request/download phases
    pub async fn download_urls(
        &self,
        urls: Vec<String>,
        output_dir: Option<&Path>,
    ) -> Vec<DownloadResult> {
        self.download_urls_internal(urls, output_dir, false).await
    }

    /// Silent version for background downloads
    pub async fn download_urls_silent(
        &self,
        urls: Vec<String>,
        output_dir: Option<&Path>,
    ) -> Vec<DownloadResult> {
        self.download_urls_internal(urls, output_dir, true).await
    }

    async fn download_urls_internal(
        &self,
        urls: Vec<String>,
        output_dir: Option<&Path>,
        silent: bool,
    ) -> Vec<DownloadResult> {
        // Phase 1: Send all requests and collect responses
        if !silent {
            println!("Phase 1: Sending requests...");
        }
        let mut valid_responses = Vec::new();
        let mut results = Vec::new();

        for url in &urls {
            if !silent {
                println!("sending request to {}, awaiting response...", url);
            }

            match self.http_client.download_silent(&url).await {
                Ok(response) => {
                    let status = response.status();
                    if !silent {
                        println!(
                            "status {} {} for {}",
                            status.as_u16(),
                            status.canonical_reason().unwrap_or(""),
                            url
                        );
                    }

                    if status.is_success() {
                        let content_length = response.content_length().unwrap_or(0);
                        let file_path = Self::determine_file_path(url, output_dir.as_deref());
                        valid_responses.push((url.clone(), response, content_length, file_path));
                    } else {
                        // Failed response - add to results as failed
                        results.push(DownloadResult {
                            url: url.clone(),
                            file_path: Self::determine_file_path(url, output_dir.as_deref()),
                            bytes_downloaded: 0,
                            success: false,
                            error: Some(format!("HTTP error: {}", status)),
                        });
                    }
                }
                Err(e) => {
                    if !silent {
                        println!("failed to connect to {}: {}", url, e);
                    }
                    // Failed request - add to results as failed
                    results.push(DownloadResult {
                        url: url.clone(),
                        file_path: Self::determine_file_path(url, output_dir.as_deref()),
                        bytes_downloaded: 0,
                        success: false,
                        error: Some(e.message),
                    });
                }
            }
        }

        // Phase 2: Download all valid responses concurrently
        if !valid_responses.is_empty() {
            if !silent {
                println!(
                    "\nPhase 2: Starting {} concurrent downloads...",
                    valid_responses.len()
                );
            }

            let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent));
            let mut download_tasks = Vec::new();

            for (url, response, content_length, file_path) in valid_responses {
                let semaphore = semaphore.clone();
                let progress_manager = self.progress_manager.clone();

                let task = tokio::spawn(async move {
                    let _permit = semaphore.acquire().await.unwrap();
                    Self::download_from_response(
                        url,
                        response,
                        content_length,
                        file_path,
                        progress_manager,
                    )
                    .await
                });

                download_tasks.push(task);
            }

            // Collect download results
            for task in download_tasks {
                match task.await {
                    Ok(result) => results.push(result),
                    Err(e) => {
                        eprintln!("Download task failed: {}", e);
                        results.push(DownloadResult {
                            url: "unknown".to_string(),
                            file_path: PathBuf::new(),
                            bytes_downloaded: 0,
                            success: false,
                            error: Some(format!("Task panicked: {}", e)),
                        });
                    }
                }
            }
        }

        results
    }

    /// Downloads from an already-received HTTP response
    async fn download_from_response(
        url: String,
        response: reqwest::Response,
        content_length: u64,
        file_path: PathBuf,
        progress_manager: Arc<MultiProgressManager>,
    ) -> DownloadResult {
        match Self::perform_download_from_response(
            &url,
            &file_path,
            response,
            content_length,
            &progress_manager,
        )
        .await
        {
            Ok(bytes_downloaded) => {
                progress_manager.finish_download(&url, true).await;
                println!("Downloaded [{}] -> {}", url, file_path.display());
                DownloadResult {
                    url,
                    file_path,
                    bytes_downloaded,
                    success: true,
                    error: None,
                }
            }
            Err(e) => {
                progress_manager.finish_download(&url, false).await;
                eprintln!("Failed to download [{}]: {}", url, e);
                DownloadResult {
                    url,
                    file_path,
                    bytes_downloaded: 0,
                    success: false,
                    error: Some(e.message),
                }
            }
        }
    }

    async fn download_single_url(
        url: String,
        http_client: HttpClient,
        progress_manager: Arc<MultiProgressManager>,
        output_dir: Option<PathBuf>,
    ) -> DownloadResult {
        let file_path = Self::determine_file_path(&url, output_dir.as_deref());

        match Self::perform_download(&url, &file_path, &http_client, &progress_manager).await {
            Ok(bytes_downloaded) => {
                progress_manager.finish_download(&url, true).await;
                println!("Downloaded [{}] -> {}", url, file_path.display());
                DownloadResult {
                    url,
                    file_path,
                    bytes_downloaded,
                    success: true,
                    error: None,
                }
            }
            Err(e) => {
                progress_manager.finish_download(&url, false).await;
                eprintln!("Failed to download [{}]: {}", url, e);
                DownloadResult {
                    url,
                    file_path,
                    bytes_downloaded: 0,
                    success: false,
                    error: Some(e.message),
                }
            }
        }
    }

    async fn perform_download(
        url: &str,
        file_path: &Path,
        http_client: &HttpClient,
        progress_manager: &MultiProgressManager,
    ) -> Result<u64, DownloadError> {
        let response = http_client.download(url).await?;
        let content_length = response.content_length().unwrap_or(0);

        // Create progress bar for this download
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

    /// Performs download from an already-received response
    async fn perform_download_from_response(
        url: &str,
        file_path: &Path,
        response: reqwest::Response,
        content_length: u64,
        progress_manager: &MultiProgressManager,
    ) -> Result<u64, DownloadError> {
        // Create progress bar for this download
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

    pub fn get_progress_manager(&self) -> Arc<MultiProgressManager> {
        self.progress_manager.clone()
    }
}
