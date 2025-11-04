//! Typhoon CLI binary entry point.

fn main() {
    if let Err(e) = typhoon_cli::run() {
        typhoon_cli::output::error(&e.to_string());
        std::process::exit(1);
    }
}
