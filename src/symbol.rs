use anyhow::Result;
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
};
use tree_sitter::{Node, Tree};

use crate::{
    language::{parsable_language::ParsableLanguage, Languages},
    symbol_kind::SymbolKind,
};

#[derive(Debug)]
/// Symbol extracted from a source file.
///
/// ## Properties:
/// * `name` (`String`): Name of the symbol,
/// * `line` (`usize`): Line number where the symbol is named,
/// * `file` (`String`): Name of the file declaring the symbol,
/// * `kind` (`symbol_kind::SymbolKind`): Kind of symbol (eg. function),
/// * `is_exported` (`bool`): true iff the symbol is usable from outside of the current scope.
/// * `scope` (`Vec<String>`): Hierarchical scope (e.g., modules, classes) where the symbol is defined.
pub struct Symbol {
    /// Name of the symbol.
    pub name: String,
    /// Line number where the symbol is named.
    pub line: usize,
    /// Name of the file declaring the symbol.
    pub file: String,
    /// Kind of symbol (eg. function).
    pub kind: SymbolKind,
    /// true iff the symbol is usable from outside of the current scope.
    pub is_exported: bool,
    /// Hierarchical scope (e.g., modules, classes) where the symbol is defined.
    pub scope: Vec<String>,
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} {:?} \x1b[1m{}\x1b[0m (l.{} in {}): \x1b[2m{}\x1b[0m",
            if self.is_exported {
                "🔑public"
            } else {
                "🔒private"
            },
            self.kind,
            self.name,
            self.line,
            self.file,
            self.scope.join("::"),
        )
    }
}

/// Gets changed symbols from a parsed file.
/// Will extracts symbols present at line which changed.
///
/// ## Parameters:
/// * `tree` (`&tree_sitter::Tree`): File parsed with tree_sitter,
/// * `file` (`&str`): Name of the file,
/// * `source` (`&str`): Content of the file,
/// * `changed_lines` (`&std::collections::HashSet<usize>`): Set of changed lines in the text,
/// * `language` (`language::Languages`): Language of the current file.
///
/// ## Returns:
/// * (`Result<Vec<Symbol>>`): List of symbol which changed. Will fail if the language was
/// incorrect.
pub fn extract_changed_symbols(
    tree: &Tree,
    file: &str,
    source: &str,
    changed_lines: &HashSet<usize>,
    language: &Languages,
) -> Result<Vec<Symbol>> {
    let cursor = tree.walk();
    let mut symbols = Vec::new();
    let mut scope_stack = language.scope_from_path(file);
    walk_tree(
        cursor.node(),
        file,
        source,
        &mut symbols,
        changed_lines,
        language,
        &mut scope_stack,
    );
    Ok(symbols)
}

fn walk_tree(
    node: Node,
    file: &str,
    source: &str,
    symbols: &mut Vec<Symbol>,
    changed_lines: &HashSet<usize>,
    language: &Languages,
    scope_stack: &mut Vec<String>,
) {
    let new_scope = language.get_name_for_node(node, source);
    if let Some(ref scope_name) = new_scope {
        scope_stack.push(scope_name.to_string());
    }
    for kind in SymbolKind::iter() {
        let expected_field = language.field_name(kind);
        if language.has_kind(node.kind(), kind) {
            if let Some(name_node) = node.child_by_field_name(expected_field) {
                let name = name_node
                    .utf8_text(source.as_bytes())
                    .unwrap_or("<unknown>")
                    .to_string();
                let line = name_node.start_position().row + 1;
                if changed_lines.iter().any(|&changed_line| {
                    let starting_row = node.start_position().row;
                    let ending_row = node.end_position().row;
                    (starting_row <= changed_line) && (changed_line <= ending_row)
                }) {
                    symbols.push(Symbol {
                        name,
                        line,
                        file: file.to_string(),
                        kind: SymbolKind::Function,
                        is_exported: language.is_exported(node, source),
                        scope: scope_stack.clone(),
                    });
                }
            }
        }
    }

    for child in node.children(&mut node.walk()) {
        walk_tree(
            child,
            file,
            source,
            symbols,
            changed_lines,
            language,
            scope_stack,
        );
    }
    if new_scope.is_some() {
        scope_stack.pop();
    }
}
