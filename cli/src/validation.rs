//! Input validation utilities
//! Validates names, identifiers, and other user inputs for CLI commands

use {anyhow::Result, regex::Regex, solana_pubkey::Pubkey, std::str::FromStr};

/// Validate program name according to Rust package naming conventions
pub fn validate_program_name(name: &str) -> Result<()> {
    // Reject empty names
    if name.is_empty() {
        anyhow::bail!("Program name cannot be empty");
    }

    // Check length
    if name.len() > 214 {
        anyhow::bail!("Program name is too long (max 214 characters)");
    }

    // Check if it starts with a letter or underscore
    if !name.chars().next().unwrap().is_ascii_alphabetic() && !name.starts_with('_') {
        anyhow::bail!("Program name must start with a letter or underscore");
    }

    // Check if it contains only valid characters (letters, numbers, hyphens, underscores)
    let valid_chars = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_-]*$").unwrap();
    if !valid_chars.is_match(name) {
        anyhow::bail!("Program name can only contain letters, numbers, hyphens, and underscores");
    }

    // Check for reserved names
    let reserved_names = [
        "test", "tests", "target", "src", "lib", "main", "cargo", "rust", "solana", "system",
        "token", "spl", "anchor", "typhoon",
    ];

    if reserved_names.contains(&name.to_lowercase().as_str()) {
        anyhow::bail!(
            "'{}' is a reserved name and cannot be used as a program name",
            name
        );
    }

    Ok(())
}

/// Validate instruction name as valid Rust identifier
pub fn validate_instruction_name(name: &str) -> Result<()> {
    // Reject empty names
    if name.is_empty() {
        anyhow::bail!("Instruction name cannot be empty");
    }

    // Check if it starts with a letter or underscore
    if !name.chars().next().unwrap().is_ascii_alphabetic() && !name.starts_with('_') {
        anyhow::bail!("Instruction name must start with a letter or underscore");
    }

    // Check if it contains only valid Rust identifier characters
    let valid_chars = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$").unwrap();
    if !valid_chars.is_match(name) {
        anyhow::bail!("Instruction name can only contain letters, numbers, and underscores");
    }

    // Check for Rust reserved keywords
    let rust_keywords = [
        "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
        "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref",
        "return", "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe",
        "use", "where", "while", "async", "await", "dyn", "try", "union", "yield",
    ];

    if rust_keywords.contains(&name) {
        anyhow::bail!(
            "'{}' is a Rust keyword and cannot be used as an instruction name",
            name
        );
    }

    Ok(())
}

/// Validate program ID as valid Solana public key
pub fn validate_program_id(program_id: &str) -> Result<()> {
    match Pubkey::from_str(program_id) {
        Ok(_) => Ok(()),
        Err(_) => anyhow::bail!(
            "Invalid program ID: '{}'. Must be a valid base58 encoded public key",
            program_id
        ),
    }
}

/// Validate workspace name for directory creation
pub fn validate_workspace_name(name: &str) -> Result<()> {
    // Reject empty names
    if name.is_empty() {
        anyhow::bail!("Workspace name cannot be empty");
    }

    // Check length
    if name.len() > 214 {
        anyhow::bail!("Workspace name is too long (max 214 characters)");
    }

    // Check if it contains only valid characters for directory names
    let valid_chars = Regex::new(r"^[a-zA-Z0-9._-]+$").unwrap();
    if !valid_chars.is_match(name) {
        anyhow::bail!(
            "Workspace name can only contain letters, numbers, dots, hyphens, and underscores"
        );
    }

    // Check for problematic names
    let problematic_names = [".", "..", "con", "prn", "aux", "nul"];
    if problematic_names.contains(&name.to_lowercase().as_str()) {
        anyhow::bail!("'{}' is not a valid workspace name", name);
    }

    Ok(())
}

/// Validate template name against available templates
pub fn validate_template_name(template: &str) -> Result<()> {
    let valid_templates = ["hello-world", "counter", "transfer", "token"];

    if !valid_templates.contains(&template) {
        anyhow::bail!(
            "Invalid template '{}'. Available templates: {}",
            template,
            valid_templates.join(", ")
        );
    }

    Ok(())
}

/// Validate language name for binding generation
pub fn validate_language_name(language: &str) -> Result<()> {
    let valid_languages = ["typescript", "ts", "swift", "kotlin", "rust"];

    if !valid_languages.contains(&language.to_lowercase().as_str()) {
        anyhow::bail!(
            "Unsupported language '{}'. Available languages: {}",
            language,
            valid_languages.join(", ")
        );
    }

    Ok(())
}
