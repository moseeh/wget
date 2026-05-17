# `rate` module

Token-bucket-style rate limiter used by both the single-URL download path and the per-task concurrent path.

## Files

- `limiter.rs` — `RateLimiter`.
- `mod.rs` — re-export.

## `RateLimiter`

```rust
RateLimiter::new("200k" | "2M" | "<bytes>") -> Result<RateLimiter, String>
async fn consume(&mut self, bytes: u64)
```

`new` parses the rate string:

- Trailing `k`/`K` → KiB (× 1024).
- Trailing `M`/`m` → MiB (× 1024 × 1024).
- Otherwise interpreted as raw bytes/sec.

`consume` is called with each chunk size as it streams off the network. It tracks bytes consumed since the last reset, computes the allowance for the elapsed time, and sleeps for the excess via `tokio::time::sleep`. Counters reset once a second so small timing errors don't accumulate.

`RateLimiter` is `Clone` so each task in the concurrent download path can own its own copy — the throttle applies per stream.

## Usage

```bash
./wget --rate-limit=200k https://example.com/file.zip
./wget --rate-limit=2M    -i list.txt
```
