use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};

mod build;
mod git;
mod metadata;
mod signing;
mod tag;
mod verify;

#[derive(Debug, Parser)]
#[command(name = "bin-release")]
#[command(about = "Release tool for lingting-ai-gateway")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Verify release metadata, signatures, repositories and dependencies.
    Verify,

    /// Create and push a release tag.
    Tag,

    /// Build a release artifact.
    Build {
        #[arg(value_enum)]
        platform: Platform,

        /// Build the slim/dynamically linked variant.
        #[arg(short, long)]
        slim: bool,
    },

}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Platform {
    Linux,
    Windows,
    Macos,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Verify => verify::run(),
        Command::Tag => tag::run(),
        Command::Build { platform, slim } => build::run(platform, slim),
    }

}