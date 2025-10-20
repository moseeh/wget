# Mirror Module

This module implements website mirroring functionality for wget-rs, allowing recursive download of entire websites while maintaining directory structure.

## Features

- **Recursive crawling** - Follows links within the same domain
- **Directory structure preservation** - Maintains website hierarchy locally
- **File filtering** - Reject specific file types with `-R` flag
- **Directory exclusion** - Skip specific directories with `-X` flag
- **Same-domain restriction** - Only mirrors content from the target domain

## Usage

```bash
# Basic mirroring
./wget --mirror https://example.com/

# Mirror with file type filtering
./wget --mirror -R jpg,gif,png https://example.com/

# Mirror excluding specific directories
./wget --mirror -X /tmp,/cache https://example.com/

# Mirror to specific directory
./wget --mirror -P ~/mirror/ https://example.com/
```

## Architecture

### `crawler.rs`
- `MirrorCrawler` - Main crawler that manages the download queue
- Handles directory creation and file saving
- Manages visited URLs to prevent infinite loops

### `parser.rs`
- Link extraction from HTML content using regex
- URL filtering and validation
- File suffix and directory exclusion logic

## Limitations

- Uses simple regex-based HTML parsing (not a full HTML parser)
- Only follows same-domain links for security
- No support for JavaScript-generated content
- No robots.txt compliance checking