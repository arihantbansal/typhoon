//! Security tools for Typhoon programs
//! Provides dependency auditing and verifiable build capabilities

use {
    crate::workspace::find_workspace_root,
    anyhow::{Context, Result},
    colored::Colorize,
    indicatif::{ProgressBar, ProgressStyle},
    std::{
        path::Path,
        process::{Command, Stdio},
    },
};

/// Run security audit on workspace dependencies
pub async fn run_audit() -> Result<()> {
    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow::anyhow!("Not in a Typhoon workspace"))?;

    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    progress.set_message("Running cargo audit...");
    let audit_passed = run_cargo_audit(&workspace_root)?;

    progress.set_message("Checking for common vulnerabilities...");
    let vuln_check_passed = check_common_vulnerabilities(&workspace_root)?;

    progress.finish_and_clear();

    if !audit_passed || !vuln_check_passed {
        anyhow::bail!("Security audit failed");
    }

    Ok(())
}

pub async fn run_verify(
    program: Option<&str>,
    repo_url: Option<&str>,
    commit_hash: Option<&str>,
    current_dir: bool,
    program_id: Option<&str>,
    cluster: &str,
) -> Result<()> {
    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow::anyhow!("Not in a Typhoon workspace"))?;

    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    // If program_id is provided, verify against deployed program
    if let Some(program_id) = program_id {
        progress.set_message(format!(
            "Verifying against deployed program {program_id}..."
        ));
        verify_deployment(program_id, cluster).await?;
    } else {
        // Run verifiable build
        progress.set_message("Running verifiable build...");
        run_verifiable_build(
            &workspace_root,
            program,
            repo_url,
            commit_hash,
            current_dir,
            &progress,
        )
        .await?;
    }

    progress.finish_and_clear();
    Ok(())
}

pub async fn verify_from_repo(
    repo_url: &str,
    program_id: &str,
    commit_hash: Option<&str>,
    cluster: &str,
    mount_path: Option<&str>,
) -> Result<()> {
    // Ensure solana-verify is installed
    ensure_solana_verify_installed()?;

    println!(
        "{} Verifying program {} against repository {}...",
        "▶".blue(),
        program_id,
        repo_url
    );

    let mut args = vec![
        "verify-from-repo",
        "--url",
        cluster,
        "--program-id",
        program_id,
        repo_url,
    ];

    // Add optional commit hash
    if let Some(commit) = commit_hash {
        args.extend_from_slice(&["--commit-hash", commit]);
    }

    // Add optional mount path
    if let Some(path) = mount_path {
        args.extend_from_slice(&["--mount-path", path]);
    }

    let output = Command::new("solana-verify")
        .args(&args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .context("Failed to execute solana-verify")?;

    if !output.status.success() {
        anyhow::bail!("Program verification failed");
    }

    println!(
        "{} Program successfully verified against repository",
        "✓".green()
    );
    Ok(())
}

fn check_solana_verify_installed() -> bool {
    Command::new("solana-verify")
        .arg("--version")
        .output()
        .map(|out| out.status.success())
        .unwrap_or(false)
}

fn install_solana_verify() -> Result<()> {
    println!("{} solana-verify not found. Installing...", "!".yellow());

    let output = Command::new("cargo")
        .args([
            "install",
            "solana-verify",
            "--git",
            "https://github.com/Ellipsis-Labs/solana-verifiable-build",
            "--rev",
            "568cb334709e88b9b45fc24f1f440eecacf5db54",
            "--force",
            "--locked",
        ])
        .output()
        .context("Failed to execute cargo install solana-verify")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        anyhow::bail!(
            "Failed to install solana-verify:\nSTDOUT: {}\nSTDERR: {}",
            stdout,
            stderr
        );
    }

    println!("{} solana-verify installed successfully", "✓".green());
    Ok(())
}

fn ensure_solana_verify_installed() -> Result<()> {
    if !check_solana_verify_installed() {
        install_solana_verify()?;

        // Verify installation worked
        if !check_solana_verify_installed() {
            anyhow::bail!(
                "solana-verify installation failed. Please install manually with:\n\
                 cargo install solana-verify --git https://github.com/Ellipsis-Labs/solana-verifiable-build \
                 --rev 568cb334709e88b9b45fc24f1f440eecacf5db54 --force --locked"
            );
        }
    }
    Ok(())
}

