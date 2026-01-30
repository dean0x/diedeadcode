//! Analysis module for dead code detection.

pub mod call_graph;
pub mod confidence;
pub mod deadness;
pub mod entry_points;
pub mod project;

use crate::config::Config;
use crate::core::{AnalysisResult, Result};
use indicatif::ProgressBar;
use std::path::PathBuf;
use std::time::Instant;

/// Main analyzer that coordinates all analysis phases.
pub struct Analyzer {
    config: Config,
    root_path: PathBuf,
}

impl Analyzer {
    /// Create a new analyzer.
    pub fn new(config: Config, root_path: PathBuf) -> Result<Self> {
        Ok(Self { config, root_path })
    }

    /// Run the full analysis pipeline.
    pub fn analyze(&mut self, progress: Option<&ProgressBar>) -> Result<AnalysisResult> {
        let start = Instant::now();

        // Phase 1: Discover files
        if let Some(pb) = progress {
            pb.set_message("Discovering files...");
        }
        let files = project::discover_files(&self.root_path, &self.config)?;

        if files.is_empty() {
            return Err(crate::core::DddError::no_files_found(self.root_path.clone()));
        }

        // Phase 2: Build call graph
        if let Some(pb) = progress {
            pb.set_message(format!("Parsing {} files...", files.len()));
        }
        let mut call_graph = call_graph::build_call_graph(&files, &self.config, progress)?;

        // Phase 3: Discover entry points
        if let Some(pb) = progress {
            pb.set_message("Discovering entry points...");
        }
        entry_points::discover_entry_points(&mut call_graph, &self.root_path, &self.config)?;

        // Phase 4: Propagate deadness
        if let Some(pb) = progress {
            pb.set_message("Analyzing reachability...");
        }
        let dead_symbols = deadness::find_dead_symbols(&call_graph, &self.config);

        // Phase 5: Score confidence
        if let Some(pb) = progress {
            pb.set_message("Scoring confidence...");
        }
        let scored_dead = confidence::score_dead_symbols(dead_symbols, &call_graph, &self.config);

        let duration = start.elapsed();

        Ok(AnalysisResult {
            dead_symbols: scored_dead,
            total_symbols: call_graph.symbol_count(),
            total_files: call_graph.files.len(),
            warnings: Vec::new(), // TODO: Collect warnings during analysis
            duration_ms: duration.as_millis() as u64,
        })
    }
}
