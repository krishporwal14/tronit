use anyhow::{bail, Result, Context};
use std::fs;
use crate::utils::*;

#[derive(Debug, Clone)]
pub struct TronitObject {
    pub obj_type: String,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct CommitMeta {
    pub tree: String,
    pub parent: Option<String>,
    pub author: Option<String>,
    pub committer: Option<String>,
    pub message: String,
}

pub fn compute_object_hash(obj_type: &str, data: &[u8]) -> String {
    let header = format!("{} {}\0", obj_type, data.len());
    let mut store = header.into_bytes();
    store.extend(data);
    sha1_hash(&store)
}

pub fn write_object(obj_type: &str, data: &[u8]) -> Result<String> {
    let header = format!("{} {}\0", obj_type, data.len());

    let mut store = header.into_bytes();
    store.extend(data);

    let hash = sha1_hash(&store);
    let compressed = compress(&store)?;

    let dir = &hash[0..2];
    let file = &hash[2..];

    let object_dir = format!(".tronit/objects/{}", dir);
    let path = format!("{}/{}", object_dir, file);

    fs::create_dir_all(&object_dir).with_context(|| format!("failed to create {}", object_dir))?;
    fs::write(&path, compressed).with_context(|| format!("failed to write object {}", path))?;

    Ok(hash)
}

pub fn read_object(hash: &str) -> Result<Vec<u8>> {
    validate_hash(hash)?;
    let path = object_path(hash);

    let data = fs::read(&path).with_context(|| format!("failed to read object {}", path))?;
    decompress(&data)
}

pub fn read_object_typed(hash: &str) -> Result<TronitObject> {
    let raw = read_object(hash)?;

    let nul_pos = raw
        .iter()
        .position(|b| *b == 0)
        .context("object missing header separator")?;

    let header = std::str::from_utf8(&raw[..nul_pos]).context("invalid object header")?;
    let mut parts = header.splitn(2, ' ');

    let obj_type = parts
        .next()
        .filter(|s| !s.is_empty())
        .context("object header missing type")?
        .to_string();

    let size_str = parts.next().context("object header missing size")?;
    let expected_size: usize = size_str
        .parse()
        .with_context(|| format!("invalid object size in header: {}", size_str))?;

    let payload = raw[nul_pos + 1..].to_vec();
    if payload.len() != expected_size {
        bail!(
            "object size mismatch: header says {}, payload is {}",
            expected_size,
            payload.len()
        );
    }

    Ok(TronitObject {
        obj_type,
        data: payload,
    })
}

pub fn write_tree(entries: &[TreeEntry]) -> Result<String> {
    let mut sorted = entries.to_vec();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));

    let mut out = Vec::new();
    for entry in sorted {
        if entry.name.contains('/') {
            bail!("tree entry name cannot contain '/': {}", entry.name);
        }

        out.extend_from_slice(entry.mode.as_bytes());
        out.push(b' ');
        out.extend_from_slice(entry.name.as_bytes());
        out.push(0);

        let raw_hash = hex::decode(&entry.hash)
            .with_context(|| format!("invalid hash in tree entry: {}", entry.hash))?;
        if raw_hash.len() != 20 {
            bail!(
                "tree entry hash must decode to 20 bytes, got {} for {}",
                raw_hash.len(),
                entry.hash
            );
        }

        out.extend_from_slice(&raw_hash);
    }

    write_object("tree", &out)
}

pub fn parse_tree(data: &[u8]) -> Result<Vec<TreeEntry>> {
    let mut i = 0usize;
    let mut entries = Vec::new();

    while i < data.len() {
        let mode_start = i;
        while i < data.len() && data[i] != b' ' {
            i += 1;
        }
        if i >= data.len() {
            bail!("malformed tree: missing mode terminator space");
        }
        let mode = std::str::from_utf8(&data[mode_start..i])
            .context("tree mode is not valid UTF-8")?
            .to_string();
        i += 1;

        let name_start = i;
        while i < data.len() && data[i] != 0 {
            i += 1;
        }
        if i >= data.len() {
            bail!("malformed tree: missing filename NUL terminator");
        }
        let name = std::str::from_utf8(&data[name_start..i])
            .context("tree filename is not valid UTF-8")?
            .to_string();
        i += 1;

        if i + 20 > data.len() {
            bail!("malformed tree: missing 20-byte object id");
        }
        let hash = hex::encode(&data[i..i + 20]);
        i += 20;

        entries.push(TreeEntry { mode, name, hash });
    }

    Ok(entries)
}

pub fn parse_commit(data: &[u8]) -> Result<CommitMeta> {
    let text = String::from_utf8(data.to_vec()).context("commit payload is not valid UTF-8")?;
    let mut lines = text.lines();

    let mut tree = String::new();
    let mut parent = None;
    let mut author = None;
    let mut committer = None;
    let mut in_headers = true;
    let mut message_lines = Vec::new();

    for line in lines.by_ref() {
        if in_headers {
            if line.is_empty() {
                in_headers = false;
                continue;
            }

            if let Some(v) = line.strip_prefix("tree ") {
                tree = v.trim().to_string();
            } else if let Some(v) = line.strip_prefix("parent ") {
                parent = Some(v.trim().to_string());
            } else if let Some(v) = line.strip_prefix("author ") {
                author = Some(v.to_string());
            } else if let Some(v) = line.strip_prefix("committer ") {
                committer = Some(v.to_string());
            }
        } else {
            message_lines.push(line.to_string());
        }
    }

    if tree.is_empty() {
        bail!("commit is missing tree field");
    }

    Ok(CommitMeta {
        tree,
        parent,
        author,
        committer,
        message: message_lines.join("\n"),
    })
}

fn object_path(hash: &str) -> String {
    format!(".tronit/objects/{}/{}", &hash[0..2], &hash[2..])
}

fn validate_hash(hash: &str) -> Result<()> {
    if hash.len() != 40 {
        bail!("hash must be 40 hex chars, got length {}", hash.len());
    }
    if !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        bail!("hash must contain only hex characters");
    }
    Ok(())
}
