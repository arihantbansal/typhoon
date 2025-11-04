//! Solana CLI toolchain checks.

use {
    crate::{
        constants::{DEPLOY_DIR, SOLANA_INSTALL_URL},
        Error, Result,
    },
    std::process::Command,
};

/// Checks if Solana CLI tools are installed.
///
/// # Errors
/// Returns an error with installation instructions if Solana CLI is not found.
pub fn check_installed() -> Result<()> {
    let output = Command::new("cargo")
        .arg("build-sbf")
        .arg("--version")
        .output();

    match output {
        Ok(output) if output.status.success() => Ok(()),
        _ => Err(Error::SolanaNotInstalled {
            url: SOLANA_INSTALL_URL.to_string(),
        }),
    }
}

/// Gets the Solana CLI version if installed.
pub fn get_version() -> Result<String> {
    let output = Command::new("cargo")
        .arg("build-sbf")
        .arg("--version")
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(Error::SolanaNotInstalled {
            url: SOLANA_INSTALL_URL.to_string(),
        })
    }
}

/// Builds the Solana program using cargo build-sbf.
///
/// # Errors
/// Returns an error if the build command fails.
pub fn build() -> Result<()> {
    println!("Building Solana program...\n");

    let status = Command::new("cargo")
        .arg("build-sbf")
        .status()
        .map_err(|e| Error::Other(anyhow::anyhow!("failed to execute 'cargo build-sbf': {e}")))?;

    if !status.success() {
        return Err(Error::BuildFailed(
            "check output above for details".to_string(),
        ));
    }

    println!("\nBuild successful.");
    println!("Program binary location: {DEPLOY_DIR}/<program-name>.so");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Only works if Solana CLI is installed
    fn test_check_installed() {
        assert!(check_installed().is_ok());
    }

    #[test]
    #[ignore] // Only works if Solana CLI is installed
    fn test_get_version() {
        let version = get_version().unwrap();
        assert!(!version.is_empty());
    }
}
