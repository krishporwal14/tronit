use std::env;
use std::fs;
use std::sync::{Mutex, OnceLock};
use anyhow::Result;
use tempfile::tempdir;

use tronit::commands::{add, branch, commit, init, status, switch};
use tronit::object;
use tronit::repo;

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn with_temp_repo<F>(f: F) -> Result<()>
where
    F: FnOnce() -> Result<()>,
{
    let _guard = test_lock().lock().expect("mutex poisoned");

    let old = env::current_dir()?;
    let dir = tempdir()?;

    env::set_current_dir(dir.path())?;
    init::run()?;

    let result = f();

    env::set_current_dir(old)?;
    result
}

#[test]
fn add_readd_replaces_index_entry() -> Result<()> {
    with_temp_repo(|| {
        fs::write("a.txt", "v1\n")?;
        add::run("a.txt")?;

        fs::write("a.txt", "v2 changed\n")?;
        add::run("a.txt")?;

        let index = fs::read_to_string(".tronit/index")?;
        let lines: Vec<&str> = index.lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].ends_with(" a.txt"));
        Ok(())
    })
}

#[test]
fn commit_writes_author_and_tree() -> Result<()> {
    with_temp_repo(|| {
        fs::create_dir_all("src")?;
        fs::write("src/main.txt", "hello\n")?;

        add::run(".")?;
        commit::run("first", Some("Krish Porwal"), Some("krish@example.com"))?;

        let head = repo::resolve_head_commit()?.expect("expected commit");
        let commit_obj = object::read_object_typed(&head)?;
        let meta = object::parse_commit(&commit_obj.data)?;

        assert!(!meta.tree.is_empty());
        assert!(meta.author.unwrap_or_default().contains("<krish@example.com>"));
        assert!(meta.committer.unwrap_or_default().contains("<krish@example.com>"));

        let tree_obj = object::read_object_typed(&meta.tree)?;
        assert_eq!(tree_obj.obj_type, "tree");

        Ok(())
    })
}

#[test]
fn branch_and_switch_update_head_symbolic_ref() -> Result<()> {
    with_temp_repo(|| {
        fs::write("a.txt", "x\n")?;
        add::run("a.txt")?;
        commit::run("base", Some("A"), Some("a@b.com"))?;

        branch::run(Some("feature"))?;
        switch::run("feature")?;

        let branch = repo::head_branch_name()?;
        assert_eq!(branch, "feature");

        Ok(())
    })
}

#[test]
fn status_reports_unstaged_and_untracked() -> Result<()> {
    with_temp_repo(|| {
        fs::write("tracked.txt", "one\n")?;
        add::run("tracked.txt")?;
        commit::run("base", Some("A"), Some("a@b.com"))?;

        fs::write("tracked.txt", "two\n")?;
        fs::write("new.txt", "new\n")?;

        let report = status::collect_status()?;

        assert!(report.unstaged.iter().any(|s| s.contains("tracked.txt")));
        assert!(report.untracked.iter().any(|s| s == "new.txt"));

        Ok(())
    })
}
