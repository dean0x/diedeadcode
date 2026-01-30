//! Configuration schema for ddd.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;

/// Main configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    /// Root directory to analyze (defaults to current directory).
    #[serde(default)]
    pub root: Option<PathBuf>,

    /// Entry point configuration.
    #[serde(default)]
    pub entry: EntryConfig,

    /// Files/directories to include.
    #[serde(default)]
    pub include: Vec<String>,

    /// Files/directories to exclude.
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Output configuration.
    #[serde(default)]
    pub output: OutputConfig,

    /// Analysis configuration.
    #[serde(default)]
    pub analysis: AnalysisConfig,

    /// Plugin configuration.
    #[serde(default)]
    pub plugins: PluginsConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            root: None,
            entry: EntryConfig::default(),
            include: vec![
                "**/*.ts".to_string(),
                "**/*.tsx".to_string(),
                "**/*.js".to_string(),
                "**/*.jsx".to_string(),
                "**/*.mts".to_string(),
                "**/*.cts".to_string(),
            ],
            exclude: vec![
                "**/node_modules/**".to_string(),
                "**/dist/**".to_string(),
                "**/build/**".to_string(),
                "**/*.d.ts".to_string(),
                "**/*.test.*".to_string(),
                "**/*.spec.*".to_string(),
                "**/__tests__/**".to_string(),
            ],
            output: OutputConfig::default(),
            analysis: AnalysisConfig::default(),
            plugins: PluginsConfig::default(),
        }
    }
}

/// Entry point configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct EntryConfig {
    /// Explicit entry point files.
    #[serde(default)]
    pub files: Vec<PathBuf>,

    /// Entry point patterns (glob).
    #[serde(default)]
    pub patterns: Vec<String>,

    /// Whether to auto-detect entry points from package.json.
    #[serde(default = "default_true")]
    pub auto_detect: bool,

    /// Exported symbols to consider as entry points.
    #[serde(default)]
    pub exports: Vec<String>,
}

/// Output configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OutputConfig {
    /// Output format: table, json, or compact.
    #[serde(default)]
    pub format: OutputFormat,

    /// Minimum confidence level to report.
    #[serde(default)]
    pub min_confidence: ConfidenceLevel,

    /// Whether to show transitive chains.
    #[serde(default = "default_true")]
    pub show_chains: bool,

    /// Maximum chain length to display.
    #[serde(default = "default_chain_length")]
    pub max_chain_length: usize,

    /// Group output by file.
    #[serde(default = "default_true")]
    pub group_by_file: bool,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            format: OutputFormat::Table,
            min_confidence: ConfidenceLevel::High,
            show_chains: true,
            max_chain_length: 5,
            group_by_file: true,
        }
    }
}

/// Output format options.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Compact,
}

/// Confidence level filter.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum ConfidenceLevel {
    Low,
    Medium,
    #[default]
    High,
}

impl ConfidenceLevel {
    /// Convert to core::Confidence type.
    pub fn to_confidence(&self) -> crate::core::Confidence {
        match self {
            ConfidenceLevel::Low => crate::core::Confidence::Low,
            ConfidenceLevel::Medium => crate::core::Confidence::Medium,
            ConfidenceLevel::High => crate::core::Confidence::High,
        }
    }
}

/// Analysis configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalysisConfig {
    /// Include type-only dead code (interfaces, type aliases).
    #[serde(default = "default_true")]
    pub include_types: bool,

    /// Analyze test files separately.
    #[serde(default)]
    pub analyze_tests: bool,

    /// Report test-only dead code.
    #[serde(default)]
    pub report_test_only: bool,

    /// Follow re-exports.
    #[serde(default = "default_true")]
    pub follow_reexports: bool,

    /// Maximum depth for transitive analysis.
    #[serde(default = "default_max_depth")]
    pub max_transitive_depth: usize,

    /// Symbols to always consider alive (never report as dead).
    #[serde(default)]
    pub ignore_symbols: HashSet<String>,

    /// Patterns for symbols to ignore.
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            include_types: true,
            analyze_tests: false,
            report_test_only: false,
            follow_reexports: true,
            max_transitive_depth: 50,
            ignore_symbols: HashSet::new(),
            ignore_patterns: vec![
                "^_".to_string(), // Private by convention
            ],
        }
    }
}

/// Plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct PluginsConfig {
    /// Enabled plugins.
    #[serde(default)]
    pub enabled: Vec<String>,

    /// Disabled plugins (overrides auto-detection).
    #[serde(default)]
    pub disabled: Vec<String>,

    /// Auto-detect plugins from package.json.
    #[serde(default = "default_true")]
    pub auto_detect: bool,

    /// Next.js plugin configuration.
    #[serde(default)]
    pub nextjs: NextJsConfig,

    /// Jest plugin configuration.
    #[serde(default)]
    pub jest: JestConfig,

    /// Express plugin configuration.
    #[serde(default)]
    pub express: ExpressConfig,
}

/// Next.js plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct NextJsConfig {
    /// Additional page directories.
    #[serde(default)]
    pub page_dirs: Vec<PathBuf>,

    /// Additional app router directories.
    #[serde(default)]
    pub app_dirs: Vec<PathBuf>,
}

/// Jest plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct JestConfig {
    /// Test file patterns.
    #[serde(default)]
    pub test_patterns: Vec<String>,

    /// Setup file patterns.
    #[serde(default)]
    pub setup_files: Vec<PathBuf>,
}

/// Express plugin configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExpressConfig {
    /// Middleware registration patterns.
    #[serde(default)]
    pub middleware_patterns: Vec<String>,
}

// Default value helpers
fn default_true() -> bool {
    true
}

fn default_chain_length() -> usize {
    5
}

fn default_max_depth() -> usize {
    50
}

impl Config {
    /// Create a minimal config for quick analysis.
    pub fn minimal() -> Self {
        Self {
            exclude: vec!["**/node_modules/**".to_string()],
            ..Default::default()
        }
    }

    /// Check if a file path should be included in analysis.
    pub fn should_include(&self, path: &std::path::Path) -> bool {
        let path_str = path.to_string_lossy();

        // Check excludes first
        for pattern in &self.exclude {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
            {
                return false;
            }
        }

        // Check includes
        if self.include.is_empty() {
            return true;
        }

        for pattern in &self.include {
            if glob::Pattern::new(pattern)
                .map(|p| p.matches(&path_str))
                .unwrap_or(false)
            {
                return true;
            }
        }

        false
    }

    /// Check if a symbol name should be ignored.
    pub fn should_ignore_symbol(&self, name: &str) -> bool {
        if self.analysis.ignore_symbols.contains(name) {
            return true;
        }

        for pattern in &self.analysis.ignore_patterns {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if re.is_match(name) {
                    return true;
                }
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.entry.auto_detect);
        assert!(config.output.show_chains);
        assert_eq!(config.output.min_confidence, ConfidenceLevel::High);
    }

    #[test]
    fn test_should_include() {
        let config = Config::default();

        assert!(config.should_include(std::path::Path::new("src/foo.ts")));
        assert!(!config.should_include(std::path::Path::new("node_modules/foo.ts")));
        assert!(!config.should_include(std::path::Path::new("src/foo.test.ts")));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("include"));
        assert!(toml_str.contains("exclude"));
    }
}
