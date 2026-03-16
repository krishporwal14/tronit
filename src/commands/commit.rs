use anyhow::{bail, Result, Context};
use chrono::Local;
use std::collections::BTreeMap;
use std::env;
use crate::index::read_index;
use crate::object::{self, TreeEntry};
use crate::repo;

pub fn run(message: &str, author_name: Option<&str>, author_email: Option<&str>) -> Result<()> {
    repo::ensure_repo()?;

    let index = read_index(".tronit/index")?;
    if index.is_empty() {
        bail!("nothing to commit");
    }

    let tree_hash = build_root_tree(&index)?;
    let parent = repo::resolve_head_commit()?;

    let (name, email) = resolve_author(author_name, author_email)?;

    let now = Local::now();
    let timestamp = now.timestamp();
    let timezone = now.format("%z").to_string();
    let ident = format!("{} <{}>", name, email);

    let mut commit_data = String::new();
    commit_data.push_str(&format!("tree {}\n", tree_hash));
    if let Some(parent_hash) = parent {
        if !parent_hash.is_empty() {
            commit_data.push_str(&format!("parent {}\n", parent_hash));
        }
    }
    commit_data.push_str(&format!("author {} {} {}\n", ident, timestamp, timezone));
    commit_data.push_str(&format!("committer {} {} {}\n\n", ident, timestamp, timezone));
    commit_data.push_str(message);
    commit_data.push('\n');

    let commit_hash = object::write_object("commit", commit_data.as_bytes())?;
    repo::update_head_commit(&commit_hash)?;

    println!("commit {}", commit_hash);
    Ok(())
}

#[derive(Default)]
struct DirNode {
    files: BTreeMap<String, String>,
    dirs: BTreeMap<String, DirNode>,
}

fn build_root_tree(index: &BTreeMap<String, String>) -> Result<String> {
    let mut root = DirNode::default();

    for (path, hash) in index {
        insert_path(&mut root, path, hash)?;
    }

    write_dir(&root)
}

fn insert_path(root: &mut DirNode, path: &str, hash: &str) -> Result<()> {
    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    if parts.is_empty() {
        bail!("invalid empty index path");
    }

    insert_parts(root, &parts, hash);
    Ok(())
}

fn insert_parts(node: &mut DirNode, parts: &[&str], hash: &str) {
    if parts.len() == 1 {
        node.files.insert(parts[0].to_string(), hash.to_string());
        return;
    }

    let child = node.dirs.entry(parts[0].to_string()).or_default();
    insert_parts(child, &parts[1..], hash);
}

fn write_dir(node: &DirNode) -> Result<String> {
    let mut entries: Vec<TreeEntry> = Vec::new();

    for (name, hash) in &node.files {
        entries.push(TreeEntry {
            mode: "100644".to_string(),
            name: name.clone(),
            hash: hash.clone(),
        });
    }

    for (name, child) in &node.dirs {
        let child_hash = write_dir(child)?;
        entries.push(TreeEntry {
            mode: "40000".to_string(),
            name: name.clone(),
            hash: child_hash,
        });
    }

    object::write_tree(&entries)
}

fn resolve_author(cli_name: Option<&str>, cli_email: Option<&str>) -> Result<(String, String)> {
    let name = cli_name
        .map(ToOwned::to_owned)
        .or_else(|| env::var("TRONIT_AUTHOR_NAME").ok())
        .filter(|s| !s.trim().is_empty())
        .context("missing author name: pass --author-name or set TRONIT_AUTHOR_NAME")?;

    let email = cli_email
        .map(ToOwned::to_owned)
        .or_else(|| env::var("TRONIT_AUTHOR_EMAIL").ok())
        .filter(|s| !s.trim().is_empty())
        .context("missing author email: pass --author-email or set TRONIT_AUTHOR_EMAIL")?;

    Ok((name, email))
}
