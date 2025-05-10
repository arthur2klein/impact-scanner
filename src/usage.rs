use anyhow::{self, bail, Result};
use std::collections::HashMap;
use std::fs;

use crate::{
    language::{parsable_language::ParsableLanguage, Languages},
    symbol::Symbol,
};
use tree_sitter::Node;
use walkdir::WalkDir;

#[derive(Debug)]
/// Usage of a symbol in a project.
///
/// ## Properties:
/// * `file` (`String`): Name of the file the symbol is used in,
/// * `line` (`usize`): Line number where the symbol is used.
pub struct Usage {
    /// Line number where the symbol is named.
    pub line: usize,
    /// Name of the file declaring the symbol.
    pub file: String,
}

#[derive(Clone, Debug)]
/// One import, or equivalent for the current language.
///
/// ## Properties:
/// * `alias` (`Option<String>`): Alias of the imported symbol, if any,
/// * `path` (`Vec<String>`): Scope and original name of the imported symbol,
/// * `is_exported` (`bool`): True iff the field can be re-imported from the current scope.
pub struct Import {
    /// Alias of the imported symbol, if any.
    pub alias: Option<String>,
    /// Scope and original name of the imported symbol.
    pub path: Vec<String>,
    /// True iff the field can be re-imported from the current scope.
    pub is_exported: bool,
}

//generic_type_with_turbofish: $ => seq(
//  field('type', choice(
//    $._type_identifier,
//    $.scoped_identifier,
//  )),
//  '::',
//  field('type_arguments', $.type_arguments),
//),
fn process_generic_type_with_turbofish(
    node: Node,
    path: &str,
    source: &str,
    language: &Languages,
    imports: &mut Vec<Import>,
) -> Result<()> {
    let Some(type_node) = node.child_by_field_name("type") else {
        bail!("field name `type` not found for a generic type with turbofish")
    };
    match type_node.kind() {
        "_type_identifier" => process_identifier(node, source, imports),
        "scoped_identifier" => process_scoped_identifier(node, path, source, language, imports),
        _ => bail!("type node of a generic type with turbofish has invalid kind"),
    }
}

//bracketed_type: $ => seq(
//  '<',
//  choice(
//    $._type,
//    $.qualified_type,
//  ),
//  '>',
//),
//
//Ignored
fn process_bracketed_type(
    _node: Node,
    _path: &str,
    _source: &str,
    _language: &Languages,
    _imports: &mut Vec<Import>,
) -> Result<()> {
    Ok(())
}

//scoped_identifier: $ => seq(
//  field('path', optional(choice(
//    $._path,
//    $.bracketed_type,
//    alias($.generic_type_with_turbofish, $.generic_type),
//  ))),
//  '::',
//  field('name', choice($.identifier, $.super)),
//),
fn process_scoped_identifier(
    node: Node,
    path: &str,
    source: &str,
    language: &Languages,
    imports: &mut Vec<Import>,
) -> Result<()> {
    if let Some(path_node) = node.child_by_field_name("path") {
        match path_node.kind() {
            "_path" => process_path(node, path, source, language, imports)?,
            "bracketed_type" => process_bracketed_type(node, path, source, language, imports)?,
            "generic_type" => {
                process_generic_type_with_turbofish(node, path, source, language, imports)?
            }
            _ => bail!("path node of a scoped identifier has invalid kind"),
        }
    }
    let Some(name_node) = node.child_by_field_name("name") else {
        bail!("field name `name` not found for a scoped identifier")
    };
    match name_node.kind() {
        "identifier" => process_identifier(node, source, imports),
        "super" => process_super(path, language, imports),
        _ => bail!("name node of a scoped identifier has invalid kind"),
    }
}

//crate: _ => 'crate',
fn process_crate(imports: &mut Vec<Import>) -> Result<()> {
    for import in imports {
        import.path.push("crate".to_string());
    }
    Ok(())
}

//super: _ => 'super',
fn process_super(path: &str, language: &Languages, imports: &mut Vec<Import>) -> Result<()> {
    let mut from_path = language.scope_from_path(path);
    from_path.pop();
    for import in imports {
        import.path.extend(from_path.clone());
    }
    Ok(())
}

//self: _ => 'self',
fn process_self(
    _node: Node,
    _path: &str,
    _source: &str,
    _language: &Languages,
    _imports: &mut Vec<Import>,
) -> Result<()> {
    Ok(())
}

