use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Manages multiple progress bars for concurrent downloads
pub struct MultiProgressManager {
    multi_progress: Arc<MultiProgress>,
    progress_bars: Arc<Mutex<HashMap<String, ProgressBar>>>,
}

impl MultiProgressManager {
    pub fn new() -> Self {
        Self {
            multi_progress: Arc::new(MultiProgress::new()),
            progress_bars: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Creates a new progress bar for a specific URL
    pub async fn create_progress_bar(&self, url: &str, content_length: u64) -> ProgressBar {
        let progress_bar = ProgressBar::new(content_length);

        // Set a more compact style for multiple downloads
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{prefix:.bold.dim} {bytes}/{total_bytes} [{bar:40.cyan/red}] {eta_precise}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        // Set a prefix with the filename from URL
        let filename = crate::utils::url::extract_filename(url);
        progress_bar.set_prefix(format!("{:<20}", filename));

        // Add to multi-progress display
        let pb = self.multi_progress.add(progress_bar.clone());

        // Store in our map
        let mut bars = self.progress_bars.lock().await;
        bars.insert(url.to_string(), pb.clone());

        pb
    }

    /// Updates progress for a specific URL
    pub async fn update_progress(&self, url: &str, downloaded: u64) {
        let bars = self.progress_bars.lock().await;
        if let Some(pb) = bars.get(url) {
            pb.set_position(downloaded);
        }
    }

    /// Marks a download as complete
    pub async fn finish_download(&self, url: &str, success: bool) {
        let mut bars = self.progress_bars.lock().await;
        if let Some(pb) = bars.remove(url) {
            if success {
                pb.finish_with_message("✓ Complete");
            } else {
                pb.finish_with_message("✗ Failed");
            }
        }
    }

    /// Gets the multi-progress instance for manual management if needed
    pub fn get_multi_progress(&self) -> Arc<MultiProgress> {
        self.multi_progress.clone()
    }
}
