use std::fs;
use std::io::Write;
use walkdir::WalkDir;

use crate::object;

pub fn run(path: &str) {
    if path == "." {
        for entry in WalkDir::new(".") {
            let entry = entry.unwrap();
            let path = entry.path();

            if path.is_file() {
                let p = path.to_str().unwrap();

                if p.contains(".tronit") {
                    continue;
                }

                add_file(p);
            }
        }
    } else {
        add_file(path);
    }
}

fn add_file(file: &str) {
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
