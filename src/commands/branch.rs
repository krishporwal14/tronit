use anyhow::{bail, Result};
use crate::repo;

pub fn run(name: Option<&str>) -> Result<()> {
    repo::ensure_repo()?;

    if let Some(branch_name) = name {
        let Some(start) = repo::resolve_head_commit()? else {
            bail!("cannot create branch without any commits");
        };
        repo::create_branch(branch_name, &start)?;
        println!("created branch {}", branch_name);
        return Ok(());
    }

    let branches = repo::list_branches()?;
    if branches.is_empty() {
        println!("no branches");
        return Ok(());
    }

    for (name, is_current) in branches {
        if is_current {
            println!("* {}", name);
        } else {
            println!("  {}", name);
        }
    }

    Ok(())
}
