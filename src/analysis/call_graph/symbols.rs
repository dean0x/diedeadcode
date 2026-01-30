//! Symbol extraction from AST.

use crate::core::{FileId, Location, SymbolId, SymbolKind, TrackedSymbol};
use oxc::ast::ast::*;
use oxc::ast_visit::walk;
use oxc::ast_visit::Visit;
use oxc::semantic::Semantic;
use oxc::span::Span;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU32, Ordering};

/// Extracts symbols from an AST.
pub struct SymbolExtractor<'a> {
    file_path: PathBuf,
    file_id: FileId,
    symbol_id_counter: &'a AtomicU32,
    source: &'a str,
    symbols: Vec<TrackedSymbol>,
    /// Map from oxc symbol ID to our symbol ID.
    pub symbol_map: HashMap<oxc::semantic::SymbolId, SymbolId>,
    /// Whether side effects were detected.
    pub has_side_effects: bool,
    /// Current scope depth (0 = module level).
    scope_depth: u32,
}

impl<'a> SymbolExtractor<'a> {
    pub fn new(
        file_path: PathBuf,
        file_id: FileId,
        symbol_id_counter: &'a AtomicU32,
        source: &'a str,
    ) -> Self {
        Self {
            file_path,
            file_id,
            symbol_id_counter,
            source,
            symbols: Vec::new(),
            symbol_map: HashMap::new(),
            has_side_effects: false,
            scope_depth: 0,
        }
    }

    pub fn extract(
        mut self,
        program: &Program<'a>,
        _semantic: &Semantic<'a>,
    ) -> (Vec<TrackedSymbol>, HashMap<oxc::semantic::SymbolId, SymbolId>, bool) {
        self.visit_program(program);
        // Note: Decorator detection would require additional AST traversal
        (self.symbols, self.symbol_map, self.has_side_effects)
    }

    fn alloc_symbol_id(&self) -> SymbolId {
        SymbolId::new(self.symbol_id_counter.fetch_add(1, Ordering::SeqCst))
    }

    fn span_to_location(&self, span: Span) -> Location {
        let (line, column) = self.offset_to_line_col(span.start);
        Location::new(
            self.file_path.clone(),
            span.start,
            span.end,
            line,
            column,
        )
    }

    fn offset_to_line_col(&self, offset: u32) -> (u32, u32) {
        let offset = offset as usize;
        let mut line = 1u32;
        let mut col = 1u32;

        for (i, ch) in self.source.char_indices() {
            if i >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        (line, col)
    }

    fn add_symbol(&mut self, name: String, kind: SymbolKind, span: Span, exported: bool) -> SymbolId {
        let id = self.alloc_symbol_id();
        let location = self.span_to_location(span);

        let mut symbol = TrackedSymbol::new(id, name, kind, location, self.file_id);
        symbol.exported = exported;

        self.symbols.push(symbol);
        id
    }

    fn is_module_level(&self) -> bool {
        self.scope_depth == 0
    }
}

impl<'a> Visit<'a> for SymbolExtractor<'a> {
    fn visit_function(&mut self, func: &Function<'a>, flags: oxc::semantic::ScopeFlags) {
        if let Some(id) = &func.id {
            let kind = SymbolKind::Function;
            self.add_symbol(id.name.to_string(), kind, id.span, false);
        }

        self.scope_depth += 1;
        walk::walk_function(self, func, flags);
        self.scope_depth -= 1;
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        if let Some(id) = &class.id {
            self.add_symbol(id.name.to_string(), SymbolKind::Class, id.span, false);
        }

        self.scope_depth += 1;
        walk::walk_class(self, class);
        self.scope_depth -= 1;
    }

