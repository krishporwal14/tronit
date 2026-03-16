use anyhow::{bail, Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const TRONIT_DIR: &str = ".tronit";
const HEAD_PATH: &str = ".tronit/HEAD";
const HEADS_DIR: &str = ".tronit/refs/heads";


pub fn init_repo() -> Result<()> {
    if Path::new(TRONIT_DIR).exists() {
        bail!("repository already exists at {}", TRONIT_DIR);
    }

    fs::create_dir_all(".tronit/objects").context("failed to create .tronit/objects")?;
    fs::create_dir_all(HEADS_DIR).context("failed to create .tronit/refs/heads")?;
    fs::write(HEAD_PATH, "ref: refs/heads/main\n").context("failed to write .tronit/HEAD")?;

    println!("Initialized empty Tronit repository.");
    Ok(())
}

pub fn ensure_repo() -> Result<()> {
    if !Path::new(TRONIT_DIR).exists() {
        bail!("not a tronit repository (missing .tronit directory)");
    }
    Ok(())
}

pub fn head_ref_path() -> Result<String> {
    ensure_repo()?;
    let head = fs::read_to_string(HEAD_PATH).context("failed to read .tronit/HEAD")?;
    let head = head.trim();

    let rel = head
        .strip_prefix("ref: ")
        .context("detached or malformed HEAD, expected symbolic ref")?;

    Ok(rel.to_string())
}

pub fn head_branch_name() -> Result<String> {
    let rel = head_ref_path()?;
    let name = rel
        .strip_prefix("refs/heads/")
        .context("HEAD does not point to refs/heads/*")?;
    Ok(name.to_string())
}

pub fn resolve_head_commit() -> Result<Option<String>> {
    let rel = head_ref_path()?;
    let full = ref_full_path(&rel);

    if !full.exists() {
        return Ok(None);
    }

    let value = fs::read_to_string(&full)
        .with_context(|| format!("failed to read {}", full.display()))?;
    let value = value.trim().to_string();

    if value.is_empty() {
        Ok(None)
    } else {
        Ok(Some(value))
    }
}

pub fn update_head_commit(hash: &str) -> Result<()> {
    let rel = head_ref_path()?;
    let full = ref_full_path(&rel);

    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    }

    fs::write(&full, format!("{}\n", hash))
        .with_context(|| format!("failed to write {}", full.display()))?;

    Ok(())
}

pub fn create_branch(name: &str, start_hash: &str) -> Result<()> {
    validate_branch_name(name)?;
    ensure_repo()?;

    let full = PathBuf::from(format!("{}/{}", HEADS_DIR, name));
    if full.exists() {
        bail!("branch already exists: {}", name);
    }

    if let Some(parent) = full.parent() {
        fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    }

    fs::write(&full, format!("{}\n", start_hash))
        .with_context(|| format!("failed to write {}", full.display()))?;

    Ok(())
}

pub fn switch_branch(name: &str) -> Result<()> {
    validate_branch_name(name)?;
    ensure_repo()?;

    let full = PathBuf::from(format!("{}/{}", HEADS_DIR, name));
    if !full.exists() {
        bail!("branch does not exist: {}", name);
    }

    fs::write(HEAD_PATH, format!("ref: refs/heads/{}\n", name))
        .context("failed to update .tronit/HEAD")?;

    Ok(())
}

pub fn list_branches() -> Result<Vec<(String, bool)>> {
    ensure_repo()?;
    let current = head_branch_name()?;

    let mut out = Vec::new();

    if Path::new(HEADS_DIR).exists() {
        for entry in WalkDir::new(HEADS_DIR).min_depth(1) {
            let entry = entry.context("failed while scanning branches")?;
            if entry.file_type().is_file() {
                let rel = entry
                    .path()
                    .strip_prefix(HEADS_DIR)
                    .context("failed to strip branch directory prefix")?
                    .to_string_lossy()
                    .replace('\\', "/")
                    .trim_start_matches('/')
                    .to_string();
                let is_current = rel == current;
                out.push((rel, is_current));
            }
        }
    }

    out.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(out)
}

fn ref_full_path(rel: &str) -> PathBuf {
    PathBuf::from(format!(".tronit/{}", rel))
}

fn validate_branch_name(name: &str) -> Result<()> {
    if name.trim().is_empty() {
        bail!("branch name cannot be empty");
    }
    if name.contains(' ') {
        bail!("branch name cannot contain spaces");
    }
    if name.contains("..") {
        bail!("branch name cannot contain '..'");
    }
    if name.starts_with('/') || name.ends_with('/') {
        bail!("branch name cannot start or end with '/'");
    }
    Ok(())
}
