use anyhow::{Context, Result};
use std::fs;

use crate::object;

pub fn run(file: &str) -> Result<()> {
    let data = fs::read(file).with_context(|| format!("failed to read file: {}", file))?;

    let hash = object::write_object("blob", &data)?;

    println!("{}", hash);
    Ok(())
}