    fn visit_variable_declaration(&mut self, decl: &VariableDeclaration<'a>) {
        let kind = match decl.kind {
            VariableDeclarationKind::Const => SymbolKind::Constant,
            _ => SymbolKind::Variable,
        };

        for declarator in &decl.declarations {
            self.extract_binding_pattern_names(&declarator.id, kind);

            // Check for side effects in initializer at module level
            if self.is_module_level() {
                if let Some(init) = &declarator.init {
                    if self.expression_has_side_effects(init) {
                        self.has_side_effects = true;
                    }
                }
            }
        }

        walk::walk_variable_declaration(self, decl);
    }

    fn visit_ts_type_alias_declaration(&mut self, decl: &TSTypeAliasDeclaration<'a>) {
        self.add_symbol(
            decl.id.name.to_string(),
            SymbolKind::Type,
            decl.id.span,
            false,
        );
        walk::walk_ts_type_alias_declaration(self, decl);
    }

    fn visit_ts_interface_declaration(&mut self, decl: &TSInterfaceDeclaration<'a>) {
        self.add_symbol(
            decl.id.name.to_string(),
            SymbolKind::Interface,
            decl.id.span,
            false,
        );
        walk::walk_ts_interface_declaration(self, decl);
    }

    fn visit_ts_enum_declaration(&mut self, decl: &TSEnumDeclaration<'a>) {
        let enum_id = self.add_symbol(
            decl.id.name.to_string(),
            SymbolKind::Enum,
            decl.id.span,
            false,
        );

        // Add enum members
        for member in &decl.body.members {
            let name = match &member.id {
                TSEnumMemberName::Identifier(id) => id.name.to_string(),
                TSEnumMemberName::String(s) => s.value.to_string(),
                TSEnumMemberName::ComputedString(s) => s.value.to_string(),
                TSEnumMemberName::ComputedTemplateString(_) => continue, // Skip computed template strings
            };
            self.add_symbol(name, SymbolKind::EnumMember, member.span, false);
        }

        let _ = enum_id; // Enum members could reference the enum
    }

    fn visit_ts_module_declaration(&mut self, decl: &TSModuleDeclaration<'a>) {
        let name = match &decl.id {
            TSModuleDeclarationName::Identifier(id) => id.name.to_string(),
            TSModuleDeclarationName::StringLiteral(s) => s.value.to_string(),
        };
        self.add_symbol(name, SymbolKind::Namespace, decl.span, false);

        self.scope_depth += 1;
        walk::walk_ts_module_declaration(self, decl);
        self.scope_depth -= 1;
    }

    fn visit_export_named_declaration(&mut self, decl: &ExportNamedDeclaration<'a>) {
        // Mark exported declarations
        if let Some(declaration) = &decl.declaration {
            // The declaration will be visited and we mark it as exported
            match declaration {
                Declaration::FunctionDeclaration(f) => {
                    if let Some(id) = &f.id {
                        let sym_id = self.add_symbol(
                            id.name.to_string(),
                            SymbolKind::Function,
                            id.span,
                            true,
                        );
                        let _ = sym_id;
                    }
                }
                Declaration::ClassDeclaration(c) => {
                    if let Some(id) = &c.id {
                        self.add_symbol(
                            id.name.to_string(),
                            SymbolKind::Class,
                            id.span,
                            true,
                        );
                    }
                }
                Declaration::VariableDeclaration(v) => {
                    let kind = match v.kind {
                        VariableDeclarationKind::Const => SymbolKind::Constant,
                        _ => SymbolKind::Variable,
                    };
                    for declarator in &v.declarations {
                        self.extract_binding_pattern_names_exported(&declarator.id, kind);
                    }
                }
                Declaration::TSTypeAliasDeclaration(t) => {
                    self.add_symbol(
                        t.id.name.to_string(),
                        SymbolKind::Type,
                        t.id.span,
                        true,
                    );
                }
                Declaration::TSInterfaceDeclaration(i) => {
                    self.add_symbol(
                        i.id.name.to_string(),
                        SymbolKind::Interface,
                        i.id.span,
                        true,
                    );
                }
                Declaration::TSEnumDeclaration(e) => {
                    self.add_symbol(
                        e.id.name.to_string(),
                        SymbolKind::Enum,
                        e.id.span,
                        true,
                    );
                }
                Declaration::TSModuleDeclaration(m) => {
                    let name = match &m.id {
                        TSModuleDeclarationName::Identifier(id) => id.name.to_string(),
                        TSModuleDeclarationName::StringLiteral(s) => s.value.to_string(),
                    };
                    self.add_symbol(name, SymbolKind::Namespace, m.span, true);
                }
                // These are less common declarations, skip for now
                Declaration::TSGlobalDeclaration(_) | Declaration::TSImportEqualsDeclaration(_) => {}
            }
        }

        // Handle export specifiers (export { foo, bar })
        for specifier in &decl.specifiers {
            // These create references to existing symbols, handled in references.rs
            let _ = specifier;
        }
    }

