//! Core type definitions for dead code analysis.
//!
//! These types form the foundation of the call graph and deadness analysis.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

/// Unique identifier for a symbol within the analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);

impl SymbolId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// The kind of symbol being tracked.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    ArrowFunction,
    Class,
    Method,
    Variable,
    Constant,
    Type,
    Interface,
    Enum,
    EnumMember,
    Namespace,
    Module,
}

impl SymbolKind {
    /// Returns true if this symbol kind can have side effects when defined.
    pub fn can_have_side_effects(&self) -> bool {
        matches!(
            self,
            SymbolKind::Class | SymbolKind::Variable | SymbolKind::Constant
        )
    }
}

/// Source location of a symbol or reference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub file_path: PathBuf,
    pub start_offset: u32,
    pub end_offset: u32,
    pub line: u32,
    pub column: u32,
}

impl Location {
    pub fn new(file_path: PathBuf, start_offset: u32, end_offset: u32, line: u32, column: u32) -> Self {
        Self {
            file_path,
            start_offset,
            end_offset,
            line,
            column,
        }
    }

    /// Format as "path:line:column" for display.
    pub fn display(&self) -> String {
        format!(
            "{}:{}:{}",
            self.file_path.display(),
            self.line,
            self.column
        )
    }
}

/// A symbol being tracked for dead code analysis.
#[derive(Debug, Clone)]
pub struct TrackedSymbol {
    pub id: SymbolId,
    pub name: String,
    pub kind: SymbolKind,
    pub location: Location,
    /// Whether this symbol is exported from its module.
    pub exported: bool,
    /// Whether this symbol is an entry point (e.g., main, framework-detected).
    pub is_entry_point: bool,
    /// Whether this symbol has decorators applied.
    pub has_decorators: bool,
    /// Whether this symbol's definition has observable side effects.
    pub has_side_effects: bool,
    /// The file ID this symbol belongs to.
    pub file_id: FileId,
}

impl TrackedSymbol {
    /// Creates a new TrackedSymbol with required fields.
    pub fn new(
        id: SymbolId,
        name: String,
        kind: SymbolKind,
        location: Location,
        file_id: FileId,
    ) -> Self {
        Self {
            id,
            name,
            kind,
            location,
            exported: false,
            is_entry_point: false,
            has_decorators: false,
            has_side_effects: false,
            file_id,
        }
    }
}

/// The kind of reference between symbols.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferenceKind {
    /// Direct function call: `foo()`.
    Call,
    /// Class instantiation: `new Foo()`.
    Instantiation,
    /// Property access: `obj.prop`.
    PropertyAccess,
    /// Type reference: `x: Foo`.
    TypeReference,
    /// Import statement: `import { foo }`.
    Import,
    /// Export statement: `export { foo }`.
    Export,
    /// Re-export: `export { foo } from './bar'`.
    ReExport,
    /// JSX element: `<Component />`.
    JsxElement,
    /// Extends clause: `class Foo extends Bar`.
    Extends,
    /// Implements clause: `class Foo implements Bar`.
    Implements,
    /// Decorator: `@decorator`.
    Decorator,
}

/// A reference from one symbol to another.
#[derive(Debug, Clone)]
pub struct SymbolReference {
    /// The symbol making the reference.
    pub from_id: SymbolId,
    /// The symbol being referenced.
    pub to_id: SymbolId,
    /// The kind of reference.
    pub kind: ReferenceKind,
    /// Whether this is a dynamic reference (e.g., bracket notation).
    pub is_dynamic: bool,
    /// Location of the reference.
    pub location: Location,
}

impl SymbolReference {
    pub fn new(
        from_id: SymbolId,
        to_id: SymbolId,
        kind: ReferenceKind,
        location: Location,
    ) -> Self {
        Self {
            from_id,
            to_id,
            kind,
            is_dynamic: false,
            location,
        }
    }
}

/// Unique identifier for a file in the analysis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

