//! Project initialization command.

use {
    crate::{keypair, templates, utils, Error, Result},
    std::path::{Path, PathBuf},
};

const TYPHOON_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Creates a new Typhoon program from a template.
///
/// # Arguments
/// * `name` - The project name
/// * `template` - Template to use ("counter" or "hello-world")
///
/// # Errors
/// Returns an error if the project name is invalid, the directory already exists,
/// template is not found, or file creation fails.
pub fn run(name: &str, template: &str) -> Result<()> {
    utils::validate_project_name(name)?;

    let project_path = PathBuf::from(name);
    if project_path.exists() {
        return Err(Error::DirectoryExists(name.to_string()));
    }

    println!("Creating Typhoon program '{name}' with template '{template}'...");

    std::fs::create_dir_all(&project_path).map_err(|e| {
        Error::Other(anyhow::anyhow!(
            "failed to create project directory '{name}': {e}"
        ))
    })?;

    // Generate keypair (same for all templates)
    let program_id = keypair::generate_program_keypair(&project_path, name)?;

    // Detect path deps (same for all templates)
    let use_path_deps = utils::is_inside_typhoon_repo(&project_path);

    // Create project based on template
    match template {
        "counter" => create_counter_project(&project_path, name, &program_id, use_path_deps)?,
        "hello-world" => {
            create_hello_world_project(&project_path, name, &program_id, use_path_deps)?
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

    println!("\nSuccessfully created Typhoon program '{name}'.");
    println!("\nNext steps:");
    println!("  cd {name}");
    println!("  typhoon build");
    println!("  typhoon test\n");

    Ok(())
}

/// Creates a counter template project.
fn create_counter_project(
    project_path: &Path,
    name: &str,
    program_id: &str,
    use_path_deps: bool,
) -> Result<()> {
    let cargo_toml = templates::render(
        templates::counter::CARGO_TOML,
        name,
        program_id,
        TYPHOON_VERSION,
        use_path_deps,
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
    let integration_test = templates::render(
        templates::counter::INTEGRATION_TEST,
        name,
        program_id,
        TYPHOON_VERSION,
        use_path_deps,
    );

    templates::create_file(&project_path.join("Cargo.toml"), &cargo_toml)?;
    templates::create_file(&project_path.join("src/lib.rs"), &lib_rs)?;
    templates::create_file(&project_path.join("build.rs"), &build_rs)?;
    templates::create_file(
        &project_path.join("tests/integration.rs"),
        &integration_test,
    )?;
    templates::create_file(
        &project_path.join(".gitignore"),
        templates::counter::GITIGNORE,
    )?;

    Ok(())
}

/// Creates a hello-world template project.
fn create_hello_world_project(
    project_path: &Path,
    name: &str,
    program_id: &str,
    use_path_deps: bool,
) -> Result<()> {
    let cargo_toml = templates::render(
        templates::hello_world::CARGO_TOML,
        name,
        program_id,
        TYPHOON_VERSION,
        use_path_deps,
    );
    let lib_rs = templates::render(
        templates::hello_world::LIB_RS,
        name,
        program_id,
        TYPHOON_VERSION,
        use_path_deps,
    );
    let integration_test = templates::render(
        templates::hello_world::INTEGRATION_TEST,
        name,
        program_id,
        TYPHOON_VERSION,
        use_path_deps,
    );

    templates::create_file(&project_path.join("Cargo.toml"), &cargo_toml)?;
    templates::create_file(&project_path.join("src/lib.rs"), &lib_rs)?;
    // Note: hello-world template does NOT include build.rs
    templates::create_file(
        &project_path.join("tests/integration.rs"),
        &integration_test,
    )?;
    templates::create_file(
        &project_path.join(".gitignore"),
        templates::hello_world::GITIGNORE,
    )?;

    Ok(())
}

/// Creates a new Typhoon workspace with the first program.
///
/// # Arguments
/// * `name` - The workspace name
/// * `template` - Template for the first program
///
/// # Errors
/// Returns an error if the workspace name is invalid, the directory already exists,
/// or file creation fails.
pub fn run_workspace(name: &str, template: &str) -> Result<()> {
    utils::validate_project_name(name)?;

    let workspace_path = PathBuf::from(name);
    if workspace_path.exists() {
        return Err(Error::DirectoryExists(name.to_string()));
    }

    println!("Creating Typhoon workspace '{name}'...");

    // Create workspace structure
    std::fs::create_dir_all(&workspace_path).map_err(|e| {
        Error::Other(anyhow::anyhow!(
            "failed to create workspace directory '{name}': {e}"
        ))
    })?;
    std::fs::create_dir_all(workspace_path.join("programs"))
        .map_err(|e| Error::Other(anyhow::anyhow!("failed to create programs directory: {e}")))?;
    std::fs::create_dir_all(workspace_path.join("tests"))
        .map_err(|e| Error::Other(anyhow::anyhow!("failed to create tests directory: {e}")))?;

    // Detect path deps
    let use_path_deps = utils::is_inside_typhoon_repo(&workspace_path);

    // Create workspace Cargo.toml
    let workspace_toml = templates::render(
        templates::workspace::CARGO_TOML,
        name,
        "", // No program_id for workspace
        TYPHOON_VERSION,
        use_path_deps,
    );
    templates::create_file(&workspace_path.join("Cargo.toml"), &workspace_toml)?;
    templates::create_file(
        &workspace_path.join(".gitignore"),
        templates::workspace::GITIGNORE,
    )?;

    // Create first program
    let program_name = format!("{name}-program");
    println!("  Creating first program '{program_name}'...");

    create_workspace_program(&workspace_path, &program_name, template, use_path_deps)?;

    println!("\nSuccessfully created Typhoon workspace '{name}'.");
    println!("\nNext steps:");
    println!("  cd {name}");
    println!("  typhoon build");
    println!("  typhoon test");
    println!("\nTo add more programs:");
    println!("  typhoon add program <name>\n");

    Ok(())
}

/// Creates a program inside a workspace.
fn create_workspace_program(
    workspace_path: &Path,
    name: &str,
    template: &str,
    use_path_deps: bool,
) -> Result<()> {
    let program_path = workspace_path.join("programs").join(name);
    std::fs::create_dir_all(&program_path)
        .map_err(|e| Error::Other(anyhow::anyhow!("failed to create program directory: {e}")))?;

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
            r#"{ path = "../../../../crates/idl-generator" }"#.to_string()
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
