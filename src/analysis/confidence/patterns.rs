//! Dynamic pattern detection for confidence scoring.

use crate::core::CallGraph;

/// Detect global dynamic patterns that affect overall confidence.
pub fn detect_dynamic_patterns(call_graph: &CallGraph) -> bool {
    // Check if any file has dynamic eval
    call_graph.files.values().any(|f| f.has_dynamic_eval)
}
