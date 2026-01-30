//! Deadness propagation using BFS from entry points.

use super::transitive::find_transitive_dead;
use crate::config::Config;
use crate::core::{CallGraph, DeadSymbol, DeadnessReason, SymbolId, TrackedSymbol};
use std::collections::{HashSet, VecDeque};

/// Find all dead symbols in the call graph.
pub fn find_dead_symbols(call_graph: &CallGraph, config: &Config) -> Vec<DeadSymbol> {
    // Phase 1: Mark all reachable symbols using BFS from entry points
    let reachable = mark_reachable_symbols(call_graph);

    // Phase 2: Collect unreachable symbols
    let unreachable: Vec<_> = call_graph
        .symbols
        .values()
        .filter(|s| !reachable.contains(&s.id))
        .filter(|s| !config.should_ignore_symbol(&s.name))
        .cloned()
        .collect();

    // Phase 3: Analyze transitive deadness
    let (directly_dead, transitively_dead) = find_transitive_dead(&unreachable, call_graph);

    // Phase 4: Create DeadSymbol instances
    let mut dead_symbols = Vec::new();

    // Add directly dead symbols
    for symbol in directly_dead {
        dead_symbols.push(create_dead_symbol(symbol, call_graph));
    }

    // Add transitively dead symbols
    for (symbol, chain, killed_by) in transitively_dead {
        dead_symbols.push(create_transitive_dead_symbol(symbol, chain, killed_by));
    }

    // Sort by file and line for consistent output
    dead_symbols.sort_by(|a, b| {
        let file_cmp = a.symbol.location.file_path.cmp(&b.symbol.location.file_path);
        if file_cmp != std::cmp::Ordering::Equal {
            return file_cmp;
        }
        a.symbol.location.line.cmp(&b.symbol.location.line)
    });

    dead_symbols
}

/// Mark all symbols reachable from entry points using BFS.
fn mark_reachable_symbols(call_graph: &CallGraph) -> HashSet<SymbolId> {
    let mut reachable = HashSet::new();
    let mut queue: VecDeque<SymbolId> = VecDeque::new();

    // Start with entry points
    for &entry_id in &call_graph.entry_points {
        if !reachable.contains(&entry_id) {
            queue.push_back(entry_id);
            reachable.insert(entry_id);
        }
    }

    // BFS traversal
    while let Some(current_id) = queue.pop_front() {
        // Get all symbols referenced by the current symbol
        for &ref_id in call_graph.get_outgoing_refs(current_id) {
            if !reachable.contains(&ref_id) {
                reachable.insert(ref_id);
                queue.push_back(ref_id);
            }
        }
    }

    reachable
}

/// Create a DeadSymbol for a directly unreachable symbol.
fn create_dead_symbol(symbol: TrackedSymbol, call_graph: &CallGraph) -> DeadSymbol {
    let reason = if symbol.exported {
        DeadnessReason::UnusedExport
    } else if is_type_symbol(&symbol) {
        DeadnessReason::UnusedType
    } else {
        let explanation = generate_unreachable_explanation(&symbol, call_graph);
        DeadnessReason::Unreachable { explanation }
    };

    // Start with base confidence of 100
    let base_confidence = 100u8;

    // Confidence will be adjusted by the confidence scorer
    DeadSymbol::new(symbol, base_confidence, reason)
}

/// Create a DeadSymbol for a transitively dead symbol.
fn create_transitive_dead_symbol(
    symbol: TrackedSymbol,
    chain: Vec<SymbolId>,
    killed_by: SymbolId,
) -> DeadSymbol {
    // Transitive dead code has slightly lower base confidence
    let base_confidence = 95u8;
    DeadSymbol::transitive(symbol, base_confidence, chain, killed_by)
}

/// Check if a symbol is a type-only symbol.
fn is_type_symbol(symbol: &TrackedSymbol) -> bool {
    matches!(
        symbol.kind,
        crate::core::SymbolKind::Type
            | crate::core::SymbolKind::Interface
    )
}

/// Generate explanation for why a symbol is unreachable.
fn generate_unreachable_explanation(symbol: &TrackedSymbol, call_graph: &CallGraph) -> String {
    let incoming = call_graph.get_incoming_refs(symbol.id);

    if incoming.is_empty() {
        if symbol.exported {
            "exported but never imported".to_string()
        } else {
            "never referenced".to_string()
        }
    } else {
        // Has references, but they're all from dead code
        format!("referenced only by {} dead symbol(s)", incoming.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{FileId, Location, SymbolKind};
    use std::path::PathBuf;

    fn make_symbol(id: u32, name: &str, kind: SymbolKind) -> TrackedSymbol {
        TrackedSymbol::new(
            SymbolId::new(id),
            name.to_string(),
            kind,
            Location::new(PathBuf::from("test.ts"), 0, 10, 1, 1),
            FileId::new(0),
        )
    }

    #[test]
    fn test_entry_point_is_not_dead() {
        let mut graph = CallGraph::new();

        let mut entry = make_symbol(0, "main", SymbolKind::Function);
        entry.is_entry_point = true;
        graph.add_symbol(entry);
        graph.mark_entry_point(SymbolId::new(0));

        let config = Config::default();
        let dead = find_dead_symbols(&graph, &config);

        assert!(dead.is_empty());
    }

    #[test]
    fn test_unreferenced_is_dead() {
        let mut graph = CallGraph::new();

        let mut entry = make_symbol(0, "main", SymbolKind::Function);
        entry.is_entry_point = true;
        graph.add_symbol(entry);
        graph.mark_entry_point(SymbolId::new(0));

        let orphan = make_symbol(1, "orphan", SymbolKind::Function);
        graph.add_symbol(orphan);

        let config = Config::default();
        let dead = find_dead_symbols(&graph, &config);

        assert_eq!(dead.len(), 1);
        assert_eq!(dead[0].symbol.name, "orphan");
    }

    #[test]
    fn test_referenced_from_entry_is_alive() {
        let mut graph = CallGraph::new();

        let mut entry = make_symbol(0, "main", SymbolKind::Function);
        entry.is_entry_point = true;
        graph.add_symbol(entry);
        graph.mark_entry_point(SymbolId::new(0));

        let helper = make_symbol(1, "helper", SymbolKind::Function);
        graph.add_symbol(helper);

        // main -> helper
        graph.add_reference(crate::core::SymbolReference::new(
            SymbolId::new(0),
            SymbolId::new(1),
            crate::core::ReferenceKind::Call,
            Location::new(PathBuf::from("test.ts"), 5, 15, 2, 1),
        ));

        let config = Config::default();
        let dead = find_dead_symbols(&graph, &config);

        assert!(dead.is_empty());
    }
}
