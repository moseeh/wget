# Resume Downloads Module

This module implements partial download resumption using HTTP Range requests.

## Features

- **Automatic resume detection** - Checks for existing partial files
- **HTTP Range requests** - Uses standard Range header for resumption
- **Progress tracking** - Correctly shows progress from resume point

## Usage

```bash
# Resume a partial download
./wget -c https://example.com/largefile.zip

# Works with rate limiting
./wget -c --rate-limit=1M https://example.com/largefile.zip

# Resume works with all output options
./wget -c -P ~/Downloads/ https://example.com/largefile.zip
```

## Implementation

The `ResumeHandler` checks file size and creates appropriate Range headers. The HTTP client handles 206 Partial Content responses and appends to existing files.