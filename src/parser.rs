use anyhow::Result;
use tree_sitter::{Parser, Tree};
use tree_sitter_rust::LANGUAGE as rust_language;

pub fn parse_rust(source: &str) -> Result<Tree> {
    let mut parser = Parser::new();
    parser.set_language(&rust_language.into())?;
    let tree = parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Parse failed"))?;
    Ok(tree)
}
