//! Framework-specific entry point detection.

use crate::config::Config;
use crate::core::{CallGraph, Result};
use crate::plugins::{detect_frameworks, FrameworkDetector};
use std::path::Path;

/// Discover entry points based on detected frameworks.
pub fn discover_framework_entry_points(
    call_graph: &mut CallGraph,
    root_path: &Path,
    config: &Config,
) -> Result<()> {
    let detectors = detect_frameworks(root_path, config)?;

    for detector in detectors {
        mark_framework_entry_points(call_graph, root_path, &*detector)?;
    }

    Ok(())
}

/// Mark entry points for a specific framework.
fn mark_framework_entry_points(
    call_graph: &mut CallGraph,
    root_path: &Path,
    detector: &dyn FrameworkDetector,
) -> Result<()> {
    let entry_patterns = detector.get_entry_patterns();

    for pattern in entry_patterns {
        let full_pattern = root_path.join(&pattern).display().to_string();

        if let Ok(glob_pattern) = glob::Pattern::new(&full_pattern) {
            // Find matching files
            let matching_file_ids: Vec<_> = call_graph
                .files
                .values()
                .filter(|f| glob_pattern.matches_path(&f.path))
                .map(|f| f.id)
                .collect();

            for file_id in matching_file_ids {
                // Get the special exports for this framework
                let special_exports = detector.get_special_exports();

                // Mark matching symbols as entry points
                let symbols_to_mark: Vec<_> = call_graph
                    .symbols
                    .values()
                    .filter(|s| {
                        s.file_id == file_id
                            && (s.exported || special_exports.contains(&s.name.as_str()))
                    })
                    .map(|s| s.id)
                    .collect();

                for id in symbols_to_mark {
                    call_graph.mark_entry_point(id);
                }
            }
        }
    }

    Ok(())
}
