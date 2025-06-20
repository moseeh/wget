use chrono::Utc;
use clap::Parser;
use std::path::PathBuf;

mod cli;
mod http;
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

    let client = http::HttpClient::new();

    // Handle the first URL for now (we'll expand this later)
    if let Some(url) = args.urls.first() {
        let file_path = determine_output_path(&args, url);

        match client.download_to_file(url, &file_path).await {
            Ok(_) => {
                let end_time = Utc::now();
                println!("finished at {}", end_time.format("%Y-%m-%d %H:%M:%S"));
            }
            Err(e) => {
                eprintln!("Download failed: {}", e);
                std::process::exit(1);
            }
        }
    }
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
