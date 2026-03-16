use anyhow::{bail, Result};
use crate::object;
use crate::repo;

pub fn run() -> Result<()> {
    let mut current = repo::resolve_head_commit()?;
    if current.is_none() {
        bail!("current branch has no commits");
    }

    while let Some(hash) = current {
        let obj = object::read_object_typed(&hash)?;
        if obj.obj_type != "commit" {
            bail!("object {} is {}, expected commit", hash, obj.obj_type);
        }

        let text = String::from_utf8(obj.data).map_err(anyhow::Error::from)?;
        println!("commit {}\n{}\n", hash, text);

        let meta = object::parse_commit(text.as_bytes())?;
        current = meta.parent;
    }

    Ok(())
}
