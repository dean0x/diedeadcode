//! CLI module for ddd command.

pub mod commands;
pub mod output;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

use crate::config::ConfidenceLevel;

/// ddd - Conservative TypeScript dead code detection.
#[derive(Parser, Debug)]
#[command(
    name = "ddd",
    version,
    about = "Conservative TypeScript dead code detection with transitive analysis and confidence scoring",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Path to analyze (defaults to current directory)
    #[arg(global = true, default_value = ".")]
    pub path: PathBuf,

    /// Path to config file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(short, long, global = true)]
    pub quiet: bool,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Initialize a new ddd.toml configuration file
    Init(InitArgs),

    /// Analyze the codebase for dead code
    Analyze(AnalyzeArgs),

    /// Watch for file changes and analyze continuously
    Watch(WatchArgs),
}

/// Arguments for the init command.
#[derive(Parser, Debug, Clone)]
pub struct InitArgs {
    /// Overwrite existing config file
    #[arg(short, long)]
    pub force: bool,

    /// Output format (toml or json)
    #[arg(short, long, default_value = "toml")]
    pub format: String,
}

/// Arguments for the analyze command.
#[derive(Parser, Debug, Clone)]
pub struct AnalyzeArgs {
    /// Output format: table, json, or compact
    #[arg(short, long, default_value = "table")]
    pub format: OutputFormat,

    /// Minimum confidence level to report: high, medium, or low
    #[arg(long, default_value = "high")]
    pub confidence: ConfidenceLevel,

    /// Show transitive dead code chains
    #[arg(long)]
    pub show_chains: bool,

    /// Only check, exit with error if dead code found
    #[arg(long)]
    pub check: bool,

    /// Show progress during analysis
    #[arg(long)]
    pub progress: bool,

    /// Number of parallel workers
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Include test files in analysis
    #[arg(long)]
    pub include_tests: bool,
}

impl Default for AnalyzeArgs {
    fn default() -> Self {
        Self {
            format: OutputFormat::Table,
            confidence: ConfidenceLevel::High,
            show_chains: false,
            check: false,
            progress: false,
            jobs: None,
            include_tests: false,
        }
    }
}

/// Arguments for the watch command.
#[derive(Parser, Debug, Clone)]
pub struct WatchArgs {
    /// Debounce delay in milliseconds
    #[arg(long, default_value = "500")]
    pub debounce: u64,

    /// Clear screen on each run
    #[arg(long)]
    pub clear: bool,
}

/// Output format for analysis results.
#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
pub enum OutputFormat {
    #[default]
    Table,
    Json,
    Compact,
}

impl Cli {
    /// Parse CLI arguments.
    pub fn parse_args() -> Self {
        Self::parse()
    }

    /// Get the effective command, defaulting to analyze.
    pub fn effective_command(&self) -> Commands {
        self.command.clone().unwrap_or(Commands::Analyze(AnalyzeArgs::default()))
    }
}

impl From<OutputFormat> for crate::config::OutputFormat {
    fn from(f: OutputFormat) -> Self {
        match f {
            OutputFormat::Table => crate::config::OutputFormat::Table,
            OutputFormat::Json => crate::config::OutputFormat::Json,
            OutputFormat::Compact => crate::config::OutputFormat::Compact,
        }
    }
}
