use std::fs;
use std::io::Write;

use crate::object;

pub fn run(file: &str) {
    let data = fs::read(file).unwrap();

    let hash = object::write_object("blob", &data);

    let entry = format!("{} {}\n", hash, file);

    fs::OpenOptions::new()
        .append(true)
        .create(true)
        .open(".tronit/index")
        .unwrap()
        .write_all(entry.as_bytes())
        .unwrap();

    println!("added {}", file);
}