    fn visit_export_default_declaration(&mut self, decl: &ExportDefaultDeclaration<'a>) {
        match &decl.declaration {
            ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
                let name = f.id.as_ref()
                    .map(|id| id.name.to_string())
                    .unwrap_or_else(|| "default".to_string());
                self.add_symbol(name, SymbolKind::Function, f.span, true);
            }
            ExportDefaultDeclarationKind::ClassDeclaration(c) => {
                let name = c.id.as_ref()
                    .map(|id| id.name.to_string())
                    .unwrap_or_else(|| "default".to_string());
                self.add_symbol(name, SymbolKind::Class, c.span, true);
            }
            ExportDefaultDeclarationKind::TSInterfaceDeclaration(i) => {
                self.add_symbol(i.id.name.to_string(), SymbolKind::Interface, i.span, true);
            }
            _ => {
                // Default export of an expression - treated as anonymous
            }
        }
        walk::walk_export_default_declaration(self, decl);
    }
}

impl<'a> SymbolExtractor<'a> {
    fn extract_binding_pattern_names(&mut self, pattern: &BindingPattern<'a>, kind: SymbolKind) {
        match &pattern.kind {
            BindingPatternKind::BindingIdentifier(id) => {
                self.add_symbol(id.name.to_string(), kind, id.span, false);
            }
            BindingPatternKind::ObjectPattern(obj) => {
                for prop in &obj.properties {
                    self.extract_binding_pattern_names(&prop.value, kind);
                }
                if let Some(rest) = &obj.rest {
                    self.extract_binding_pattern_names(&rest.argument, kind);
                }
            }
            BindingPatternKind::ArrayPattern(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.extract_binding_pattern_names(elem, kind);
                }
                if let Some(rest) = &arr.rest {
                    self.extract_binding_pattern_names(&rest.argument, kind);
                }
            }
            BindingPatternKind::AssignmentPattern(assign) => {
                self.extract_binding_pattern_names(&assign.left, kind);
            }
        }
    }

    fn extract_binding_pattern_names_exported(&mut self, pattern: &BindingPattern<'a>, kind: SymbolKind) {
        match &pattern.kind {
            BindingPatternKind::BindingIdentifier(id) => {
                self.add_symbol(id.name.to_string(), kind, id.span, true);
            }
            BindingPatternKind::ObjectPattern(obj) => {
                for prop in &obj.properties {
                    self.extract_binding_pattern_names_exported(&prop.value, kind);
                }
            }
            BindingPatternKind::ArrayPattern(arr) => {
                for elem in arr.elements.iter().flatten() {
                    self.extract_binding_pattern_names_exported(elem, kind);
                }
            }
            BindingPatternKind::AssignmentPattern(assign) => {
                self.extract_binding_pattern_names_exported(&assign.left, kind);
            }
        }
    }

    fn expression_has_side_effects(&self, expr: &Expression<'a>) -> bool {
        match expr {
            Expression::CallExpression(_) => true,
            Expression::NewExpression(_) => true,
            Expression::AssignmentExpression(_) => true,
            Expression::UpdateExpression(_) => true,
            Expression::AwaitExpression(_) => true,
            Expression::YieldExpression(_) => true,
            _ => false,
        }
    }
}
