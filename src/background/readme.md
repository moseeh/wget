# Background Module

This module implements background download functionality for wget-rs, allowing downloads to continue in the background while logging progress to `wget-log`.

## Features

- **Background processing** - Downloads continue without terminal output
- **Logging to wget-log** - All progress and errors logged to file
- **Mirror support** - Background mirroring with detailed logging
- **Concurrent downloads** - Background support for input file processing
- **Error handling** - Failed downloads logged with error details

## Usage

```bash
# Background single download
./wget -B https://example.com/file.zip

# Background mirror
./wget -B --mirror https://example.com/

# Background input file processing
./wget -B -i urls.txt

# Background mirror with filtering
./wget -B --mirror -R js,css https://example.com/
```

## Architecture

### `logger.rs`
- `BackgroundLogger` - Handles all logging to wget-log file
- Timestamped log entries for all operations
- Separate methods for different log types (start, success, error)

### `process.rs`
- `BackgroundProcessor` - Main coordinator for background operations
- Handles all download types (single, concurrent, mirror)
- Integrates with existing download and mirror modules
- Silent operation with comprehensive logging

## Log Format

The `wget-log` file contains timestamped entries:
```
2025-10-20 11:39:14 Starting download: https://example.com/file.zip
2025-10-20 11:39:16 Downloaded [https://example.com/file.zip] - 1024 bytes
2025-10-20 11:39:28 Starting mirror: https://example.com/
2025-10-20 11:39:51 Mirror completed successfully
```