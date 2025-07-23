//! Client binding generation for Typhoon programs
//! Supports TypeScript, Swift, Kotlin, and Rust client libraries

use {
    crate::{validation, workspace::find_workspace_root},
    anyhow::Result,
    colored::Colorize,
    indicatif::{ProgressBar, ProgressStyle},
    serde_json::Value,
    std::{fs, path::Path},
};

/// Generate client bindings for multiple languages
/// Creates SDK packages for interacting with Typhoon programs
pub async fn generate_bindings(languages: &[String], output_dir: Option<&Path>) -> Result<()> {
    // Validate languages
    for language in languages {
        validation::validate_language_name(language)?;
    }

    let workspace_root =
        find_workspace_root()?.ok_or_else(|| anyhow::anyhow!("Not in a Typhoon workspace"))?;

    let progress = ProgressBar::new_spinner();
    progress.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );

    // Determine output directory
    let output_path = if let Some(dir) = output_dir {
        dir.to_path_buf()
    } else {
        workspace_root.join("sdk")
    };

    // Create output directory
    fs::create_dir_all(&output_path)?;

    // Find all IDL files
    let idl_dir = workspace_root.join("target").join("idl");
    if !idl_dir.exists() {
        anyhow::bail!("No IDL files found. Run 'typhoon build --idl' first.");
    }

    let idl_files: Vec<_> = fs::read_dir(&idl_dir)?
        .filter_map(|entry| {
            entry.ok().and_then(|e| {
                let path = e.path();
                if path.extension()?.to_str()? == "json" {
                    Some(path)
                } else {
                    None
                }
            })
        })
        .collect();

    if idl_files.is_empty() {
        anyhow::bail!("No IDL files found in target/idl/");
    }

    // Enhanced progress tracking
    let total_tasks = idl_files.len() * languages.len();
    progress.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:30.cyan/blue}] {pos}/{len} {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );
    progress.set_length(total_tasks as u64);

    let mut completed = 0;
    for idl_path in &idl_files {
        let program_name = idl_path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid IDL filename"))?;

        for language in languages {
            progress.set_position(completed);
            progress.set_message(format!(
                "Generating {language} bindings for {program_name}..."
            ));

            match language.as_str() {
                "typescript" | "ts" => {
                    generate_typescript_bindings(idl_path, &output_path, program_name)?;
                }
                "swift" => {
                    generate_swift_bindings(idl_path, &output_path, program_name)?;
                }
                "kotlin" => {
                    generate_kotlin_bindings(idl_path, &output_path, program_name)?;
                }
                "rust" => {
                    generate_rust_bindings(idl_path, &output_path, program_name)?;
                }
                _ => {
                    eprintln!("{} Unsupported language: {}", "!".yellow(), language);
                    continue;
                }
            }
            completed += 1;
        }
    }

    progress.finish_with_message(format!(
        "Generated bindings for {} programs in {} languages",
        idl_files.len(),
        languages.len()
    ));

    progress.finish_and_clear();
    println!(
        "{} Generated bindings in {}",
        "✓".green(),
        output_path.display()
    );
    Ok(())
}

fn generate_typescript_bindings(
    idl_path: &Path,
    output_dir: &Path,
    program_name: &str,
) -> Result<()> {
    let ts_dir = output_dir.join("typescript").join(program_name);
    fs::create_dir_all(&ts_dir)?;

    // Read and parse IDL
    let idl_content = fs::read_to_string(idl_path)?;
    let idl: Value = serde_json::from_str(&idl_content)?;

    // Generate TypeScript client
    let ts_content = generate_typescript_client(&idl, program_name)?;
    fs::write(ts_dir.join("index.ts"), ts_content)?;

    // Generate types
    let types_content = generate_typescript_types(&idl)?;
    fs::write(ts_dir.join("types.ts"), types_content)?;

    // Generate package.json
    let package_json = format!(
        r#"{{
  "name": "@typhoon/{program_name}-sdk",
  "version": "0.1.0",
  "description": "TypeScript SDK for {program_name} program",
  "main": "lib/index.js",
  "types": "lib/index.d.ts",
  "scripts": {{
    "build": "tsc",
    "test": "jest"
  }},
  "dependencies": {{
    "@solana/web3.js": "^1.87.0",
    "@coral-xyz/borsh": "^0.29.0"
  }},
  "devDependencies": {{
    "typescript": "^5.0.0",
    "@types/jest": "^29.0.0",
    "jest": "^29.0.0"
  }}
}}"#
    );

    fs::write(ts_dir.join("package.json"), package_json)?;

    // Generate tsconfig.json
    let tsconfig = r#"{
  "compilerOptions": {
    "target": "es2020",
    "module": "commonjs",
    "lib": ["es2020"],
    "outDir": "./lib",
    "rootDir": "./",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true,
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true
  },
  "include": ["**/*.ts"],
  "exclude": ["node_modules", "lib"]
}"#;

    fs::write(ts_dir.join("tsconfig.json"), tsconfig)?;

    Ok(())
}

