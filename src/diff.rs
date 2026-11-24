use anyhow::{bail, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::posture::{assertion_set, Cost, Direction};

#[derive(Serialize, Clone)]
pub struct Change {
    pub description: String,
    pub dimension: String,
    pub value: i32,
    pub line: usize,
    pub direction: String,
}

#[derive(Serialize, Clone)]
pub struct DiffResult {
    pub file: String,
    pub net_cost: i32,
    pub changes: Vec<Change>,
}

/// Compare two versions of a source file and return the security delta.
/// Uses a multiset diff so removing one of two identical assertions is counted.
pub fn compare_files(old: &str, new: &str) -> Result<DiffResult> {
    let old_set = assertion_set(old);
    let new_set = assertion_set(new);

    let mut old_map: HashMap<String, Vec<usize>> = HashMap::new();
    let mut new_map: HashMap<String, Vec<usize>> = HashMap::new();
    for (k, l) in old_set {
        old_map.entry(k).or_default().push(l);
    }
    for (k, l) in new_set {
        new_map.entry(k).or_default().push(l);
    }

    let mut changes = Vec::new();
    let mut net = 0i32;

    // Removals
    for (key, old_lines) in &old_map {
        let new_count = new_map.get(key).map(|v| v.len()).unwrap_or(0);
        let old_count = old_lines.len();
        if old_count <= new_count {
            continue;
        }
        let removed = old_count - new_count;
        if let Some(c) = Cost::of(key, Direction::Removed) {
            let anchor = *old_lines.last().unwrap();
            for _ in 0..removed {
                net += c.value;
                changes.push(Change {
                    description: c.desc.to_string(),
                    dimension: c.dimension.to_string(),
                    value: c.value,
                    line: anchor,
                    direction: "removed".into(),
                });
            }
        }
    }

    // Additions
    for (key, new_lines) in &new_map {
        let old_count = old_map.get(key).map(|v| v.len()).unwrap_or(0);
        let new_count = new_lines.len();
        if new_count <= old_count {
            continue;
        }
        let added = new_count - old_count;
        if let Some(c) = Cost::of(key, Direction::Added) {
            let anchor = *new_lines.last().unwrap();
            for _ in 0..added {
                net += c.value;
                changes.push(Change {
                    description: c.desc.to_string(),
                    dimension: c.dimension.to_string(),
                    value: c.value,
                    line: anchor,
                    direction: "added".into(),
                });
            }
        }
    }

    Ok(DiffResult {
        file: String::new(),
        net_cost: net,
        changes,
    })
}

pub fn check(root: &Path, file: Option<&Path>) -> Result<Vec<DiffResult>> {
    let status = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output();
    if status.map(|o| !o.status.success()).unwrap_or(true) {
        bail!("Not in a git repository. Run `cushion demo` for a self-contained example, or `git init` first.");
    }

    let targets: Vec<PathBuf> = match file {
        Some(f) => vec![f.to_path_buf()],
        None => {
            let out = Command::new("git")
                .arg("-C")
                .arg(root)
                .args(["diff", "--name-only", "HEAD"])
                .output()?;
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .filter(|l| l.ends_with(".py"))
                .map(|l| root.join(l))
                .collect()
        }
    };

    let mut results = Vec::new();
    for target in targets {
        let rel = target.strip_prefix(root).unwrap_or(&target);
        let rel_s = rel.to_string_lossy().into_owned();
        let old = git_show(root, &rel_s).unwrap_or_default();
        let new = std::fs::read_to_string(&target).unwrap_or_default();
        let mut r = compare_files(&old, &new)?;
        r.file = rel_s;
        if !r.changes.is_empty() {
            results.push(r);
        }
    }
    Ok(results)
}

fn git_show(root: &Path, path: &str) -> Option<String> {
    let out = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["show", &format!("HEAD:{}", path)])
        .output()
        .ok()?;
    if out.status.success() {
        Some(String::from_utf8_lossy(&out.stdout).into_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_removed_auth() {
        let old = "from flask_login import login_required\n@login_required\ndef f(): pass\n";
        let new = "def f(): pass\n";
        let r = compare_files(old, new).unwrap();
        assert!(r.net_cost < 0);
        assert!(r.changes.iter().any(|c| c.dimension == "auth_coverage"));
    }

    #[test]
    fn detects_disabled_tls() {
        let old = "requests.get('x', verify=True)\n";
        let new = "requests.get('x', verify=False)\n";
        let r = compare_files(old, new).unwrap();
        assert!(r.net_cost < 0);
        assert!(r.changes.iter().any(|c| c.dimension == "crypto_strength"));
    }

    #[test]
    fn multiset_diff_catches_partial_removal() {
        let old = "@login_required\ndef a(): pass\n@login_required\ndef b(): pass\n";
        let new = "@login_required\ndef a(): pass\ndef b(): pass\n";
        let r = compare_files(old, new).unwrap();
        assert!(r.net_cost < 0);
    }

    #[test]
    fn no_change_means_zero_cost() {
        let src = "def f(): pass\n";
        let r = compare_files(src, src).unwrap();
        assert_eq!(r.net_cost, 0);
    }

    #[test]
    fn added_auth_is_refund() {
        let old = "def f(): pass\n";
        let new = "@login_required\ndef f(): pass\n";
        let r = compare_files(old, new).unwrap();
        assert!(r.net_cost > 0);
    }
}