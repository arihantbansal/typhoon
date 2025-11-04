//! Colored terminal output helpers.

use console::{style, Emoji};

pub static SUCCESS: Emoji = Emoji("✓", "+");
pub static ERROR: Emoji = Emoji("✗", "x");
pub static INFO: Emoji = Emoji("ℹ", "i");
pub static WARNING: Emoji = Emoji("⚠", "!");
pub static ARROW: Emoji = Emoji("→", ">");

pub fn success(msg: &str) {
    println!("{} {}", style(SUCCESS).green().bold(), msg);
}

pub fn error(msg: &str) {
    eprintln!("{} {}", style(ERROR).red().bold(), msg);
}

pub fn info(msg: &str) {
    println!("{} {}", style(INFO).cyan().bold(), msg);
}

pub fn warning(msg: &str) {
    println!("{} {}", style(WARNING).yellow().bold(), msg);
}

pub fn step(msg: &str) {
    println!("{} {}", style(ARROW).dim(), style(msg).dim());
}

pub fn command(cmd: &str) {
    println!("  {}", style(cmd).cyan());
}

pub fn header(msg: &str) {
    println!("\n{}", style(msg).bold());
}