fn generate_typescript_client(idl: &Value, program_name: &str) -> Result<String> {
    let program_id = idl["metadata"]["address"]
        .as_str()
        .unwrap_or("11111111111111111111111111111111");

    let mut client = format!(
        r#"import {{ Connection, PublicKey, Transaction, TransactionInstruction, Keypair }} from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';
import {{ {} }} from './types';

export const PROGRAM_ID = new PublicKey('{}');

export class {}Client {{
  constructor(
    private connection: Connection,
    private programId: PublicKey = PROGRAM_ID
  ) {{}}
"#,
        generate_type_imports(idl)?,
        program_id,
        to_pascal_case(program_name)
    );

    // Generate methods for each instruction
    if let Some(instructions) = idl["instructions"].as_array() {
        for instruction in instructions {
            let method = generate_typescript_method(instruction)?;
            client.push_str(&method);
        }
    }

    client.push_str("}\n");
    Ok(client)
}

fn generate_typescript_types(idl: &Value) -> Result<String> {
    let mut types = String::from("import { PublicKey } from '@solana/web3.js';\n\n");

    // Generate account types
    if let Some(accounts) = idl["accounts"].as_array() {
        for account in accounts {
            let type_def = generate_typescript_type(account)?;
            types.push_str(&type_def);
            types.push('\n');
        }
    }

    // Generate instruction argument types
    if let Some(instructions) = idl["instructions"].as_array() {
        for instruction in instructions {
            if let Some(args) = instruction["args"].as_array() {
                if !args.is_empty() {
                    let args_type = generate_instruction_args_type(instruction)?;
                    types.push_str(&args_type);
                    types.push('\n');
                }
            }
        }
    }

    Ok(types)
}

fn generate_typescript_method(instruction: &Value) -> Result<String> {
    let name = instruction["name"].as_str().unwrap_or("unknown");
    let pascal_name = to_pascal_case(name);

    let mut method = format!("\n  async {}(", to_camel_case(name));

    // Add accounts parameter
    method.push_str("accounts: {");
    if let Some(accounts) = instruction["accounts"].as_array() {
        for account in accounts {
            let account_name = account["name"].as_str().unwrap_or("unknown");
            method.push_str(&format!(
                "\n    {}: PublicKey;",
                to_camel_case(account_name)
            ));
        }
    }
    method.push_str("\n  }");

    // Add args parameter if there are any
    if let Some(args) = instruction["args"].as_array() {
        if !args.is_empty() {
            method.push_str(&format!(", args: {pascal_name}Args"));
        }
    }

    method.push_str(") {\n");
    method.push_str("    // TODO: Implement instruction encoding\n");
    method.push_str("    const instruction = new TransactionInstruction({\n");
    method.push_str("      keys: [\n");

    if let Some(accounts) = instruction["accounts"].as_array() {
        for account in accounts {
            let account_name = account["name"].as_str().unwrap_or("unknown");
            let is_mut = account["isMut"].as_bool().unwrap_or(false);
            let is_signer = account["isSigner"].as_bool().unwrap_or(false);
            method.push_str(&format!(
                "        {{ pubkey: accounts.{}, isWritable: {}, isSigner: {} }},\n",
                to_camel_case(account_name),
                is_mut,
                is_signer
            ));
        }
    }

    method.push_str("      ],\n");
    method.push_str("      programId: this.programId,\n");
    method.push_str("      data: Buffer.alloc(0), // TODO: Encode instruction data\n");
    method.push_str("    });\n");
    method.push_str("    return instruction;\n");
    method.push_str("  }\n");

    Ok(method)
}

fn generate_swift_bindings(idl_path: &Path, output_dir: &Path, program_name: &str) -> Result<()> {
    let swift_dir = output_dir.join("swift").join(program_name);
    fs::create_dir_all(&swift_dir)?;

    // Generate Swift package structure
    let package_swift = format!(
        r#"// swift-tools-version: 5.7
import PackageDescription

let package = Package(
    name: "{}SDK",
    platforms: [
        .macOS(.v12),
        .iOS(.v15)
    ],
    products: [
        .library(
            name: "{}SDK",
            targets: ["{}SDK"]
        ),
    ],
    dependencies: [
        .package(url: "https://github.com/solana-mobile/solana-swift", from: "1.0.0")
    ],
    targets: [
        .target(
            name: "{}SDK",
            dependencies: [
                .product(name: "Solana", package: "solana-swift")
            ]
        ),
        .testTarget(
            name: "{}SDKTests",
            dependencies: ["{}SDK"]
        ),
    ]
)"#,
        to_pascal_case(program_name),
        to_pascal_case(program_name),
        to_pascal_case(program_name),
        to_pascal_case(program_name),
        to_pascal_case(program_name),
        to_pascal_case(program_name)
    );

    fs::write(swift_dir.join("Package.swift"), package_swift)?;

    // Create Sources directory
    let sources_dir = swift_dir
        .join("Sources")
        .join(format!("{}SDK", to_pascal_case(program_name)));
    fs::create_dir_all(&sources_dir)?;

    // Generate Swift client
    let swift_content = format!(
        r#"import Foundation
import Solana

public struct {}Client {{
    private let connection: Connection
    private let programId: PublicKey
    
    public init(connection: Connection, programId: PublicKey = "{}") {{
        self.connection = connection
        self.programId = programId
    }}
    
    // TODO: Implement methods for each instruction
}}
"#,
        to_pascal_case(program_name),
        idl_path.file_stem().unwrap().to_str().unwrap()
    );

    fs::write(sources_dir.join("Client.swift"), swift_content)?;

    Ok(())
}

fn generate_kotlin_bindings(idl_path: &Path, output_dir: &Path, program_name: &str) -> Result<()> {
    let kotlin_dir = output_dir.join("kotlin").join(program_name);
    fs::create_dir_all(&kotlin_dir)?;

    // Generate build.gradle.kts
    let build_gradle = format!(
        r#"plugins {{
    kotlin("jvm") version "1.9.0"
    `maven-publish`
}}

group = "com.typhoon.{program_name}"
version = "0.1.0"

repositories {{
    mavenCentral()
}}

dependencies {{
    implementation("com.solana:solana-kotlin:1.0.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.7.0")
    testImplementation(kotlin("test"))
}}

tasks.test {{
    useJUnitPlatform()
}}"#
    );

    fs::write(kotlin_dir.join("build.gradle.kts"), build_gradle)?;

    // Set up Maven-style directory layout
    let src_dir = kotlin_dir
        .join("src")
        .join("main")
        .join("kotlin")
        .join("com")
        .join("typhoon")
        .join(program_name);
    fs::create_dir_all(&src_dir)?;

    // Generate Kotlin client
    let kotlin_content = format!(
        r#"package com.typhoon.{}

import com.solana.core.PublicKey
import com.solana.core.Transaction
import com.solana.rpc.Connection

class {}Client(
    private val connection: Connection,
    private val programId: PublicKey = PublicKey("{}")
) {{
    // TODO: Implement methods for each instruction
}}
"#,
        program_name,
        to_pascal_case(program_name),
        idl_path.file_stem().unwrap().to_str().unwrap()
    );

    fs::write(src_dir.join("Client.kt"), kotlin_content)?;

    Ok(())
}