//  metavariable: _ => /\$[a-zA-Z_]\w*/,
fn process_metavariable(node: Node, source: &str, imports: &mut Vec<Import>) -> Result<()> {
    let value = node.utf8_text(source.as_bytes()).map(|v| v.to_string())?;
    for import in imports.iter_mut() {
        import.path.push(value.clone());
    }
    Ok(())
}

//  _path: $ => choice(
//    $.self,
//    alias(choice(...primitiveTypes), $.identifier),
//    $.metavariable,
//    $.super,
//    $.crate,
//    $.identifier,
//    $.scoped_identifier,
//    $._reserved_identifier,
//  ),
fn process_path(
    node: Node,
    path: &str,
    source: &str,
    language: &Languages,
    imports: &mut Vec<Import>,
) -> Result<()> {
    match node.kind() {
        "self" => process_self(node, path, source, language, imports),
        "metavariable" => process_metavariable(node, source, imports),
        "super" => process_super(path, language, imports),
        "crate" => process_crate(imports),
        "identifier" => process_identifier(node, source, imports),
        "scoped_identifier" => process_scoped_identifier(node, path, source, language, imports),
        "_reserved_identifier" => process_identifier(node, source, imports),
        _ => bail!("path has invalid kind"),
    }
}

//  identifier: _ => /(r#)?[_\p{XID_Start}][_\p{XID_Continue}]*/,
fn process_identifier(node: Node, source: &str, imports: &mut Vec<Import>) -> Result<()> {
    let value = get_value_of_identifier(node, source)?;
    for import in imports {
        import.path.push(value.clone());
    }
    Ok(())
}

fn get_value_of_identifier(node: Node, source: &str) -> Result<String> {
    node.utf8_text(source.as_bytes())
        .map(|v| v.to_string())
        .or_else(|e| bail!(e))
}

//   use_wildcard: $ => seq(
//     optional(seq(optional($._path), '::')),
//     '*',
//   ),
fn process_use_wildcard(
    node: Node,
    path: &str,
    source: &str,
    language: &Languages,
    imports: &mut Vec<Import>,
) -> Result<()> {
    if node.child_count() > 1 {
        let Some(path_node) = node.child(0) else {
            bail!("can not get the first child of a use wildcard with more than one child")
        };
        process_path(path_node, path, source, language, imports)?;
    }
    for import in imports.iter_mut() {
        import.path.push("*".to_string());
    }
    Ok(())
}

//   use_as_clause: $ => seq(
//     field('path', $._path),
//     'as',
//     field('alias', $.identifier),
//   ),
fn process_use_as_clause(
    node: Node,
    path: &str,
    source: &str,
    language: &Languages,
    imports: &mut Vec<Import>,
) -> Result<()> {
    let Some(alias_node) = node.child_by_field_name("alias") else {
        bail!("field name `alias` not found for a use as clause")
    };
    let alias = get_value_of_identifier(alias_node, source)?;
    for import in imports.iter_mut() {
        import.alias = Some(alias.clone());
    }
    let Some(path_node) = node.child_by_field_name("path") else {
        bail!("field name `path` not found for a use as clause")
    };
    process_path(path_node, path, source, language, imports)?;
    Ok(())
}

//   scoped_use_list: $ => seq(
//     field('path', optional($._path)),
//     '::',
//     field('list', $.use_list),
//   ),
fn process_scoped_use_list(
    node: Node,
    path: &str,
    source: &str,
    language: &Languages,
    imports: &mut Vec<Import>,
) -> Result<()> {
    if let Some(path_node) = node.child_by_field_name("path") {
        process_path(path_node, path, source, language, imports)?;
    }
    let Some(list) = node.child_by_field_name("list") else {
        bail!("field name `list` not found for a scoped use list")
    };
    process_use_list(list, path, source, language, imports)
}

//   use_list: $ => seq(
//     '{',
//     sepBy(',', choice(
//       $._use_clause,
//     )),
//     optional(','),
//     '}',
//   ),
fn process_use_list(
    node: Node,
    path: &str,
    source: &str,
    language: &Languages,
    imports: &mut Vec<Import>,
) -> Result<()> {
    let mut cursor = node.walk();
    let cloned: Vec<Import> = imports.iter().cloned().collect();
    imports.clear();
    for child in node.children(&mut cursor) {
        if child.kind() == "_use_clause" {
            let mut imports_part = cloned.iter().cloned().collect();
            process_use_clause(child, path, source, language, &mut imports_part)?;
            imports.extend(imports_part);
        }
    }
    Ok(())
}

