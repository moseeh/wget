use crate::background::BackgroundLogger;
use crate::cli::Cli;
use crate::download::ConcurrentDownloadManager;
use crate::http::HttpClient;
use crate::mirror::MirrorCrawler;
use crate::utils::url::extract_filename;
use std::path::PathBuf;

pub struct BackgroundProcessor {
    logger: BackgroundLogger,
}

impl BackgroundProcessor {
    pub fn new() -> Self {
        Self {
            logger: BackgroundLogger::new(),
        }
    }

    pub async fn process_urls(&self, args: &Cli, urls: &[String]) -> u32 {
        let mut failed_count = 0;
        let client = HttpClient::new();

        for url in urls {
            self.logger.log_start(url);
            let file_path = self.determine_output_path(args, url);

            match client.download_to_file_silent(url, &file_path).await {
                Ok(bytes) => {
                    self.logger.log_success(url, bytes);
                }
                Err(e) => {
                    self.logger.log_error(url, &e.to_string());
                    failed_count += 1;
                }
            }
        }

        failed_count
    }

    pub async fn process_file_urls(&self, args: &Cli, urls: Vec<String>) -> u32 {
        let download_manager = ConcurrentDownloadManager::new(4);
        let output_dir = args.directory_prefix.as_deref();
        
        self.logger.log(&format!("Processing {} URLs concurrently", urls.len()));
        let results = download_manager.download_urls_silent(urls, output_dir).await;

        let successful = results.iter().filter(|r| r.success).count();
        let failed = results.len() - successful;
        let total_bytes: u64 = results.iter().map(|r| r.bytes_downloaded).sum();

        self.logger.log(&format!("Completed: {} successful, {} failed, {} bytes", 
                                successful, failed, total_bytes));

        for result in results.iter().filter(|r| !r.success) {
            self.logger.log_error(&result.url, 
                result.error.as_ref().unwrap_or(&"Unknown error".to_string()));
        }

        failed as u32
    }

    pub async fn process_mirror(&self, args: &Cli) -> Result<(), Box<dyn std::error::Error>> {
        let url = &args.urls[0];
        self.logger.log_mirror_start(url);
        
        let mut crawler = MirrorCrawler::new_silent(url, args.directory_prefix.as_deref())?;
        crawler.mirror_silent(&args.reject_suffixes, &args.exclude_dirs, &self.logger).await?;
        
        self.logger.log_mirror_complete();
        Ok(())
    }

    fn determine_output_path(&self, args: &Cli, url: &str) -> PathBuf {
        if let Some(output) = &args.output {
            return output.clone();
        }

        let filename = extract_filename(url);
        if let Some(dir) = &args.directory_prefix {
            dir.join(filename)
        } else {
            PathBuf::from(filename)
        }
    }
}