impl FileId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// Metadata about an analyzed file.
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub id: FileId,
    pub path: PathBuf,
    /// Whether this file has top-level side effects.
    pub has_side_effects: bool,
    /// Whether eval or similar dynamic code execution was detected.
    pub has_dynamic_eval: bool,
    /// Symbols defined in this file.
    pub symbols: Vec<SymbolId>,
}

/// Dynamic pattern that reduces confidence in analysis.
#[derive(Debug, Clone)]
pub struct DynamicPattern {
    pub kind: DynamicPatternKind,
    pub location: Location,
    pub affected_symbols: Vec<SymbolId>,
}

/// Types of dynamic patterns that affect analysis confidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicPatternKind {
    /// Bracket property access: `obj[key]`.
    BracketAccess,
    /// `eval()` call.
    Eval,
    /// `new Function()` call.
    FunctionConstructor,
    /// `Reflect.get/set/apply` etc.
    Reflect,
    /// `Object.keys/values/entries` iteration.
    ObjectIteration,
    /// String literal property access that might match symbols.
    StringPropertyAccess,
    /// `require()` with non-literal argument.
    DynamicRequire,
    /// `import()` with non-literal argument.
    DynamicImport,
}

/// The complete call graph for a project.
#[derive(Debug)]
pub struct CallGraph {
    /// All tracked symbols by ID.
    pub symbols: HashMap<SymbolId, TrackedSymbol>,
    /// All references between symbols.
    pub references: Vec<SymbolReference>,
    /// Entry point symbols that are always considered live.
    pub entry_points: HashSet<SymbolId>,
    /// Dynamic patterns detected during analysis.
    pub dynamic_patterns: Vec<DynamicPattern>,
    /// Files in the analysis.
    pub files: HashMap<FileId, FileInfo>,
    /// Reverse index: symbol -> symbols that reference it.
    pub incoming_refs: HashMap<SymbolId, Vec<SymbolId>>,
    /// Forward index: symbol -> symbols it references.
    pub outgoing_refs: HashMap<SymbolId, Vec<SymbolId>>,
    /// Next symbol ID to allocate.
    next_symbol_id: u32,
    /// Next file ID to allocate.
    next_file_id: u32,
}

impl Default for CallGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl CallGraph {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            references: Vec::new(),
            entry_points: HashSet::new(),
            dynamic_patterns: Vec::new(),
            files: HashMap::new(),
            incoming_refs: HashMap::new(),
            outgoing_refs: HashMap::new(),
            next_symbol_id: 0,
            next_file_id: 0,
        }
    }

    /// Allocate a new symbol ID.
    pub fn alloc_symbol_id(&mut self) -> SymbolId {
        let id = SymbolId::new(self.next_symbol_id);
        self.next_symbol_id += 1;
        id
    }

    /// Allocate a new file ID.
    pub fn alloc_file_id(&mut self) -> FileId {
        let id = FileId::new(self.next_file_id);
        self.next_file_id += 1;
        id
    }

    /// Add a symbol to the graph.
    pub fn add_symbol(&mut self, symbol: TrackedSymbol) {
        let id = symbol.id;
        if symbol.is_entry_point {
            self.entry_points.insert(id);
        }
        self.symbols.insert(id, symbol);
    }

    /// Add a reference between symbols.
    pub fn add_reference(&mut self, reference: SymbolReference) {
        let from_id = reference.from_id;
        let to_id = reference.to_id;

        self.incoming_refs
            .entry(to_id)
            .or_default()
            .push(from_id);
        self.outgoing_refs
            .entry(from_id)
            .or_default()
            .push(to_id);

        self.references.push(reference);
    }

    /// Add a file to the graph.
    pub fn add_file(&mut self, file: FileInfo) {
        self.files.insert(file.id, file);
    }

    /// Mark a symbol as an entry point.
    pub fn mark_entry_point(&mut self, id: SymbolId) {
        self.entry_points.insert(id);
        if let Some(symbol) = self.symbols.get_mut(&id) {
            symbol.is_entry_point = true;
        }
    }

    /// Find an exported symbol by name in a file.
    pub fn find_export(&self, file_id: FileId, name: &str) -> Option<SymbolId> {
        self.files.get(&file_id).and_then(|file| {
            file.symbols.iter().find_map(|&sym_id| {
                let symbol = self.symbols.get(&sym_id)?;
                if symbol.exported && symbol.name == name {
                    Some(sym_id)
                } else {
                    None
                }
            })
        })
    }

    /// Get all symbols that reference the given symbol.
    pub fn get_incoming_refs(&self, id: SymbolId) -> &[SymbolId] {
        self.incoming_refs.get(&id).map_or(&[], |v| v.as_slice())
    }

    /// Get all symbols that the given symbol references.
    pub fn get_outgoing_refs(&self, id: SymbolId) -> &[SymbolId] {
        self.outgoing_refs.get(&id).map_or(&[], |v| v.as_slice())
    }

    /// Get total symbol count.
    pub fn symbol_count(&self) -> usize {
        self.symbols.len()
    }

    /// Get total reference count.
    pub fn reference_count(&self) -> usize {
        self.references.len()
    }
}

