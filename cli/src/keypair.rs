//! Keypair generation utilities.

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use {
    crate::{constants::DEPLOY_DIR, Error, Result},
    solana_keypair::{Keypair, Signer},
    std::{fs, io::Write, path::Path},
};

/// Generates a new Solana keypair and saves it to the project's target/deploy directory.
///
/// This follows Solana's standard convention where `cargo build-sbf` expects keypairs
/// to be located at `target/deploy/{program_name}-keypair.json`.
///
/// Returns the base58-encoded program ID derived from the keypair's public key.
///
/// # Errors
/// Returns an error if directory or file creation fails.
pub fn generate_program_keypair(project_path: &Path, project_name: &str) -> Result<String> {
    let keypair = Keypair::new();
    let program_id = keypair.pubkey().to_string();

    let deploy_dir = project_path.join(DEPLOY_DIR);
    fs::create_dir_all(&deploy_dir).map_err(|e| {
        Error::Other(anyhow::anyhow!(
            "failed to create {DEPLOY_DIR} directory: {e}"
        ))
    })?;

    // Convert dashes to underscores to match Solana's binary naming convention
    let binary_name = project_name.replace('-', "_");
    let keypair_filename = format!("{binary_name}-keypair.json");
    let keypair_path = deploy_dir.join(&keypair_filename);

    let keypair_bytes: Vec<u8> = keypair.to_bytes().to_vec();
    let keypair_json = serde_json::to_string(&keypair_bytes)
        .map_err(|e| Error::Other(anyhow::anyhow!("failed to serialize keypair: {e}")))?;

    // Write keypair with restrictive permissions for security
    write_keypair_securely(&keypair_path, &keypair_json)?;

    println!("  Generated keypair at: {DEPLOY_DIR}/{keypair_filename}");
    println!("  Program ID: {program_id}");

    Ok(program_id)
}

/// Writes a keypair file with restrictive permissions.
///
/// On Unix systems, creates the file with mode 0600 (owner read/write only).
/// On Windows, uses default permissions (Windows file permissions are more complex
/// and typically handled via ACLs at a higher level).
///
/// # Errors
/// Returns an error if file creation or writing fails.
fn write_keypair_securely(keypair_path: &Path, keypair_json: &str) -> Result<()> {
    #[cfg(unix)]
    {
        use std::fs::OpenOptions;

        // Create file with restrictive permissions (0600 = owner read/write only)
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(keypair_path)
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    Error::Other(anyhow::anyhow!(
                        "keypair file already exists at {}\n\n\
                        Remove the existing file or use a different project name.",
                        keypair_path.display()
                    ))
                } else {
                    Error::Other(anyhow::anyhow!(
                        "failed to create keypair file at {}: {}",
                        keypair_path.display(),
                        e
                    ))
                }
            })?;

        file.write_all(keypair_json.as_bytes()).map_err(|e| {
            Error::Other(anyhow::anyhow!(
                "failed to write keypair data to {}: {}",
                keypair_path.display(),
                e
            ))
        })?;
    }

    #[cfg(not(unix))]
    {
        // On Windows, use standard write. Windows permissions are typically
        // managed through ACLs which require more complex setup.
        // For now, rely on default NTFS permissions which restrict to user.
        fs::write(keypair_path, keypair_json).map_err(|e| {
            Error::Other(anyhow::anyhow!(
                "failed to write keypair file to {}: {}",
                keypair_path.display(),
                e
            ))
        })?;
    }

    Ok(())
}
