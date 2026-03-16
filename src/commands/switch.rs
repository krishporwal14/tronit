use anyhow::Result;
use crate::repo;

pub fn run(name: &str) -> Result<()> {
    repo::switch_branch(name)?;
    println!("switched to branch {}", name);
    println!("note: working tree checkout is not implemented yet");
    Ok(())
}
