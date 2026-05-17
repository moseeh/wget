# wget-rs

A subset of GNU `wget` reimplemented in Rust. Built for the 01-edu wget project: implements single-file downloads, batch downloads from a list, background mode, rate limiting, and website mirroring.

## Supported flags

| Flag | Description |
|------|-------------|
| `-O <file>` | Save the downloaded file under a specific name |
| `-P <dir>` | Save into the given directory |
| `-B` | Download in the background; output goes to `wget-log` |
| `-i <file>` | Read URLs to download from `<file>` (one per line) |
| `--rate-limit=<rate>` | Throttle download speed; accepts `k`/`M` suffixes (e.g. `200k`, `2M`) |
| `-c` | Continue a partial download using HTTP Range |
| `--mirror` | Recursively mirror a website |
| `-R <suffixes>` | (with `--mirror`) Reject file extensions; comma-separated |
| `-X <dirs>` | (with `--mirror`) Skip path prefixes; comma-separated |
| `--convert-links` | (with `--mirror`) Rewrite links in HTML/CSS for offline viewing |
| `--timeout=<secs>` | HTTP timeout in seconds (default `30`) |
| `--user-agent=<str>` | Override the User-Agent header |

## Build

```bash
cargo build --release
# binary lands at ./target/release/wget
```

## Examples

Single download:

```bash
./wget https://pbs.twimg.com/media/EMtmPFLWkAA8CIS.jpg
```

```
start at 2026-05-17 22:32:31
sending request, awaiting response... status 200 OK
content size: 56370 [~0.05MB]
saving file to: ./EMtmPFLWkAA8CIS.jpg
 55.05 KiB / 55.05 KiB [==================] 100% 1.24 MiB/s 0s

Downloaded [https://pbs.twimg.com/media/EMtmPFLWkAA8CIS.jpg]
finished at 2026-05-17 22:32:33
```

Save under a different name, in a specific directory:

```bash
./wget -O=meme.jpg https://pbs.twimg.com/media/EMtmPFLWkAA8CIS.jpg
./wget -P=~/Downloads/ https://pbs.twimg.com/media/EMtmPFLWkAA8CIS.jpg
```

Throttled download:

```bash
./wget --rate-limit=300k https://assets.01-edu.org/wgetDataSamples/20MB.zip
```

Background download — returns the shell immediately and writes to `wget-log`:

```bash
./wget -B https://assets.01-edu.org/wgetDataSamples/20MB.zip
# Output will be written to "wget-log".
```

Batch download from a file (run concurrently):

```bash
cat downloads.txt
# https://assets.01-edu.org/wgetDataSamples/Image_20MB.zip
# https://assets.01-edu.org/wgetDataSamples/20MB.zip
./wget -i=downloads.txt
```

Mirror a website:

```bash
./wget --mirror https://trypap.com/
./wget --mirror -R=gif https://oct82.com/
./wget --mirror -X=/img https://trypap.com/
./wget --mirror --convert-links http://corndog.io/
```

## Behavior notes

- Command-line URLs are downloaded sequentially.
- URLs from an `-i` file are downloaded concurrently (default 4 in flight).
- `-B` is implemented by re-execing the process under `setsid` with stdout/stderr redirected to `wget-log`; the progress bar is suppressed in that path so the log stays clean.
- `--convert-links` rewrites `<a href>`, `<link href>`, `<img src>` and CSS `url(...)` references to relative local paths. External links and `mailto:` / `data:` / `javascript:` / fragment links are left alone.
- `--mirror` writes files as raw bytes — images, fonts and other binaries round-trip intact. Link extraction only runs on responses identified as HTML.
- Times are local-clock, formatted `yyyy-mm-dd hh:mm:ss`.

## Module layout

| Path | Purpose |
|------|---------|
| [`src/cli/`](src/cli/readme.md) | Clap-based argument parsing and validation |
| [`src/http/`](src/http/readme.md) | `HttpClient` — request, stream-to-file, resume, rate-limit |
| [`src/io/`](src/io/readme.md) | Reading URL lists from a file |
| [`src/download/`](src/download/readme.md) | Concurrent download manager used by `-i` |
| [`src/mirror/`](src/mirror/readme.md) | Recursive crawler, link extraction, link rewriting |
| [`src/rate/`](src/rate/readme.md) | Token-bucket rate limiter |
| [`src/resume/`](src/resume/readme.md) | HTTP Range helpers for `-c` |
| [`src/utils/`](src/utils/readme.md) | URL filename extraction, save-path formatting |

## Dependencies

- `clap` — CLI parsing
- `reqwest` (+ `stream` feature) — HTTP client
- `tokio` — async runtime
- `indicatif` — progress bars
- `chrono` — timestamps
- `futures-util` — stream combinators
- `url` — URL parsing
- `regex` — HTML/CSS link extraction
- `libc` — `setsid(2)` for `-B`
