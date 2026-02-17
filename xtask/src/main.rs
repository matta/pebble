use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Hello world
    Hello,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Hello => println!("Hello from xtask!"),
    }
    Ok(())
}
