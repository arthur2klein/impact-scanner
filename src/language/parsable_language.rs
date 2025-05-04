use anyhow::Result;
use tree_sitter::{Node, Tree};

use crate::symbol_kind::SymbolKind;

/// Trait for a supported language.
pub trait ParsableLanguage {
    /// Returns true iff the symbol will be made accessible from outside of the current scope.
    ///
    /// ## Parameters:
    /// * `node` (`tree_sitter::Node`): File parsed by tree-sitter,
    /// * `source` (`&str`): Content of the current file.
    ///
    /// ## Returns:
    /// * (`bool`): true iff the symbol is public.
    fn is_exported(&self, node: Node, source: &str) -> bool;

    /// Returns the tree-sitter field name used to find the name of a given symbol kind.
    ///
    /// ## Parameters:
    /// * `kind` (`&symbol_kind::SymbolKind`): Kind of symbol to get the name of.
    ///
    /// ## Returns:
    /// * (`String`): Field name containing the name of a given tree-sitter symbol kind.
    fn field_name(&self, kind: &SymbolKind) -> String;

    /// Checks if the given tree-sitter kind identifies a given `SymbolKind`.
    ///
    /// ## Parameters:
    /// * `tree_sitter_kind` (`&str`): Name of the tree-sitter kind related to the given argument.
    /// * `kind` (`&symbol_kind::SymbolKind`): Kind of symbol to get the tree-sitter kind name of.
    ///
    /// ## Returns:
    /// * (`bool`): true iff the given tree-sitter kind correspond to the wanted kind.
    fn has_kind(&self, tree_sitter_kind: &str, kind: &SymbolKind) -> bool;

    /// Parse a file as a `tree_sitter::Tree`.
    ///
    /// ## Parameters:
    /// * `source` (`&str`): Content of the file.
    ///
    /// ## Returns:
    /// * (`Result<tree_sitter::Tree>`): Given file parsed by tree-sitter.
    fn parse(&self, source: &str) -> Result<Tree>;
}
