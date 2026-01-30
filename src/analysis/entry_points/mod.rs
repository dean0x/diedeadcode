//! Entry point discovery for dead code analysis.

mod framework;
mod package_json;

use crate::config::Config;
use crate::core::{CallGraph, Result};
use std::path::Path;

/// Discover and mark entry points in the call graph.
pub fn discover_entry_points(
    call_graph: &mut CallGraph,
    root_path: &Path,
    config: &Config,
) -> Result<()> {
    // 1. Explicit entry points from config
    for entry_file in &config.entry.files {
        mark_file_exports_as_entry_points(call_graph, entry_file);
    }

    // 2. Entry points from patterns
    for pattern in &config.entry.patterns {
        mark_pattern_as_entry_points(call_graph, root_path, pattern);
    }

    // 3. Auto-detect from package.json
    if config.entry.auto_detect {
        let package_json_path = root_path.join("package.json");
        if package_json_path.exists() {
            package_json::mark_package_json_entry_points(call_graph, &package_json_path)?;
        }
    }

    // 4. Framework-specific entry points
    if config.plugins.auto_detect || !config.plugins.enabled.is_empty() {
        framework::discover_framework_entry_points(call_graph, root_path, config)?;
    }

    // 5. Mark explicitly exported symbols from config
    for export_name in &config.entry.exports {
        mark_export_as_entry_point(call_graph, export_name);
    }

    // 6. Files with side effects are implicit entry points
    mark_side_effect_files_as_entry_points(call_graph);

    Ok(())
}

/// Mark all exports from a file as entry points.
fn mark_file_exports_as_entry_points(call_graph: &mut CallGraph, file_path: &Path) {
    // Find the file in the graph
    let file_id = call_graph
        .files
        .values()
        .find(|f| f.path == file_path)
        .map(|f| f.id);

    if let Some(file_id) = file_id {
        // Mark all exported symbols from this file as entry points
        let symbols_to_mark: Vec<_> = call_graph
            .symbols
            .values()
            .filter(|s| s.file_id == file_id && s.exported)
            .map(|s| s.id)
            .collect();

        for id in symbols_to_mark {
            call_graph.mark_entry_point(id);
        }
    }
}

/// Mark exports matching a glob pattern as entry points.
fn mark_pattern_as_entry_points(call_graph: &mut CallGraph, root_path: &Path, pattern: &str) {
    let full_pattern = root_path.join(pattern).display().to_string();

    if let Ok(glob_pattern) = glob::Pattern::new(&full_pattern) {
        let matching_files: Vec<_> = call_graph
            .files
            .values()
            .filter(|f| glob_pattern.matches_path(&f.path))
            .map(|f| f.id)
            .collect();

        for file_id in matching_files {
            let symbols_to_mark: Vec<_> = call_graph
                .symbols
                .values()
                .filter(|s| s.file_id == file_id && s.exported)
                .map(|s| s.id)
                .collect();

            for id in symbols_to_mark {
                call_graph.mark_entry_point(id);
            }
        }
    }
}

/// Mark a named export as an entry point across all files.
fn mark_export_as_entry_point(call_graph: &mut CallGraph, export_name: &str) {
    let symbols_to_mark: Vec<_> = call_graph
        .symbols
        .values()
        .filter(|s| s.exported && s.name == export_name)
        .map(|s| s.id)
        .collect();

    for id in symbols_to_mark {
        call_graph.mark_entry_point(id);
    }
}

/// Mark files with side effects as having entry points.
fn mark_side_effect_files_as_entry_points(call_graph: &mut CallGraph) {
    let side_effect_file_ids: Vec<_> = call_graph
        .files
        .values()
        .filter(|f| f.has_side_effects)
        .map(|f| f.id)
        .collect();

    for file_id in side_effect_file_ids {
        // Mark all top-level symbols in files with side effects
        let symbols_to_mark: Vec<_> = call_graph
            .symbols
            .values()
            .filter(|s| s.file_id == file_id)
            .map(|s| s.id)
            .collect();

        for id in symbols_to_mark {
            call_graph.mark_entry_point(id);
        }
    }
}
