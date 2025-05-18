pub use std::collections::HashSet;
use std::path::PathBuf;

use crate::language::parsable_language::ParsableLanguage;
use anyhow::Result;
use clap::Parser;
use language::get_language_for_file;

mod git;
mod language;
mod symbol;
mod symbol_kind;
mod usage;

#[derive(Parser, Debug)]
#[command(name = "impact-scanner")]
#[command(about = "Analyze what code is affected by your staged changes", long_about = None)]
/// Arguments received by the main command.
///
/// ## Arguments:
/// - `debug` (`bool`): true to display more info, defaults to false,
/// - `path` (`String`): Path to the project to analyze, defaults to current directory.
struct Args {
    #[arg(short, long)]
    /// Display more information.
    debug: bool,
    #[arg(short, long, default_value_t = String::from("."))]
    /// Path to the project to analyze.
    path: String,
    #[arg(short, long)]
    /// Show usage of symbols
    usage: bool,
}

/// Get changed symbols in the given file.
///
/// ## Parameters:
/// * `file` (`&std::path::PathBuf`): Name of the file,
/// * `language` (`&language::Languages`): Language of the file,
/// * `changed_lines` (`&Vec<usize>`): List of lines with staged changes in the file,
/// * `debug` (`bool`): true iff more information should be displayed.
///
/// ## Returns:
/// * (`Result<Vec<symbol::Symbol>>`): List of symbols that changed in the given file.
fn symbols_from_changes(
    file: &PathBuf,
    language: &language::Languages,
    changed_lines: &Vec<usize>,
    debug: bool,
) -> Result<Vec<symbol::Symbol>> {
    if debug {
        println!("Processing {:?}", file);
        println!("Language is {:?}", language);
    }
    let source = std::fs::read_to_string(&file)?;
    let tree = language.parse(&source)?;

    let changed_lines: HashSet<usize> = changed_lines.iter().copied().collect();
    symbol::extract_changed_symbols(&tree, file, &source, &changed_lines, &language)
}

/// Runs the main impact-scanner command with the arguments from `Args`.
/// - Get staged changes,
/// - Deduce changed symbols,
/// - Display them.
///
/// ## Returns:
/// - (`Result<()>`): Ok if no critical error, else description of the error.
fn main() -> Result<()> {
    let args = Args::parse();
    let changed_map = git::get_changed_lines(&PathBuf::from(&args.path))?;
    if args.debug {
        println!("Changed lines: {:?}", changed_map);
    }

    for file in changed_map.keys() {
        let language: language::Languages = get_language_for_file(file);
        match symbols_from_changes(file, &language, &changed_map[file], args.debug) {
            Ok(changed_symbols) => {
                println!("✏️ Changed symbols in {file:?}:");
                for symbol in changed_symbols {
                    println!("   - {symbol},");
                    if args.usage {
                        let usage = usage::find_symbol_usages(
                            &PathBuf::from(&args.path),
                            &symbol,
                            &language,
                        );
                        eprintln!("DEBUGPRINT[30]: main.rs:79: usage={:#?}", usage);
                    }
                }
            }
            Err(error) => println!("❌ File {file:?} gives error {error:?}"),
        }
    }

    Ok(())
}
