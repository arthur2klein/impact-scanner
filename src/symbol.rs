use anyhow::Result;
use std::collections::HashSet;
use tree_sitter::{Node, Tree};

#[derive(Debug)]
pub struct Symbol {
    pub name: String,
    pub line: usize,
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

fn walk_tree(node: Node, source: &str, symbols: &mut Vec<Symbol>, changed_lines: &HashSet<usize>) {
    if node.kind() == "function_item" {
        if let Some(name_node) = node.child_by_field_name("name") {
            let name = name_node
                .utf8_text(source.as_bytes())
                .unwrap_or("<unknown>")
                .to_string();
            let line = name_node.start_position().row + 1;
            if changed_lines.contains(&line) {
                symbols.push(Symbol { name, line });
            }
        }
    }

    for child in node.children(&mut node.walk()) {
        walk_tree(child, source, symbols, changed_lines);
    }
}
