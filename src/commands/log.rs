use std::fs;

use crate::object;

pub fn run() {
    let mut current = fs::read_to_string(".tronit/refs/heads/main").unwrap();

    loop {
        let obj = object::read_object(current.trim());

        let content = obj.split(|&b| b == 0).nth(1).unwrap();

        let text = String::from_utf8_lossy(content);

        println!("commit {}\n{}\n", current.trim(), text);

        let parent_line = text.lines().find(|l| l.starts_with("parent"));

        if let Some(p) = parent_line {
            current = p.replace("parent ", "");
        } else {
            break;
        }
    }
}
