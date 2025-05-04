use anyhow::Result;
use rust::RustLanguage;
use tree_sitter::{Node, Tree};
use unknown::UnknownLanguage;

use crate::symbol_kind::SymbolKind;

pub mod language;
mod rust;
mod unknown;

#[derive(Debug)]
pub enum Languages {
    Rust(RustLanguage),
    Unknown(UnknownLanguage),
}

impl language::Language for Languages {
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

    fn treesitter_kind(&self, kind: &SymbolKind) -> String {
        match &self {
            Languages::Rust(language) => language.treesitter_kind(kind),
            Languages::Unknown(language) => language.treesitter_kind(kind),
        }
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        match &self {
            Languages::Rust(language) => language.parse(source),
            Languages::Unknown(language) => language.parse(source),
        }
    }
}

pub fn get_language_for_file(name: &String) -> Languages {
    match name.rsplit_once(".") {
        Some((_, "rs")) => Languages::Rust(RustLanguage {}),
        _ => Languages::Unknown(UnknownLanguage {}),
    }
}
