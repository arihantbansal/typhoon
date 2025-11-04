//! Program build command.

use crate::{checks, output, utils, Result};

/// Builds the Typhoon program using cargo build-sbf.
///
/// # Errors
/// Returns an error if not in a Rust project, Solana CLI is not installed,
/// or the build fails.
pub fn run() -> Result<()> {
    utils::check_rust_project()?;
    checks::solana::check_installed()?;

    if !utils::has_typhoon_dependency()? {
        output::warning("This doesn't appear to be a Typhoon project");
    }

    checks::solana::build()?;

    Ok(())
}
