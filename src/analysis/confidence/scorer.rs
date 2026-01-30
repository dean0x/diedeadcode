//! Confidence scoring for dead code analysis.
//!
//! Applies conservative rules to reduce false positives.

use super::patterns::detect_dynamic_patterns;
use crate::config::Config;
use crate::core::{CallGraph, Confidence, DeadSymbol, DynamicPatternKind};

/// Score dead symbols for confidence.
pub fn score_dead_symbols(
    mut dead_symbols: Vec<DeadSymbol>,
    call_graph: &CallGraph,
    config: &Config,
) -> Vec<DeadSymbol> {
    // Detect file-level dynamic patterns
    let file_has_eval: std::collections::HashSet<_> = call_graph
        .files
        .values()
        .filter(|f| f.has_dynamic_eval)
        .map(|f| f.id)
        .collect();

    // Check for global dynamic patterns
    let has_global_dynamic = detect_dynamic_patterns(call_graph);

    for dead in &mut dead_symbols {
        let mut score = dead.confidence_score as i32;

        // Rule 1: Decorators reduce confidence (frameworks might use them)
        if dead.symbol.has_decorators {
            score -= 20;
        }

        // Rule 2: Exported symbols are more likely to be used externally
        if dead.symbol.exported {
            score -= 10;
        }

        // Rule 3: File has eval() - major confidence reduction
        if file_has_eval.contains(&dead.symbol.file_id) {
            score -= 30;
        }

        // Rule 4: Global dynamic patterns affect all symbols
        if has_global_dynamic {
            score -= 15;
        }

        // Rule 5: Transitive dead code is slightly less certain
        if matches!(dead.reason, crate::core::DeadnessReason::Transitive { .. }) {
            score -= 5;
        }

        // Rule 6: Type-only symbols (interfaces, types) are less risky to remove
        if is_type_only(&dead.symbol.kind) {
            score += 5;
        }

        // Rule 7: Private-by-convention (starts with _) is more likely dead
        if dead.symbol.name.starts_with('_') && !dead.symbol.name.starts_with("__") {
            score += 5;
        }

        // Rule 8: Check for dynamic pattern matches in the call graph
        for pattern in &call_graph.dynamic_patterns {
            if pattern.affected_symbols.contains(&dead.symbol.id) {
                match pattern.kind {
                    DynamicPatternKind::Eval | DynamicPatternKind::FunctionConstructor => {
                        score -= 40; // Very uncertain
                    }
                    DynamicPatternKind::Reflect => {
                        score -= 30;
                    }
                    DynamicPatternKind::BracketAccess | DynamicPatternKind::StringPropertyAccess => {
                        score -= 20;
                    }
                    DynamicPatternKind::ObjectIteration => {
                        score -= 15;
                    }
                    DynamicPatternKind::DynamicImport | DynamicPatternKind::DynamicRequire => {
                        score -= 25;
                    }
                }
            }
        }

        // Rule 9: Default exports without names are harder to track
        if dead.symbol.name == "default" {
            score -= 10;
        }

        // Rule 10: Class methods vs standalone functions
        // Methods are more likely to be called dynamically
        if dead.symbol.kind == crate::core::SymbolKind::Method {
            score -= 5;
        }

        // Clamp score to valid range
        dead.confidence_score = score.clamp(0, 100) as u8;
        dead.confidence = Confidence::from_score(dead.confidence_score);
    }

    // Apply config-based filtering
    if !config.analysis.include_types {
        dead_symbols.retain(|d| !is_type_only(&d.symbol.kind));
    }

    dead_symbols
}

/// Check if a symbol kind is type-only (no runtime impact).
fn is_type_only(kind: &crate::core::SymbolKind) -> bool {
    matches!(
        kind,
        crate::core::SymbolKind::Type | crate::core::SymbolKind::Interface
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{DeadnessReason, FileId, Location, SymbolId, SymbolKind, TrackedSymbol};
    use std::path::PathBuf;

    fn make_dead_symbol(name: &str, exported: bool, has_decorators: bool) -> DeadSymbol {
        let mut symbol = TrackedSymbol::new(
            SymbolId::new(0),
            name.to_string(),
            SymbolKind::Function,
            Location::new(PathBuf::from("test.ts"), 0, 10, 1, 1),
            FileId::new(0),
        );
        symbol.exported = exported;
        symbol.has_decorators = has_decorators;

        DeadSymbol::new(
            symbol,
            100,
            DeadnessReason::Unreachable {
                explanation: "never referenced".to_string(),
            },
        )
    }

    #[test]
    fn test_base_confidence() {
        let dead = make_dead_symbol("foo", false, false);
        assert_eq!(dead.confidence_score, 100);
        assert_eq!(dead.confidence, Confidence::High);
    }

    #[test]
    fn test_exported_reduces_confidence() {
        let graph = CallGraph::new();
        let config = Config::default();
        let dead_symbols = vec![make_dead_symbol("foo", true, false)];

        let scored = score_dead_symbols(dead_symbols, &graph, &config);

        assert_eq!(scored[0].confidence_score, 90);
    }

    #[test]
    fn test_decorators_reduce_confidence() {
        let graph = CallGraph::new();
        let config = Config::default();
        let dead_symbols = vec![make_dead_symbol("foo", false, true)];

        let scored = score_dead_symbols(dead_symbols, &graph, &config);

        assert_eq!(scored[0].confidence_score, 80);
    }

    #[test]
    fn test_combined_penalties() {
        let graph = CallGraph::new();
        let config = Config::default();
        // Exported + decorators
        let dead_symbols = vec![make_dead_symbol("foo", true, true)];

        let scored = score_dead_symbols(dead_symbols, &graph, &config);

        // 100 - 10 (exported) - 20 (decorators) = 70
        assert_eq!(scored[0].confidence_score, 70);
        assert_eq!(scored[0].confidence, Confidence::Medium);
    }

    #[test]
    fn test_private_convention_bonus() {
        let graph = CallGraph::new();
        let config = Config::default();
        let dead_symbols = vec![make_dead_symbol("_privateHelper", false, false)];

        let scored = score_dead_symbols(dead_symbols, &graph, &config);

        // 100 + 5 (private convention) = 100 (clamped)
        assert_eq!(scored[0].confidence_score, 100);
    }
}
