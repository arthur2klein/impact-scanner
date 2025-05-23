use std::path::PathBuf;

use anyhow::Result;
use parsable_language::ParsableLanguage;
use rust::RustLanguage;
use tree_sitter::{Node, Tree};
use unknown::UnknownLanguage;

use crate::symbol_kind::SymbolKind;

pub mod parsable_language;
mod rust;
mod unknown;

#[derive(Debug)]
/// Enum of supported languages
pub enum Languages {
    /// Rust is the language the functionalities will be first implemented for.
    Rust(RustLanguage),
    /// Most method return dummy values, parse indicates that the language is not known.
    Unknown(UnknownLanguage),
}

impl ParsableLanguage for Languages {
    fn is_exported(&self, node: Node, source: &str) -> bool {
        match &self {
            Languages::Rust(language) => language.is_exported(node, source),
            Languages::Unknown(language) => language.is_exported(node, source),
        }
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        match &self {
            Languages::Rust(language) => language.parse(source),
            Languages::Unknown(language) => language.parse(source),
        }
    }

    fn get_scope_name_for_node(&self, node: Node, source: &str) -> Option<String> {
        match &self {
            Languages::Rust(language) => language.get_scope_name_for_node(node, source),
            Languages::Unknown(language) => language.get_scope_name_for_node(node, source),
        }
    }

    fn get_name_node_of_symbol<'a>(
        &self,
        node: &Node<'a>,
    ) -> Option<(Node<'a>, &'static SymbolKind)> {
        match &self {
            Languages::Rust(language) => language.get_name_node_of_symbol(node),
            Languages::Unknown(language) => language.get_name_node_of_symbol(node),
        }
    }

    fn scope_from_path(&self, file_path: &PathBuf) -> Vec<String> {
        match &self {
            Languages::Rust(language) => language.scope_from_path(file_path),
            Languages::Unknown(language) => language.scope_from_path(file_path),
        }
    }
}

/// Returns the language used in a given file.
/// Will use the file extension.
///
/// ## Parameters:
/// * `file_name` (`&std::path::PathBuf`): Name of the file to get the language from.
///
/// ## Returns:
/// * (`Languages`): Language identified in the given file. If identification fails,
/// `Languages::Unknown` will be returned.
pub fn get_language_for_file(file_name: &PathBuf) -> Languages {
    match file_name.extension().and_then(|v| v.to_str()) {
        Some("rs") => Languages::Rust(RustLanguage {}),
        _ => Languages::Unknown(UnknownLanguage {}),
    }
}
