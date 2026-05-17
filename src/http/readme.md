# `http` module

HTTP client built on top of [`reqwest`](https://docs.rs/reqwest), streaming responses to disk with optional rate limiting and resume.

## Files

- `client.rs` — `HttpClient` and `DownloadError`.
- `mod.rs` — re-exports `HttpClient`.

## `HttpClient`

```rust
HttpClient::new()
HttpClient::with_config(rate_limit: Option<&str>, user_agent: Option<String>, timeout_secs: u64)

async fn download(&self, url: &str) -> Result<Response, DownloadError>
async fn download_silent(&self, url: &str) -> Result<Response, DownloadError>
async fn download_to_file_with_resume(
    &self,
    url: &str,
    file_path: &Path,
    resume: bool,
    show_progress: bool,
) -> Result<u64, DownloadError>
```

- `download` prints `sending request, awaiting response... status N XX` to stdout.
- `download_silent` is identical but emits no output — used by the concurrent path and the mirror crawler, which print their own context.
- `download_to_file_with_resume` is the full workflow used by single-file downloads:
  - Honours `resume` by reading the existing file size and sending `Range: bytes=N-`.
  - When `show_progress` is `true`, draws an `indicatif` bar showing bytes, percentage, throughput, and ETA. Suppress it in background mode (`-B`) so the log file stays clean.
  - Applies the rate limiter (if configured on the client) per streamed chunk.

## `DownloadError`

Wraps a single `String` message and implements `Display` and `std::error::Error`. Anywhere a network read, HTTP status check, or file write fails, the error is folded into this type with a context-prefixed message.

## Notes

- The underlying `reqwest::Client` is built once with the configured timeout and User-Agent; cloning `HttpClient` is cheap because both the inner client and the rate limiter are reference-counted / `Clone`.
- For mirror mode the body is consumed in full via `response.bytes()` so binary files round-trip intact.
