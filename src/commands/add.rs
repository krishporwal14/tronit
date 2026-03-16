use anyhow::{bail, Result, Context};
use std::fs;
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};
use walkdir::WalkDir;

use crate::object;

pub fn run(path: &str) -> Result<()> {
    let files = collect_files(path)?;
    if files.is_empty() {
        bail!("no files to add");
    }

    let mut index = read_index(".tronit/index")?;

    for file in files {
        let normalized = normalize_path(&file);
        let data = fs::read(&file)
            .with_context(|| format!("failed to read file '{}'", file.display()))?;
        let hash = object::write_object("blob", &data)?;
        index.insert(normalized.clone(), hash.clone());
        println!("added {} {}", hash, normalized);
    }

    write_index(".tronit/index", &index)?;
    Ok(())
}

fn collect_files(path: &str) -> Result<Vec<PathBuf>> {
    let input = Path::new(path);

    if !input.exists() {
        bail!("path does not exist: {}", path);
    }

    let mut files = Vec::new();

    if input.is_file() {
        if !is_inside_tronit(input) {
            files.push(input.to_path_buf());
        }
    } else {
        for entry in WalkDir::new(input) {
            let entry = entry.with_context(|| format!("walkdir failed under {}", path))?;
            let p = entry.path();

            if entry.file_type().is_file() && !is_inside_tronit(p) {
                files.push(p.to_path_buf());
            }
        }
    }

    files.sort_by_key(|p| normalize_path(p));
    Ok(files)
}

fn is_inside_tronit(path: &Path) -> bool {
    path.components().any(|c| match c {
        Component::Normal(seg) => seg.to_string_lossy().eq_ignore_ascii_case(".tronit"),
        _ => false,
    })
}

fn normalize_path(path: &Path) -> String {
    let mut s = path.to_string_lossy().replace('\\', "/");
    while s.starts_with("./") {
        s = s[2..].to_string();
    }
    s
}

fn read_index(index_path: &str) -> Result<BTreeMap<String, String>> {
    let mut map = BTreeMap::new();

    if !Path::new(index_path).exists() {
        return Ok(map);
    }

    let text =
        fs::read_to_string(index_path).with_context(|| format!("failed to read {}", index_path))?;

    for (line_no, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let (hash, path) = line
            .split_once(' ')
            .with_context(|| format!("malformed index line {}: {}", line_no + 1, line))?;

        if hash.len() != 40 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("invalid hash at index line {}: {}", line_no + 1, hash);
        }

        map.insert(path.to_string(), hash.to_string());
    }

    Ok(map)
}

fn write_index(index_path: &str, map: &BTreeMap<String, String>) -> Result<()> {
    let mut out = String::new();
    for (path, hash) in map {
        out.push_str(hash);
        out.push(' ');
        out.push_str(path);
        out.push('\n');
    }

    fs::write(index_path, out).with_context(|| format!("failed to write {}", index_path))?;
    Ok(())
}
