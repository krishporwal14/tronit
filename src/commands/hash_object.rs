use std::fs;

use crate::object;

pub fn run(file: &str) {
    let data = fs::read(file).unwrap();

    let hash = object::write_object("blob", &data);

    println!("{}", hash);
}
