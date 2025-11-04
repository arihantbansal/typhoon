//! Project initialization command.

use {
    crate::{keypair, templates, utils, Error, Result},
    std::path::PathBuf,
};

const TYPHOON_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Creates a new Typhoon program with the counter template.
///
/// # Errors
/// Returns an error if the project name is invalid, the directory already exists,
/// or file creation fails.
pub fn run(name: &str) -> Result<()> {
    utils::validate_project_name(name)?;

    let project_path = PathBuf::from(name);
    if project_path.exists() {
        return Err(Error::DirectoryExists(name.to_string()));
    }

    println!("Creating Typhoon program '{name}'...");

    std::fs::create_dir_all(&project_path).map_err(|e| {
        Error::Other(anyhow::anyhow!(
            "failed to create project directory '{name}': {e}"
        ))
    })?;

    let program_id = keypair::generate_program_keypair(&project_path)?;

    let cargo_toml = templates::render(
        templates::counter::CARGO_TOML,
        name,
        &program_id,
        TYPHOON_VERSION,
    );
    let lib_rs = templates::render(
        templates::counter::LIB_RS,
        name,
        &program_id,
        TYPHOON_VERSION,
    );
    let build_rs = templates::render(
        templates::counter::BUILD_RS,
        name,
        &program_id,
        TYPHOON_VERSION,
    );
    let integration_test = templates::render(
        templates::counter::INTEGRATION_TEST,
        name,
        &program_id,
        TYPHOON_VERSION,
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

    println!("\nSuccessfully created Typhoon program '{name}'.");
    println!("\nNext steps:");
    println!("  cd {name}");
    println!("  typhoon build");
    println!("  typhoon test\n");

    Ok(())
}
