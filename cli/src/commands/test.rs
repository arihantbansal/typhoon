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

    let is_workspace = utils::is_workspace()?;

    // Validate that programs have been built before running tests
    if !is_workspace {
        // For single programs, check that the specific binary exists
        let package_name = utils::get_package_name()?;
        // Solana replaces dashes with underscores in binary names
        let binary_name = package_name.replace('-', "_");
        let so_path = format!("{}/{}.so", constants::DEPLOY_DIR, binary_name);

        if !Path::new(&so_path).exists() {
            return Err(Error::ProgramNotBuilt(so_path));
        }
    } else {
        // For workspaces, check that at least one .so file exists
        let deploy_dir = Path::new(constants::DEPLOY_DIR);

        if !deploy_dir.exists() {
            return Err(Error::Other(anyhow::anyhow!(
                "target/deploy/ directory not found\n\n\
                Have you run 'typhoon build' yet?"
            )));
        }

        let has_programs = std::fs::read_dir(deploy_dir)
            .map_err(|e| {
                Error::Other(anyhow::anyhow!(
                    "failed to read target/deploy/ directory: {e}"
                ))
            })?
            .filter_map(|entry| entry.ok())
            .any(|entry| entry.path().extension().map_or(false, |ext| ext == "so"));

        if !has_programs {
            return Err(Error::Other(anyhow::anyhow!(
                "no program binaries found in target/deploy/\n\n\
                Have you run 'typhoon build' yet?"
            )));
        }
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
