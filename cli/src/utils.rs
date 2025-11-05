//! Shared utilities for the CLI.

use {
    crate::{constants::CARGO_TOML, Error, Result},
    std::path::Path,
    toml::Value,
};

/// Validates a project name for Rust crate naming conventions and security.
///
/// # Errors
/// Returns an error if the name is invalid, is a keyword, or contains path traversal attempts.
pub fn validate_project_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(Error::InvalidProjectName(
            "name cannot be empty".to_string(),
        ));
    }

    if name.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(Error::InvalidProjectName(
            "name cannot start with a digit".to_string(),
        ));
    }

    // Security: prevent path traversal
    if name.contains('/') || name.contains('\\') || name.contains("..") {
        return Err(Error::InvalidProjectName(
            "name cannot contain path separators or relative paths".to_string(),
        ));
    }

    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(Error::InvalidProjectName(
            "name can only contain alphanumeric characters, hyphens, and underscores".to_string(),
        ));
    }

    let keywords = [
        "abstract", "as", "async", "await", "become", "box", "break", "const", "continue", "crate",
        "do", "dyn", "else", "enum", "extern", "false", "final", "fn", "for", "if", "impl", "in",
        "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "try", "type",
        "typeof", "unsafe", "unsized", "use", "virtual", "where", "while", "yield",
    ];

    if keywords.contains(&name) {
        return Err(Error::InvalidProjectName(
            "name cannot be a Rust keyword".to_string(),
        ));
    }

    Ok(())
}

/// Checks if the current directory contains a Rust project.
///
/// # Errors
/// Returns an error if no Cargo.toml is found.
pub fn check_rust_project() -> Result<()> {
    if !Path::new(CARGO_TOML).exists() {
        Err(Error::NotInProject)
    } else {
        Ok(())
    }
}

/// Parses a Cargo.toml file and returns the parsed TOML value.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn parse_cargo_toml() -> Result<Value> {
    let content = std::fs::read_to_string(CARGO_TOML)?;
    Ok(toml::from_str(&content)?)
}

/// Extracts the package name from Cargo.toml.
///
/// # Errors
/// Returns an error if the package name cannot be found.
pub fn get_package_name() -> Result<String> {
    let toml = parse_cargo_toml()?;
    toml.get("package")
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .map(String::from)
        .ok_or_else(|| Error::InvalidCargoToml("missing package name field".to_string()))
}

/// Checks if Cargo.toml has a typhoon dependency.
///
/// # Errors
/// Returns an error if Cargo.toml cannot be read or parsed.
pub fn has_typhoon_dependency() -> Result<bool> {
    let toml = parse_cargo_toml()?;
    let has_dep = toml
        .get("dependencies")
        .and_then(|deps| deps.get("typhoon"))
        .is_some();
    Ok(has_dep)
}

/// Detects if a project path will be inside the Typhoon repository.
///
/// This checks if the typhoon workspace structure exists relative to where
/// the new project will be created, specifically looking for a workspace
/// Cargo.toml with typhoon crate members.
///
/// # Arguments
/// * `project_path` - The path where the new project will be created
///
/// Returns true if the project would be inside the typhoon repo, false otherwise.
pub fn is_inside_typhoon_repo(project_path: &Path) -> bool {
    // From the new project's location, check if ../../crates/lib/Cargo.toml exists
    let typhoon_lib = project_path.join("../../crates/lib/Cargo.toml");
    if !typhoon_lib.exists() {
        return false;
    }

    // Check if ../../Cargo.toml exists and is a workspace
    let workspace_toml = project_path.join("../../Cargo.toml");
    if !workspace_toml.exists() {
        return false;
    }

    // Read and parse the workspace Cargo.toml
    if let Ok(content) = std::fs::read_to_string(&workspace_toml) {
        if let Ok(toml) = toml::from_str::<Value>(&content) {
            // Check if it has workspace.members that includes "crates/*"
            if let Some(workspace) = toml.get("workspace") {
                if let Some(members) = workspace.get("members") {
                    if let Some(members_array) = members.as_array() {
                        return members_array
                            .iter()
                            .any(|m| m.as_str() == Some("crates/*") || m.as_str() == Some("cli"));
                    }
                }
            }
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_project_name_valid() {
        assert!(validate_project_name("my-project").is_ok());
        assert!(validate_project_name("my_project").is_ok());
        assert!(validate_project_name("project123").is_ok());
    }

    #[test]
    fn test_validate_project_name_empty() {
        assert!(validate_project_name("").is_err());
    }

    #[test]
    fn test_validate_project_name_starts_with_digit() {
        assert!(validate_project_name("123project").is_err());
    }

    #[test]
    fn test_validate_project_name_path_traversal() {
        assert!(validate_project_name("../evil").is_err());
        assert!(validate_project_name("foo/bar").is_err());
        assert!(validate_project_name("foo\\bar").is_err());
        assert!(validate_project_name("..").is_err());
    }

    #[test]
    fn test_validate_project_name_keywords() {
        assert!(validate_project_name("async").is_err());
        assert!(validate_project_name("impl").is_err());
        assert!(validate_project_name("fn").is_err());
    }

    #[test]
    fn test_validate_project_name_invalid_chars() {
        assert!(validate_project_name("my project").is_err()); // space
        assert!(validate_project_name("my@project").is_err()); // @
        assert!(validate_project_name("my.project").is_err()); // .
    }
}
