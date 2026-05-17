# `download` module

Concurrent download manager used by the `-i` flag. Single-URL downloads from the command line go through `http::HttpClient` directly; this module exists to handle the batch case.

## Files

- `client.rs` — `ConcurrentDownloadManager` and `DownloadResult`.
- `progress.rs` — `MultiProgressManager`, a thin wrapper around `indicatif::MultiProgress` for keyed progress bars.
- `mod.rs` — re-exports.

## `ConcurrentDownloadManager`

```rust
ConcurrentDownloadManager::with_config(
    max_concurrent: usize,
    rate_limit: Option<&str>,
    user_agent: Option<String>,
    timeout_secs: u64,
)

async fn download_urls(&self, urls: Vec<String>, output_dir: Option<&Path>) -> Vec<DownloadResult>
```

`download_urls` runs in two passes:

1. Sequentially issues HEAD-like GETs to each URL via `HttpClient::download_silent`. Successful responses (with their `Content-Length`) are kept; failures are recorded as `DownloadResult { success: false, .. }` and reported in the final list.
2. The collected responses are streamed to disk concurrently, gated by a `tokio::sync::Semaphore(max_concurrent)`. Each task gets its own clone of the configured rate limiter so per-connection throttling actually applies.

Before phase 2 the manager prints a single line shaped like the spec example:

```
content size: [10485760, 20971520]
```

Each successful task prints `finished <path>` on completion.

## `DownloadResult`

```rust
pub struct DownloadResult {
    pub url: String,
    pub success: bool,
}
```

The caller (`main.rs::process_urls_concurrently`) uses these to print the trailing `Download finished: [...]` line and to return a failure count for the process exit code.

## `MultiProgressManager`

```rust
MultiProgressManager::new()
async fn create_progress_bar(&self, url: &str, content_length: u64) -> ProgressBar
async fn finish_download(&self, url: &str, success: bool)
```

Wraps `indicatif::MultiProgress`. Bars are stored under their URL so completion can be marked without re-passing the handle. Bar style includes bytes, percentage, throughput, and ETA — matching the foreground template in `http::HttpClient`.
