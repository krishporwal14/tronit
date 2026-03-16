use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub fn read_index(index_path: &str) -> Result<BTreeMap<String, String>> {
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

pub fn write_index(index_path: &str, map: &BTreeMap<String, String>) -> Result<()> {
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
