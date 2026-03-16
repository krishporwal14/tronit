use anyhow::{Context, Result};
use crate::object;

pub fn run(hash: &str) -> Result<()> {
    let obj = object::read_object_typed(hash)?;

    match obj.obj_type.as_str() {
        "blob" | "commit" => {
            println!("{}", String::from_utf8_lossy(&obj.data));
        }
        "tree" => {
            let entries = object::parse_tree(&obj.data)?;
            for entry in entries {
                let kind = if entry.mode == "40000" { "tree" } else { "blob" };
                println!("{} {} {}\t{}", entry.mode, kind, entry.hash, entry.name);
            }
        }
        other => {
            let text = String::from_utf8(obj.data.clone())
                .with_context(|| format!("object type {} is not valid utf-8", other))?;
            println!("{}", text);
        }
    }

    Ok(())
}
