use anyhow::Result;
use git2::{DiffOptions, Repository};
use std::collections::HashMap;

pub fn get_changed_lines() -> Result<HashMap<String, Vec<usize>>> {
    let repo = Repository::open(".")?;
    let index = repo.index()?;
    let head = repo.head()?.peel_to_tree()?;
    let diff = repo.diff_tree_to_index(Some(&head), Some(&index), Some(&mut DiffOptions::new()))?;

    let mut result = HashMap::new();

    diff.foreach(
        &mut |_delta, _progress| true,
        None,
        Some(&mut |delta, hunk| {
            let path = delta
                .new_file()
                .path()
                .and_then(|p| p.to_str())
                .unwrap_or("<unknown>")
                .to_string();

            let start = hunk.new_start() as usize;
            let count = hunk.new_lines() as usize;
            let lines: &mut Vec<usize> = result.entry(path).or_default();
            lines.extend(start..start + count);
            true
        }),
        None,
    )?;
    Ok(result)
}
