use crate::http::HttpClient;
use crate::mirror::parser;
use crate::utils::url::extract_filename;
use std::collections::{HashSet, VecDeque};
use std::path::{Path, PathBuf};
use tokio::fs;
use url::Url;

pub struct MirrorCrawler {
    client: HttpClient,
    visited: HashSet<String>,
    queue: VecDeque<String>,
    base_url: Url,
    output_dir: PathBuf,
}

impl MirrorCrawler {
    pub fn new(base_url: &str, output_dir: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        let parsed_url = Url::parse(base_url)?;
        let dir = output_dir.unwrap_or_else(|| Path::new(".")).to_path_buf();
        
        Ok(MirrorCrawler {
            client: HttpClient::new(),
            visited: HashSet::new(),
            queue: VecDeque::new(),
            base_url: parsed_url,
            output_dir: dir,
        })
    }

    pub fn new_silent(base_url: &str, output_dir: Option<&Path>) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new(base_url, output_dir)
    }

    pub async fn mirror(
        &mut self,
        reject_suffixes: &Option<String>,
        exclude_dirs: &Option<String>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.mirror_internal(reject_suffixes, exclude_dirs, false, None).await
    }

    pub async fn mirror_silent(
        &mut self,
        reject_suffixes: &Option<String>,
        exclude_dirs: &Option<String>,
        logger: &crate::background::BackgroundLogger,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.mirror_internal(reject_suffixes, exclude_dirs, true, Some(logger)).await
    }

    async fn mirror_internal(
        &mut self,
        reject_suffixes: &Option<String>,
        exclude_dirs: &Option<String>,
        silent: bool,
        logger: Option<&crate::background::BackgroundLogger>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.queue.push_back(self.base_url.to_string());
        
        while let Some(url) = self.queue.pop_front() {
            if self.visited.contains(&url) {
                continue;
            }
            
            if parser::should_reject_file(&url, reject_suffixes) {
                continue;
            }
            
            if parser::should_exclude_directory(&url, exclude_dirs) {
                continue;
            }
            
            self.visited.insert(url.clone());
            
            match self.download_and_parse_internal(&url, silent, logger).await {
                Ok(links) => {
                    for link in links {
                        if !self.visited.contains(&link) {
                            self.queue.push_back(link);
                        }
                    }
                }
                Err(e) => {
                    if silent {
                        if let Some(logger) = logger {
                            logger.log_error(&url, &e.to_string());
                        }
                    } else {
                        eprintln!("Failed to download {}: {}", url, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn download_and_parse(&self, url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        self.download_and_parse_internal(url, false, None).await
    }

    async fn download_and_parse_internal(
        &self, 
        url: &str, 
        silent: bool, 
        logger: Option<&crate::background::BackgroundLogger>
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response = self.client.download_silent(url).await?;
        let file_path = self.get_local_path(url);
        
        // Create directory structure
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        let content = response.text().await?;
        
        // Save file
        fs::write(&file_path, &content).await?;
        
        if silent {
            if let Some(logger) = logger {
                logger.log(&format!("Downloaded: {} -> {}", url, file_path.display()));
            }
        } else {
            println!("Downloaded: {} -> {}", url, file_path.display());
        }
        
        // Extract links if it's HTML
        let links = if self.is_html_content(url, &content) {
            parser::extract_links(&content, &self.base_url)
        } else {
            Vec::new()
        };
        
        Ok(links)
    }

    fn get_local_path(&self, url: &str) -> PathBuf {
        if let Ok(parsed) = Url::parse(url) {
            let mut path = self.output_dir.clone();
            
            if let Some(host) = parsed.host_str() {
                path.push(host);
            }
            
            let url_path = parsed.path();
            if url_path == "/" || url_path.is_empty() {
                path.push("index.html");
            } else {
                let segments: Vec<&str> = url_path.split('/').filter(|s| !s.is_empty()).collect();
                for segment in segments {
                    path.push(segment);
                }
                
                // If path doesn't have extension, treat as directory with index.html
                if path.extension().is_none() {
                    path.push("index.html");
                }
            }
            
            path
        } else {
            self.output_dir.join(extract_filename(url))
        }
    }

    fn is_html_content(&self, url: &str, content: &str) -> bool {
        url.ends_with(".html") || 
        url.ends_with(".htm") || 
        content.trim_start().starts_with("<!DOCTYPE") ||
        content.contains("<html")
    }
}