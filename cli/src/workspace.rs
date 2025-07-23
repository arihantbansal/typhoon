//! Workspace management for Typhoon projects
//! Handles creation and configuration of multi-program workspaces

use {
    crate::{templates, validation},
    anyhow::{Context, Result},
    std::{fs, path::Path},
    toml::toml,
};

/// Create a new Typhoon workspace with optional template
/// Sets up directory structure and configuration files
pub async fn create_workspace(
    name: &str,
    template: Option<&str>,
    from: Option<&str>,
) -> Result<()> {
    // Ensure workspace name is valid
    validation::validate_workspace_name(name)?;

    // Check template validity if specified
    if let Some(template_name) = template {
        validation::validate_template_name(template_name)?;
    }

    let workspace_path = Path::new(name);

    if workspace_path.exists() {
        anyhow::bail!("Directory '{}' already exists", name);
    }

    // Set up main workspace directory
    fs::create_dir_all(workspace_path).context("Failed to create workspace directory")?;

    // Handle remote template cloning
    if let Some(repo_url) = from {
        clone_template(repo_url, workspace_path).await?;
        return Ok(());
    }

    // Generate workspace Cargo.toml configuration
    let workspace_toml = toml! {
        [workspace]
        resolver = "2"
        members = []

        [workspace.package]
        version = "0.1.0"
        edition = "2021"
        license = "Apache-2.0"
        homepage = ""
        documentation = ""
        repository = ""
        authors = []

        [workspace.dependencies]
        typhoon = { version = "0.1.0-alpha" }
        bytemuck = "1.20"
        pinocchio = "0.6"
        litesvm = "0.6"
        solana-sdk = "2.1"
        "solana-program-test" = "2.1"
    };

    let cargo_toml_path = workspace_path.join("Cargo.toml");
    fs::write(&cargo_toml_path, toml::to_string_pretty(&workspace_toml)?)
        .context("Failed to write Cargo.toml")?;

    // Add version control ignore file
    fs::write(
        workspace_path.join(".gitignore"),
        templates::GITIGNORE_TEMPLATE,
    )
    .context("Failed to write .gitignore")?;

    // Generate basic documentation
    let readme_content = format!(
        "# {name}\n\nA Typhoon workspace for Solana programs.\n\n## Getting Started\n\n```bash\n# Build all programs\ntyphoon build\n\n# Run tests\ntyphoon test\n```\n"
    );
    fs::write(workspace_path.join("README.md"), readme_content)
        .context("Failed to write README.md")?;

    // Set up directory for programs
    fs::create_dir_all(workspace_path.join("programs"))
        .context("Failed to create programs directory")?;

    // Set up CI configuration
    let cargo_make_dir = workspace_path.join("cargo-make");
    fs::create_dir_all(&cargo_make_dir).context("Failed to create cargo-make directory")?;

    fs::write(
        cargo_make_dir.join("ci.toml"),
        templates::CARGO_MAKE_CI_TEMPLATE,
    )
    .context("Failed to write cargo-make ci.toml")?;

    // Generate workspace-level build configuration
    fs::write(
        workspace_path.join("Makefile.toml"),
        templates::WORKSPACE_MAKEFILE_TEMPLATE,
    )
    .context("Failed to write Makefile.toml")?;

    // Generate Typhoon-specific configuration
    let typhoon_config = toml! {
        [workspace]
        name = name
        programs = []

        [program_ids]

        [build]
        idl = true
        "idl-out" = "target/idl"

        [test]
        command = "cargo test-sbf"

        [bindings]
        languages = ["typescript"]
        output = "sdk"
    };

    fs::write(
        workspace_path.join("typhoon.toml"),
        toml::to_string_pretty(&typhoon_config)?,
    )
    .context("Failed to write typhoon.toml")?;

    // Add initial program if template specified
    if let Some(template_name) = template {
        let program_path = workspace_path.join("programs").join(template_name);
        crate::scaffold::create_program_in_path(&program_path, template_name, Some(template_name))
            .await?;

        // Register program in workspace configuration
        let mut workspace_toml =
            toml::from_str::<toml::Value>(&fs::read_to_string(&cargo_toml_path)?)?;
        if let Some(members) = workspace_toml
            .get_mut("workspace")
            .and_then(|w| w.get_mut("members"))
            .and_then(|m| m.as_array_mut())
        {
            members.push(toml::Value::String(format!("programs/{template_name}")));
        }
        fs::write(&cargo_toml_path, toml::to_string_pretty(&workspace_toml)?)?;
    }

    Ok(())
}

/// Clone remote template repository
/// Removes .git directory after cloning
async fn clone_template(repo_url: &str, target_path: &Path) -> Result<()> {
    use git2::Repository;

    Repository::clone(repo_url, target_path).context("Failed to clone template repository")?;

    // Clean up git metadata
    let git_dir = target_path.join(".git");
    if git_dir.exists() {
        fs::remove_dir_all(git_dir)?;
    }

    Ok(())
}

/// Find the root directory of the current Typhoon workspace
/// Searches up the directory tree for typhoon.toml or workspace Cargo.toml
pub fn find_workspace_root() -> Result<Option<std::path::PathBuf>> {
    let current_dir = std::env::current_dir()?;
    let mut path = current_dir.as_path();

    loop {
        let typhoon_toml = path.join("typhoon.toml");
        let cargo_toml = path.join("Cargo.toml");

        if typhoon_toml.exists() {
            return Ok(Some(path.to_path_buf()));
        }

        // Verify if Cargo.toml defines a workspace
        if cargo_toml.exists() {
            let content = fs::read_to_string(&cargo_toml)?;
            if content.contains("[workspace]") {
                return Ok(Some(path.to_path_buf()));
            }
        }

        match path.parent() {
            Some(p) => path = p,
            None => break,
        }
    }

    Ok(None)
}
