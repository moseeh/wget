use crate::http::HttpClient;
use crate::mirror::parser;
use crate::utils::url::extract_filename;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::{Path, PathBuf};
use tokio::fs;
use url::Url;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssetKind {
    Html,
    Css,
    Other,
}

pub struct MirrorCrawler {
    client: HttpClient,
    visited: HashSet<String>,
    queue: VecDeque<String>,
    base_url: Url,
    output_dir: PathBuf,
    // url -> (local path, asset kind) — used by --convert-links
    saved: HashMap<String, (PathBuf, AssetKind)>,
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
            saved: HashMap::new(),
        })
    }

    pub async fn mirror(
        &mut self,
        reject_suffixes: &Option<String>,
        exclude_dirs: &Option<String>,
        convert_links: bool,
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

            match self.download_and_parse(&url).await {
                Ok(links) => {
                    for link in links {
                        if !self.visited.contains(&link) {
                            self.queue.push_back(link);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to download {}: {}", url, e);
                }
            }
        }

        if convert_links {
            self.rewrite_links().await?;
        }
        Ok(())
    }

    async fn download_and_parse(
        &mut self,
        url: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let response = self.client.download_silent(url).await?;

        // Determine asset kind from Content-Type before consuming the body
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_lowercase();

        let kind_from_header = if content_type.contains("text/html") || content_type.contains("application/xhtml") {
            Some(AssetKind::Html)
        } else if content_type.contains("text/css") {
            Some(AssetKind::Css)
        } else if content_type.is_empty() {
            None
        } else {
            Some(AssetKind::Other)
        };

        let file_path = self.get_local_path(url);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Always fetch as bytes — binary-safe
        let bytes = response.bytes().await?;

        // Fallback sniff if header didn't tell us
        let kind = kind_from_header.unwrap_or_else(|| {
            let head = std::str::from_utf8(&bytes[..bytes.len().min(512)])
                .unwrap_or("")
                .trim_start()
                .to_ascii_lowercase();
            if head.starts_with("<!doctype") || head.contains("<html") {
                AssetKind::Html
            } else {
                AssetKind::Other
            }
        });

        fs::write(&file_path, &bytes).await?;
        self.saved.insert(url.to_string(), (file_path.clone(), kind));

        println!("Downloaded: {} -> {}", url, file_path.display());

        // Only parse HTML for new links to follow
        let links = if kind == AssetKind::Html {
            // Safe lossy decode for link extraction — HTML link attributes are ASCII
            let content = String::from_utf8_lossy(&bytes);
            parser::extract_links(&content, &self.base_url)
        } else {
            Vec::new()
        };

        Ok(links)
    }

    async fn rewrite_links(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Build url -> local path map for rewriting
        let url_to_path: HashMap<&str, &Path> = self
            .saved
            .iter()
            .map(|(u, (p, _))| (u.as_str(), p.as_path()))
            .collect();

        for (_url, (path, kind)) in &self.saved {
            if *kind != AssetKind::Html && *kind != AssetKind::Css {
                continue;
            }
            let raw = match fs::read(path).await {
                Ok(b) => b,
                Err(_) => continue,
            };
            let content = String::from_utf8_lossy(&raw).into_owned();

            let rewritten = match kind {
                AssetKind::Html => parser::rewrite_html_links(&content, &self.base_url, path, &url_to_path),
                AssetKind::Css => parser::rewrite_css_links(&content, &self.base_url, path, &url_to_path),
                _ => continue,
            };

            fs::write(path, rewritten).await?;
        }
        Ok(())
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
}
