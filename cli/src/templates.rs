pub const GITIGNORE_TEMPLATE: &str = r#"
# Rust
target/
Cargo.lock
**/*.rs.bk

# Solana
.anchor/
.cache/
node_modules/
test-ledger/

# IDL
target/idl/
target/deploy/

# Environment
.env
.env.local

# IDE
.vscode/
.idea/
*.swp
*.swo
*~

# OS
.DS_Store
Thumbs.db
"#;

pub const WORKSPACE_MAKEFILE_TEMPLATE: &str = r#"[env]
CARGO_MAKE_WORKSPACE_EMULATION = true

[tasks.build]
command = "typhoon"
args = ["build"]

[tasks.test]
command = "typhoon"
args = ["test"]

[tasks.idl]
command = "typhoon"
args = ["idl"]
"#;

pub const CARGO_MAKE_CI_TEMPLATE: &str = r#"[tasks.test]
command = "cargo"
args = ["test-sbf"]

[tasks.fmt]
command = "cargo"
args = ["fmt", "--all"]

[tasks.build]
command = "cargo"
args = ["build-sbf"]

[tasks.check]
command = "cargo"
args = ["check"]
"#;

pub const PROGRAM_MAKEFILE_TEMPLATE: &str = r#"extend = [{ path = "../cargo-make/ci.toml" }]

[tasks.build]
command = "cargo"
args = ["build-sbf"]

[tasks.test]
command = "cargo"
args = ["test-sbf"]

[tasks.idl]
command = "cargo"
args = ["build", "--release"]
"#;

pub fn generate_build_rs_template(program_name: &str) -> String {
    format!(
        r#"use std::{{env, fs, path::Path}};

fn main() {{
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();

    let target_dir = Path::new(&manifest_dir).join("target");
    let idl_dir = target_dir.join("idl");

    fs::create_dir_all(&idl_dir).unwrap();

    let idl = typhoon_idl_generator::generate(manifest_dir).unwrap();

    fs::write(idl_dir.join("{}.json"), idl).unwrap();
}}
"#,
        program_name.replace("-", "_")
    )
}

pub const HELLO_WORLD_TEMPLATE: &str = r#"#![no_std]

use typhoon::prelude::*;

program_id!("22222222222222222222222222222222222222222222");

nostd_panic_handler!();
no_allocator!();

#[context]
pub struct InitializeContext {
    pub payer: Mut<Signer>,
    pub system_program: Program<System>,
}

handlers! {
    initialize,
}

pub fn initialize(ctx: InitializeContext) -> ProgramResult {
    msg!("Hello, Typhoon!");
    Ok(())
}
"#;

pub const COUNTER_TEMPLATE: &str = r#"#![no_std]

use {
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
};

program_id!("22222222222222222222222222222222222222222222");

nostd_panic_handler!();
no_allocator!();

#[context]
pub struct InitContext {
    pub payer: Mut<Signer>,
    #[constraint(
        init,
        payer = payer,
    )]
    pub counter: Mut<Account<Counter>>,
    pub system: Program<System>,
}

#[context]
pub struct CounterMutContext {
    pub counter: Mut<Account<Counter>>,
}

#[context]
pub struct DestinationContext {
    pub destination: Mut<SystemAccount>,
}

handlers! {
    initialize,
    increment,
    close
}

pub fn initialize(_: InitContext) -> ProgramResult {
    Ok(())
}

pub fn increment(ctx: CounterMutContext) -> ProgramResult {
    ctx.counter.mut_data()?.count += 1;

    Ok(())
}

pub fn close(
    CounterMutContext { counter }: CounterMutContext,
    DestinationContext { destination }: DestinationContext,
) -> ProgramResult {
    counter.close(&destination)?;

    Ok(())
}

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
pub struct Counter {
    pub count: u64,
}
"#;

pub const TRANSFER_TEMPLATE: &str = r#"#![no_std]

use typhoon::prelude::*;

program_id!("22222222222222222222222222222222222222222222");

nostd_panic_handler!();
no_allocator!();

#[context]
pub struct TransferContext {
    pub from: Mut<Signer>,
    pub to: Mut<SystemAccount>,
    pub system_program: Program<System>,
}

handlers! {
    transfer,
}

pub fn transfer(ctx: TransferContext, amount: u64) -> ProgramResult {
    // Transfer SOL from 'from' to 'to'
    ctx.from.transfer_lamports(&ctx.to, amount)?;
    
    msg!("Transferred {} lamports", amount);
    Ok(())
}
"#;

pub const TOKEN_TEMPLATE: &str = r#"#![no_std]

use {
    bytemuck::{AnyBitPattern, NoUninit},
    typhoon::prelude::*,
};

program_id!("22222222222222222222222222222222222222222222");

nostd_panic_handler!();
no_allocator!();

#[context]
pub struct InitializeTokenContext {
    pub payer: Mut<Signer>,
    #[constraint(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<TokenMint>(),
    )]
    pub mint: Mut<Account<TokenMint>>,
    pub system_program: Program<System>,
}

#[context]
pub struct CreateAccountContext {
    pub payer: Mut<Signer>,
    pub owner: SystemAccount,
    #[constraint(
        init,
        payer = payer,
        space = 8 + std::mem::size_of::<TokenAccount>(),
    )]
    pub token_account: Mut<Account<TokenAccount>>,
    pub mint: Account<TokenMint>,
    pub system_program: Program<System>,
}

#[context]
pub struct MintToContext {
    pub authority: Mut<Signer>,
    pub mint: Mut<Account<TokenMint>>,
    pub to: Mut<Account<TokenAccount>>,
}

#[context]
pub struct TransferContext {
    pub authority: Mut<Signer>,
    pub from: Mut<Account<TokenAccount>>,
    pub to: Mut<Account<TokenAccount>>,
}

handlers! {
    initialize_token,
    create_account,
    mint_to,
    transfer,
}

pub fn initialize_token(ctx: InitializeTokenContext, decimals: u8) -> ProgramResult {
    let mint = ctx.mint.mut_data()?;
    mint.supply = 0;
    mint.decimals = decimals;
    mint.mint_authority = *ctx.payer.key();
    mint.is_initialized = true;
    Ok(())
}

pub fn create_account(ctx: CreateAccountContext) -> ProgramResult {
    let account = ctx.token_account.mut_data()?;
    account.mint = *ctx.mint.key();
    account.owner = *ctx.owner.key();
    account.amount = 0;
    account.is_initialized = true;
    Ok(())
}

pub fn mint_to(ctx: MintToContext, amount: u64) -> ProgramResult {
    let mint = ctx.mint.mut_data()?;
    
    // Check authority
    if mint.mint_authority != *ctx.authority.key() {
        return Err(ProgramError::InvalidAccountData);
    }
    
    // Update supply
    mint.supply = mint.supply.checked_add(amount)
        .ok_or(ProgramError::InvalidAccountData)?;
    
    // Update token account
    let account = ctx.to.mut_data()?;
    account.amount = account.amount.checked_add(amount)
        .ok_or(ProgramError::InvalidAccountData)?;
    
    Ok(())
}

pub fn transfer(ctx: TransferContext, amount: u64) -> ProgramResult {
    let from_account = ctx.from.mut_data()?;
    let to_account = ctx.to.mut_data()?;
    
    // Check owner
    if from_account.owner != *ctx.authority.key() {
        return Err(ProgramError::InvalidAccountData);
    }
    
    // Check same mint
    if from_account.mint != to_account.mint {
        return Err(ProgramError::InvalidAccountData);
    }
    
    // Transfer tokens
    from_account.amount = from_account.amount.checked_sub(amount)
        .ok_or(ProgramError::InsufficientFunds)?;
    to_account.amount = to_account.amount.checked_add(amount)
        .ok_or(ProgramError::InvalidAccountData)?;
    
    Ok(())
}

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
pub struct TokenMint {
    pub supply: u64,
    pub decimals: u8,
    pub is_initialized: bool,
    pub mint_authority: Pubkey,
}

#[derive(NoUninit, AnyBitPattern, AccountState, Copy, Clone)]
#[repr(C)]
pub struct TokenAccount {
    pub mint: Pubkey,
    pub owner: Pubkey,
    pub amount: u64,
    pub is_initialized: bool,
}
"#;

pub fn generate_test_template(program_name: &str, template: Option<&str>) -> String {
    match template {
        Some("hello-world") | None => generate_hello_world_test_template(program_name),
        Some("counter") => generate_counter_test_template(program_name),
        Some("transfer") => generate_transfer_test_template(program_name),
        Some("token") => generate_token_test_template(program_name),
        _ => generate_hello_world_test_template(program_name),
    }
}

fn generate_program_id_reader_code(program_name: &str) -> String {
    format!(
        r#"fn read_program_id() -> Pubkey {{
    // Read program ID from typhoon.toml
    let mut toml_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    toml_path.pop(); // Go up to workspace root
    toml_path.push("typhoon.toml");

    if let Ok(content) = fs::read_to_string(&toml_path) {{
        if let Ok(config) = toml::from_str::<toml::Value>(&content) {{
            if let Some(program_ids) = config.get("program_ids") {{
                if let Some(program_id_str) = program_ids.get("{program_name}") {{
                    if let Some(id_str) = program_id_str.as_str() {{
                        if let Ok(pubkey) = Pubkey::from_str(id_str) {{
                            return pubkey;
                        }}
                    }}
                }}
            }}
        }}
    }}

    // Fallback to reading from source file
    let mut lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    lib_path.push("src/lib.rs");
    
    if let Ok(content) = fs::read_to_string(&lib_path) {{
        if let Some(start) = content.find("program_id!(\"") {{
            let start = start + 13; // Length of "program_id!(\""
            if let Some(end) = content[start..].find("\"") {{
                let id_str = &content[start..start + end];
                if let Ok(pubkey) = Pubkey::from_str(id_str) {{
                    return pubkey;
                }}
            }}
        }}
    }}

    panic!("Could not find program ID in typhoon.toml or source file");
}}"#,
    )
}

pub fn generate_hello_world_test_template(program_name: &str) -> String {
    let program_name_snake = program_name.replace("-", "_");
    let program_id_reader = generate_program_id_reader_code(program_name);

    format!(
        r#"use {{
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_native_token::LAMPORTS_PER_SOL,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    std::{{fs, path::PathBuf, str::FromStr}},
    toml,
    typhoon_instruction_builder::generate_instructions_client,
}};

fn read_program() -> Vec<u8> {{
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("target/deploy/{program_name_snake}.so");

    std::fs::read(so_path).unwrap()
}}

{program_id_reader}

generate_instructions_client!({program_name_snake});

#[test]
fn integration_test() {{
    let mut svm = LiteSVM::new();
    let admin_kp = Keypair::new();
    let admin_pk = admin_kp.pubkey();

    svm.airdrop(&admin_pk, 10 * LAMPORTS_PER_SOL).unwrap();

    let program_bytes = read_program();
    let program_id = read_program_id();

    svm.add_program(program_id, &program_bytes);

    let ix = InitializeInstruction {{
        payer: admin_pk,
        system_program: solana_system_interface::program::ID,
    }}.into_instruction();
    let hash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&admin_pk), &[&admin_kp], hash);

    let res = svm.send_transaction(tx).unwrap();

    assert!(res.logs[0].contains(&format!("Program {{}} invoke [1]", program_id)));
    assert_eq!(res.logs[1], "Program log: Hello, Typhoon!");
}}
"#
    )
}

pub fn generate_counter_test_template(program_name: &str) -> String {
    let program_name_snake = program_name.replace("-", "_");
    let program_id_reader = generate_program_id_reader_code(program_name);

    format!(
        r#"use {{
    {program_name_snake}::Counter,
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_native_token::LAMPORTS_PER_SOL,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    std::{{fs, path::PathBuf, str::FromStr}},
    toml,
    typhoon::lib::RefFromBytes,
    typhoon_instruction_builder::generate_instructions_client,
}};

fn read_program() -> Vec<u8> {{
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("target/deploy/{program_name_snake}.so");

    std::fs::read(so_path).unwrap()
}}

{program_id_reader}

generate_instructions_client!({program_name_snake});

#[test]
fn integration_test() {{
    let mut svm = LiteSVM::new();
    let admin_kp = Keypair::new();
    let admin_pk = admin_kp.pubkey();

    svm.airdrop(&admin_pk, 10 * LAMPORTS_PER_SOL).unwrap();

    let program_bytes = read_program();
    let program_id = read_program_id();
    svm.add_program(program_id, &program_bytes);

    // Create the counter
    let counter_kp = Keypair::new();
    let counter_pk = counter_kp.pubkey();
    let ix = InitializeInstruction {{
        payer: admin_pk,
        counter: counter_pk,
        system: solana_system_interface::program::ID,
    }}
    .into_instruction();
    let hash = svm.latest_blockhash();
    let tx =
        Transaction::new_signed_with_payer(&[ix], Some(&admin_pk), &[&admin_kp, &counter_kp], hash);
    svm.send_transaction(tx).unwrap();

    let raw_account = svm.get_account(&counter_pk).unwrap();
    let counter_account: &Counter = Counter::read(raw_account.data.as_slice()).unwrap();
    assert!(counter_account.count == 0);

    // Increment the counter
    let ix = IncrementInstruction {{
        counter: counter_pk,
    }}
    .into_instruction();
    let hash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&admin_pk), &[&admin_kp], hash);
    svm.send_transaction(tx).unwrap();

    let raw_account = svm.get_account(&counter_pk).unwrap();
    let counter_account: &Counter = Counter::read(raw_account.data.as_slice()).unwrap();
    assert!(counter_account.count == 1);

    // Close the counter
    let ix = CloseInstruction {{
        counter: counter_pk,
        destination: admin_pk,
    }}
    .into_instruction();
    let hash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&admin_pk), &[&admin_kp], hash);
    svm.send_transaction(tx).unwrap();

    let raw_account = svm.get_account(&counter_pk).unwrap();
    assert_eq!(raw_account.owner, solana_system_interface::program::ID);
    assert_eq!(raw_account.lamports, 0);
}}
"#
    )
}

pub fn generate_transfer_test_template(program_name: &str) -> String {
    let program_name_snake = program_name.replace("-", "_");
    let program_id_reader = generate_program_id_reader_code(program_name);

    format!(
        r#"use {{
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_native_token::LAMPORTS_PER_SOL,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    std::{{fs, path::PathBuf, str::FromStr}},
    toml,
    {program_name_snake}::PodU64,
    typhoon_instruction_builder::generate_instructions_client,
}};

fn read_program() -> Vec<u8> {{
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("target/deploy/{program_name_snake}.so");

    std::fs::read(so_path).unwrap()
}}

{program_id_reader}

generate_instructions_client!(
    {program_name_snake},
    [transfer_sol_with_cpi, transfer_sol_with_program]
);

#[test]
fn integration_test() {{
    let mut svm = LiteSVM::new();
    let admin_kp = Keypair::new();
    let admin_pk = admin_kp.pubkey();

    let recipient_kp = Keypair::new();
    let recipient_pk = recipient_kp.pubkey();

    svm.airdrop(&admin_pk, 10 * LAMPORTS_PER_SOL).unwrap();

    let program_bytes = read_program();
    let program_id = read_program_id();

    svm.add_program(program_id, &program_bytes);

    let admin_balance = svm.get_balance(&admin_pk).unwrap_or_default();
    let recipient_balance = svm.get_balance(&recipient_pk).unwrap_or_default();
    assert_eq!(admin_balance, 10 * LAMPORTS_PER_SOL);
    assert_eq!(recipient_balance, 0);

    // Transfer with CPI

    let amount = LAMPORTS_PER_SOL;
    let ix = TransferSolWithCpiInstruction {{
        arg_0: amount.into(),
        payer: admin_pk,
        recipient: recipient_pk,
        system: solana_system_interface::program::ID,
    }}
    .into_instruction();

    let hash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&admin_pk), &[&admin_kp], hash);

    let res = svm.send_transaction(tx);
    assert!(res.is_ok());

    let admin_balance = svm.get_balance(&admin_pk).unwrap_or_default();
    let recipient_balance = svm.get_balance(&recipient_pk).unwrap_or_default();
    assert!(admin_balance > 8 * LAMPORTS_PER_SOL);
    assert_eq!(recipient_balance, LAMPORTS_PER_SOL);

    // Transfer with program

    let program_acc_kp = Keypair::new();
    let program_acc_pk = program_acc_kp.pubkey();

    let pre_ix = solana_system_interface::instruction::create_account(
        &admin_pk,
        &program_acc_pk,
        LAMPORTS_PER_SOL,
        0,
        &program_id,
    );

    let amount = LAMPORTS_PER_SOL;
    let ix = TransferSolWithProgramInstruction {{
        arg_0: amount.into(),
        payer: program_acc_pk,
        recipient: admin_pk,
    }}
    .into_instruction();
    let hash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(
        &[pre_ix, ix],
        Some(&admin_pk),
        &[&admin_kp, &program_acc_kp],
        hash,
    );

    let res = svm.send_transaction(tx);
    assert!(res.is_ok());

    let admin_balance = svm.get_balance(&admin_pk).unwrap_or_default();
    let program_acc = svm.get_balance(&program_acc_pk).unwrap_or_default();
    assert!(admin_balance > 8 * LAMPORTS_PER_SOL);
    assert_eq!(program_acc, 0);
}}
"#
    )
}

pub fn generate_token_test_template(program_name: &str) -> String {
    let program_name_snake = program_name.replace("-", "_");
    let program_id_reader = generate_program_id_reader_code(program_name);

    format!(
        r#"use {{
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_native_token::LAMPORTS_PER_SOL,
    solana_pubkey::Pubkey,
    solana_signer::Signer,
    solana_transaction::Transaction,
    std::{{fs, path::PathBuf, str::FromStr}},
    toml,
    {program_name_snake}::MintFromEscrowArgs,
    typhoon_instruction_builder::generate_instructions_client,
}};

fn read_program() -> Vec<u8> {{
    let mut so_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    so_path.push("target/deploy/{program_name_snake}.so");

    std::fs::read(so_path).unwrap()
}}

{program_id_reader}

generate_instructions_client!({program_name_snake});

#[test]
fn integration_test() {{
    let mut svm = LiteSVM::new();
    let admin_kp = Keypair::new();
    let admin_pk = admin_kp.pubkey();

    let owner_kp = Keypair::new();
    let owner_pk = owner_kp.pubkey();

    svm.airdrop(&admin_pk, 10 * LAMPORTS_PER_SOL).unwrap();

    let program_bytes = read_program();
    let program_id = read_program_id();
    svm.add_program(program_id, &program_bytes);

    // Test mint from escrow instruction
    let mint_kp = Keypair::new();
    let mint_pk = mint_kp.pubkey();

    let args = MintFromEscrowArgs {{
        amount: 1_000_000,
        decimals: 6,
    }};

    let ix = MintFromEscrowInstruction {{
        args,
        payer: admin_pk,
        owner: owner_pk,
        mint: mint_pk,
        escrow: Pubkey::find_program_address(&[b"escrow"], &program_id).0,
        token_account: spl_associated_token_account::get_associated_token_address(&owner_pk, &mint_pk),
        token_program: spl_token::id(),
        ata_program: spl_associated_token_account::id(),
        system_program: solana_system_interface::program::ID,
    }}
    .into_instruction();

    let hash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(&[ix], Some(&admin_pk), &[&admin_kp, &mint_kp], hash);

    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "Mint from escrow should succeed");

    // Verify mint account was created
    let mint_account = svm.get_account(&mint_pk);
    assert!(mint_account.is_some(), "Mint account should exist");

    // Verify token account was created and has tokens
    let token_account_pk = spl_associated_token_account::get_associated_token_address(&owner_pk, &mint_pk);
    let token_account = svm.get_account(&token_account_pk);
    assert!(token_account.is_some(), "Token account should exist");
}}
"#
    )
}
