//! The `ddd init` command implementation.

use crate::cli::InitArgs;
use crate::config::generate_default_config;
use crate::core::{DddError, Result};
use colored::Colorize;
use std::path::Path;

/// Run the init command.
pub fn run_init(args: &InitArgs, path: &Path) -> Result<()> {
    let config_filename = match args.format.as_str() {
        "json" => "ddd.json",
        _ => "ddd.toml",
    };

    let config_path = path.join(config_filename);

    if config_path.exists() && !args.force {
        return Err(DddError::config_error(format!(
            "Config file already exists: {}. Use --force to overwrite.",
            config_path.display()
        )));
    }

    let content = match args.format.as_str() {
        "json" => {
            let config = crate::config::Config::default();
            serde_json::to_string_pretty(&config).map_err(|e| {
                DddError::config_error(format!("Failed to serialize config: {}", e))
            })?
        }
        _ => generate_default_config(),
    };

    std::fs::write(&config_path, content)
        .map_err(|e| DddError::io_error(config_path.clone(), e))?;

    println!(
        "{} Created configuration file: {}",
        "✓".green().bold(),
        config_path.display()
    );

    println!();
    println!("Next steps:");
    println!("  1. Edit {} to configure entry points", config_filename);
    println!("  2. Run {} to analyze your codebase", "ddd analyze".cyan());

    Ok(())
}
