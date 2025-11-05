//! Add subcommands for workspace management.

use {
    crate::{keypair, templates, utils, Error, Result},
    std::path::{Path, PathBuf},
};

const TYPHOON_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Adds a new program to the current workspace.
///
/// # Errors
/// Returns an error if not in a workspace, the program name is invalid,
/// or file creation fails.
pub fn run_program(name: &str, template: &str) -> Result<()> {
    utils::validate_project_name(name)?;

    // Check if we're in a workspace
    if !is_in_workspace()? {
        return Err(Error::Other(anyhow::anyhow!(
            "not in a workspace\n\n\
            This command must be run from the root of a Typhoon workspace.\n\
            To create a workspace, use: typhoon init --workspace <name>"
        )));
    }

    let workspace_path = PathBuf::from(".");

    // Check for both programs/ (plural) and program/ (singular) directories
    let programs_dir = workspace_path.join("programs");
    let program_dir = workspace_path.join("program");

    let (base_dir, dir_name) = if programs_dir.exists() {
        (programs_dir, "programs")
    } else if program_dir.exists() {
        (program_dir, "program")
    } else {
        return Err(Error::Other(anyhow::anyhow!(
            "programs/ or program/ directory not found\n\n\
            Expected workspace structure with programs/ or program/ directory."
        )));
    };

    // Security: Verify that the programs directory is actually within the workspace
    // This prevents symlink attacks where programs/ could point outside the workspace
    let canonical_base = base_dir.canonicalize().map_err(|e| {
        Error::Other(anyhow::anyhow!(
            "failed to resolve {dir_name}/ directory path: {e}"
        ))
    })?;

    let canonical_workspace = workspace_path
        .canonicalize()
        .map_err(|e| Error::Other(anyhow::anyhow!("failed to resolve workspace path: {e}")))?;

    if !canonical_base.starts_with(&canonical_workspace) {
        return Err(Error::Other(anyhow::anyhow!(
            "{dir_name}/ directory is outside the workspace\n\n\
            This may indicate a symlink attack or misconfigured workspace structure."
        )));
    }

    println!("Adding program '{name}' to workspace...");

    // Detect if using path deps
    let use_path_deps = utils::is_inside_typhoon_repo(&workspace_path);

    // Create program
    create_workspace_program(&workspace_path, name, template, use_path_deps, dir_name)?;

    println!("\nSuccessfully added program '{name}'.");
    println!("\nThe program has been added to the workspace members.");
    println!("Build it with: typhoon build\n");

    Ok(())
}

/// Checks if the current directory is a Typhoon workspace.
fn is_in_workspace() -> Result<bool> {
    let toml = utils::parse_cargo_toml()?;
    Ok(toml.get("workspace").is_some())
}

/// Creates a program inside a workspace.
fn create_workspace_program(
    workspace_path: &Path,
    name: &str,
    template: &str,
    use_path_deps: bool,
    programs_dir_name: &str,
) -> Result<()> {
    let program_path = workspace_path.join(programs_dir_name).join(name);

    // Use create_dir (not create_dir_all) to atomically check and create
    // This prevents race conditions where another process creates the directory
    // between our check and creation
    match std::fs::create_dir(&program_path) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(Error::DirectoryExists(format!(
                "{programs_dir_name}/{name}"
            )));
        }
        Err(e) => {
            return Err(Error::Other(anyhow::anyhow!(
                "failed to create program directory: {e}"
            )));
        }
    }

    // Generate keypair in workspace's target/deploy
    let program_id = keypair::generate_program_keypair(workspace_path, name)?;

    // Create program files based on template
    match template {
        "counter" => create_workspace_counter(&program_path, name, &program_id, use_path_deps)?,
        "hello-world" => {
            create_workspace_hello_world(&program_path, name, &program_id, use_path_deps)?
        }
        _ => {
            return Err(Error::Other(anyhow::anyhow!(
                "template '{template}' not found\n\n\
                Available templates:\n\
                  - counter      Full-featured with state management\n\
                  - hello-world  Minimal program with single instruction"
            )))
        }
    }

    Ok(())
}

/// Creates a counter template program in a workspace.
fn create_workspace_counter(
    program_path: &Path,
    name: &str,
    program_id: &str,
    use_path_deps: bool,
) -> Result<()> {
    // Workspace programs use workspace dependencies
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition.workspace = true

[lib]
crate-type = ["cdylib", "lib"]

[lints]
workspace = true

[dependencies]
bytemuck.workspace = true
typhoon.workspace = true

[build-dependencies]
typhoon-idl-generator = {typhoon_idl_dep}
"#,
        name = name,
        typhoon_idl_dep = if use_path_deps {
            r#"{ path = "../../../crates/idl-generator" }"#.to_string()
        } else {
            format!(r#""{TYPHOON_VERSION}""#)
        }
    );

    let lib_rs = templates::render(
        templates::counter::LIB_RS,
        name,
        program_id,
        TYPHOON_VERSION,
        use_path_deps,
    );

    let build_rs = templates::render(
        templates::counter::BUILD_RS,
        name,
        program_id,
        TYPHOON_VERSION,
        use_path_deps,
    );

    templates::create_file(&program_path.join("Cargo.toml"), &cargo_toml)?;
    templates::create_file(&program_path.join("src/lib.rs"), &lib_rs)?;
    templates::create_file(&program_path.join("build.rs"), &build_rs)?;

    Ok(())
}

/// Creates a hello-world template program in a workspace.
fn create_workspace_hello_world(
    program_path: &Path,
    name: &str,
    program_id: &str,
    use_path_deps: bool,
) -> Result<()> {
    // Workspace programs use workspace dependencies (no build-dependencies for hello-world)
    let cargo_toml = format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition.workspace = true

[lib]
crate-type = ["cdylib"]

[lints]
workspace = true

[dependencies]
typhoon.workspace = true
"#
    );

    let lib_rs = templates::render(
        templates::hello_world::LIB_RS,
        name,
        program_id,
        TYPHOON_VERSION,
        use_path_deps,
    );

    templates::create_file(&program_path.join("Cargo.toml"), &cargo_toml)?;
    templates::create_file(&program_path.join("src/lib.rs"), &lib_rs)?;
    // Note: hello-world template does NOT include build.rs

    Ok(())
}
