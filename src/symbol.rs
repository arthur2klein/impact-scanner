use anyhow::Result;
use std::collections::HashSet;
use tree_sitter::{Node, Tree};

#[derive(PartialEq, Eq, Debug)]
pub enum SymbolKind {
    Function,
}

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub line: usize,
    pub kind: SymbolKind,
    pub is_exported: bool,
}

pub fn extract_changed_symbols(
    tree: &Tree,
    source: &str,
    changed_lines: &HashSet<usize>,
) -> Result<Vec<Symbol>> {
    let cursor = tree.walk();
    let mut symbols = Vec::new();

    walk_tree(cursor.node(), source, &mut symbols, changed_lines);
    Ok(symbols)
}

fn is_exported(node: Node, source: &str) -> bool {
    for i in 0..node.child_count() {
        let child = node.child(i).unwrap();
        if child.kind() == "visibility_modifier" {
            let text = child.utf8_text(source.as_bytes()).unwrap_or("");
            return text.starts_with("pub");
        }
    }
    false
}

fn walk_tree(node: Node, source: &str, symbols: &mut Vec<Symbol>, changed_lines: &HashSet<usize>) {
    if node.kind() == "function_item" {
        if let Some(name_node) = node.child_by_field_name("name") {
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
                    kind: Kind::Function,
                    is_exported: is_exported(node, source),
                });
            }
        }
    }

    for child in node.children(&mut node.walk()) {
        walk_tree(child, source, symbols, changed_lines);
    }
}
