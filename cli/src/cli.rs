//! CLI definition and command routing.

use {
    crate::Result,
    clap::{Parser, Subcommand},
};

/// Typhoon CLI entry point.
#[derive(Parser)]
#[command(name = "typhoon")]
#[command(about = "Typhoon Solana Framework CLI")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Available CLI commands.
#[derive(Subcommand)]
enum Commands {
    /// Create a new Typhoon program
    Init {
        /// Name of the program to create
        name: String,
    },
    /// Build the Typhoon program
    Build,
    /// Run tests for the Typhoon program
    Test,
    /// Remove build artifacts
    Clean,
}

/// Run the CLI application.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { name } => crate::commands::init::run(&name),
        Commands::Build => crate::commands::build::run(),
        Commands::Test => crate::commands::test::run(),
        Commands::Clean => crate::commands::clean::run(),
    }
}
