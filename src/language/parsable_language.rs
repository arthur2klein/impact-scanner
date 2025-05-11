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

    /// Parse a file as a `tree_sitter::Tree`.
    ///
    /// ## Parameters:
    /// * `source` (`&str`): Content of the file.
    ///
    /// ## Returns:
    /// * (`Result<tree_sitter::Tree>`): Given file parsed by tree-sitter.
    fn parse(&self, source: &str) -> Result<Tree>;

    /// Optionally returns the name associated with a node if it represents one.
    ///
    /// ## Parameters:
    /// * `node` (`tree_sitter::Node`): Node to get the name of.
    /// * `source` (`&str`): Content of the file.
    ///
    /// ## Returns:
    /// * (`Option<String>`): Name of the given node, or None if given node is not a scope name.
    fn get_scope_name_for_node(&self, node: Node, source: &str) -> Option<String>;

    /// Returns the node containing the name of the given symbol, if the symbol is used by the
    /// program.
    ///
    /// ## Parameters:
    /// * `node` (`tree_sitter::Node`): Node to get the name node of.
    ///
    /// ## Returns:
    /// * (`Option<(Node, &SymbolKind)>`): None if the symbol is not interesting, else node
    ///   containing the name of the given node, and kind of symbol represented by the node.
    fn get_name_node_of_symbol<'a>(
        &self,
        node: &Node<'a>,
    ) -> Option<(Node<'a>, &'static SymbolKind)>;

    /// Returns the scope to deduce from file name alone for the entirety of the file.
    ///
    /// ## Parameters:
    /// * `file_path` (`&str`): Name of the file.
    ///
    /// ## Returns:
    /// * (`Vec<String>`): Scope of the given file constructed from the given path.
    fn scope_from_path(&self, file_path: &str) -> Vec<String>;
}
