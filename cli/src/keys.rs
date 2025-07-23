use {
    crate::workspace::find_workspace_root,
    anyhow::{anyhow, Context, Result},
    colored::Colorize,
    serde::{Deserialize, Serialize},
    solana_keypair::{read_keypair_file, write_keypair_file, Keypair},
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    std::{fs, path::Path, str::FromStr},
};

#[derive(Debug, Serialize, Deserialize)]
struct TyphoonConfig {
    workspace: WorkspaceSection,
    #[serde(default)]
    programs: std::collections::HashMap<String, ProgramConfig>,
    #[serde(default)]
    program_ids: std::collections::HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkspaceSection {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ProgramConfig {
    path: String,
}

/// Lists all program keys in the workspace
pub fn list() -> Result<()> {
    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow!("Not in a Typhoon workspace"))?;

    let typhoon_toml_path = workspace_root.join("typhoon.toml");
    let config: TyphoonConfig = if typhoon_toml_path.exists() {
        let content =
            fs::read_to_string(&typhoon_toml_path).context("Failed to read typhoon.toml")?;
        toml::from_str(&content).context("Failed to parse typhoon.toml")?
    } else {
        return Err(anyhow!("No typhoon.toml found in workspace"));
    };

    println!("{}", "Program Keys:".bold().cyan());
    println!();

    if config.programs.is_empty() {
        println!("  No programs found in workspace");
        return Ok(());
    }

    // Show program IDs from typhoon.toml
    if !config.program_ids.is_empty() {
        for (name, program_id) in &config.program_ids {
            println!("  {}: {}", name.bold(), program_id.bright_green());
        }
    }

    // Show programs without declared IDs (fallback to source file)
    let mut has_undeclared = false;
    for (name, program) in &config.programs {
        if !config.program_ids.contains_key(name) {
            if !has_undeclared {
                println!();
                println!("  {}:", "[from source]".bold().yellow());
                has_undeclared = true;
            }

            match read_program_id(&workspace_root.join(&program.path)) {
                Ok(program_id) => {
                    println!(
                        "  {}: {}",
                        name.bold(),
                        program_id.to_string().bright_green()
                    );
                }
                Err(e) => {
                    println!("  {}: {} ({})", name.bold(), "ERROR".red(), e);
                }
            }
        }
    }

    Ok(())
}

/// Syncs program IDs from keypair files to source code and typhoon.toml
pub fn sync(program_name: Option<String>) -> Result<()> {
    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow!("Not in a Typhoon workspace"))?;

    let typhoon_toml_path = workspace_root.join("typhoon.toml");
    let mut config: TyphoonConfig = if typhoon_toml_path.exists() {
        let content =
            fs::read_to_string(&typhoon_toml_path).context("Failed to read typhoon.toml")?;
        toml::from_str(&content).context("Failed to parse typhoon.toml")?
    } else {
        return Err(anyhow!("No typhoon.toml found in workspace"));
    };

    let programs_to_sync: Vec<_> = if let Some(name) = program_name {
        // Sync specific program
        config
            .programs
            .iter()
            .filter(|(prog_name, _)| prog_name == &&name)
            .collect()
    } else {
        // Sync all programs
        config.programs.iter().collect()
    };

    if programs_to_sync.is_empty() {
        return Err(anyhow!("No programs found to sync"));
    }

    println!("{}", "Syncing program keys...".bold().cyan());
    println!();

    for (name, program) in programs_to_sync {
        let program_path = workspace_root.join(&program.path);
        let keypair_path = program_path.join("keypair.json");

        // Read or create the keypair file
        let keypair = match read_keypair_file(&keypair_path) {
            Ok(kp) => kp,
            Err(_) => {
                println!(
                    "  {} No keypair file found for {}, generating new keypair...",
                    "⚠".yellow(),
                    name.bold()
                );

                // Ensure the program directory exists
                if let Some(parent) = keypair_path.parent() {
                    fs::create_dir_all(parent).with_context(|| {
                        format!(
                            "Failed to create directory for keypair: {}",
                            parent.display()
                        )
                    })?;
                }

                let new_keypair = Keypair::new();
                write_keypair_file(&new_keypair, &keypair_path).map_err(|e| {
                    anyhow!(
                        "Unable to create program keypair at {}: {}",
                        keypair_path.display(),
                        e
                    )
                })?;
                new_keypair
            }
        };

        let program_id = keypair.pubkey();

        // Update the program's lib.rs file with the new program ID
        update_program_id(&program_path, &program_id)?;

        // Update typhoon.toml with the new program ID
        config
            .program_ids
            .insert(name.clone(), program_id.to_string());

        println!(
            "  {} {} -> {}",
            "✓".green(),
            name.bold(),
            program_id.to_string().bright_green()
        );
    }

    // Write the updated configuration back to typhoon.toml
    let updated_content =
        toml::to_string_pretty(&config).context("Failed to serialize typhoon.toml")?;
    fs::write(&typhoon_toml_path, updated_content)
        .context("Failed to write updated typhoon.toml")?;

    println!();
    println!(
        "{}",
        "Sync complete! Program IDs updated in typhoon.toml"
            .bold()
            .green()
    );

    Ok(())
}

/// Reads the current program ID from the source file
fn read_program_id(program_path: &Path) -> Result<Pubkey> {
    let lib_path = program_path.join("src/lib.rs");
    let content = fs::read_to_string(&lib_path)
        .with_context(|| format!("Failed to read {}", lib_path.display()))?;

    // Look for program_id!("...") macro with optional semicolon
    let program_id_regex = regex::Regex::new(r#"program_id!\s*\(\s*"([^"]+)"\s*\)\s*;?"#)?;

    if let Some(captures) = program_id_regex.captures(&content) {
        let id_str = captures.get(1).unwrap().as_str();
        Pubkey::from_str(id_str).with_context(|| format!("Invalid program ID: {id_str}"))
    } else {
        Err(anyhow!(
            "No program_id! macro found in {}",
            lib_path.display()
        ))
    }
}

/// Updates the program ID in the source file
fn update_program_id(program_path: &Path, new_id: &Pubkey) -> Result<()> {
    let lib_path = program_path.join("src/lib.rs");
    let content = fs::read_to_string(&lib_path)
        .with_context(|| format!("Failed to read {}", lib_path.display()))?;

    // Replace program_id!("...") with the new ID, including optional semicolon
    let program_id_regex = regex::Regex::new(r#"program_id!\s*\(\s*"[^"]+"\s*\)\s*;?"#)?;
    let new_content = program_id_regex.replace(&content, format!("program_id!(\"{new_id}\");"));

    if new_content == content {
        return Err(anyhow!(
            "No program_id! macro found in {}",
            lib_path.display()
        ));
    }

    fs::write(&lib_path, new_content.as_ref())
        .with_context(|| format!("Failed to write {}", lib_path.display()))?;

    Ok(())
}
