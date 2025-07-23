//! Test execution for Typhoon programs
//! Handles running cargo test-sbf for Solana programs

use {
    crate::workspace::find_workspace_root,
    anyhow::{Context, Result},
    colored::Colorize,
    indicatif::{ProgressBar, ProgressStyle},
    std::{path::Path, process::Command},
};

/// Run tests for programs in the workspace
/// Can target specific program and/or test
pub async fn run_tests(program: Option<&str>, test_name: Option<&str>) -> Result<()> {
    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow::anyhow!("Not in a Typhoon workspace"))?;

    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    if let Some(program_name) = program {
        // Execute tests for specified program
        progress.set_message(format!("Running tests for '{program_name}'..."));
        run_program_tests(&workspace_root, program_name, test_name, &progress)?;
    } else {
        // Execute tests for all programs
        progress.set_message("Running all tests...");
        run_all_tests(&workspace_root, test_name, &progress)?;
    }

    progress.finish_and_clear();
    Ok(())
}

/// Run tests for a specific program
fn run_program_tests(
    workspace_root: &Path,
    program_name: &str,
    test_name: Option<&str>,
    progress: &ProgressBar,
) -> Result<()> {
    let program_path = workspace_root.join("programs").join(program_name);

    if !program_path.exists() {
        anyhow::bail!("Program '{}' not found", program_name);
    }

    progress.set_message(format!("Testing {program_name}..."));

    let mut cmd = Command::new("cargo");
    cmd.arg("test-sbf");

    if let Some(test) = test_name {
        cmd.arg(test);
    }

    let output = cmd
        .current_dir(&program_path)
        .env("RUST_LOG", "off") // Suppress verbose logging
        .output()
        .context("Failed to execute cargo test-sbf")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        eprintln!("{stdout}");
        eprintln!("{stderr}");
        anyhow::bail!("Tests failed for {}", program_name);
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Extract test summary from output
        if let Some(summary_line) = stdout
            .lines()
            .rev()
            .find(|line| line.contains("test result:"))
        {
            println!("{} {} - {}", "✓".green(), program_name, summary_line.trim());
        } else {
            println!("{} {} - tests passed", "✓".green(), program_name);
        }
    }

    Ok(())
}

/// Run tests for all programs in workspace
fn run_all_tests(
    workspace_root: &Path,
    test_name: Option<&str>,
    progress: &ProgressBar,
) -> Result<()> {
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
        println!("{} No programs found to test", "!".yellow());
        return Ok(());
    }

    let mut failed_programs = Vec::new();

    for program_name in &programs {
        match run_program_tests(workspace_root, program_name, test_name, progress) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{} {} - {}", "x".red(), program_name, e);
                failed_programs.push(program_name.clone());
            }
        }
    }

    if !failed_programs.is_empty() {
        anyhow::bail!(
            "Tests failed for {} programs: {}",
            failed_programs.len(),
            failed_programs.join(", ")
        );
    }

    println!(
        "{} All tests passed ({} programs)",
        "✓".green().bold(),
        programs.len()
    );
    Ok(())
}

/// Run a specific test in a specific program
/// Used for targeted test execution
pub fn run_specific_test(program: &str, test: &str) -> Result<()> {
    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow::anyhow!("Not in a Typhoon workspace"))?;

    let program_path = workspace_root.join("programs").join(program);

    if !program_path.exists() {
        anyhow::bail!("Program '{}' not found", program);
    }

    println!(
        "{} Running test '{}' in '{}'...",
        "▶".blue().bold(),
        test,
        program
    );

    let output = Command::new("cargo")
        .args(["test-sbf", "--", test, "--exact"])
        .current_dir(&program_path)
        .output()
        .context("Failed to execute test")?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        eprintln!("{stdout}");
        eprintln!("{stderr}");
        anyhow::bail!("Test failed");
    }

    println!("{stdout}");
    Ok(())
}
