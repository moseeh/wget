# Rate Limiting Module

This module implements download rate limiting functionality to control bandwidth usage.

## Features

- **Configurable rate limits** - Support for k/M suffixes (e.g., 200k, 2M)
- **Smooth throttling** - Uses token bucket algorithm for consistent rates
- **Async-friendly** - Non-blocking rate limiting with tokio sleep

## Usage

```bash
# Limit to 200 KB/s
./wget --rate-limit=200k https://example.com/file.zip

# Limit to 2 MB/s
./wget --rate-limit=2M https://example.com/file.zip

# Works with all download modes
./wget --rate-limit=500k --mirror https://example.com/
```

## Implementation

The `RateLimiter` tracks bytes consumed over time and introduces delays when the rate exceeds the configured limit. It resets counters periodically to prevent accumulation of small timing errors.