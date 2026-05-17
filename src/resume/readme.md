# `resume` module

HTTP Range helpers powering `-c` (continue partial download).

## Files

- `handler.rs` — `ResumeHandler` (associated functions only, no state).
- `mod.rs` — re-export.

## `ResumeHandler`

```rust
async fn get_resume_position(file_path: &Path) -> u64
fn     create_range_header(start: u64) -> Option<String>
```

- `get_resume_position` returns the current size of `file_path`, or `0` if the file doesn't exist.
- `create_range_header` returns `Some("bytes=<start>-")` when `start > 0`, otherwise `None`.

## How it's wired

`HttpClient::download_to_file_with_resume(url, path, resume=true, …)` calls `get_resume_position`, sends the request with the `Range` header when non-zero, opens the local file in append mode, and pre-positions the progress bar at the resume offset. A `206 Partial Content` response is accepted in addition to `200`.

## Usage

```bash
./wget -c https://example.com/largefile.zip
./wget -c --rate-limit=1M https://example.com/largefile.zip
./wget -c -P ~/Downloads/ https://example.com/largefile.zip
```

Note: the server must honour `Range` requests. If it doesn't, the file will be overwritten from scratch with `200 OK`.
