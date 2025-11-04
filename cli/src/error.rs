//! Structured error types for the CLI.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("directory '{0}' already exists")]
    DirectoryExists(String),

    #[error("invalid project name: {0}")]
    InvalidProjectName(String),

    #[error("Cargo.toml not found in current directory")]
    NotInProject,

    #[error(
        "Solana CLI tools not installed\n\nInstall them with:\n  sh -c \"$(curl -sSfL {url})\""
    )]
    SolanaNotInstalled { url: String },

    #[error("build failed: {0}")]
    BuildFailed(String),

    #[error("tests failed: {0}")]
    TestsFailed(String),

    #[error("program binary not found at: {0}\n\nHave you run 'typhoon build' yet?")]
    ProgramNotBuilt(String),

    #[error("template '{0}' not found")]
    TemplateNotFound(String),

    #[error("invalid Cargo.toml: {0}")]
    InvalidCargoToml(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    TomlParse(#[from] toml::de::Error),

    #[error(transparent)]
    JsonSerialize(#[from] serde_json::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