/// Confidence level for dead code detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    /// 0-49: Likely false positive, many dynamic patterns detected.
    Low,
    /// 50-79: Review recommended, some uncertainty.
    Medium,
    /// 80-100: Safe to remove, high certainty.
    High,
}

impl Confidence {
    /// Create confidence from a numeric score (0-100).
    pub fn from_score(score: u8) -> Self {
        match score {
            0..=49 => Confidence::Low,
            50..=79 => Confidence::Medium,
            _ => Confidence::High, // 80+ including clamped values
        }
    }

    /// Get the minimum score for this confidence level.
    pub fn min_score(&self) -> u8 {
        match self {
            Confidence::Low => 0,
            Confidence::Medium => 50,
            Confidence::High => 80,
        }
    }

    /// Get a human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Confidence::Low => "low",
            Confidence::Medium => "medium",
            Confidence::High => "high",
        }
    }
}

/// Reason why a symbol is considered dead.
#[derive(Debug, Clone)]
pub enum DeadnessReason {
    /// No references found from entry points.
    Unreachable {
        explanation: String,
    },
    /// Dead because all callers are dead.
    Transitive {
        /// Chain of dead symbols leading to this one.
        chain: Vec<SymbolId>,
    },
    /// Exported but never imported.
    UnusedExport,
    /// Type/interface that is never referenced.
    UnusedType,
}

impl DeadnessReason {
    pub fn description(&self) -> String {
        match self {
            DeadnessReason::Unreachable { explanation } => explanation.clone(),
            DeadnessReason::Transitive { chain } => {
                format!("transitively dead via {} callers", chain.len())
            }
            DeadnessReason::UnusedExport => "exported but never imported".to_string(),
            DeadnessReason::UnusedType => "type is never referenced".to_string(),
        }
    }
}

/// A symbol that has been determined to be dead.
#[derive(Debug, Clone)]
pub struct DeadSymbol {
    /// The symbol that is dead.
    pub symbol: TrackedSymbol,
    /// Confidence level.
    pub confidence: Confidence,
    /// Numeric confidence score (0-100).
    pub confidence_score: u8,
    /// Why this symbol is dead.
    pub reason: DeadnessReason,
    /// If transitively dead, which dead symbol caused this.
    pub killed_by: Option<SymbolId>,
}

impl DeadSymbol {
    pub fn new(
        symbol: TrackedSymbol,
        confidence_score: u8,
        reason: DeadnessReason,
    ) -> Self {
        Self {
            symbol,
            confidence: Confidence::from_score(confidence_score),
            confidence_score,
            reason,
            killed_by: None,
        }
    }

