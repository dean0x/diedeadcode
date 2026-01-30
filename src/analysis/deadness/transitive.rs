//! Transitive deadness analysis.
//!
//! A symbol is transitively dead if all of its callers are dead.

use crate::core::{CallGraph, SymbolId, TrackedSymbol};
use std::collections::{HashMap, HashSet};

/// Categorize unreachable symbols into directly dead and transitively dead.
///
/// Returns:
/// - Directly dead symbols (have no live callers)
/// - Transitively dead symbols with their chain and the symbol that "killed" them
pub fn find_transitive_dead(
    unreachable: &[TrackedSymbol],
    call_graph: &CallGraph,
) -> (
    Vec<TrackedSymbol>,
    Vec<(TrackedSymbol, Vec<SymbolId>, SymbolId)>,
) {
    let unreachable_ids: HashSet<SymbolId> = unreachable.iter().map(|s| s.id).collect();

    // Build reverse dependency map: symbol -> symbols that reference it
    let mut referenced_by: HashMap<SymbolId, Vec<SymbolId>> = HashMap::new();
    for symbol in unreachable {
        referenced_by.insert(symbol.id, Vec::new());
    }

    for symbol in unreachable {
        for &ref_id in call_graph.get_incoming_refs(symbol.id) {
            if unreachable_ids.contains(&ref_id) {
                referenced_by.entry(symbol.id).or_default().push(ref_id);
            }
        }
    }

    // Symbols with no references from within the unreachable set are "root dead"
    let mut root_dead: Vec<SymbolId> = Vec::new();
    let mut has_unreachable_refs: Vec<SymbolId> = Vec::new();

    for symbol in unreachable {
        let refs = referenced_by.get(&symbol.id).map_or(0, |v| v.len());
        if refs == 0 {
            root_dead.push(symbol.id);
        } else {
            has_unreachable_refs.push(symbol.id);
        }
    }

    // For symbols with unreachable refs, determine if they're transitively dead
    // and build their kill chain
    let symbol_map: HashMap<SymbolId, &TrackedSymbol> =
        unreachable.iter().map(|s| (s.id, s)).collect();

    let mut directly_dead = Vec::new();
    let mut transitively_dead = Vec::new();

    // Root dead symbols are directly dead
    for id in root_dead {
        if let Some(symbol) = symbol_map.get(&id) {
            directly_dead.push((*symbol).clone());
        }
    }

    // Symbols only referenced by other dead symbols are transitively dead
    for id in has_unreachable_refs {
        if let Some(symbol) = symbol_map.get(&id) {
            // Find the chain - which dead symbols reference this one?
            let chain: Vec<SymbolId> = referenced_by
                .get(&id)
                .cloned()
                .unwrap_or_default();

            if let Some(&killed_by) = chain.first() {
                transitively_dead.push(((*symbol).clone(), chain, killed_by));
            } else {
                // No chain means it's directly dead
                directly_dead.push((*symbol).clone());
            }
        }
    }

    (directly_dead, transitively_dead)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{FileId, Location, ReferenceKind, SymbolKind, SymbolReference};
    use std::path::PathBuf;

    fn make_symbol(id: u32, name: &str) -> TrackedSymbol {
        TrackedSymbol::new(
            SymbolId::new(id),
            name.to_string(),
            SymbolKind::Function,
            Location::new(PathBuf::from("test.ts"), 0, 10, 1, 1),
            FileId::new(0),
        )
    }

    #[test]
    fn test_isolated_symbols_are_directly_dead() {
        let graph = CallGraph::new();
        let unreachable = vec![make_symbol(0, "foo"), make_symbol(1, "bar")];

        let (directly_dead, transitively_dead) = find_transitive_dead(&unreachable, &graph);

        assert_eq!(directly_dead.len(), 2);
        assert!(transitively_dead.is_empty());
    }

    #[test]
    fn test_transitive_dead_detection() {
        let mut graph = CallGraph::new();

        // foo -> bar (foo calls bar, but foo is never called)
        graph.add_reference(SymbolReference::new(
            SymbolId::new(0),
            SymbolId::new(1),
            ReferenceKind::Call,
            Location::new(PathBuf::from("test.ts"), 0, 10, 1, 1),
        ));

        let unreachable = vec![make_symbol(0, "foo"), make_symbol(1, "bar")];

        let (directly_dead, transitively_dead) = find_transitive_dead(&unreachable, &graph);

        // foo is directly dead (nothing calls it within the dead set)
        assert_eq!(directly_dead.len(), 1);
        assert_eq!(directly_dead[0].name, "foo");

        // bar is transitively dead (only called by dead foo)
        assert_eq!(transitively_dead.len(), 1);
        assert_eq!(transitively_dead[0].0.name, "bar");
    }

}
