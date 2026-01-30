//! The `ddd analyze` command implementation.

use crate::analysis::Analyzer;
use crate::cli::output::OutputWriter;
use crate::cli::AnalyzeArgs;
use crate::config::Config;
use crate::core::Result;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::time::Instant;

/// Run the analyze command.
pub fn run_analyze(args: &AnalyzeArgs, path: &Path, config: &Config, verbose: bool) -> Result<i32> {
    let start = Instant::now();

    // Setup progress bar if requested
    let progress = if args.progress {
        let pb = ProgressBar::new_spinner();
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.cyan} {msg}")
                .unwrap(),
        );
        pb.set_message("Discovering files...");
        Some(pb)
    } else {
        None
    };

    // Create analyzer
    let mut analyzer = Analyzer::new(config.clone(), path.to_path_buf())?;

    // Update progress
    if let Some(ref pb) = progress {
        pb.set_message("Parsing files...");
    }

    // Run analysis
    let result = analyzer.analyze(progress.as_ref())?;

    // Finish progress
    if let Some(pb) = progress {
        pb.finish_and_clear();
    }

    let duration = start.elapsed();

    // Filter by confidence level
    let min_confidence = args.confidence.to_confidence();
    let filtered_dead = result.filter_by_confidence(min_confidence);

    // Write output
    let mut writer = OutputWriter::new(args.format.into(), verbose);
    writer.write_result(&result, &filtered_dead, args.show_chains)?;

    // Print summary
    if !args.check {
        let (high, medium, low) = result.count_by_confidence();
        eprintln!();
        eprintln!(
            "Analyzed {} symbols in {} files ({:.2}s)",
            result.total_symbols,
            result.total_files,
            duration.as_secs_f64()
        );
        eprintln!(
            "Dead code: {} high, {} medium, {} low confidence",
            high, medium, low
        );
    }

    // Print warnings
    for warning in &result.warnings {
        eprintln!("Warning: {}", warning.message);
    }

    // Return exit code
    if args.check && !filtered_dead.is_empty() {
        Ok(1)
    } else {
        Ok(0)
    }
}
