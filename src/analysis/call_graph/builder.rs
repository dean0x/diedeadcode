//! Call graph builder coordinating parsing and semantic analysis.

use super::references::{ImportInfo, ReferenceExtractor};
use super::symbols::SymbolExtractor;
use crate::analysis::project::get_source_type;
use crate::config::Config;
use crate::core::{CallGraph, DddError, FileId, FileInfo, Result};
use dashmap::DashMap;
use indicatif::ProgressBar;
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

/// Build a call graph from a list of files.
pub fn build_call_graph(
    files: &[PathBuf],
    config: &Config,
    progress: Option<&ProgressBar>,
) -> Result<CallGraph> {
    let file_id_counter = AtomicU32::new(0);
    let symbol_id_counter = AtomicU32::new(0);

    // Map from file path to file ID
    let file_id_map: DashMap<PathBuf, FileId> = DashMap::new();

    // Parse all files in parallel
    let file_analyses: Vec<_> = files
        .par_iter()
        .filter_map(|path| {
            let file_id = FileId::new(file_id_counter.fetch_add(1, Ordering::SeqCst));
            file_id_map.insert(path.clone(), file_id);

            match analyze_file(path, file_id, &symbol_id_counter, config) {
                Ok(analysis) => Some(analysis),
                Err(e) => {
                    // Log parse errors but continue
                    eprintln!("Warning: Failed to parse {}: {}", path.display(), e);
                    None
                }
            }
        })
        .collect();

    if let Some(pb) = progress {
        pb.set_message(format!("Building call graph from {} files...", file_analyses.len()));
    }

    // Build the call graph from analyzed files
    let mut graph = CallGraph::new();

    // First pass: add all symbols
    for analysis in &file_analyses {
        graph.add_file(analysis.file_info.clone());

        for symbol in &analysis.symbols {
            graph.add_symbol(symbol.clone());
        }
    }

    // Second pass: resolve imports and add references
    let path_to_file_id: HashMap<PathBuf, FileId> = file_id_map.into_iter().collect();

    for analysis in &file_analyses {
        // Add intra-file references
        for reference in &analysis.references {
            graph.add_reference(reference.clone());
        }

        // Resolve imports to symbols in other files
        for import in &analysis.imports {
            if let Some(target_file_id) = resolve_import_path(&import.resolved_path, &path_to_file_id) {
                if let Some(target_symbol_id) = graph.find_export(target_file_id, &import.imported_name) {
                    let reference = crate::core::SymbolReference {
                        from_id: import.local_symbol_id,
                        to_id: target_symbol_id,
                        kind: crate::core::ReferenceKind::Import,
                        is_dynamic: import.is_dynamic,
                        location: import.location.clone(),
                    };
                    graph.add_reference(reference);
                }
            }
        }
    }

    Ok(graph)
}

/// Analysis result for a single file.
struct FileAnalysis {
    file_info: FileInfo,
    symbols: Vec<crate::core::TrackedSymbol>,
    references: Vec<crate::core::SymbolReference>,
    imports: Vec<ImportInfo>,
}

/// Analyze a single file.
fn analyze_file(
    path: &Path,
    file_id: FileId,
    symbol_id_counter: &AtomicU32,
    _config: &Config,
) -> Result<FileAnalysis> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| DddError::io_error(path.to_path_buf(), e))?;

    let source_type = get_source_type(path);
    let allocator = Allocator::default();

    // Parse the file
    let parser_ret = Parser::new(&allocator, &source, source_type).parse();

    if !parser_ret.errors.is_empty() {
        let first_error = &parser_ret.errors[0];
        return Err(DddError::parse_error(
            path.to_path_buf(),
            first_error.to_string(),
            0, // TODO: Get actual line/column from error
            0,
        ));
    }

    // Build semantic information
    let semantic_ret = SemanticBuilder::new()
        .build(&parser_ret.program);

    let semantic = semantic_ret.semantic;

    // Extract symbols
    let symbol_extractor = SymbolExtractor::new(
        path.to_path_buf(),
        file_id,
        symbol_id_counter,
        &source,
    );
    let (symbols, symbol_map, has_side_effects) = symbol_extractor.extract(&parser_ret.program, &semantic);

    // Extract references
    let reference_extractor = ReferenceExtractor::new(
        path.to_path_buf(),
        file_id,
        &symbol_map,
        &source,
    );
    let (references, imports, has_dynamic_eval) = reference_extractor.extract(&parser_ret.program, &semantic, path);

    // Build file info
    let file_info = FileInfo {
        id: file_id,
        path: path.to_path_buf(),
        has_side_effects,
        has_dynamic_eval,
        symbols: symbols.iter().map(|s| s.id).collect(),
    };

    Ok(FileAnalysis {
        file_info,
        symbols,
        references,
        imports,
    })
}

/// Resolve an import path to a file ID.
fn resolve_import_path(
    resolved_path: &Path,
    path_to_file_id: &HashMap<PathBuf, FileId>,
) -> Option<FileId> {
    // Try exact match first
    if let Some(&id) = path_to_file_id.get(resolved_path) {
        return Some(id);
    }

    // Try with different extensions
    let extensions = ["ts", "tsx", "js", "jsx", "mts", "cts"];
    let stem = resolved_path.file_stem()?.to_str()?;
    let parent = resolved_path.parent()?;

    for ext in extensions {
        let candidate = parent.join(format!("{}.{}", stem, ext));
        if let Some(&id) = path_to_file_id.get(&candidate) {
            return Some(id);
        }
    }

    // Try index file
    for ext in extensions {
        let candidate = resolved_path.join(format!("index.{}", ext));
        if let Some(&id) = path_to_file_id.get(&candidate) {
            return Some(id);
        }
    }

    None
}
