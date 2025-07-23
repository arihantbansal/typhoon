//! Typhoon CLI entry point
//! Handles command parsing and delegates to appropriate modules

use {
    anyhow::Result,
    clap::Parser,
    colored::Colorize,
    typhoon_cli::{AddSubcommand, Cli, Command, KeysSubcommand, SecuritySubcommand},
};

/// Main entry point for the Typhoon CLI
/// Parses command line arguments and executes the appropriate command
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::New {
            name,
            program,
            template,
            from,
        } => {
            println!(
                "{} Creating new {}...",
                "▶".blue().bold(),
                if program { "program" } else { "workspace" }
            );
            if program {
                typhoon_cli::scaffold::create_program(&name, template.as_deref(), from.as_deref())
                    .await?;
            } else {
                typhoon_cli::workspace::create_workspace(
                    &name,
                    template.as_deref(),
                    from.as_deref(),
                )
                .await?;
            }
            println!(
                "{} Successfully created {}",
                "✓".green().bold(),
                name.green()
            );
        }
        Command::Add { subcommand } => match subcommand {
            AddSubcommand::Program { name, template } => {
                println!("{} Adding new program '{}'...", "▶".blue().bold(), name);
                typhoon_cli::scaffold::add_program(&name, template.as_deref()).await?;
                println!(
                    "{} Successfully added program {}",
                    "✓".green().bold(),
                    name.green()
                );
            }
            AddSubcommand::Instruction { program, name } => {
                println!(
                    "{} Adding instruction '{}' to program '{}'...",
                    "▶".blue().bold(),
                    name,
                    program
                );
                typhoon_cli::scaffold::add_instruction(&program, &name).await?;
                println!(
                    "{} Successfully added instruction {}",
                    "✓".green().bold(),
                    name.green()
                );
            }
        },
        Command::Build { program, idl } => {
            println!("{} Building...", "▶".blue().bold());
            typhoon_cli::build::build(program.as_deref(), idl).await?;
            println!("{} Build successful", "✓".green().bold());
        }
        Command::Test { program, test } => {
            println!("{} Running tests...", "▶".blue().bold());
            typhoon_cli::test::run_tests(program.as_deref(), test.as_deref()).await?;
            println!("{} Tests passed", "✓".green().bold());
        }
        Command::Security { subcommand } => match subcommand {
            SecuritySubcommand::Audit => {
                println!("{} Running security audit...", "▶".blue().bold());
                typhoon_cli::security::run_audit().await?;
                println!("{} Security audit complete", "✓".green().bold());
            }
            SecuritySubcommand::Verify {
                program,
                repo_url,
                commit_hash,
                current_dir,
                program_id,
                cluster,
            } => {
                println!("{} Running verifiable build...", "▶".blue().bold());
                typhoon_cli::security::run_verify(
                    program.as_deref(),
                    repo_url.as_deref(),
                    commit_hash.as_deref(),
                    current_dir,
                    program_id.as_deref(),
                    &cluster,
                )
                .await?;
                println!("{} Verification complete", "✓".green().bold());
            }
            SecuritySubcommand::VerifyRepo {
                repo_url,
                program_id,
                commit_hash,
                cluster,
                mount_path,
            } => {
                println!(
                    "{} Verifying program against repository...",
                    "▶".blue().bold()
                );
                typhoon_cli::security::verify_from_repo(
                    &repo_url,
                    &program_id,
                    commit_hash.as_deref(),
                    &cluster,
                    mount_path.as_deref(),
                )
                .await?;
                println!("{} Repository verification complete", "✓".green().bold());
            }
        },
        Command::GenerateBindings { languages, output } => {
            println!("{} Generating client bindings...", "▶".blue().bold());
            typhoon_cli::bindings::generate_bindings(&languages, output.as_deref()).await?;
            println!("{} Bindings generated successfully", "✓".green().bold());
        }
        Command::Idl { program } => {
            println!("{} Generating IDL...", "▶".blue().bold());
            typhoon_cli::build::generate_idl(program.as_deref()).await?;
            println!("{} IDL generated successfully", "✓".green().bold());
        }
        Command::Deploy {
            program: _,
            network: _,
        } => {
            // TODO: Implement deployment functionality
            anyhow::bail!("Deploy command is not yet implemented")
        }
        Command::Keys { subcommand } => match subcommand {
            KeysSubcommand::List => {
                typhoon_cli::keys::list()?;
            }
            KeysSubcommand::Sync { program_name } => {
                typhoon_cli::keys::sync(program_name)?;
            }
        },
    }

    Ok(())
}
