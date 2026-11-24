use anyhow::Result;
use axum::{response::Html, routing::get, Json, Router};
use serde::Serialize;
use std::path::{Path, PathBuf};

const INDEX: &str = include_str!("ui/ui.html");

#[derive(Serialize)]
struct State {
    ok: bool,
    budget: u32,
    spent: u32,
    remaining: i32,
    pct_remaining: f32,
    level: String,
    entries: Vec<EntryView>,
    message: Option<String>,
}

#[derive(Serialize)]
struct EntryView {
    timestamp: String,
    summary: String,
    net_cost: i32,
    level: String,
}

pub async fn serve(port: u16) -> Result<()> {
    let app = Router::new()
        .route("/", get(index))
        .route("/api/state", get(state));

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("✓ Cushion UI → http://localhost:{}", port);
    println!("  Press Ctrl+C to stop.");
    axum::serve(listener, app).await?;
    Ok(())
}

async fn index() -> Html<&'static str> {
    Html(INDEX)
}

async fn state() -> Json<State> {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    match build_state(&root) {
        Ok(s) => Json(s),
        Err(e) => Json(State {
            ok: false,
            budget: 0,
            spent: 0,
            remaining: 0,
            pct_remaining: 0.0,
            level: "unknown".into(),
            entries: vec![],
            message: Some(format!("{}", e)),
        }),
    }
}

fn build_state(root: &Path) -> Result<State> {
    let cfg = crate::config::load(root)?;
    let entries = crate::ledger::load(root)?;
    let spent = crate::ledger::spent_so_far(&entries);
    let remaining = cfg.budget.total as i32 - spent as i32;
    let pct = (remaining as f32 / cfg.budget.total as f32 * 100.0).max(0.0);

    let level = if pct < 15.0 {
        "red"
    } else if pct < 40.0 {
        "yellow"
    } else {
        "green"
    };

    let entries_view: Vec<EntryView> = entries
        .iter()
        .rev()
        .take(10)
        .map(|e| EntryView {
            timestamp: e.timestamp.clone(),
            summary: e.summary.clone(),
            net_cost: e.net_cost,
            level: e.verdict.clone(),
        })
        .collect();

    Ok(State {
        ok: true,
        budget: cfg.budget.total,
        spent,
        remaining,
        pct_remaining: pct,
        level: level.into(),
        entries: entries_view,
        message: None,
    })
}