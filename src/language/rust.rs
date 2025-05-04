use crate::symbol_kind::SymbolKind;

use super::parsable_language::ParsableLanguage;
use anyhow::{anyhow, Result};
use tree_sitter::{Node, Parser, Tree};
use tree_sitter_rust::LANGUAGE as rust_language;

#[derive(Debug)]
pub struct RustLanguage {}

impl ParsableLanguage for RustLanguage {
    fn is_exported(&self, node: Node, source: &str) -> bool {
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            if child.kind() == "visibility_modifier" {
                let text = child.utf8_text(source.as_bytes()).unwrap_or("");
                return text.starts_with("pub");
            }
        }
        false
    }

    fn field_name(&self, kind: &SymbolKind) -> String {
        match kind {
            SymbolKind::Function => "name".to_string(),
        }
    }

    fn has_kind(&self, tree_sitter_kind: &str, kind: &SymbolKind) -> bool {
        match kind {
            SymbolKind::Function => "function_item" == tree_sitter_kind,
        }
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser.set_language(&rust_language.into())?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| anyhow!("Parse failed"))?;
        Ok(tree)
    }
}
