//! Template rendering and file creation utilities.

pub mod counter;

use {
    crate::{Error, Result},
    std::{fs, path::Path},
};

/// Replaces template variables in a string.
pub fn render(template: &str, project_name: &str, program_id: &str, version: &str) -> String {
    // Module names in Rust must use underscores, not dashes
    let module_name = project_name.replace('-', "_");

    template
        .replace("{{project_name}}", project_name)
        .replace("{{module_name}}", &module_name)
        .replace("{{program_id}}", program_id)
        .replace("{{typhoon_version}}", version)
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
