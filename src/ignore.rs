use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct IgnoreMatcher {
    rules: Vec<String>,
}

impl IgnoreMatcher {
    pub fn from_repo_root() -> Result<Self> {
        let mut rules = vec![
            ".tronit".to_string(),
            ".git".to_string(),
            "target".to_string(),
        ];

        let path = Path::new(".tronitignore");
        if path.exists() {
            let raw = fs::read_to_string(path).context("failed to read .tronitignore")?;
            for line in raw.lines() {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    continue;
                }
                rules.push(trimmed.replace('\\', "/"));
            }
        }

        Ok(Self { rules })
    }

    pub fn is_ignored(&self, path: &Path) -> bool {
        let normalized = normalize_path(path);
        if normalized.is_empty() {
            return false;
        }

        for rule in &self.rules {
            let r = rule.trim().trim_matches('/').to_string();
            if r.is_empty() {
                continue;
            }

            if r.contains('/') {
                if normalized == r || normalized.starts_with(&(r.clone() + "/")) {
                    return true;
                }
            } else if normalized.split('/').any(|seg| seg == r) {
                return true;
            }
        }

        false
    }
}

pub fn normalize_path(path: &Path) -> String {
    let mut s = path.to_string_lossy().replace('\\', "/");
    while s.starts_with("./") {
        s = s[2..].to_string();
    }
    s.trim_start_matches('/').to_string()
}
