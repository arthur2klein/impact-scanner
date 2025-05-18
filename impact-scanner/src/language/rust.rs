use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crate::symbol_kind::SymbolKind;

use super::parsable_language::ParsableLanguage;
use anyhow::{anyhow, Result};
use tree_sitter::{Node, Parser, Tree};
use tree_sitter_rust::LANGUAGE as rust_language;

#[derive(Debug)]
pub struct RustLanguage {}

impl ParsableLanguage for RustLanguage {
    fn is_exported(&self, node: Node, source: &str) -> bool {
        let _test = 0;
        for i in 0..node.child_count() {
            let child = node.child(i).unwrap();
            if child.kind() == "visibility_modifier" {
                let text = child.utf8_text(source.as_bytes()).unwrap_or("");
                return text.starts_with("pub");
            }
        }
        false
    }

    fn parse(&self, source: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser.set_language(&rust_language.into())?;
        let tree = parser
            .parse(source, None)
            .ok_or_else(|| anyhow!("Parse failed"))?;
        Ok(tree)
    }

    fn get_scope_name_for_node(&self, node: Node, source: &str) -> Option<String> {
        match node.kind() {
            "mod_item" | "struct_item" | "enum_item" | "trait_item" | "function_item" => {
                if let Some(name_node) = node.child_by_field_name("name") {
                    return name_node
                        .utf8_text(source.as_bytes())
                        .ok()
                        .map(|s| s.to_string());
                }
            }
            "impl_item" => {
                if let Some(name_node) = node.child_by_field_name("type") {
                    return name_node
                        .utf8_text(source.as_bytes())
                        .ok()
                        .map(|s| s.to_string());
                }
            }
            _ => {}
        }
        None
    }

    fn get_name_node_of_symbol<'a>(
        &self,
        node: &Node<'a>,
    ) -> Option<(Node<'a>, &'static SymbolKind)> {
        if "function_item" == node.kind() {
            if let Some(name_node) = node.child_by_field_name("name".to_string()) {
                return Some((name_node, &SymbolKind::Function));
            }
        }
        None
    }

    fn scope_from_path(&self, file_path: &PathBuf) -> Vec<String> {
        let mut components = file_path
            .components()
            .skip_while(|c| {
                matches!(
                    c.as_os_str().to_str(),
                    Some("src") | Some("tests") | Some("src-bin") | Some(".")
                )
            })
            .collect::<Vec<_>>();
        if let Some(file_stem) = file_path.file_stem().and_then(|s| s.to_str()) {
            components.pop();
            if file_stem != "mod" && file_stem != "lib" && file_stem != "main" {
                components.push(std::path::Component::Normal(OsStr::new(file_stem)));
            }
        }
        std::iter::once("crate".to_string())
            .chain(
                components
                    .into_iter()
                    .filter_map(|c| c.as_os_str().to_str().map(|s| s.to_string())),
            )
            .collect()
    }
}
