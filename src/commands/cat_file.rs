use crate::object;

pub fn run(hash: &str) {
    let obj = object::read_object(hash);

    let content = obj.split(|&b| b== 0).nth(1).unwrap();

    println!("{}", String::from_utf8_lossy(content));
}
