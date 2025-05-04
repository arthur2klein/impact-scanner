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

    fn treesitter_kind(&self, _kind: &SymbolKind) -> String {
        "".to_string()
    }

    fn parse(&self, _source: &str) -> Result<Tree> {
        bail!("Unknown language")
    }
}
