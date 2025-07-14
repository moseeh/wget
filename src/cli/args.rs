use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(name = "wget", about = "A simple wget clone", version = "0.1.0")]
pub struct Cli {
    /// URL(s) to download
    #[arg(help = "URL(s) to download")]
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

    /// Reject file suffixes during mirror (-R)
    #[arg(
        short = 'R',
        long,
        help = "Comma-separated list of file suffixes to reject"
    )]
    pub reject_suffixes: Option<String>,

    /// Exclude specific directories during mirror (-X)
    #[arg(
        short = 'X',
        long,
        help = "Comma-separated list of directories to exclude"
    )]
    pub exclude_dirs: Option<String>,

    /// Convert links in mirrored pages to offline-friendly versions (--convert-links)
    #[arg(long, help = "Convert links in mirrored files for offline viewing")]
    pub convert_links: bool,
}

impl Cli {
    pub fn validate(&self) -> Result<(), String> {
        // Must have either direct URLs or an input file
        if self.urls.is_empty() && self.input_file.is_none() {
            return Err("Provide at least one URL or use -i with a file containing URLs.".into());
        }

        // Input file must exist
        if let Some(file) = &self.input_file {
            if !file.exists() {
                return Err(format!("Input file {:?} does not exist", file));
            }
        }

        // Directory must exist if -P is used
        if let Some(dir) = &self.directory_prefix {
            if !dir.exists() || !dir.is_dir() {
                return Err(format!(
                    "Directory {:?} does not exist or is not a folder",
                    dir
                ));
            }
        }

        // Validate rate-limit format (e.g., 200k or 2M)
        if let Some(rate) = &self.rate_limit {
            let valid = rate.ends_with('k') || rate.ends_with('M');
            if !valid {
                return Err("Rate limit must end with 'k' or 'M' (e.g., 400k, 2M)".into());
            }
            let number_part = &rate[..rate.len() - 1];
            if number_part.parse::<u64>().is_err() {
                return Err("Rate limit must start with a valid number (e.g., 400k)".into());
            }
        }

        // Mirror mode validation
        if self.mirror {
            if self.urls.len() != 1 {
                return Err("Mirror mode requires exactly one URL".into());
            }
        } else {
            // These flags only make sense *with* mirror
            if self.reject_suffixes.is_some() || self.exclude_dirs.is_some() || self.convert_links {
                return Err(
                    "Flags -R, -X, and --convert-links can only be used with --mirror".into(),
                );
            }
        }

        Ok(())
    }
}
