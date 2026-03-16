use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use crate::ignore::{normalize_path, IgnoreMatcher};
use crate::index::{read_index, write_index};
use crate::object;

pub fn run(path: &str) -> Result<()> {
    let matcher = IgnoreMatcher::from_repo_root()?;
    let files = collect_files(path, &matcher)?;

    if files.is_empty() {
        bail!("no files to add");
    }

    let mut index = read_index(".tronit/index")?;

    for file in files {
        let normalized = normalize_path(&file);
        let data = std::fs::read(&file)
            .with_context(|| format!("failed to read file '{}'", file.display()))?;
        let hash = object::write_object("blob", &data)?;
        index.insert(normalized.clone(), hash.clone());
        println!("added {} {}", hash, normalized);
    }

    write_index(".tronit/index", &index)?;
    Ok(())
}

fn collect_files(path: &str, matcher: &IgnoreMatcher) -> Result<Vec<PathBuf>> {
    let input = Path::new(path);

    if !input.exists() {
        bail!("path does not exist: {}", path);
    }

    let mut files = Vec::new();

    if input.is_file() {
        if !matcher.is_ignored(input) {
            files.push(input.to_path_buf());
        }
    } else {
        let iter = WalkDir::new(input)
            .into_iter()
            .filter_entry(|entry| !matcher.is_ignored(entry.path()));

        for entry in iter {
            let entry = entry.with_context(|| format!("walkdir failed under {}", path))?;
            if entry.file_type().is_file() && !matcher.is_ignored(entry.path()) {
                files.push(entry.path().to_path_buf());
            }
        }
    }

    files.sort_by_key(|p| normalize_path(p));
    Ok(files)
}
