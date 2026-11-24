use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub budget: Budget,
    #[serde(default)]
    pub thresholds: Thresholds,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Budget {
    pub total: u32,
    #[serde(default = "default_window")]
    pub window_days: u32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Thresholds {
    /// Warn when remaining/total drops below this fraction
    #[serde(default = "default_warn")]
    pub warn: f32,
    /// Block when remaining/total drops below this fraction
    #[serde(default = "default_block")]
    pub block: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self { warn: 0.40, block: 0.15 }
    }
}

fn default_window() -> u32 { 30 }
fn default_warn() -> f32 { 0.40 }
fn default_block() -> f32 { 0.15 }

pub fn init(total: u32) -> Result<()> {
    let cfg = Config {
        budget: Budget { total, window_days: 30 },
        thresholds: Thresholds::default(),
    };
    let toml = toml::to_string_pretty(&cfg)?;
    std::fs::write(".cushion.toml", toml)?;
    std::fs::create_dir_all(".cushion")?;
    std::fs::write(".cushion/ledger.jsonl", "")?;
    Ok(())
}

pub fn load(root: &Path) -> Result<Config> {
    let path = root.join(".cushion.toml");
    let content = std::fs::read_to_string(&path).with_context(|| {
        format!("Missing {}. Run `cushion init` first.", path.display())
    })?;
    Ok(toml::from_str(&content)?)
}