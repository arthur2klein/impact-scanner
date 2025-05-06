use anyhow::{bail, Result};
use tree_sitter::{Node, Tree};

use crate::symbol_kind::SymbolKind;

use super::parsable_language::ParsableLanguage;

#[derive(Debug)]
pub struct UnknownLanguage {}

impl ParsableLanguage for UnknownLanguage {
    fn is_exported(&self, _node: Node, _source: &str) -> bool {
        false
    }

    fn field_name(&self, _kind: &SymbolKind) -> String {
        "".to_string()
    }

    fn has_kind(&self, _tree_sitter_kind: &str, _kind: &SymbolKind) -> bool {
        false
    }

    fn parse(&self, _source: &str) -> Result<Tree> {
        bail!("Unknown language")
    }

    fn get_name_for_node(&self, _node: Node, _source: &str) -> Option<String> {
        None
    }

    fn scope_from_path(&self, _file_path: &str) -> Vec<String> {
        Vec::new()
    }
}
