use clap::Parser;

mod cli;

fn main() {
    let args = cli::Cli::parse();
    if let Err(e) = args.validate() {
        eprintln!("Argument error: {}", e);
        std::process::exit(1);
    }
    println!("CLI parsed and validated successfully!");
    println!("{:#?}", args);
}