fn generate_rust_bindings(idl_path: &Path, output_dir: &Path, program_name: &str) -> Result<()> {
    let rust_dir = output_dir.join("rust").join(program_name);
    fs::create_dir_all(&rust_dir)?;

    // Generate Cargo.toml
    let cargo_toml = format!(
        r#"[package]
name = "{program_name}-client"
version = "0.1.0"
edition = "2021"

[dependencies]
solana-sdk = "2.1"
solana-client = "2.1"
borsh = "1.5"
thiserror = "1.0"
"#
    );

    fs::write(rust_dir.join("Cargo.toml"), cargo_toml)?;

    // Set up Rust library structure
    let src_dir = rust_dir.join("src");
    fs::create_dir_all(&src_dir)?;

    // Generate lib.rs
    let lib_content = format!(
        r#"use solana_sdk::{{
    instruction::{{AccountMeta, Instruction}},
    pubkey::Pubkey,
    signer::Signer,
}};

pub const PROGRAM_ID: Pubkey = solana_sdk::pubkey!("{}");

pub struct {}Client {{
    program_id: Pubkey,
}}

impl {}Client {{
    pub fn new() -> Self {{
        Self {{
            program_id: PROGRAM_ID,
        }}
    }}

    pub fn with_program_id(program_id: Pubkey) -> Self {{
        Self {{ program_id }}
    }}

