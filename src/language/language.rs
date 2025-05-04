use anyhow::Result;
use tree_sitter::{Node, Tree};

use crate::symbol_kind::SymbolKind;

///
pub trait Language {
    fn is_exported(&self, node: Node, source: &str) -> bool;
    fn field_name(&self, kind: &SymbolKind) -> String;
    fn treesitter_kind(&self, kind: &SymbolKind) -> String;
    fn parse(&self, source: &str) -> Result<Tree>;
}
