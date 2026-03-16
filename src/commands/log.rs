use anyhow::{bail, Context, Result};
use std::fs;

use crate::object;

pub fn run() -> Result<()> {
    let mut current = fs::read_to_string(".tronit/refs/heads/main")
        .context("failed to read .tronit/refs/heads/main")?
        .trim()
        .to_string();

    if current.is_empty() {
        bail!("main branch has no commits");
    }

    loop {
        let obj = object::read_object_typed(&current)?;
        if obj.obj_type != "commit" {
            bail!("object {} is {}, expected commit", current, obj.obj_type);
        }

        let text = String::from_utf8(obj.data).context("commit payload is not valid utf-8")?;
        println!("commit {}\n{}\n", current, text);

        let parent = text
            .lines()
            .find_map(|line| line.strip_prefix("parent ").map(|s| s.trim().to_string()));

        match parent {
            Some(p) if !p.is_empty() => current = p,
            _ => break,
        }
    }

    Ok(())
}
