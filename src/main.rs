mod budget;
mod config;
mod diff;
mod ledger;
mod posture;
mod web;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "cushion",
    about = "Cushion — your code's security budget, at a glance",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a cushion in the current repo
    Init {
        #[arg(long, default_value_t = 1000)]
        budget: u32,
    },
    /// Check current changes against the safety cushion
    Check {
        /// Specific file to check (default: all changed .py files)
        #[arg(long)]
        file: Option<PathBuf>,
        /// Don't record to the ledger (preview only)
        #[arg(long)]
        dry_run: bool,
    },
    /// Self-contained demo — no git required
    Demo,
    /// Launch the friendly web UI
    Ui {
        #[arg(long, default_value_t = 7878)]
        port: u16,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { budget } => {
            config::init(budget)?;
            println!("✓ Initialized .cushion.toml with budget {}", budget);
            println!("✓ Created .cushion/ledger.jsonl");
            println!("\nNext: run `cushion check` after making changes.");
        }

        Commands::Check { file, dry_run } => {
            let root = std::env::current_dir()?;
            let cfg = config::load(&root)?;
            let diffs = diff::check(&root, file.as_deref())?;

            if diffs.is_empty() {
                println!("🟢 No security-relevant changes.");
                return Ok(());
            }

            let verdict = budget::evaluate(&cfg, &diffs, &root)?;
            budget::print_human(&verdict, &diffs);

            if !dry_run {
                let entry = ledger::Entry {
                    timestamp: chrono::Utc::now().to_rfc3339(),
                    summary: verdict.summary.clone(),
                    net_cost: verdict.net_cost,
                    spent_total: verdict.spent_after,
                    verdict: verdict.level.to_string(),
                    prev_hash: String::new(),
                    hash: String::new(),
                };
                ledger::append(&root, entry)?;
            }

            if verdict.blocked {
                std::process::exit(1);
            }
        }

        Commands::Demo => run_demo()?,

        Commands::Ui { port } => {
            web::serve(port).await?;
        }
    }

    Ok(())
}

fn run_demo() -> anyhow::Result<()> {
    use std::env;

    let tmp = env::temp_dir().join("cushion-demo");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp)?;
    env::set_current_dir(&tmp)?;

    config::init(50)?; // Small budget so the demo visibly transitions green → yellow

    // Best-effort git init so `cushion check` also works here
    let _ = std::process::Command::new("git")
        .arg("-C").arg(&tmp).args(["init", "-q"]).status();

    let empty = "";
    let v1 = r#"from flask import Flask
from flask_login import login_required

app = Flask(__name__)

@app.route("/admin")
@login_required
def admin():
    return "admin panel"
"#;
    let v2 = r#"from flask import Flask
import requests

app = Flask(__name__)

@app.route("/admin")
def admin():
    return "admin panel"

def fetch_data():
    return requests.get("https://internal.api", verify=False)
"#;

    println!("\n═══════════════════════════════════════");
    println!("  Cushion — Live Demo");
    println!("═══════════════════════════════════════\n");

    let cfg = config::load(&tmp)?;

    println!("📝 Step 1: A file is added with good security (login required).");
    let r1 = diff::compare_files(empty, v1)?;
    let v = budget::evaluate(&cfg, &[r1.clone()], &tmp)?;
    budget::print_human(&v, &[r1]);

    println!("📝 Step 2: Login protection is removed and TLS verification is disabled.");
    let r2 = diff::compare_files(v1, v2)?;
    let v = budget::evaluate(&cfg, &[r2.clone()], &tmp)?;
    budget::print_human(&v, &[r2]);

    let entry = ledger::Entry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        summary: v.summary.clone(),
        net_cost: v.net_cost,
        spent_total: v.spent_after,
        verdict: v.level.to_string(),
        prev_hash: String::new(),
        hash: String::new(),
    };
    ledger::append(&tmp, entry)?;

    // Write v2 as the current state so `cushion check` later works
    std::fs::write(tmp.join("app.py"), v2)?;

    println!("\n✅ Demo complete.\n");
    println!("   See the friendly UI:");
    println!("     cd {}", tmp.display());
    println!("     cushion ui\n");
    println!("   Then open http://localhost:7878\n");

    Ok(())
}