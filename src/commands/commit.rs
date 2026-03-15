use std::fs;
use chrono::Utc;

use crate::object;

pub fn run(message: &str) {
    let index = fs::read_to_string(".tronit/index").unwrap();

    let tree_hash = object::write_object("tree", index.as_bytes());

    let parent = fs::read_to_string(".tronit/refs/heads/main").unwrap_or_default();

    let timestamp = Utc::now().timestamp();

    let commit_data = format!(
        "tree {}\nparent {}\nauthor tronit {}\n\n{}\n",
        tree_hash,
        parent.trim(),
        timestamp,
        message
    );

    let commit_hash = object::write_object("commit", commit_data.as_bytes());
    let hash_copy = commit_hash.clone();
    
    fs::write(".tronit/refs/heads/main", commit_hash).unwrap();

    println!("commit {}", hash_copy);
}
