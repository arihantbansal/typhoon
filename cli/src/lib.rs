use clap::Parser;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Parser)]
pub enum Command {
    /// Create a new Typhoon project
    Init { name: String },
}
