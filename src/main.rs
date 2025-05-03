use std::collections::HashSet;

use anyhow::Result;
use clap::Parser;

mod git;
mod impact;
mod parser;
mod symbol;

#[derive(Parser, Debug)]
#[command(name = "impact-scanner")]
#[command(about = "Analyze what code is affected by your staged changes", long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let _args = Args::parse();
    let changed_map = git::get_changed_lines()?;

    for file in changed_map.keys() {
        let source = std::fs::read_to_string(&file)?;
        let tree = parser::parse_rust(&source)?;

        let changed_lines: HashSet<usize> = changed_map[file].iter().copied().collect();
        let changed_symbols = symbol::extract_changed_symbols(&tree, &source, &changed_lines)?;

        println!("✏️  Changed symbols in {file:?}: {:?}", changed_symbols);
    }

    Ok(())
}
