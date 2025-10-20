use std::path::Path;
use tokio::fs;

pub struct ResumeHandler;

impl ResumeHandler {
    pub async fn get_resume_position(file_path: &Path) -> u64 {
        if let Ok(metadata) = fs::metadata(file_path).await {
            metadata.len()
        } else {
            0
        }
    }

    pub fn create_range_header(start: u64) -> Option<String> {
        if start > 0 {
            Some(format!("bytes={}-", start))
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub async fn should_resume(file_path: &Path) -> bool {
        file_path.exists() && Self::get_resume_position(file_path).await > 0
    }
}