async fn run_verifiable_build(
    workspace_root: &Path,
    program: Option<&str>,
    repo_url: Option<&str>,
    commit_hash: Option<&str>,
    current_dir: bool,
    progress: &ProgressBar,
) -> Result<()> {
    // Ensure solana-verify is installed
    ensure_solana_verify_installed()?;

    let programs_dir = workspace_root.join("programs");
    if !programs_dir.exists() {
        anyhow::bail!("No programs directory found");
    }

    let programs_to_verify = if let Some(program_name) = program {
        vec![program_name.to_string()]
    } else {
        // Get all programs
        std::fs::read_dir(&programs_dir)?
            .filter_map(|entry| {
                entry.ok().and_then(|e| {
                    if e.path().is_dir() {
                        e.file_name().into_string().ok()
                    } else {
                        None
                    }
                })
            })
            .collect()
    };

    for program_name in programs_to_verify {
        progress.set_message(format!("Verifying {program_name}..."));

        let program_path = programs_dir.join(&program_name);

        let mut args = vec!["build"];

        // Add library name
        let library_name = program_name.replace("-", "_");
        args.extend_from_slice(&["--library-name", &library_name]);

        // Handle verification source
        match (current_dir, repo_url) {
            (true, _) => {
                // Use current directory - no additional args needed
            }
            (false, Some(url)) => {
                args.extend_from_slice(&["--repository-url", url]);
                if let Some(commit) = commit_hash {
                    args.extend_from_slice(&["--commit-hash", commit]);
                }
            }
            (false, None) => {
                // Default to current directory if no repo specified
            }
        }

        let output = Command::new("solana-verify")
            .args(&args)
            .current_dir(&program_path)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output()
            .context("Failed to run verifiable build")?;

        if !output.status.success() {
            anyhow::bail!("Verifiable build failed for {}", program_name);
        }

        println!("{} {} verified", "✓".green(), program_name);
    }

    Ok(())
}

fn run_cargo_audit(workspace_root: &Path) -> Result<bool> {
    println!("{} Running dependency audit...", "◆".blue());

    // Check if cargo-audit is installed
    let check_audit = Command::new("cargo").args(["audit", "--version"]).output();

    if check_audit.is_err() || !check_audit.unwrap().status.success() {
        println!("{} cargo-audit not installed. Installing...", "⚠".yellow());

        let install = Command::new("cargo")
            .args(["install", "cargo-audit"])
            .output()
            .context("Failed to install cargo-audit")?;

        if !install.status.success() {
            eprintln!("{} Failed to install cargo-audit", "x".red());
            return Ok(false);
        }
    }

    let output = Command::new("cargo")
        .args(["audit"])
        .current_dir(workspace_root)
        .output()
        .context("Failed to run cargo audit")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("{} Dependency audit failed:", "x".red());
        eprintln!("{stdout}");
        eprintln!("{stderr}");
        return Ok(false);
    }

    println!("{} Dependency audit passed", "✓".green());
    Ok(true)
}

fn check_common_vulnerabilities(workspace_root: &Path) -> Result<bool> {
    println!("{} Checking for common vulnerabilities...", "◆".blue());

    let mut issues_found = false;
    let programs_dir = workspace_root.join("programs");

    if programs_dir.exists() {
        for entry in std::fs::read_dir(&programs_dir)? {
            let entry = entry?;
            if !entry.path().is_dir() {
                continue;
            }

            let lib_path = entry.path().join("src").join("lib.rs");
            if lib_path.exists() {
                let content = std::fs::read_to_string(&lib_path)?;

                // Check for common issues
                if content.contains("unsafe ") && !content.contains("#![forbid(unsafe_code)]") {
                    println!(
                        "{} {} uses unsafe code without forbid directive",
                        "!".yellow(),
                        entry.file_name().to_string_lossy()
                    );
                    issues_found = true;
                }

                if !content.contains("nostd_panic_handler!()") {
                    println!(
                        "{} {} missing nostd_panic_handler",
                        "!".yellow(),
                        entry.file_name().to_string_lossy()
                    );
                    issues_found = true;
                }

                // Check for proper account validation
                if content.contains("constraint") {
                    // Basic check for constraint usage
                    println!(
                        "{} {} uses constraints",
                        "✓".green(),
                        entry.file_name().to_string_lossy()
                    );
                } else if content.contains("#[context]") {
                    println!(
                        "{} {} may need additional account validation",
                        "!".yellow(),
                        entry.file_name().to_string_lossy()
                    );
                }
            }
        }
    }

    if !issues_found {
        println!("{} No common vulnerabilities found", "✓".green());
    }

    Ok(!issues_found)
}

pub async fn verify_deployment(program_id: &str, network: &str) -> Result<()> {
    println!("{} Verifying deployment on {}...", "▶".blue(), network);

    let output = Command::new("solana")
        .args(["program", "show", program_id, "--url", network])
        .output()
        .context("Failed to verify deployment")?;

    if !output.status.success() {
        anyhow::bail!("Failed to verify program deployment");
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("{stdout}");

    println!("{} Program verified on {}", "✓".green(), network);
    Ok(())
}

pub async fn run_security_checks(verify: bool, audit: bool) -> Result<()> {
    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow::anyhow!("Not in a Typhoon workspace"))?;

    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    let mut checks_passed = true;

    if audit {
        progress.set_message("Running cargo audit...");
        if !run_cargo_audit(&workspace_root)? {
            checks_passed = false;
        }
    }

    if verify {
        progress.set_message("Running verifiable build...");
        if run_verifiable_build(&workspace_root, None, None, None, false, &progress)
            .await
            .is_err()
        {
            checks_passed = false;
        }
    }

    // Run additional security checks
    progress.set_message("Checking for common vulnerabilities...");
    if !check_common_vulnerabilities(&workspace_root)? {
        checks_passed = false;
    }

    progress.finish_and_clear();

    if !checks_passed {
        anyhow::bail!("Security checks failed");
    }

    Ok(())
}
