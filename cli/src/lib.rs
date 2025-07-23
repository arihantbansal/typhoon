//! Typhoon CLI library
//! Defines command line interface structure and exports all CLI modules

use {
    clap::{Parser, Subcommand},
    std::path::PathBuf,
};

/// Version string extracted from Cargo.toml at compile time
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main CLI structure that defines the typhoon command and its subcommands
#[derive(Debug, Parser)]
#[command(name = "typhoon")]
#[command(about = "Typhoon CLI - A Solana Sealevel Framework", long_about = None)]
#[command(version = VERSION)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

/// All available top-level commands in the Typhoon CLI
#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create a new Typhoon workspace or program
    New {
        /// Name of the workspace or program
        name: String,
        /// Create a single program (default is workspace)
        #[arg(short, long)]
        program: bool,
        /// Template to use (defaults to hello-world)
        #[arg(short, long)]
        template: Option<String>,
        /// Use a remote template repository
        #[arg(long)]
        from: Option<String>,
    },
    /// Add a new program or instruction to existing workspace
    Add {
        #[command(subcommand)]
        subcommand: AddSubcommand,
    },
    /// Build the workspace or specific program
    Build {
        /// Program to build (builds all if not specified)
        #[arg(short, long)]
        program: Option<String>,
        /// Generate IDL after building
        #[arg(long)]
        idl: bool,
    },
    /// Run tests
    Test {
        /// Program to test (tests all if not specified)
        #[arg(short, long)]
        program: Option<String>,
        /// Run specific test
        #[arg(short, long)]
        test: Option<String>,
    },
    /// Security checks and verifiable build
    Security {
        #[command(subcommand)]
        subcommand: SecuritySubcommand,
    },
    /// Generate client bindings
    GenerateBindings {
        /// Languages to generate bindings for
        #[arg(short, long, value_delimiter = ',')]
        languages: Vec<String>,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Generate IDL files
    Idl {
        /// Program to generate IDL for (generates all if not specified)
        #[arg(short, long)]
        program: Option<String>,
    },
    /// Deploy program(s)
    Deploy {
        /// Program to deploy (deploys all if not specified)
        #[arg(short, long)]
        program: Option<String>,
        /// Network to deploy to
        #[arg(short, long, default_value = "localnet")]
        network: String,
    },
}

/// Subcommands for the 'add' command
#[derive(Debug, Subcommand)]
pub enum AddSubcommand {
    /// Add a new program to the workspace
    Program {
        /// Name of the program
        name: String,
        /// Template to use
        #[arg(short, long)]
        template: Option<String>,
    },
    /// Add a new instruction to a program
    Instruction {
        /// Program to add instruction to
        program: String,
        /// Name of the instruction
        name: String,
    },
}

/// Subcommands for the 'security' command
#[derive(Debug, Subcommand)]
pub enum SecuritySubcommand {
    /// Run security audit on dependencies
    Audit,
    /// Run verifiable build
    Verify {
        /// Program to verify (verifies all if not specified)
        #[arg(short, long)]
        program: Option<String>,
        /// Repository URL to verify against
        #[arg(long)]
        repo_url: Option<String>,
        /// Git commit hash to verify against
        #[arg(long)]
        commit_hash: Option<String>,
        /// Use current directory for verification
        #[arg(long)]
        current_dir: bool,
        /// Program ID to verify against deployed program
        #[arg(long)]
        program_id: Option<String>,
        /// Solana cluster to verify against
        #[arg(long, default_value = "mainnet-beta")]
        cluster: String,
    },
    /// Verify deployed program matches repository
    VerifyRepo {
        /// Repository URL
        repo_url: String,
        /// Program ID on chain
        program_id: String,
        /// Git commit hash (defaults to latest)
        #[arg(long)]
        commit_hash: Option<String>,
        /// Solana cluster
        #[arg(long, default_value = "mainnet-beta")]
        cluster: String,
        /// Mount path for verification
        #[arg(long)]
        mount_path: Option<String>,
    },
}

pub mod bindings;
pub mod build;
pub mod scaffold;
pub mod security;
pub mod templates;
pub mod test;
pub mod validation;
pub mod workspace;
