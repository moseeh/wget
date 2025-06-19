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
}
