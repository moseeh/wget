use std::path::Path;

pub fn extract_filename(url: &str) -> String {
    // Parse URL and extract filename from path
    if let Ok(parsed) = url::Url::parse(url) {
        let path = parsed.path();
        if let Some(filename) = Path::new(path).file_name() {
            if let Some(filename_str) = filename.to_str() {
                if !filename_str.is_empty() {
                    return filename_str.to_string();
                }
            }
        }
    }

    // Fallback: use last segment of URL or default name
    if let Some(filename) = url.split('/').last() {
        if !filename.is_empty() && filename.contains('.') {
            return filename.to_string();
        }
    }

    // Default filename
    "index.html".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_filename() {
        assert_eq!(extract_filename("https://example.com/file.jpg"), "file.jpg");
        assert_eq!(
            extract_filename("https://example.com/path/to/document.pdf"),
            "document.pdf"
        );
        assert_eq!(extract_filename("https://example.com/"), "index.html");
        assert_eq!(extract_filename("https://example.com"), "index.html");
    }
}
