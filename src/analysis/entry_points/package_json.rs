//! Entry point extraction from package.json.

use crate::config::extract_entry_points_from_package_json;
use crate::core::{CallGraph, Result};
use std::path::Path;

/// Mark entry points from package.json in the call graph.
pub fn mark_package_json_entry_points(call_graph: &mut CallGraph, package_json_path: &Path) -> Result<()> {
    let entry_config = extract_entry_points_from_package_json(package_json_path)?;

    for entry_file in &entry_config.files {
        // Find matching file in the call graph
        let file_id = call_graph
            .files
            .values()
            .find(|f| paths_match(&f.path, entry_file))
            .map(|f| f.id);

        if let Some(file_id) = file_id {
            // Mark all exports as entry points
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

    Ok(())
}

/// Check if two paths refer to the same file, handling extension normalization.
fn paths_match(path1: &Path, path2: &Path) -> bool {
    // Direct match
    if path1 == path2 {
        return true;
    }

    // Try canonicalizing
    if let (Ok(p1), Ok(p2)) = (path1.canonicalize(), path2.canonicalize()) {
        if p1 == p2 {
            return true;
        }
    }

    // Try matching without extension (for ts/js equivalence)
    let stem1 = path1.file_stem();
    let stem2 = path2.file_stem();
    let parent1 = path1.parent();
    let parent2 = path2.parent();

    if stem1 == stem2 && parent1 == parent2 {
        return true;
    }

    false
}