//   _use_clause: $ => choice(
//     $._path,
//     $.use_as_clause,
//     $.use_list,
//     $.scoped_use_list,
//     $.use_wildcard,
//   ),
fn process_use_clause(
    node: Node,
    path: &str,
    source: &str,
    language: &Languages,
    imports: &mut Vec<Import>,
) -> Result<()> {
    match node.kind() {
        "_path" => process_path(node, path, source, language, imports),
        "use_as_clause" => process_use_as_clause(node, path, source, language, imports),
        "use_list" => process_use_list(node, path, source, language, imports),
        "scoped_use_list" => process_scoped_use_list(node, path, source, language, imports),
        "use_wildcard" => process_use_wildcard(node, path, source, language, imports),
        _ => bail!("use clause has invalid kind {:?}", node.kind()),
    }
}

//  use_declaration: $ => seq(
//     optional($.visibility_modifier),
//     'use',
//     field('argument', $._use_clause),
//     ';',
//   ),
fn process_use_declaration(
    node: Node,
    path: &str,
    source: &str,
    language: &Languages,
) -> Result<Vec<Import>> {
    let is_exported = node
        .child(0)
        .map(|arg| arg.kind() == "visibility_modifier")
        .unwrap_or(false);
    let Some(argument) = node.child_by_field_name("argument") else {
        bail!("field name `argument` not found for a use declaration")
    };
    let mut imports = vec![Import {
        alias: None,
        path: Vec::new(),
        is_exported,
    }];
    process_use_clause(argument, path, source, language, &mut imports)?;
    Ok(imports)
}

pub fn extract_use_map(
    node: Node,
    path: &str,
    file: &str,
    source: &str,
    use_map: &mut HashMap<String, Vec<String>>,
    language: &Languages,
) {
    if node.kind() == "use_declaration" {
        let imports = process_use_declaration(node, path, source, language);
        eprintln!("DEBUGPRINT[60]: usage.rs:353: imports={:#?}", imports);
    }
    for child in node.named_children(&mut node.walk()) {
        extract_use_map(child, path, file, source, use_map, language);
    }
}

pub fn find_symbol_usages(project_root: &str, symbol: &Symbol, language: &Languages) -> Vec<Usage> {
    let mut usages: Vec<Usage> = vec![];

    for entry in WalkDir::new(project_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.path().extension().map(|ext| ext == "rs").unwrap_or(false))
    {
        let path = entry.path();
        let source_code = match fs::read_to_string(path) {
            Ok(code) => code,
            Err(_) => continue,
        };
        let tree = match language.parse(&source_code).ok() {
            Some(tree) => tree,
            None => continue,
        };
        let root_node = tree.root_node();

        let path_string = path.to_str().unwrap_or_default();
        eprintln!(
            "DEBUGPRINT[46]: usage.rs:128: path_string={:#?}",
            path_string
        );
        let mut use_map = HashMap::new();
        extract_use_map(
            root_node,
            &path_string,
            path_string,
            &source_code,
            &mut use_map,
            &language,
        );
        eprintln!("DEBUGPRINT[45]: usage.rs:192: use_map={:#?}", use_map);

        let mut cursor = root_node.walk();
        let mut stack = vec![root_node];

        while let Some(node) = stack.pop() {
            match node.kind() {
                "call_expression" => {
                    if let Some(fn_node) = node.child_by_field_name("function") {
                        let name = &source_code[fn_node.start_byte()..fn_node.end_byte()];
                        if name == symbol.name {
                            if let Some(use_path) = use_map.get(name) {
                                if use_path.contains(&symbol.name) {
                                    usages.push(Usage {
                                        file: use_path.join("::"),
                                        line: node.start_position().row + 1,
                                    });
                                }
                            } else if symbol.file.ends_with(
                                path.file_name().unwrap_or_default().to_str().unwrap_or(""),
                            ) {
                                usages.push(Usage {
                                    file: path.to_string_lossy().into_owned(),
                                    line: node.start_position().row + 1,
                                });
                            }
                        }
                    }
                }
                "use_declaration" => {
                    let path_str = path.to_string_lossy().into_owned();
                    if fs::canonicalize(&symbol.file).unwrap_or_default()
                        != fs::canonicalize(&path).unwrap_or_default()
                    {
                        let text = &source_code[node.start_byte()..node.end_byte()];
                        if text.contains(&symbol.name) {
                            usages.push(Usage {
                                file: path_str,
                                line: node.start_position().row + 1,
                            });
                        }
                    }
                }
                _ => {}
            }
            for child in node.children(&mut cursor) {
                stack.push(child);
            }
        }
    }
    usages
}
