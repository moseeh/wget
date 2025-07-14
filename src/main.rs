use chrono::Utc;
use clap::Parser;
use download::ConcurrentDownloadManager;
use std::path::PathBuf;

mod cli;
mod download;
mod http;
mod io;
mod utils;

#[tokio::main]
async fn main() {
    let start_time = Utc::now();
    println!("start at {}", start_time.format("%Y-%m-%d %H:%M:%S"));

    let args = cli::Cli::parse();
    if let Err(e) = args.validate() {
        eprintln!("Argument error: {}", e);
        std::process::exit(1);
    }

    let mut failed_downloads = 0;

    // Process command line URLs sequentially (like real wget)
    if !args.urls.is_empty() {
        println!(
            "Processing {} command line URLs sequentially...",
            args.urls.len()
        );
        failed_downloads += process_urls_sequentially(&args, &args.urls).await;
    }

    // Process input file URLs concurrently (for efficiency)
    if let Some(input_file) = &args.input_file {
        match io::read_urls_from_file(input_file).await {
            Ok(file_urls) => {
                if !file_urls.is_empty() {
                    println!(
                        "Read {} URLs from file: {}",
                        file_urls.len(),
                        input_file.display()
                    );
                    println!("Processing file URLs concurrently...");
                    failed_downloads += process_urls_concurrently(&args, file_urls).await;
                }
            }
            Err(e) => {
                eprintln!("Error reading URLs from file: {}", e);
                std::process::exit(1);
            }
        }
    }

    // Check if we have any URLs to process
    if args.urls.is_empty() && args.input_file.is_none() {
        eprintln!("No URLs to download");
        std::process::exit(1);
    }

    let end_time = Utc::now();
    println!("finished at {}", end_time.format("%Y-%m-%d %H:%M:%S"));

    // Exit with error code if any downloads failed (like real wget)
    if failed_downloads > 0 {
        std::process::exit(1);
    }
}

/// Process URLs sequentially (for command line URLs)
async fn process_urls_sequentially(args: &cli::Cli, urls: &[String]) -> u32 {
    let client = http::HttpClient::new();
    let mut failed_count = 0;

    for url in urls {
        let file_path = determine_output_path(args, url);

        match client.download_to_file(url, &file_path).await {
            Ok(_) => {
                println!("Downloaded [{}]", url);
            }
            Err(e) => {
                eprintln!("Download failed for [{}]: {}", url, e);
                failed_count += 1;
            }
        }
    }

    failed_count
}

/// Process URLs concurrently (for input file URLs)
async fn process_urls_concurrently(args: &cli::Cli, urls: Vec<String>) -> u32 {
    if urls.is_empty() {
        return 0;
    }

    // Create concurrent download manager with reasonable concurrency limit
    let max_concurrent = 4; // Can be made configurable later
    let download_manager = ConcurrentDownloadManager::new(max_concurrent);

    // Determine output directory
    let output_dir = args.directory_prefix.as_deref();

    // Start downloads
    let results = download_manager.download_urls(urls, output_dir).await;

    // Count failures and print summary
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;
    let total_bytes: u64 = results.iter().map(|r| r.bytes_downloaded).sum();

    println!("\nConcurrent Download Summary:");
    println!("  Successful: {}", successful);
    println!("  Failed: {}", failed);
    println!(
        "  Total bytes: {} ({:.2} MB)",
        total_bytes,
        total_bytes as f64 / 1_048_576.0
    );

    // Print failed downloads
    if failed > 0 {
        println!("\nFailed downloads:");
        for result in results.iter().filter(|r| !r.success) {
            println!(
                "  {} - {}",
                result.url,
                result
                    .error
                    .as_ref()
                    .unwrap_or(&"Unknown error".to_string())
            );
        }
    }

    failed as u32
}

fn determine_output_path(args: &cli::Cli, url: &str) -> PathBuf {
    if let Some(output) = &args.output {
        // -O flag: use specified filename
        return output.clone();
    }

    // Extract filename from URL
    let filename = utils::url::extract_filename(url);

    if let Some(dir) = &args.directory_prefix {
        // -P flag: save to specified directory
        dir.join(filename)
    } else {
        // Default: save to current directory
        PathBuf::from(filename)
    }
}
