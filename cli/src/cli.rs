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
        /// Template to use (hello-world or counter)
        #[arg(short, long, default_value = "counter")]
        template: String,
        /// Create a workspace instead of a single program
        #[arg(short, long)]
        workspace: bool,
    },
    /// Add a program to the current workspace
    Add {
        #[command(subcommand)]
        command: AddCommands,
    },
    /// Build the Typhoon program
    Build,
    /// Run tests for the Typhoon program
    Test,
    /// Remove build artifacts
    Clean,
}

/// Add subcommands.
#[derive(Subcommand)]
enum AddCommands {
    /// Add a new program to the workspace
    Program {
        /// Name of the program to add
        name: String,
        /// Template to use (hello-world or counter)
        #[arg(short, long, default_value = "counter")]
        template: String,
    },
}

/// Run the CLI application.
pub fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            name,
            template,
            workspace,
        } => {
            if workspace {
                crate::commands::init::run_workspace(&name, &template)
            } else {
                crate::commands::init::run(&name, &template)
            }
        }
        Commands::Add { command } => match command {
            AddCommands::Program { name, template } => {
                crate::commands::add::run_program(&name, &template)
            }
        },
        Commands::Build => crate::commands::build::run(),
        Commands::Test => crate::commands::test::run(),
        Commands::Clean => crate::commands::clean::run(),
    }
}
