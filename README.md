# Cushion

**Your code's security budget, at a glance.**

Cushion is not another vulnerability scanner. It measures something no other tool does: **how much security each change spends.**

Every repository starts with a *cushion* — a budget of, say, 1000 safety points. Each change is compiled into a **posture delta**: removing an auth check costs points, disabling TLS costs points, adding a validation refunds them. When the cushion runs low, Cushion tells you — in plain English, on a dashboard anyone can read.

```
🟡  Security weakened — 40 points spent.
    Safety cushion: 0 → 40 / 1000 (960 remaining)

    Changes:
    − [auth_coverage]   Login protection removed (app.py line 6)
    − [crypto_strength] TLS verification disabled (app.py line 10)

    ⚠️  Getting close to the cushion limit.
```

> **First time here?** → [`docs/USAGE.md`](docs/USAGE.md) — a complete guide for technical and non-technical readers.

---

## Why this exists

Scanners answer: **"is there a vulnerability here?"**
Cushion answers: **"how much security did this change cost, and can we afford it?"**

That distinction matters most in AI-assisted development, where agents make hundreds of small changes — each one shaving a little security away without ever tripping a scanner rule. The code looks clean. The cushion is nearly empty.

Cushion is built around three ideas no existing tool combines:

1. **A budget, not a finding.** Security becomes a finite, visible resource — not a pass/fail gate.
2. **A stateful ledger.** The cushion is consumed over the repository's lifetime, hash-chained so it cannot be quietly refunded.
3. **An agent-queryable API.** The primary consumer is an AI agent asking *"will this change fit in the cushion?"* — before writing code.

It is a **resource management layer** that runs alongside Semgrep, Snyk, Gitleaks, and Trivy — not a replacement for any of them.

---

## 30-second demo (no setup)

```bash
cargo install --path .
cushion demo
```

The demo writes to your OS temp directory — it never touches your current project. It shows the full lifecycle: a change that strengthens security (refund), then a change that weakens it (spend), then points you at the friendly dashboard.

---

## Install

