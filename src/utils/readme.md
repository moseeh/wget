# 🛠️ `utils` Module — Utility Functions for wget-rs

This module provides utility functions and helper components used throughout the `wget-rs` application. It contains common functionality that supports various operations across different modules.

## ✨ Features

* URL parsing and filename extraction
* File path manipulation utilities
* Common helper functions for string processing
* Cross-module utility support
* Reusable components for consistent behavior

## 📁 Structure

* `utils/url.rs`: URL manipulation and parsing utilities
* `utils/mod.rs`: Exports utility functions

## 🔧 Core Components

### URL Utilities (`utils/url.rs`)

#### `extract_filename(url: &str) -> String`
Extracts a filename from a URL for saving downloaded files:
* **Input**: URL string (e.g., `"https://example.com/path/file.zip"`)
* **Output**: Filename string (e.g., `"file.zip"`)
* **Features**:
  - Handles URLs with query parameters
  - Provides fallback names for URLs without clear filenames
  - Removes URL encoding from filenames
  - Handles edge cases like trailing slashes

## ✅ How to Use

```rust
use crate::utils::url::extract_filename;

fn main() {
    // Basic filename extraction
    let filename = extract_filename("https://example.com/downloads/file.zip");
    println!("Filename: {}", filename); // Output: "file.zip"
    
    // URL with query parameters
    let filename = extract_filename("https://example.com/file.pdf?version=1.2");
    println!("Filename: {}", filename); // Output: "file.pdf"
    
    // URL without clear filename
    let filename = extract_filename("https://example.com/api/data");
    println!("Filename: {}", filename); // Output: "data" or fallback name
}
```

## 🔍 Filename Extraction Logic

1. **Parse URL**: Extract the path component from the URL
2. **Remove Query**: Strip query parameters and fragments
3. **Get Last Segment**: Take the last path segment as filename
4. **Fallback Handling**: Provide default names for unclear cases
5. **Sanitization**: Ensure filename is valid for filesystem

## 📚 Notes

* Designed to work with various URL formats and edge cases
* Provides consistent filename extraction across the application
* Handles both absolute and relative URL scenarios
* Integrates with file saving operations in HTTP and download modules
* Extensible design allows for additional utility functions

## 🚀 Future Extensions

This module can be extended with additional utilities such as:
* File size formatting functions
* Time/date formatting helpers
* Path validation utilities
* String sanitization functions
* Configuration parsing helpers

This module ensures consistent and reliable utility operations across wget-rs.
