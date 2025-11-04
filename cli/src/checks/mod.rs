//! Prerequisite checks for development environment.

pub mod solana;

use crate::Result;

/// Runs all prerequisite checks.
pub fn run_all() -> Result<()> {
    solana::check_installed()?;
    Ok(())
}
