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

    fn field_name(&self, kind: &SymbolKind) -> String {
        match &self {
            Languages::Rust(language) => language.field_name(kind),
            Languages::Unknown(language) => language.field_name(kind),
        }
    }

    fn has_kind(&self, tree_sitter_kind: &str, kind: &SymbolKind) -> bool {
        match &self {
            Languages::Rust(language) => language.has_kind(tree_sitter_kind, kind),
            Languages::Unknown(language) => language.has_kind(tree_sitter_kind, kind),
        }
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        match &self {
            Languages::Rust(language) => language.parse(source),
            Languages::Unknown(language) => language.parse(source),
        }
    }
}

/// Returns the language used in a given file.
/// Will use the file extension.
///
/// ## Parameters:
/// * `file_name` (`&str`): Name of the file to get the language from.
///
/// ## Returns:
/// * (`Languages`): Language identified in the given file. If identification fails,
/// `Languages::Unknown` will be returned.
pub fn get_language_for_file(file_name: &str) -> Languages {
    match file_name.rsplit_once(".") {
        Some((_, "rs")) => Languages::Rust(RustLanguage {}),
        _ => Languages::Unknown(UnknownLanguage {}),
    }
}
