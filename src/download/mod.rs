pub mod client;
pub mod progress;
pub mod resume;

// Concurrent download functionality (for future use)
#[allow(unused_imports)]
pub use client::{ConcurrentDownloadManager, DownloadResult};
#[allow(unused_imports)]
pub use progress::MultiProgressManager;
