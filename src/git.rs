use git2::{DiffDelta, DiffHunk, DiffLine, DiffOptions, Repository};
use std::collections::HashMap;

/// Returns the lines that changed in a git repository.
///
/// ## Parameters:
/// * `path` (`&str`): Path to the git repository.
///
/// ## Returns:
/// * (`anyhow::Result<std::collections::HashMap<String, Vec<usize>>>`): Map associating file names
/// to a list of changed lines in git repository. Line numbers are lines in the staged version of
/// the repo.
pub fn get_changed_lines(path: &str) -> anyhow::Result<HashMap<String, Vec<usize>>> {
    let repo = Repository::open(path)?;
    let index = repo.index()?;
    let head = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head), Some(&index), Some(&mut DiffOptions::new()))?;
    let mut result: HashMap<String, Vec<usize>> = HashMap::new();
    diff.foreach(
        &mut |_delta, _progress| true,
        None,
        None,
        Some(
            &mut |delta: DiffDelta, _hunk: Option<DiffHunk>, line: DiffLine| {
                if line.origin() == '+' {
                    if let Some(path) = delta.new_file().path().and_then(|p| p.to_str()) {
                        let line_num = line.new_lineno().unwrap_or(0) as usize;
                        if line_num > 0 {
                            result.entry(path.to_string()).or_default().push(line_num);
                        }
                    }
                }
                true
            },
        ),
    )?;

    Ok(result)
}
