//! ddd - Conservative TypeScript dead code detection CLI.

use diedeadcode::cli::{commands, Cli, Commands};
use diedeadcode::config::load_config;
use miette::Result;
use std::process::ExitCode;

fn main() -> ExitCode {
    // Setup miette for pretty error output
    miette::set_panic_hook();

    match run() {
        Ok(code) => ExitCode::from(code as u8),
        Err(e) => {
            eprintln!("{:?}", e);
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<i32> {
    let cli = Cli::parse_args();

    // Resolve the path
    let path = if cli.path.is_absolute() {
        cli.path.clone()
    } else {
        std::env::current_dir()
            .map_err(|e| miette::miette!("Failed to get current directory: {}", e))?
            .join(&cli.path)
    };

    // Load config
    let (config, config_path) = if let Some(ref config_file) = cli.config {
        let cfg = diedeadcode::config::loader::load_config(config_file.parent().unwrap_or(&path))?;
        (cfg.0, Some(config_file.clone()))
    } else {
        load_config(&path)?
    };

    if cli.verbose {
        if let Some(ref cfg_path) = config_path {
            eprintln!("Using config: {}", cfg_path.display());
        }
    }

    // Execute the command
    let command = cli.effective_command();

    match command {
        Commands::Init(args) => {
            commands::run_init(&args, &path)?;
            Ok(0)
        }
        Commands::Analyze(args) => {
            Ok(commands::run_analyze(&args, &path, &config, cli.verbose)?)
        }
        Commands::Watch(args) => {
            Ok(commands::run_watch(&args, &path, &config)?)
        }
    }
}
