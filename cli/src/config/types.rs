//! Configuration type definitions.

use serde::Deserialize;

/// Main configuration structure.
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub build: BuildConfig,

    #[serde(default)]
    pub test: TestConfig,

    #[serde(default)]
    pub project: ProjectConfig,
}

/// Build configuration.
#[derive(Debug, Deserialize)]
pub struct BuildConfig {
    #[serde(default = "default_true")]
    pub release: bool,

    #[serde(default)]
    pub features: Vec<String>,
}

impl Default for BuildConfig {
    fn default() -> Self {
        Self {
            release: true,
            features: Vec::new(),
        }
    }
}

/// Test configuration.
#[derive(Debug, Deserialize, Default)]
pub struct TestConfig {
    #[serde(default = "default_litesvm")]
    pub validator: String,
}

/// Project configuration.
#[derive(Debug, Deserialize, Default)]
pub struct ProjectConfig {
    pub name: Option<String>,
}

fn default_true() -> bool {
    true
}

fn default_litesvm() -> String {
    "litesvm".to_string()
}