    // TODO: Implement instruction builders
}}
"#,
        idl_path.file_stem().unwrap().to_str().unwrap(),
        to_pascal_case(program_name),
        to_pascal_case(program_name)
    );

    fs::write(src_dir.join("lib.rs"), lib_content)?;

    Ok(())
}

// Utility functions for string case conversion
/// Convert snake_case to PascalCase
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

/// Convert snake_case to camelCase
fn to_camel_case(s: &str) -> String {
    let pascal = to_pascal_case(s);
    let mut chars = pascal.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

fn generate_type_imports(idl: &Value) -> Result<String> {
    let mut imports = Vec::new();

    if let Some(accounts) = idl["accounts"].as_array() {
        for account in accounts {
            if let Some(name) = account["name"].as_str() {
                imports.push(to_pascal_case(name));
            }
        }
    }

    Ok(imports.join(", "))
}

fn generate_typescript_type(account: &Value) -> Result<String> {
    let name = account["name"].as_str().unwrap_or("Unknown");
    let mut type_def = format!("export interface {} {{\n", to_pascal_case(name));

    if let Some(fields) = account["type"]["fields"].as_array() {
        for field in fields {
            let field_name = field["name"].as_str().unwrap_or("unknown");
            let field_type = map_to_typescript_type(field["type"].as_str().unwrap_or("unknown"));
            type_def.push_str(&format!(
                "  {}: {};\n",
                to_camel_case(field_name),
                field_type
            ));
        }
    }

    type_def.push_str("}\n");
    Ok(type_def)
}

fn generate_instruction_args_type(instruction: &Value) -> Result<String> {
    let name = instruction["name"].as_str().unwrap_or("unknown");
    let mut type_def = format!("export interface {}Args {{\n", to_pascal_case(name));

    if let Some(args) = instruction["args"].as_array() {
        for arg in args {
            let arg_name = arg["name"].as_str().unwrap_or("unknown");
            let arg_type = map_to_typescript_type(arg["type"].as_str().unwrap_or("unknown"));
            type_def.push_str(&format!("  {}: {};\n", to_camel_case(arg_name), arg_type));
        }
    }

    type_def.push_str("}\n");
    Ok(type_def)
}

fn map_to_typescript_type(rust_type: &str) -> &str {
    match rust_type {
        "u8" | "u16" | "u32" | "u64" | "u128" | "i8" | "i16" | "i32" | "i64" | "i128" => "number",
        "bool" => "boolean",
        "String" => "string",
        "Pubkey" | "publicKey" => "PublicKey",
        _ => "any",
    }
}
