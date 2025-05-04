use std::collections::HashSet;

use crate::language::language::Language;
use anyhow::Result;
use clap::Parser;
use language::get_language_for_file;

mod git;
mod impact;
mod language;
mod symbol;
mod symbol_kind;

#[derive(Parser, Debug)]
#[command(name = "impact-scanner")]
#[command(about = "Analyze what code is affected by your staged changes", long_about = None)]
struct Args {
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let changed_map = git::get_changed_lines()?;
    if args.debug {
        println!("Changed lines: {:?}", changed_map);
    }

    for file in changed_map.keys() {
        let language: language::Languages = get_language_for_file(file);
        if args.debug {
            println!("Processing {:?}", file);
            println!("Language is {:?}", language);
        }
        let source = std::fs::read_to_string(&file)?;
        let tree = language.parse(&source)?;

        let changed_lines: HashSet<usize> = changed_map[file].iter().copied().collect();
        let changed_symbols =
            symbol::extract_changed_symbols(&tree, file, &source, &changed_lines, &language)?;

        println!("✏️ Changed symbols in {file:?}:");
        for symbol in changed_symbols {
            println!("   - {symbol},");
        }
    }

    Ok(())
}
