use std::collections::HashSet;
use url::Url;

pub fn extract_links(html: &str, base_url: &Url) -> Vec<String> {
    let mut links = HashSet::new();
    
    // Simple regex-based link extraction
    let patterns = [
        r#"href\s*=\s*["']([^"']+)["']"#,
        r#"src\s*=\s*["']([^"']+)["']"#,
    ];
    
    for pattern in &patterns {
        if let Ok(re) = regex::Regex::new(pattern) {
            for cap in re.captures_iter(html) {
                if let Some(link) = cap.get(1) {
                    let link_str = link.as_str();
                    if let Ok(absolute_url) = base_url.join(link_str) {
                        let url_str = absolute_url.to_string();
                        if is_valid_mirror_url(&url_str, base_url) {
                            links.insert(url_str);
                        }
                    }
                }
            }
        }
    }
    
    links.into_iter().collect()
}

fn is_valid_mirror_url(url: &str, base_url: &Url) -> bool {
    if let Ok(parsed) = Url::parse(url) {
        // Only mirror URLs from the same domain
        parsed.domain() == base_url.domain() && 
        parsed.scheme() == base_url.scheme()
    } else {
        false
    }
}

pub fn should_reject_file(url: &str, reject_suffixes: &Option<String>) -> bool {
    if let Some(suffixes) = reject_suffixes {
        let reject_list: Vec<&str> = suffixes.split(',').collect();
        for suffix in reject_list {
            if url.ends_with(&format!(".{}", suffix.trim())) {
                return true;
            }
        }
    }
    false
}

pub fn should_exclude_directory(url: &str, exclude_dirs: &Option<String>) -> bool {
    if let Some(dirs) = exclude_dirs {
        let exclude_list: Vec<&str> = dirs.split(',').collect();
        for dir in exclude_list {
            if url.contains(&format!("/{}/", dir.trim())) {
                return true;
            }
        }
    }
    false
}