Requires [Rust](https://rustup.rs/) (stable, 1.75+).

```bash
cargo install --path .
```

That's it. `cushion` is now on your PATH, available in every project on your machine.

Verify from any directory:

```bash
cd ~
cushion --version
```

If you see `'cushion' is not recognized` — **close and reopen your terminal.** PATH changes only apply to new shells.

---

## Use it on your project

```bash
cd path/to/your-python-project

# One-time setup
cushion init --budget 1000
git add .cushion.toml .cushion/
git commit -m "add cushion security budget"

# Daily loop
cushion check --dry-run     # preview the cost — doesn't record
cushion check               # record it
git commit -am "..."        # consume it

# Dashboard
cushion ui                  # http://localhost:7878
```

**The one rule:** run `cushion check` (without `--dry-run`) **exactly once**, right before committing. Run it twice and the cost is double-recorded. Skip it and the cost is never recorded.

Full walkthrough, team setup, and CI integration → [`docs/USAGE.md`](docs/USAGE.md).

---

## The dashboard (for non-engineers)

`cushion ui` opens a dashboard designed to be readable by anyone — no security glossary required.

- **A ring** showing the percentage of cushion left: 🟢 green, 🟡 yellow, 🔴 red
- **A verdict** in one sentence: *"You're safe to make changes"* / *"Getting close to your limit"*
- **A plain-English history** of recent changes with points spent or earned

The point is that a product manager can ask *"the cushion is at 20% — should we schedule cleanup work?"* without knowing what a CVE is.

---

## What it checks (v0.1)

Cushion recognizes **9 security-relevant patterns** in Python, priced in safety points:

| Pattern | Cost when added | Refund when removed |
|---|---|---|
| `@login_required` / `@requires_auth` / `@auth_required` / `@permission_required` / `@require_auth` | +15 | −15 |
| `subprocess.run/call/Popen(..., shell=True)` | −30 | +30 |
| `requests.*(..., verify=False)` | −25 | +25 |
| `eval(...)` | −30 | +30 |
| `exec(...)` | −30 | +30 |
| `DEBUG = True` | −20 | +20 |
| `allow_origins = "*"` | −25 | +25 |
| `import pickle` | −15 | +15 |
| `import marshal` | −15 | +15 |

**Negative spends the cushion. Positive refunds it.**

That's deliberately small. The value isn't the rule count — it's the model. Adding more rules (and more languages) is straightforward: new tree-sitter grammar + a few entries in `src/posture.rs`.

---

## Commands

| Command | What it does |
|---|---|
| `cushion init [--budget N]` | Create `.cushion.toml` and `.cushion/ledger.jsonl` |
| `cushion check [--file F] [--dry-run]` | Evaluate current changes against the cushion |
| `cushion demo` | Self-contained example — no git, no setup |
| `cushion ui [--port P]` | Launch the friendly web dashboard |

**Exit codes:** `0` = green or yellow (allowed) · `1` = red (blocked). Ready for any CI.

---

## Configuration

`.cushion.toml`:

```toml
[budget]
total = 1000
window_days = 30

[thresholds]
warn = 0.40   # warn when < 40% remaining
block = 0.15  # block when < 15% remaining
```

Tune `total` up if your team is hitting the block threshold too often. Loosen `warn` / `block` to see fewer yellow warnings. Cushion is a signal, not a jail.

---

## Team setup

**`.cushion.toml` and `.cushion/ledger.jsonl` must be committed to git.**

The ledger is the shared budget. If it's gitignored, every developer has a private counter and the concept collapses. Do not add `.cushion/` to your project's `.gitignore` — only `.cushion/*.tmp` and `.cushion/*.swp`.

**Optional — never forget again** (pre-commit):

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: cushion
        name: cushion check
        entry: cushion check
        language: system
        pass_filenames: false
        always_run: true
```

**GitHub Actions:**

```yaml
name: Cushion
on: [pull_request]
jobs:
  cushion:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install --path .
      - run: cushion check --dry-run
```

`--dry-run` on PRs — the ledger should only grow on `main`. A PR previews its cost; the merge records it.

---

## How it works

```
┌────────────────────────────────────────────────────────────┐
│  tree-sitter AST  →  posture delta  →  budget ledger       │
│  (Python parser)     (9 patterns)     (hash-chained)       │
├────────────────────────────────────────────────────────────┤
│  CLI (clap)  ·  Web UI (axum + embedded HTML)  ·  CI ready │
└────────────────────────────────────────────────────────────┘
```

1. **AST diff** — parses old and new versions with tree-sitter, extracts security-relevant assertions, and computes a multiset diff. Removing one of two `@login_required` decorators is counted; regex tools miss it.
2. **Cost mapping** — each assertion change is priced by `src/posture.rs`.
3. **Ledger** — the net cost is appended to a hash-chained, tamper-evident file. History cannot be quietly rewritten.
4. **Verdict** — green / yellow / red based on remaining cushion.

---

## What Cushion does *not* do

To be honest about scope:

| Does | Does **not** |
|---|---|
| Track cumulative security spend | Find CVEs in dependencies |
| Price 9 security-relevant Python patterns | Detect hardcoded secrets |
| Give a plain-English dashboard | Scan TypeScript, Rust, Go |
| Block commits when cushion is low | Replace Semgrep / Snyk / Gitleaks |
| Tamper-evident ledger | Analyze code logic for bugs |
| Expose an agent-queryable budget | Run static analysis beyond its 9 rules |

Use it **alongside** your existing scanners. It answers a question they don't.

---

## Development

```bash
make            # show all targets
make release    # optimized build → target/release/cushion
make test       # unit tests
make verify     # fmt-check + clippy + tests
make demo       # see it work
make ui         # launch the dashboard
make ci         # what GitHub Actions runs
```

---

## License

MIT — see [`LICENSE`](LICENSE).