    /// Create a transitively dead symbol.
    pub fn transitive(
        symbol: TrackedSymbol,
        confidence_score: u8,
        chain: Vec<SymbolId>,
        killed_by: SymbolId,
    ) -> Self {
        Self {
            symbol,
            confidence: Confidence::from_score(confidence_score),
            confidence_score,
            reason: DeadnessReason::Transitive { chain },
            killed_by: Some(killed_by),
        }
    }
}

/// Result of dead code analysis.
#[derive(Debug)]
pub struct AnalysisResult {
    /// Dead symbols found.
    pub dead_symbols: Vec<DeadSymbol>,
    /// Total symbols analyzed.
    pub total_symbols: usize,
    /// Total files analyzed.
    pub total_files: usize,
    /// Global warnings (e.g., eval detected).
    pub warnings: Vec<AnalysisWarning>,
    /// Analysis duration.
    pub duration_ms: u64,
}

impl AnalysisResult {
    /// Get dead symbols filtered by minimum confidence.
    pub fn filter_by_confidence(&self, min_confidence: Confidence) -> Vec<&DeadSymbol> {
        self.dead_symbols
            .iter()
            .filter(|d| d.confidence >= min_confidence)
            .collect()
    }

    /// Count dead symbols by confidence level.
    pub fn count_by_confidence(&self) -> (usize, usize, usize) {
        let mut low = 0;
        let mut medium = 0;
        let mut high = 0;

        for dead in &self.dead_symbols {
            match dead.confidence {
                Confidence::Low => low += 1,
                Confidence::Medium => medium += 1,
                Confidence::High => high += 1,
            }
        }

        (high, medium, low)
    }
}

/// Warning generated during analysis.
#[derive(Debug, Clone)]
pub struct AnalysisWarning {
    pub kind: WarningKind,
    pub message: String,
    pub location: Option<Location>,
}

/// Types of analysis warnings.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WarningKind {
    /// eval() or similar detected.
    DynamicCodeExecution,
    /// Unparseable file.
    ParseError,
    /// Unresolvable import.
    UnresolvedImport,
    /// Circular dependency detected.
    CircularDependency,
    /// Configuration issue.
    ConfigWarning,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confidence_from_score() {
        assert_eq!(Confidence::from_score(0), Confidence::Low);
        assert_eq!(Confidence::from_score(49), Confidence::Low);
        assert_eq!(Confidence::from_score(50), Confidence::Medium);
        assert_eq!(Confidence::from_score(79), Confidence::Medium);
        assert_eq!(Confidence::from_score(80), Confidence::High);
        assert_eq!(Confidence::from_score(100), Confidence::High);
    }

    #[test]
    fn test_call_graph_allocation() {
        let mut graph = CallGraph::new();

        let id1 = graph.alloc_symbol_id();
        let id2 = graph.alloc_symbol_id();

        assert_eq!(id1.0, 0);
        assert_eq!(id2.0, 1);
    }

    #[test]
    fn test_call_graph_references() {
        let mut graph = CallGraph::new();

        let file_id = graph.alloc_file_id();
        let sym1 = graph.alloc_symbol_id();
        let sym2 = graph.alloc_symbol_id();

        let location = Location::new(
            PathBuf::from("test.ts"),
            0,
            10,
            1,
            1,
        );

        graph.add_symbol(TrackedSymbol::new(
            sym1,
            "foo".to_string(),
            SymbolKind::Function,
            location.clone(),
            file_id,
        ));

        graph.add_symbol(TrackedSymbol::new(
            sym2,
            "bar".to_string(),
            SymbolKind::Function,
            location.clone(),
            file_id,
        ));

        graph.add_reference(SymbolReference::new(
            sym1,
            sym2,
            ReferenceKind::Call,
            location,
        ));

        assert_eq!(graph.get_incoming_refs(sym2), &[sym1]);
        assert_eq!(graph.get_outgoing_refs(sym1), &[sym2]);
    }
}
