//! Build system for Typhoon programs
//! Handles compilation of Solana programs and IDL generation

use {
    crate::workspace::find_workspace_root,
    anyhow::{Context, Result},
    colored::Colorize,
    indicatif::{ProgressBar, ProgressStyle},
    std::{path::Path, process::Command},
};

/// Build programs in the workspace
/// Optionally builds a specific program or generates IDL files
pub async fn build(program: Option<&str>, generate_idl: bool) -> Result<()> {
    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow::anyhow!("Not in a Typhoon workspace"))?;

    // Automatically sync keys before building
    println!("{} Syncing program keys before build...", "▶".blue().bold());
    if let Err(e) = crate::keys::sync(program.map(|s| s.to_string())) {
        eprintln!("Warning: Failed to sync keys before build: {e}");
    }

    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    // Prefer cargo-make for builds when available
    let use_cargo_make = Command::new("cargo")
        .args(["make", "--version"])
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false);

    if let Some(program_name) = program {
        // Build specific program
        progress.set_message(format!("Building program '{program_name}'..."));
        if use_cargo_make {
            build_program_with_cargo_make(&workspace_root, program_name, &progress)?;
        } else {
            build_program(&workspace_root, program_name, &progress)?;
        }
    } else {
        // Build all programs
        progress.set_message("Building all programs...");
        if use_cargo_make {
            build_all_programs_with_cargo_make(&workspace_root, &progress)?;
        } else {
            build_all_programs(&workspace_root, &progress)?;
        }
    }

    if generate_idl {
        progress.set_message("Generating IDL files...");
        generate_idl_files(&workspace_root, program).await?;
    }

    progress.finish_and_clear();
    Ok(())
}

fn build_program(workspace_root: &Path, program_name: &str, progress: &ProgressBar) -> Result<()> {
    let program_path = workspace_root.join("programs").join(program_name);

    if !program_path.exists() {
        anyhow::bail!("Program '{}' not found", program_name);
    }

    progress.set_message(format!("Compiling {program_name}..."));

    // Check if cargo build-sbf is available
    if Command::new("cargo")
        .args(["--list"])
        .output()
        .map(|out| !String::from_utf8_lossy(&out.stdout).contains("build-sbf"))
        .unwrap_or(true)
    {
        anyhow::bail!(
            "cargo build-sbf not found. Please install with:\n\
             sh -c \"$(curl -sSfL https://release.anza.xyz/stable/install)\"\n\
             OR\n\
             cargo install solana-cli"
        );
    }

    let output = Command::new("cargo")
        .args(["build-sbf"])
        .current_dir(&program_path)
        .output()
        .context("Failed to execute cargo build-sbf")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        if stderr.contains("typhoon") && stderr.contains("not found") {
            anyhow::bail!(
                "Typhoon framework not found. This might be because:\n\
                 1. Typhoon is still in development and not published to crates.io\n\
                 2. You need to build Typhoon locally first\n\
                 \n\
                 Original error: {}",
                stderr
            );
        }

        anyhow::bail!("Build failed for {}:\n{}\n{}", program_name, stdout, stderr);
    }

    // Copy the built program to workspace target directory
    let built_so = program_path
        .join("target")
        .join("deploy")
        .join(format!("{}.so", program_name.replace("-", "_")));

    if built_so.exists() {
        let workspace_deploy = workspace_root.join("target").join("deploy");
        std::fs::create_dir_all(&workspace_deploy)?;

        std::fs::copy(
            &built_so,
            workspace_deploy.join(built_so.file_name().unwrap()),
        )?;
    }

    Ok(())
}

fn build_all_programs(workspace_root: &Path, progress: &ProgressBar) -> Result<()> {
    let programs_dir = workspace_root.join("programs");

    if !programs_dir.exists() {
        anyhow::bail!("No programs directory found");
    }

    let programs: Vec<_> = std::fs::read_dir(&programs_dir)?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                if e.path().is_dir() {
                    e.file_name().into_string().ok()
                } else {
                    None
                }
            })
        })
        .collect();

    if programs.is_empty() {
        println!("{} No programs found to build", "!".yellow());
        return Ok(());
    }

    // Configure progress bar for multiple programs
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:30.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    progress.set_length(programs.len() as u64);

    for (i, program_name) in programs.iter().enumerate() {
        progress.set_position(i as u64);
        progress.set_message(format!("Building {program_name}"));
        build_program(workspace_root, program_name, progress)?;
    }

    progress.finish_with_message(format!("Built {} programs", programs.len()));
    println!("{} Built {} programs", "✓".green(), programs.len());
    Ok(())
}

/// Generate IDL files for programs
/// Entry point for the idl command
pub async fn generate_idl(program: Option<&str>) -> Result<()> {
    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow::anyhow!("Not in a Typhoon workspace"))?;

    generate_idl_files(&workspace_root, program).await
}

/// Generate IDL files for specified program or all programs
async fn generate_idl_files(workspace_root: &Path, program: Option<&str>) -> Result<()> {
    let idl_dir = workspace_root.join("target").join("idl");
    std::fs::create_dir_all(&idl_dir)?;

    if let Some(program_name) = program {
        generate_program_idl(workspace_root, program_name, &idl_dir)?;
    } else {
        // Generate IDL for all programs
        let programs_dir = workspace_root.join("programs");

        if programs_dir.exists() {
            for entry in std::fs::read_dir(&programs_dir)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    if let Some(program_name) = entry.file_name().to_str() {
                        generate_program_idl(workspace_root, program_name, &idl_dir)?;
                    }
                }
            }
        }
    }

    Ok(())
}

/// Generate IDL for a specific program by running its build.rs
fn generate_program_idl(workspace_root: &Path, program_name: &str, idl_dir: &Path) -> Result<()> {
    let program_path = workspace_root.join("programs").join(program_name);

    // Execute cargo build to trigger IDL generation via build.rs
    let output = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(&program_path)
        .output()
        .context("Failed to generate IDL")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("IDL generation failed for {}: {}", program_name, stderr);
    }

    // Move generated IDL to central workspace location
    let program_idl = program_path
        .join("target")
        .join("idl")
        .join(format!("{}.json", program_name.replace("-", "_")));

    if program_idl.exists() {
        std::fs::copy(&program_idl, idl_dir.join(program_idl.file_name().unwrap()))?;
    }

    Ok(())
}

/// Build a single program using cargo-make
fn build_program_with_cargo_make(
    workspace_root: &Path,
    program_name: &str,
    progress: &ProgressBar,
) -> Result<()> {
    let program_path = workspace_root.join("programs").join(program_name);

    if !program_path.exists() {
        anyhow::bail!("Program '{}' not found", program_name);
    }

    progress.set_message(format!("Building {program_name} with cargo-make..."));

    let output = Command::new("cargo")
        .args(["make", "build"])
        .current_dir(&program_path)
        .output()
        .context("Failed to execute cargo make build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!("Build failed for {}:\n{}\n{}", program_name, stdout, stderr);
    }

    Ok(())
}

/// Build all programs using cargo-make at workspace level
fn build_all_programs_with_cargo_make(workspace_root: &Path, progress: &ProgressBar) -> Result<()> {
    progress.set_message("Building all programs with cargo-make...");

    let output = Command::new("cargo")
        .args(["make", "build"])
        .current_dir(workspace_root)
        .output()
        .context("Failed to execute cargo make build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!("Build failed:\n{}\n{}", stdout, stderr);
    }

    println!("{} Built all programs with cargo-make", "✓".green());
    Ok(())
}
