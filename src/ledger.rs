use anyhow::Result;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entry {
    pub timestamp: String,
    pub summary: String,
    pub net_cost: i32,
    pub spent_total: u32,
    pub verdict: String,
    pub prev_hash: String,
    pub hash: String,
}

pub fn path(root: &Path) -> PathBuf {
    root.join(".cushion").join("ledger.jsonl")
}

pub fn load(root: &Path) -> Result<Vec<Entry>> {
    let p = path(root);
    if !p.exists() {
        return Ok(vec![]);
    }
    let content = std::fs::read_to_string(&p)?;
    let mut entries = Vec::new();
    let mut prev_hash = String::from("0");
    for (i, line) in content.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let entry: Entry = serde_json::from_str(line)
            .map_err(|e| anyhow::anyhow!("Corrupt ledger at line {}: {}", i + 1, e))?;
        if entry.prev_hash != prev_hash {
            anyhow::bail!("🔒 Ledger tampering detected at line {}: hash chain broken", i + 1);
        }
        let expected = compute_hash(&entry, &entry.prev_hash);
        if entry.hash != expected {
            anyhow::bail!("🔒 Ledger tampering detected at line {}: signature invalid", i + 1);
        }
        prev_hash = entry.hash.clone();
        entries.push(entry);
    }
    Ok(entries)
}

pub fn append(root: &Path, mut entry: Entry) -> Result<()> {
    let entries = load(root)?;
    let prev_hash = entries.last().map(|e| e.hash.clone()).unwrap_or_else(|| "0".into());
    entry.prev_hash = prev_hash.clone();
    entry.hash = compute_hash(&entry, &prev_hash);
    let p = path(root);
    std::fs::create_dir_all(p.parent().unwrap())?;
    use std::io::Write;
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&p)?;
    writeln!(f, "{}", serde_json::to_string(&entry)?)?;
    Ok(())
}

fn compute_hash(entry: &Entry, prev_hash: &str) -> String {
    let mut h = Sha256::new();
    h.update(prev_hash.as_bytes());
    h.update(entry.timestamp.as_bytes());
    h.update(entry.summary.as_bytes());
    h.update(entry.net_cost.to_string().as_bytes());
    h.update(entry.spent_total.to_string().as_bytes());
    h.update(entry.verdict.as_bytes());
    hex::encode(h.finalize())
}

pub fn spent_so_far(entries: &[Entry]) -> u32 {
    entries.last().map(|e| e.spent_total).unwrap_or(0)
}