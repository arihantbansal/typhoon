//! Typhoon CLI library interface.
//!
//! This library provides the core functionality for the Typhoon CLI tool.
//! It can be used programmatically or through the `typhoon` binary.

pub mod checks;
pub mod cli;
pub mod commands;
pub mod error;
pub mod output;
pub mod templates;

mod constants;
mod keypair;
mod utils;

pub use {
    cli::run,
    error::{Error, Result},
};
