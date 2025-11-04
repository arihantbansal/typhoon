//! Test execution command.

use {
    crate::{constants, utils, Error, Result},
    std::{path::Path, process::Command},
};

/// Runs integration tests for the Typhoon program.
///
/// # Errors
/// Returns an error if not in a Rust project, the program is not built,
/// or tests fail.
pub fn run() -> Result<()> {
    utils::check_rust_project()?;

    let package_name = utils::get_package_name()?;
    // Solana replaces dashes with underscores in binary names
    let binary_name = package_name.replace('-', "_");
    let so_path = format!("{}/{}.so", constants::DEPLOY_DIR, binary_name);

    if !Path::new(&so_path).exists() {
        return Err(Error::ProgramNotBuilt(so_path));
    }

    println!("Running tests...\n");

    let status = Command::new("cargo")
        .arg("test")
        .arg("--")
        .arg("--nocapture")
        .status()
        .map_err(|e| Error::Other(anyhow::anyhow!("failed to execute 'cargo test': {e}")))?;

    if !status.success() {
        return Err(Error::Other(anyhow::anyhow!(
            "tests failed. See output above for details"
        )));
    }

    println!("\nAll tests passed.");

    Ok(())
}
