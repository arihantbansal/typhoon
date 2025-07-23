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

program_id!("11111111111111111111111111111111");

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

program_id!("Fg6PaFpoGXkYsidMpWTK6W2BeZ7FEfcYkg476zPFsLnS");

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

program_id!("11111111111111111111111111111111");

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

program_id!("11111111111111111111111111111111");

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

pub fn generate_test_template(program_name: &str) -> String {
    format!(
        r#"use {{
    litesvm::LiteSVM,
    solana_sdk::{{
        instruction::{{AccountMeta, Instruction}},
        pubkey::Pubkey,
        signature::{{Keypair, Signer}},
        transaction::Transaction,
    }},
    typhoon_instruction_builder::generate_instructions_client,
}};

generate_instructions_client!(
    CLIENT = {},
    PATH = "../target/idl/{}.json"
);

#[test]
fn test_initialize() {{
    let mut svm = LiteSVM::new();
    svm.airdrop(&svm.payer().pubkey(), 10_000_000_000).unwrap();

    // TODO: Implement test logic
    
    assert!(true);
}}
"#,
        program_name.replace("-", "_"),
        program_name.replace("-", "_")
    )
}
