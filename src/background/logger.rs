use chrono::Utc;
use std::fs::OpenOptions;
use std::io::Write;

pub struct BackgroundLogger {
    log_file: String,
}

impl BackgroundLogger {
    pub fn new() -> Self {
        Self {
            log_file: "wget-log".to_string(),
        }
    }

    pub fn log(&self, message: &str) {
        let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S");
        let log_entry = format!("{} {}\n", timestamp, message);
        
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
        {
            let _ = file.write_all(log_entry.as_bytes());
        }
    }

    pub fn log_start(&self, url: &str) {
        self.log(&format!("Starting download: {}", url));
    }

    pub fn log_success(&self, url: &str, bytes: u64) {
        self.log(&format!("Downloaded [{}] - {} bytes", url, bytes));
    }

    pub fn log_error(&self, url: &str, error: &str) {
        self.log(&format!("Failed [{}]: {}", url, error));
    }

    pub fn log_mirror_start(&self, url: &str) {
        self.log(&format!("Starting mirror: {}", url));
    }

    pub fn log_mirror_complete(&self) {
        self.log("Mirror completed successfully");
    }
}