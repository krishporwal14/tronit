use anyhow::{bail, Context, Result};
use std::fs;
use std::path::Path;

pub fn init_repo() -> Result<()> {
    if Path::new(".tronit").exists() {
        bail!("repository already exists at .tronit");
    }

    fs::create_dir_all(".tronit/objects").context("failed to create .tronit/objects")?;
    fs::create_dir_all(".tronit/refs/heads").context("failed to create .tronit/refs/heads")?;

    fs::write(".tronit/HEAD", "ref: refs/heads/main\n").context("failed to write .tronit/HEAD")?;

    println!("Initialized empty Tronit repository.");
    Ok(())
}
