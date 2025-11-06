//! Template rendering and file creation utilities.
//!
//! ## Solana Version
//!
//! The templates contain hard-coded Solana version "3.0" to match the
//! Typhoon framework's target Solana version. When upgrading Solana support,
//! the version strings in the template files must be updated manually:
//! - `cli/templates/workspace/cargo.toml.template`
//! - `cli/templates/counter/cargo.toml.template`
//! - `cli/templates/hello-world/cargo.toml.template`

pub mod counter;
pub mod hello_world;
pub mod workspace;

use {
    crate::{Error, Result},
    std::{fs, path::Path},
};

/// Replaces template variables in a string.
///
/// If `use_path_deps` is true, uses local path dependencies for typhoon crates.
/// Otherwise, uses version strings for published crates.
pub fn render(
    template: &str,
    project_name: &str,
    program_id: &str,
    version: &str,
    use_path_deps: bool,
) -> String {
    // Module names in Rust must use underscores, not dashes
    let module_name = project_name.replace('-', "_");

    // Conditional dependency format
    let (typhoon_dep, typhoon_idl_dep, typhoon_instruction_builder_dep) = if use_path_deps {
        (
            r#"{ path = "../../crates/lib" }"#.to_string(),
            r#"{ path = "../../crates/idl-generator" }"#.to_string(),
            r#"{ path = "../../crates/instruction-builder" }"#.to_string(),
        )
    } else {
        (
            format!(r#""{version}""#),
            format!(r#""{version}""#),
            format!(r#""{version}""#),
        )
    };

    template
        .replace("{{project_name}}", project_name)
        .replace("{{module_name}}", &module_name)
        .replace("{{program_id}}", program_id)
        .replace("{{typhoon_version}}", version)
        .replace("{{typhoon_dep}}", &typhoon_dep)
        .replace("{{typhoon_idl_dep}}", &typhoon_idl_dep)
        .replace(
            "{{typhoon_instruction_builder_dep}}",
            &typhoon_instruction_builder_dep,
        )
}

/// Creates a file with the given content, creating parent directories if needed.
///
/// # Errors
/// Returns an error if file creation or directory creation fails.
pub fn create_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            Error::Other(anyhow::anyhow!(
                "failed to create directory {}: {}",
                parent.display(),
                e
            ))
        })?;
    }
    fs::write(path, content).map_err(|e| {
        Error::Other(anyhow::anyhow!(
            "failed to write file {}: {}",
            path.display(),
            e
        ))
    })?;
    Ok(())
}
