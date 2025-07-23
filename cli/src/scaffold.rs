//! Project scaffolding for Typhoon programs
//! Creates new programs, workspaces, and instructions with proper templates

use {
    crate::{templates, validation, workspace::find_workspace_root},
    anyhow::{Context, Result},
    std::{fs, path::Path},
    toml::toml,
};

/// Create a new standalone program
/// Entry point for standalone program creation
pub async fn create_program(name: &str, template: Option<&str>, _from: Option<&str>) -> Result<()> {
    let program_path = Path::new(name);
    create_program_in_path(program_path, name, template).await
}

/// Create a program at the specified path with template
/// Core program creation logic used by both standalone and workspace scenarios
pub async fn create_program_in_path(path: &Path, name: &str, template: Option<&str>) -> Result<()> {
    // Ensure program name meets requirements
    validation::validate_program_name(name)?;

    // Check template validity if specified
    if let Some(template_name) = template {
        validation::validate_template_name(template_name)?;
    }

    if path.exists() {
        anyhow::bail!("Directory '{}' already exists", path.display());
    }

    fs::create_dir_all(path).context("Failed to create program directory")?;

    // Generate Cargo.toml with Typhoon dependencies
    let lib_name = name.replace("-", "_");
    let cargo_toml = toml! {
        [package]
        name = name
        version = "0.1.0"
        edition = "2021"

        [lib]
        name = lib_name
        "crate-type" = ["cdylib", "lib"]

        [dependencies]
        typhoon = { version = "0.1.0-alpha" }
        bytemuck = { version = "1.20", features = ["derive"] }
        pinocchio = "0.6"

        ["build-dependencies"]
        "typhoon-idl-generator" = "0.1.0-alpha"

        ["dev-dependencies"]
        litesvm = "0.6"
        "solana-sdk" = "2.1"
        "typhoon-instruction-builder" = "0.1.0-alpha"
    };

    fs::write(
        path.join("Cargo.toml"),
        toml::to_string_pretty(&cargo_toml)?,
    )
    .context("Failed to write Cargo.toml")?;

    // Set up source code directory
    let src_dir = path.join("src");
    fs::create_dir_all(&src_dir).context("Failed to create src directory")?;

    // Generate main program file from template
    let lib_content = match template {
        Some("counter") => templates::COUNTER_TEMPLATE,
        Some("transfer") => templates::TRANSFER_TEMPLATE,
        Some("token") => templates::TOKEN_TEMPLATE,
        _ => templates::HELLO_WORLD_TEMPLATE,
    };

    fs::write(src_dir.join("lib.rs"), lib_content).context("Failed to write lib.rs")?;

    // Generate build script for IDL generation
    let build_rs_content = templates::generate_build_rs_template(name);
    fs::write(path.join("build.rs"), build_rs_content).context("Failed to write build.rs")?;

    // Set up test directory structure
    let tests_dir = path.join("tests");
    fs::create_dir_all(&tests_dir).context("Failed to create tests directory")?;

    // Generate basic integration test
    let test_content = templates::generate_test_template(name);
    fs::write(tests_dir.join("integration.rs"), test_content)
        .context("Failed to write integration test")?;

    // Generate cargo-make configuration
    fs::write(
        path.join("Makefile.toml"),
        templates::PROGRAM_MAKEFILE_TEMPLATE,
    )
    .context("Failed to write Makefile.toml")?;

    Ok(())
}

