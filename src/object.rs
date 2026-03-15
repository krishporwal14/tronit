use std::fs;
use crate::utils::*;

pub fn write_object(obj_type: &str, data: &[u8]) -> String {
    let header = format!("{} {}\0", obj_type, data.len());

    let mut store = header.into_bytes();
    store.extend(data);

    let hash = sha1_hash(&store);

    let compressed = compress(&store);

    let dir = &hash[0..2];
    let file = &hash[2..];

    let path = format!(".tronit/objects/{}/{}", dir, file);

    fs::create_dir_all(format!(".tronit/objects/{}", dir)).unwrap();

    fs::write(path, compressed).unwrap();

    hash
}

pub fn read_object(hash: &str) -> Vec<u8> {
    let path = format!(".tronit/objects/{}/{}", &hash[0..2], &hash[2..]);

    let data = fs::read(path).unwrap();

    decompress(&data)
}
