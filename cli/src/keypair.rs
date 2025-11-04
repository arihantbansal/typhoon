//! Keypair generation utilities.

use {
    crate::{
        constants::{KEYPAIR_DIR, PROGRAM_KEYPAIR_FILE},
        Error, Result,
    },
    solana_keypair::{Keypair, Signer},
    std::{fs, path::Path},
};

/// Generates a new Solana keypair and saves it to the project's .keypairs directory.
///
/// Returns the base58-encoded program ID derived from the keypair's public key.
///
/// # Errors
/// Returns an error if directory or file creation fails.
pub fn generate_program_keypair(project_path: &Path) -> Result<String> {
    let keypair = Keypair::new();
    let program_id = keypair.pubkey().to_string();

    let keypair_dir = project_path.join(KEYPAIR_DIR);
    fs::create_dir_all(&keypair_dir).map_err(|e| {
        Error::Other(anyhow::anyhow!(
            "failed to create {KEYPAIR_DIR} directory: {e}"
        ))
    })?;

    let keypair_path = keypair_dir.join(PROGRAM_KEYPAIR_FILE);
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

    println!("  Generated keypair at: {KEYPAIR_DIR}/{PROGRAM_KEYPAIR_FILE}");
    println!("  Program ID: {program_id}");

    Ok(program_id)
}
