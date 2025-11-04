//! Clean command to remove build artifacts.

use {
    crate::{output, utils, Result},
    std::process::Command,
};

/// Cleans build artifacts and target directory.
///
/// # Errors
/// Returns an error if not in a Rust project or clean command fails.
pub fn run() -> Result<()> {
    utils::check_rust_project()?;

    output::info("Cleaning build artifacts...");

    let status = Command::new("cargo").arg("clean").status().map_err(|e| {
        crate::Error::Other(anyhow::anyhow!("failed to execute 'cargo clean': {e}"))
    })?;

    if !status.success() {
        return Err(crate::Error::Other(anyhow::anyhow!(
            "clean failed. See output above for details"
        )));
    }

    output::success("Build artifacts removed");

    Ok(())
}
