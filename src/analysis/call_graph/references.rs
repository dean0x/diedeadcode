//! Reference extraction from AST.

use crate::core::{FileId, Location, SymbolId, SymbolReference};
use oxc::ast::ast::*;
use oxc::ast_visit::walk;
use oxc::ast_visit::Visit;
use oxc::semantic::Semantic;
use oxc::span::Span;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Extracts references between symbols from an AST.
pub struct ReferenceExtractor<'a> {
    file_path: PathBuf,
    source: &'a str,
    references: Vec<SymbolReference>,
    imports: Vec<ImportInfo>,
    /// Whether dynamic eval was detected.
    pub has_dynamic_eval: bool,
}

/// Information about an import.
#[derive(Debug, Clone)]
pub struct ImportInfo {
    pub local_symbol_id: SymbolId,
    pub imported_name: String,
    pub resolved_path: PathBuf,
    pub is_dynamic: bool,
    pub location: Location,
}

impl<'a> ReferenceExtractor<'a> {
    pub fn new(
        file_path: PathBuf,
        _file_id: FileId,
        _symbol_map: &'a HashMap<oxc::semantic::SymbolId, SymbolId>,
        source: &'a str,
    ) -> Self {
        Self {
            file_path,
            source,
            references: Vec::new(),
            imports: Vec::new(),
            has_dynamic_eval: false,
        }
    }

    pub fn extract(
        mut self,
        program: &Program<'a>,
        _semantic: &Semantic<'a>,
        file_path: &Path,
    ) -> (Vec<SymbolReference>, Vec<ImportInfo>, bool) {
        // First, collect imports
        for stmt in &program.body {
            if let Statement::ImportDeclaration(import) = stmt {
                self.process_import(import, file_path);
            }
        }

        // Then walk the AST for references
        self.visit_program(program);

        (self.references, self.imports, self.has_dynamic_eval)
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

    fn process_import(&mut self, import: &ImportDeclaration<'_>, file_path: &Path) {
        let source_value = import.source.value.as_str();
        let resolved_path = resolve_import_specifier(source_value, file_path);

        if let Some(specifiers) = &import.specifiers {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(spec) => {
                        // import { foo } from './bar' or import { foo as bar } from './bar'
                        let imported_name = spec.imported.name().to_string();
                        let local_name = spec.local.name.to_string();

                        // Create a placeholder symbol ID - will be resolved in builder
                        let local_id = SymbolId::new(u32::MAX - self.imports.len() as u32);

                        self.imports.push(ImportInfo {
                            local_symbol_id: local_id,
                            imported_name,
                            resolved_path: resolved_path.clone(),
                            is_dynamic: false,
                            location: self.span_to_location(spec.span),
                        });

                        let _ = local_name;
                    }
                    ImportDeclarationSpecifier::ImportDefaultSpecifier(spec) => {
                        // import foo from './bar'
                        let local_id = SymbolId::new(u32::MAX - self.imports.len() as u32);

                        self.imports.push(ImportInfo {
                            local_symbol_id: local_id,
                            imported_name: "default".to_string(),
                            resolved_path: resolved_path.clone(),
                            is_dynamic: false,
                            location: self.span_to_location(spec.span),
                        });
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(spec) => {
                        // import * as foo from './bar'
                        let local_id = SymbolId::new(u32::MAX - self.imports.len() as u32);

                        self.imports.push(ImportInfo {
                            local_symbol_id: local_id,
                            imported_name: "*".to_string(),
                            resolved_path: resolved_path.clone(),
                            is_dynamic: false,
                            location: self.span_to_location(spec.span),
                        });
                    }
                }
            }
        }
    }

}

impl<'a> Visit<'a> for ReferenceExtractor<'a> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        // Check for eval()
        if let Expression::Identifier(id) = &expr.callee {
            if id.name == "eval" {
                self.has_dynamic_eval = true;
            }
        }

        // Check for new Function()
        if let Expression::Identifier(id) = &expr.callee {
            if id.name == "Function" {
                self.has_dynamic_eval = true;
            }
        }

        // Check for Reflect.* calls
        if let Expression::StaticMemberExpression(member) = &expr.callee {
            if let Expression::Identifier(obj) = &member.object {
                if obj.name == "Reflect" {
                    self.has_dynamic_eval = true;
                }
            }
        }

        walk::walk_call_expression(self, expr);
    }

    fn visit_import_expression(&mut self, expr: &ImportExpression<'a>) {
        // Dynamic import
        if let Expression::StringLiteral(lit) = &expr.source {
            let resolved = resolve_import_specifier(&lit.value, &self.file_path);
            let local_id = SymbolId::new(u32::MAX - self.imports.len() as u32);

            self.imports.push(ImportInfo {
                local_symbol_id: local_id,
                imported_name: "*".to_string(),
                resolved_path: resolved,
                is_dynamic: true,
                location: self.span_to_location(expr.span),
            });
        } else {
            // Non-literal dynamic import - we can't resolve it
            self.has_dynamic_eval = true;
        }

        walk::walk_import_expression(self, expr);
    }

    fn visit_computed_member_expression(&mut self, expr: &ComputedMemberExpression<'a>) {
        // Bracket access like obj[key] where key is not a literal
        if !matches!(&expr.expression, Expression::StringLiteral(_) | Expression::NumericLiteral(_)) {
            // This is dynamic property access - reduces confidence
            // We note this but don't treat it as eval-level danger
        }
        walk::walk_computed_member_expression(self, expr);
    }

    fn visit_jsx_element(&mut self, elem: &JSXElement<'a>) {
        // JSX element usage creates a reference
        if let JSXElementName::Identifier(id) = &elem.opening_element.name {
            // Only uppercase identifiers are components
            if id.name.chars().next().map_or(false, |c| c.is_uppercase()) {
                // This would be a reference to the component
                // We'd need to resolve it through scope
            }
        }
        walk::walk_jsx_element(self, elem);
    }
}

/// Resolve an import specifier to a file path.
fn resolve_import_specifier(specifier: &str, from_file: &Path) -> PathBuf {
    let from_dir = from_file.parent().unwrap_or(Path::new("."));

    if specifier.starts_with('.') {
        // Relative import
        from_dir.join(specifier)
    } else if specifier.starts_with('/') {
        // Absolute import (rare in TS/JS)
        PathBuf::from(specifier)
    } else {
        // Package import - would need to look in node_modules
        // For now, return a placeholder path
        PathBuf::from(format!("node_modules/{}", specifier))
    }
}
