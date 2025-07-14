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

    /// Downloads multiple URLs concurrently
    pub async fn download_urls(
        &self,
        urls: Vec<String>,
        output_dir: Option<&Path>,
    ) -> Vec<DownloadResult> {
        let semaphore = Arc::new(tokio::sync::Semaphore::new(self.max_concurrent));
        let mut tasks = Vec::new();

        for url in urls {
            let semaphore = semaphore.clone();
            let http_client = self.http_client.clone();
            let progress_manager = self.progress_manager.clone();
            let output_dir = output_dir.map(|p| p.to_path_buf());

            let task = tokio::spawn(async move {
                let _permit = semaphore.acquire().await.unwrap();
                Self::download_single_url(url, http_client, progress_manager, output_dir).await
            });

            tasks.push(task);
        }

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("Task failed: {}", e);
                    // Create a failed result for the task that panicked
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

        results
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
