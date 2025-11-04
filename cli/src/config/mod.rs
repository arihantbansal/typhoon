//! Configuration discovery and loading.

pub mod types;

use crate::{constants::TYPHOON_TOML, Result};
pub use types::Config;

/// Loads configuration from Typhoon.toml if it exists, otherwise returns defaults.
///
/// # Errors
/// Returns an error if the config file exists but is malformed.
pub fn load() -> Result<Config> {
    if std::path::Path::new(TYPHOON_TOML).exists() {
        let content = std::fs::read_to_string(TYPHOON_TOML)?;
        let config = toml::from_str(&content)?;
        Ok(config)
    } else {
        Ok(Config::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.build.release);
        assert_eq!(config.build.features.len(), 0);
        assert_eq!(config.test.validator, "litesvm");
    }

    #[test]
    fn test_config_parsing() {
        let toml_str = r#"
[build]
release = false
features = ["logging"]

[test]
validator = "test-validator"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(!config.build.release);
        assert_eq!(config.build.features, vec!["logging"]);
        assert_eq!(config.test.validator, "test-validator");
    }
}
