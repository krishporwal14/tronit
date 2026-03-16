use anyhow::{Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use walkdir::WalkDir;
use crate::ignore::{normalize_path, IgnoreMatcher};
use crate::index::read_index;
use crate::object;
use crate::repo;

#[derive(Debug, Default)]
pub struct StatusReport {
    pub branch: String,
    pub staged: Vec<String>,
    pub unstaged: Vec<String>,
    pub untracked: Vec<String>,
}

pub fn run() -> Result<()> {
    let report = collect_status()?;

    println!("On branch {}", report.branch);
    println!();

    if report.staged.is_empty() {
        println!("No changes staged for commit");
    } else {
        println!("Changes to be committed:");
        for line in &report.staged {
            println!("  {}", line);
        }
    }

    println!();

    if report.unstaged.is_empty() {
        println!("No changes not staged for commit");
    } else {
        println!("Changes not staged for commit:");
        for line in &report.unstaged {
            println!("  {}", line);
        }
    }

    println!();

    if report.untracked.is_empty() {
        println!("No untracked files");
    } else {
        println!("Untracked files:");
        for line in &report.untracked {
            println!("  {}", line);
        }
    }

    Ok(())
}

pub fn collect_status() -> Result<StatusReport> {
    repo::ensure_repo()?;

    let branch = repo::head_branch_name()?;
    let index_map = read_index(".tronit/index")?;
    let head_map = load_head_tree_map()?;

    let staged = compute_staged_changes(&head_map, &index_map);
    let unstaged = compute_unstaged_changes(&index_map)?;
    let untracked = compute_untracked_files(&index_map)?;

    Ok(StatusReport {
        branch,
        staged,
        unstaged,
        untracked,
    })
}

fn load_head_tree_map() -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();

    let head = repo::resolve_head_commit()?;
    let Some(commit_hash) = head else {
        return Ok(out);
    };

    let obj = object::read_object_typed(&commit_hash)?;
    if obj.obj_type != "commit" {
        return Ok(out);
    }

    let meta = object::parse_commit(&obj.data)?;
    flatten_tree(&meta.tree, "", &mut out)?;

    Ok(out)
}

fn flatten_tree(tree_hash: &str, prefix: &str, out: &mut BTreeMap<String, String>) -> Result<()> {
    let obj = object::read_object_typed(tree_hash)?;
    if obj.obj_type != "tree" {
        return Ok(());
    }

    let entries = object::parse_tree(&obj.data)?;
    for e in entries {
        let path = if prefix.is_empty() {
            e.name.clone()
        } else {
            format!("{}/{}", prefix, e.name)
        };

        if e.mode == "40000" || e.mode == "040000" {
            flatten_tree(&e.hash, &path, out)?;
        } else {
            out.insert(path, e.hash);
        }
    }

    Ok(())
}

fn compute_staged_changes(
    head_map: &BTreeMap<String, String>,
    index_map: &BTreeMap<String, String>,
) -> Vec<String> {
    let mut out = Vec::new();

    let mut paths = BTreeSet::new();
    for p in head_map.keys() {
        paths.insert(p.clone());
    }
    for p in index_map.keys() {
        paths.insert(p.clone());
    }

    for p in paths {
        match (head_map.get(&p), index_map.get(&p)) {
            (None, Some(_)) => out.push(format!("new file: {}", p)),
            (Some(_), None) => out.push(format!("deleted: {}", p)),
            (Some(h), Some(i)) if h != i => out.push(format!("modified: {}", p)),
            _ => {}
        }
    }

    out
}

fn compute_unstaged_changes(index_map: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let mut out = Vec::new();

    for (path, staged_hash) in index_map {
        let p = Path::new(path);
        if !p.exists() {
            out.push(format!("deleted: {}", path));
            continue;
        }
        if !p.is_file() {
            continue;
        }

        let data = std::fs::read(p)
            .with_context(|| format!("failed to read tracked file {}", path))?;
        let work_hash = object::compute_object_hash("blob", &data);

        if &work_hash != staged_hash {
            out.push(format!("modified: {}", path));
        }
    }

    Ok(out)
}

fn compute_untracked_files(index_map: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let matcher = IgnoreMatcher::from_repo_root()?;
    let mut out = Vec::new();

    let iter = WalkDir::new(".")
        .into_iter()
        .filter_entry(|entry| !matcher.is_ignored(entry.path()));

    for entry in iter {
        let entry = entry.context("walkdir failed while collecting untracked files")?;
        if entry.file_type().is_file() && !matcher.is_ignored(entry.path()) {
            let p = normalize_path(entry.path());
            if !index_map.contains_key(&p) {
                out.push(p);
            }
        }
    }

    out.sort();
    Ok(out)
}
