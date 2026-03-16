use anyhow::Result;
use crate::repo;

pub fn run() -> Result<()> {
    repo::init_repo()
}
