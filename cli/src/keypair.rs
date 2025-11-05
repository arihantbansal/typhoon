//! Keypair generation utilities.

use {
    crate::{constants::DEPLOY_DIR, Error, Result},
    solana_keypair::{Keypair, Signer},
    std::{fs, path::Path},
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

    fs::write(&keypair_path, keypair_json).map_err(|e| {
        Error::Other(anyhow::anyhow!(
            "failed to write keypair file to {}: {}",
            keypair_path.display(),
            e
        ))
    })?;

    println!("  Generated keypair at: {DEPLOY_DIR}/{keypair_filename}");
    println!("  Program ID: {program_id}");

    Ok(program_id)
}
