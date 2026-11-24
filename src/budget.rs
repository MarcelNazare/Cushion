use anyhow::Result;
use std::path::Path;

use crate::config::Config;
use crate::diff::{Change, DiffResult};
use crate::ledger;

pub struct Verdict {
    pub net_cost: i32,
    pub spent_before: u32,
    pub spent_after: u32,
    pub remaining: i32,
    pub total_budget: u32,
    pub level: &'static str,
    pub blocked: bool,
    pub summary: String,
    pub changes: Vec<Change>,
}

pub fn evaluate(cfg: &Config, diffs: &[DiffResult], root: &Path) -> Result<Verdict> {
    let entries = ledger::load(root)?;
    let spent_before = ledger::spent_so_far(&entries);

    let net_cost: i32 = diffs.iter().map(|d| d.net_cost).sum();

    // Negative net_cost → security weakened → spend increases
    // Positive net_cost → security strengthened → spend decreases (refund)
    let new_spent = (spent_before as i32 - net_cost).max(0) as u32;
    let remaining = cfg.budget.total as i32 - new_spent as i32;

    let ratio = remaining as f32 / cfg.budget.total as f32;
    let (level, blocked) = if remaining < 0 {
        ("red", true)
    } else if ratio < cfg.thresholds.block {
        ("red", true)
    } else if ratio < cfg.thresholds.warn {
        ("yellow", false)
    } else {
        ("green", false)
    };

    let mut changes = Vec::new();
    for d in diffs {
        for c in &d.changes {
            changes.push(c.clone());
        }
    }

    let summary = if net_cost < 0 {
        format!("Security weakened — {} points spent.", -net_cost)
    } else if net_cost > 0 {
        format!("Security strengthened — {} points earned.", net_cost)
    } else {
        "No net security impact.".to_string()
    };

    Ok(Verdict {
        net_cost,
        spent_before,
        spent_after: new_spent,
        remaining,
        total_budget: cfg.budget.total,
        level,
        blocked,
        summary,
        changes,
    })
}

pub fn print_human(v: &Verdict, diffs: &[DiffResult]) {
    let icon = match v.level {
        "green" => "🟢",
        "yellow" => "🟡",
        _ => "🔴",
    };

    println!("\n{}  {}", icon, v.summary);

    // Show where the cushion was before, and where it is now.
    if v.spent_before == v.spent_after {
        println!(
            "   Safety cushion: {} / {} ({} remaining)",
            v.spent_after, v.total_budget, v.remaining
        );
    } else {
        println!(
            "   Safety cushion: {} → {} / {} ({} remaining)",
            v.spent_before, v.spent_after, v.total_budget, v.remaining
        );
    }

    // Use v.changes directly — no need to re-flatten diffs.
    if !v.changes.is_empty() {
        // Build a lookup from change → file so we can label lines with the right filename.
        let mut file_for_change: Vec<&str> = Vec::new();
        for d in diffs {
            for _ in &d.changes {
                file_for_change.push(if d.file.is_empty() { "file" } else { &d.file });
            }
        }

        println!("\n   Changes:");
        for (i, c) in v.changes.iter().enumerate() {
            let sign = if c.value < 0 { "−" } else { "+" };
            let file = file_for_change.get(i).copied().unwrap_or("file");
            println!(
                "   {} [{}] {} ({} line {})",
                sign, c.dimension, c.description, file, c.line
            );
        }
    }

    match v.level {
        "red" => {
            println!("\n   ⛔ This change is blocked.");
            println!("      To override: raise `budget.total` in .cushion.toml");
        }
        "yellow" => println!("\n   ⚠️  Getting close to the cushion limit."),
        _ => {}
    }
    println!();
}