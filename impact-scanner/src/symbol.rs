use anyhow::Result;
use impact_scanner_derive::TestBuilder;
use std::{
    collections::HashSet,
    fmt::{Display, Formatter},
    path::PathBuf,
};
use tree_sitter::{Node, Tree};

use crate::{
    language::{parsable_language::ParsableLanguage, Languages},
    symbol_kind::SymbolKind,
};

#[derive(Debug, Clone, TestBuilder)]
/// Symbol extracted from a source file.
///
/// ## Properties:
/// * `naming` (`Option<String>`): Alias of the symbol, if any,
/// * `line` (`usize`): Line number where the symbol is named,
/// * `file` (`std::path::PathBuf`): Name of the file the symbol was found in.
/// * `kind` (`symbol_kind::SymbolKind`): Kind of symbol (eg. function),
/// * `is_exported` (`bool`): true iff the symbol is usable from outside of the current scope.
/// * `scope` (`Vec<String>`): Hierarchical scope (e.g., modules, classes) where the symbol is defined.
pub struct Symbol {
    /// Alias of the imported symbol, if any.
    pub naming: Option<String>,
    /// Line number where the symbol is named.
    pub line: usize,
    /// Name of the file the symbol was found in.
    pub file: PathBuf,
    #[builder(default = SymbolKind::Function)]
    /// Kind of symbol (eg. function).
    pub kind: SymbolKind,
    /// true iff the symbol is usable from outside of the current scope.
    pub is_exported: bool,
    /// Hierarchical scope (e.g., modules, classes) where the symbol is defined.
    pub scope: Vec<String>,
}

impl Symbol {
    /// Name of the symbol, taking into account aliases.
    ///
    /// ## Returns:
    /// * (`String`): Alias if defined, else name of the symbol.
    pub fn name(&self) -> String {
        self.naming.clone().unwrap_or(match self.scope.last() {
            Some(v) => v.to_string(),
            None => "<invalid>".to_string(),
        })
    }
}

impl Display for Symbol {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "{} {:?} \x1b[1m{}\x1b[0m ({}:{}): \x1b[2m{}\x1b[0m",
            if self.is_exported {
                "🔑public"
            } else {
                "🔒private"
            },
            self.kind,
            self.name(),
            self.file.to_str().unwrap_or("<invalid>"),
            self.line,
            self.scope.join("::"),
        )
    }
}

fn walk_tree<'a, F>(
    node: Node<'a>,
    file: &PathBuf,
    source: &str,
    symbols: &mut Vec<Symbol>,
    language: &Languages,
    scope_stack: &mut Vec<String>,
    get_name_and_kind_if_interesting: &F,
) where
    F: Fn(&Node<'a>) -> Option<(Node<'a>, &'static SymbolKind)>,
{
    let new_scope = language.get_scope_name_for_node(node, source);
    if let Some(ref scope_name) = new_scope {
        scope_stack.push(scope_name.to_string());
    }
    if let Some((name_node, kind)) = get_name_and_kind_if_interesting(&node) {
        let name = name_node
            .utf8_text(source.as_bytes())
            .unwrap_or("<unknown>")
            .to_string();
        let line = name_node.start_position().row + 1;
        symbols.push(Symbol {
            naming: Some(name),
            line,
            file: file.clone(),
            kind: *kind,
            is_exported: language.is_exported(node, source),
            scope: scope_stack.clone(),
        });
    } else {
        for child in node.children(&mut node.walk()) {
            walk_tree(
                child,
                file,
                source,
                symbols,
                language,
                scope_stack,
                get_name_and_kind_if_interesting,
            );
        }
    }
    if new_scope.is_some() {
        scope_stack.pop();
    }
}

/// Gets symbols from a parsed file contained in a node matching the given closure.
///
/// ## Parameters:
/// * `tree` (`&tree_sitter::Tree`): File parsed with tree_sitter,
/// * `file` (`&std::path::PathBuf`): Name of the file,
/// * `source` (`&str`): Content of the file,
/// * `language` (`language::Languages`): Language of the current file,
/// * `add_path_to_scope` (`bool`): True the scope of resulting symbols should begin with the path.
/// * `get_name_and_kind_if_interesting` (`Fn(&Node) -> Option<(Node, &SymbolKind)>`): None if not
///   an interesting node, else node with the name of the given node, and kind of symbol extracted.
///
/// ## Returns:
/// * (`Result<Vec<Symbol>>`): List of symbol which matching the closure.
pub fn extract_symbols<'a, F>(
    tree: &'a Tree,
    file: &std::path::PathBuf,
    source: &str,
    language: &Languages,
    add_path_to_scope: bool,
    get_name_and_kind_if_interesting: F,
) -> Result<Vec<Symbol>>
where
    F: Fn(&Node<'a>) -> Option<(Node<'a>, &'static SymbolKind)>,
{
    let cursor = tree.walk();
    let mut symbols = Vec::new();
    let mut scope_stack = if add_path_to_scope {
        language.scope_from_path(file)
    } else {
        Vec::new()
    };
    walk_tree(
        cursor.node(),
        file,
        source,
        &mut symbols,
        language,
        &mut scope_stack,
        &get_name_and_kind_if_interesting,
    );
    Ok(symbols)
}

/// Gets changed symbols from a parsed file.
/// Will extracts symbols present at line which changed.
///
/// ## Parameters:
/// * `tree` (`&tree_sitter::Tree`): File parsed with tree_sitter,
/// * `file` (`&std::path::PathBuf`): Name of the file,
/// * `source` (`&str`): Content of the file,
/// * `changed_lines` (`&std::collections::HashSet<usize>`): Set of changed lines in the text,
/// * `language` (`language::Languages`): Language of the current file.
///
/// ## Returns:
/// * (`Result<Vec<Symbol>>`): List of symbol which changed. Will fail if the language was
/// incorrect.
pub fn extract_changed_symbols<'a>(
    tree: &'a Tree,
    file: &PathBuf,
    source: &str,
    changed_lines: &HashSet<usize>,
    language: &Languages,
) -> Result<Vec<Symbol>> {
    extract_symbols(
        tree,
        file,
        source,
        language,
        true,
        &(|node: &Node<'a>| {
            if changed_lines.iter().any(|&changed_line| {
                let starting_row = node.start_position().row;
                let ending_row = node.end_position().row;
                (starting_row <= changed_line) && (changed_line <= ending_row)
            }) {
                language.get_name_node_of_symbol(node)
            } else {
                None
            }
        }),
    )
}
