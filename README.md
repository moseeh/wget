# 🚀 wget-rs — A Modern wget Implementation in Rust

A high-performance, feature-rich implementation of the classic `wget` utility, built with Rust for speed, safety, and concurrency.

## ✨ Features

### 🌐 Core Download Functionality
* **Single file downloads** with progress tracking
* **Multiple file downloads** from command line arguments
* **Batch downloads** from input files (`-i` flag)
* **Sequential processing** for command line URLs (wget-compatible)
* **Concurrent processing** for file-based URLs (performance optimized)

### 🎛️ Advanced Options
* **Custom output names** (`-O` flag)
* **Directory specification** (`-P` flag)
* **Background downloads** (`-B` flag) with logging
* **Rate limiting** (`--rate-limit`) with k/M suffixes
* **Website mirroring** (`--mirror`) with filtering options

### 🔧 Technical Highlights
* **Asynchronous I/O** using Tokio for high performance
* **Streaming downloads** for memory efficiency
* **Multi-progress bars** for concurrent download tracking
* **Two-phase download process** for clean user experience
* **Comprehensive error handling** with detailed reporting

## 🏗️ Architecture

The project is organized into focused modules:

### 📁 Module Structure

* **[`cli/`](src/cli/readme.md)** — Command-line argument parsing and validation
* **[`http/`](src/http/readme.md)** — HTTP client and network operations
* **[`io/`](src/io/readme.md)** — File I/O and URL input processing
* **[`download/`](src/download/readme.md)** — Concurrent download management and progress tracking
* **[`utils/`](src/utils/readme.md)** — Utility functions and helpers

## 🚀 Quick Start

### Installation
```bash
git clone <repository-url>
cd wget-rs
cargo build --release
```

### Basic Usage
```bash
# Download a single file
./wget https://example.com/file.zip

# Download with custom name
./wget -O myfile.zip https://example.com/file.zip

# Download to specific directory
./wget -P ~/Downloads/ https://example.com/file.zip

# Download multiple files from input file
./wget -i downloads.txt

# Background download with rate limiting
./wget -B --rate-limit=500k https://example.com/largefile.zip
```

## 📋 Command Line Options

| Flag | Description | Example |
|------|-------------|---------|
| `-O <file>` | Save as specific filename | `./wget -O image.jpg <url>` |
| `-P <dir>` | Save to directory | `./wget -P ~/Downloads/ <url>` |
| `-i <file>` | Read URLs from file | `./wget -i urls.txt` |
| `-B` | Download in background | `./wget -B <url>` |
| `--rate-limit=<rate>` | Limit download speed | `./wget --rate-limit=200k <url>` |
| `--mirror` | Mirror entire website | `./wget --mirror <url>` |
| `-R <suffixes>` | Reject file types | `./wget --mirror -R=jpg,gif <url>` |
| `-X <dirs>` | Exclude directories | `./wget --mirror -X=/tmp,/cache <url>` |

## 🎯 Key Differentiators

### vs. Original wget
* **Concurrent file downloads** from input files (original wget is sequential)
* **Modern progress bars** with multi-download support
* **Two-phase download process** prevents UI conflicts
* **Better error reporting** with detailed summaries

### vs. Other Download Tools
* **True wget compatibility** for single downloads
* **Rust performance and safety** benefits
* **Async/await architecture** for efficiency
* **Modular design** for maintainability

## 🔄 Download Behavior

### Command Line URLs (Sequential)
```bash
./wget url1 url2 url3
# Downloads: url1 → url2 → url3 (one after another)
```

### Input File URLs (Concurrent)
```bash
./wget -i urls.txt
# Phase 1: Send all requests, collect responses
# Phase 2: Download all files simultaneously
```

## 📊 Example Output

### Single Download
```
start at 2024-01-15 10:30:45
sending request, awaiting response... status 200 OK
content size: 1048576 [~1.00MB]
saving file to: ./file.zip
1.00 MiB / 1.00 MiB [████████████████████] 100.00% 2.5 MiB/s 0s

Downloaded [https://example.com/file.zip]
finished at 2024-01-15 10:30:47
```

### Concurrent Downloads
```
start at 2024-01-15 10:30:45
Read 3 URLs from file: downloads.txt
Processing file URLs concurrently...
Phase 1: Sending requests...
sending request to https://example.com/file1.zip, awaiting response...
status 200 OK for https://example.com/file1.zip
sending request to https://example.com/file2.zip, awaiting response...
status 200 OK for https://example.com/file2.zip

Phase 2: Starting 2 concurrent downloads...
file1.zip            512 KiB / 1.0 MiB [████████████████████] 50.0% 1.2 MiB/s 1s
file2.zip            256 KiB / 2.0 MiB [██████████          ] 25.0% 800 KiB/s 3s

Concurrent Download Summary:
  Successful: 2
  Failed: 0
  Total bytes: 3145728 (3.00 MB)
finished at 2024-01-15 10:30:52
```

## 🧪 Testing

### Run Tests
```bash
cargo test
```

### Manual Testing
```bash
# Test single download
./wget https://httpbin.org/bytes/1024

# Test concurrent downloads
echo -e "https://httpbin.org/bytes/1024\nhttps://httpbin.org/json" > test.txt
./wget -i test.txt

# Test with directory
./wget -P /tmp/ https://httpbin.org/uuid

# Test background mode
./wget -B https://httpbin.org/bytes/2048
cat wget-log
```

## 🛠️ Development

### Prerequisites
* Rust 1.70+
* Tokio runtime
* Internet connection for testing

### Dependencies
* `clap` — Command line parsing
* `reqwest` — HTTP client
* `tokio` — Async runtime
* `indicatif` — Progress bars
* `chrono` — Date/time handling
* `futures-util` — Stream utilities

### Building
```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run with logging
RUST_LOG=debug cargo run -- <url>
```

## 📚 Documentation

Each module contains detailed documentation:
* **[CLI Module](src/cli/readme.md)** — Argument parsing and validation
* **[HTTP Module](src/http/readme.md)** — Network operations and client
* **[I/O Module](src/io/readme.md)** — File operations and URL reading
* **[Download Module](src/download/readme.md)** — Concurrent download management
* **[Utils Module](src/utils/readme.md)** — Utility functions and helpers

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

* Original GNU wget team for the inspiration
* Rust community for excellent async ecosystem
* Contributors and testers

---

**Built with ❤️ and ⚡ Rust**
