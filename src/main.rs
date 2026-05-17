use chrono::Local;
use clap::Parser;
use download::ConcurrentDownloadManager;
use mirror::MirrorCrawler;
use std::path::PathBuf;

mod cli;
mod download;
mod http;
mod io;
mod mirror;
mod rate;
mod resume;
mod utils;

const BACKGROUND_ENV: &str = "WGET_IN_BACKGROUND";

#[tokio::main]
async fn main() {
    let args = cli::Cli::parse();
    if let Err(e) = args.validate() {
        eprintln!("Argument error: {}", e);
        std::process::exit(1);
    }

    // -B handling: parent process re-execs self under setsid with stdout/stderr
    // redirected to wget-log, then returns to the shell.
    if args.background && std::env::var_os(BACKGROUND_ENV).is_none() {
        match spawn_background() {
            Ok(()) => {
                println!("Output will be written to \"wget-log\".");
                return;
            }
            Err(e) => {
                eprintln!("Failed to start background process: {}", e);
                std::process::exit(1);
            }
        }
    }

    // In background-child mode the progress bar would just spam the log file
    // with carriage-return redraws — suppress it.
    let in_background = std::env::var_os(BACKGROUND_ENV).is_some();
    let show_progress = !in_background;

    let start_time = Local::now();
    println!("start at {}", start_time.format("%Y-%m-%d %H:%M:%S"));

    let mut failed_downloads = 0;

    if args.mirror {
        if let Err(e) = process_mirror_mode(&args).await {
            eprintln!("Mirror failed: {}", e);
            std::process::exit(1);
        }
    } else {
        if !args.urls.is_empty() {
            failed_downloads += process_urls_sequentially(&args, &args.urls, show_progress).await;
        }

        if let Some(input_file) = &args.input_file {
            match io::read_urls_from_file(input_file).await {
                Ok(file_urls) => {
                    if !file_urls.is_empty() {
                        failed_downloads += process_urls_concurrently(&args, file_urls).await;
                    }
                }
                Err(e) => {
                    eprintln!("Error reading URLs from file: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }

    let end_time = Local::now();
    println!("finished at {}", end_time.format("%Y-%m-%d %H:%M:%S"));

    if failed_downloads > 0 {
        std::process::exit(1);
    }
}

fn spawn_background() -> std::io::Result<()> {
    use std::os::unix::process::CommandExt;
    use std::process::{Command, Stdio};

    let exe = std::env::current_exe()?;
    let args: Vec<String> = std::env::args().skip(1).collect();

    let log = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("wget-log")?;
    let log_err = log.try_clone()?;

    let mut cmd = Command::new(exe);
    cmd.args(&args)
        .env(BACKGROUND_ENV, "1")
        .stdin(Stdio::null())
        .stdout(Stdio::from(log))
        .stderr(Stdio::from(log_err));

    unsafe {
        cmd.pre_exec(|| {
            // Detach from the controlling terminal so the child survives
            // after the parent (the shell-invoked process) exits.
            if libc::setsid() < 0 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }

    cmd.spawn()?;
    Ok(())
}

async fn process_urls_sequentially(args: &cli::Cli, urls: &[String], show_progress: bool) -> u32 {
    let client = http::HttpClient::with_config(
        args.rate_limit.as_deref(),
        args.user_agent.clone(),
        args.timeout,
    );
    let mut failed_count = 0;

    for url in urls {
        let file_path = determine_output_path(args, url);

        if let Err(e) = client
            .download_to_file_with_resume(url, &file_path, args.continue_download, show_progress)
            .await
        {
            eprintln!("Download failed for [{}]: {}", url, e);
            failed_count += 1;
        }
    }

    failed_count
}

async fn process_urls_concurrently(args: &cli::Cli, urls: Vec<String>) -> u32 {
    if urls.is_empty() {
        return 0;
    }

    let download_manager = ConcurrentDownloadManager::with_config(
        4,
        args.rate_limit.as_deref(),
        args.user_agent.clone(),
        args.timeout,
    );

    let output_dir = args.directory_prefix.as_deref();
    let results = download_manager.download_urls(urls, output_dir).await;

    let urls_str: Vec<&str> = results.iter().map(|r| r.url.as_str()).collect();
    println!("\nDownload finished: {:?}", urls_str);

    results.iter().filter(|r| !r.success).count() as u32
}

async fn process_mirror_mode(args: &cli::Cli) -> Result<(), Box<dyn std::error::Error>> {
    let url = &args.urls[0];
    let mut crawler = MirrorCrawler::new(url, args.directory_prefix.as_deref())?;
    crawler
        .mirror(&args.reject_suffixes, &args.exclude_dirs, args.convert_links)
        .await?;
    Ok(())
}

fn determine_output_path(args: &cli::Cli, url: &str) -> PathBuf {
    if let Some(output) = &args.output {
        return output.clone();
    }

    let filename = utils::url::extract_filename(url);

    if let Some(dir) = &args.directory_prefix {
        dir.join(filename)
    } else {
        PathBuf::from(filename)
    }
}
