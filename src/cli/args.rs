use clap::{Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "wget", about = "A simple wget clone", version = "0.1.0")]
pub struct Cli {
    /// URL(s) to download
    #[arg(required = true, help = "URL(s) to download")]
    pub urls: Vec<String>,

    /// Save file as a specific name (-O)
    #[arg(
        short = 'O',
        long,
        help = "Write documents to file",
        conflicts_with = "directory_prefix"
    )]
    pub output: Option<PathBuf>,
    
    /// Directory to save the downloaded file (-P)
    #[arg(
        short = 'P',
        long,
        help = "Directory prefix to save files to",
        conflicts_with = "output"
    )]
    pub directory_prefix: Option<PathBuf>,

    /// Download in background (-B)
    #[arg(
        short = 'B',
        long,
        help = "Download in background (output to wget-log)"
    )]
    pub background: bool,

    /// Limit download rate (--rate-limit=200k/2M)
    #[arg(long, help = "Limit download rate (e.g., 200k, 2M)")]
    pub rate_limit: Option<String>,

    /// Input file with list of URLs to download (-i)
    #[arg(short = 'i', long, help = "File containing list of URLs to download")]
    pub input_file: Option<PathBuf>,

    /// Enable mirror mode (--mirror)
    #[arg(long, help = "Enable website mirroring")]
    pub mirror: bool,
}