/// Add a new program to existing workspace
/// Updates workspace configuration and creates program
pub async fn add_program(name: &str, template: Option<&str>) -> Result<()> {
    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow::anyhow!("Not in a Typhoon workspace"))?;

    let programs_dir = workspace_root.join("programs");
    if !programs_dir.exists() {
        fs::create_dir_all(&programs_dir)?;
    }

    let program_path = programs_dir.join(name);
    create_program_in_path(&program_path, name, template).await?;

    // Add program to workspace members
    let cargo_toml_path = workspace_root.join("Cargo.toml");
    let mut workspace_toml = toml::from_str::<toml::Value>(&fs::read_to_string(&cargo_toml_path)?)?;

    if let Some(members) = workspace_toml
        .get_mut("workspace")
        .and_then(|w| w.get_mut("members"))
        .and_then(|m| m.as_array_mut())
    {
        members.push(toml::Value::String(format!("programs/{name}")));
    }

    fs::write(&cargo_toml_path, toml::to_string_pretty(&workspace_toml)?)?;

    // Update Typhoon configuration if present
    let typhoon_toml_path = workspace_root.join("typhoon.toml");
    if typhoon_toml_path.exists() {
        let mut typhoon_toml =
            toml::from_str::<toml::Value>(&fs::read_to_string(&typhoon_toml_path)?)?;

        if let Some(programs) = typhoon_toml
            .get_mut("workspace")
            .and_then(|w| w.get_mut("programs"))
            .and_then(|p| p.as_array_mut())
        {
            programs.push(toml::Value::String(name.to_string()));
        }

        fs::write(&typhoon_toml_path, toml::to_string_pretty(&typhoon_toml)?)?;
    }

    Ok(())
}

/// Add a new instruction to an existing program
/// Modifies the program's lib.rs to include new instruction handler
pub async fn add_instruction(program: &str, instruction_name: &str) -> Result<()> {
    // Ensure instruction name is valid Rust identifier
    validation::validate_instruction_name(instruction_name)?;

    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow::anyhow!("Not in a Typhoon workspace"))?;

    let program_lib_path = workspace_root
        .join("programs")
        .join(program)
        .join("src")
        .join("lib.rs");

    if !program_lib_path.exists() {
        anyhow::bail!("Program '{}' not found", program);
    }

    let lib_content = fs::read_to_string(&program_lib_path)?;

    // Locate handlers macro in program file
    let handlers_start = lib_content
        .find("handlers! {")
        .ok_or_else(|| anyhow::anyhow!("Could not find handlers! macro in program"))?;

    let handlers_end = lib_content[handlers_start..]
        .find("}")
        .ok_or_else(|| anyhow::anyhow!("Could not find end of handlers! macro"))?
        + handlers_start;

    // Parse current handler list
    let handlers_content = &lib_content[handlers_start..handlers_end];
    let existing_handlers: Vec<&str> = handlers_content
        .lines()
        .skip(1) // Skip "handlers! {"
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.is_empty() && !trimmed.starts_with("//") {
                Some(trimmed.trim_end_matches(','))
            } else {
                None
            }
        })
        .collect();

    // Include new instruction in handler list
    let mut all_handlers = existing_handlers;
    all_handlers.push(instruction_name);

    // Reconstruct handlers macro with new instruction
    let new_handlers = format!(
        "handlers! {{\n{}\n}}",
        all_handlers
            .iter()
            .map(|h| format!("    {h},"))
            .collect::<Vec<_>>()
            .join("\n")
    );

    // Update file with new handlers macro
    let new_lib_content = format!(
        "{}{}{}",
        &lib_content[..handlers_start],
        new_handlers,
        &lib_content[handlers_end + 1..]
    );

    // Generate context struct and handler function
    let context_name = format!("{}Context", to_pascal_case(instruction_name));
    let new_context_and_handler = format!(
        r#"
#[context]
pub struct {context_name} {{
    // Add your accounts here
    pub signer: Mut<Signer>,
}}

pub fn {instruction_name}(ctx: {context_name}) -> ProgramResult {{
    // TODO: Implement {instruction_name} instruction
    Ok(())
}}
"#
    );

    // Find appropriate insertion point in file
    let insert_position = if let Some(pos) = new_lib_content.rfind("#[derive(") {
        // Place before existing account definitions
        new_lib_content[..pos].rfind('\n').unwrap_or(pos)
    } else {
        // Append to end of file
        new_lib_content.len()
    };

    let final_content = format!(
        "{}{}{}",
        &new_lib_content[..insert_position],
        new_context_and_handler,
        &new_lib_content[insert_position..]
    );

    fs::write(&program_lib_path, final_content)?;

    Ok(())
}

/// Convert snake_case identifier to PascalCase for struct names
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}
