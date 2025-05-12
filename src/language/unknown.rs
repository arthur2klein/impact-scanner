use std::path::PathBuf;

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

    fn parse(&self, _source: &str) -> Result<Tree> {
        bail!("Unknown language")
    }

    fn get_scope_name_for_node(&self, _node: Node, _source: &str) -> Option<String> {
        None
    }

    fn get_name_node_of_symbol<'a>(
        &self,
        _node: &Node<'a>,
    ) -> Option<(Node<'a>, &'static SymbolKind)> {
        None
    }

    fn scope_from_path(&self, _file_path: &PathBuf) -> Vec<String> {
        Vec::new()
    }
}
