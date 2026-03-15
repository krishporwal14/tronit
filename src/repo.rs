use std::fs;

pub fn init_repo() {
    fs::create_dir(".tronit").unwrap();
    fs::create_dir(".tronit/objects").unwrap();
    fs::create_dir(".tronit/refs").unwrap();
    fs::create_dir(".tronit/refs/heads").unwrap();

    fs::write(".tronit/HEAD", "ref: refs/heads/main\n").unwrap();

    println!("Initialized empty Tronit repository.");
}
