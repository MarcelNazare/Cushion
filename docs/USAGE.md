# How to Use Cushion

**Cushion is your code's security budget.** It tracks how much security each change spends — so a team can see when they're running low *before* it becomes a problem.

This guide covers everything: installing it, using it on a real project, reading the dashboard, and putting it in your pipeline. Skip to whatever section you need.

---

## Table of contents

1. [The 30-second version](#1-the-30-second-version)
2. [What Cushion actually measures](#2-what-cushion-actually-measures)
3. [Install it once](#3-install-it-once)
4. [Try it without touching anything (demo)](#4-try-it-without-touching-anything-demo)
5. [Use it on your own project](#5-use-it-on-your-own-project)
6. [Reading the dashboard (for non-engineers)](#6-reading-the-dashboard-for-non-engineers)
7. [The daily workflow](#7-the-daily-workflow)
8. [Understanding the verdicts](#8-understanding-the-verdicts)
9. [Configuration](#9-configuration)
10. [Team setup](#10-team-setup)
11. [Put it in your pipeline (CI)](#11-put-it-in-your-pipeline-ci)
12. [Troubleshooting](#12-troubleshooting)
13. [FAQ](#13-faq)
14. [What Cushion does *not* do](#14-what-cushion-does-not-do)

---

## 1. The 30-second version

If you read nothing else:

```bash
cargo install --path .              # install once
cd your-python-project
cushion init --budget 1000          # set up once per project
# ... make some changes to a .py file ...
cushion check                       # see if the change fits your budget
cushion ui                          # open the friendly dashboard
```

That's it. Everything below is detail.

---

## 2. What Cushion actually measures

Every other security tool asks: **"is there a vulnerability here?"**

Cushion asks: **"how much security did this change cost, and can we afford it?"**

Each repository starts with a **cushion** — a budget of, say, 1000 safety points. Every change is priced:

- Removing an auth decorator → **spends** 15 points
- Disabling TLS verification → **spends** 25 points
- Adding an auth check → **refunds** 15 points

The cushion is tracked in a **ledger** that lives in your repo, so the whole team sees the same number. When the cushion runs low, Cushion tells you — in plain English.

**Who this is for:**

- **Engineers** — see the security cost of a change before committing
- **Engineering managers** — see a single number for "how much safety does our code have left"
- **Non-technical stakeholders** — open a dashboard and read a percentage, no glossary needed
- **AI agents** — query the budget before writing code, so they stay inside it

---

## 3. Install it once

You need [Rust](https://rustup.rs/) installed. Then, from the Cushion source directory:

```bash
cargo install --path .
```

This puts the `cushion` command on your PATH.

**Verify it works from any directory:**

```bash
cd ~
cushion --version
# → cushion 0.1.0
```

If you get `'cushion' is not recognized` or `command not found`:

- **Windows:** close and reopen the terminal. PATH updates only happen on a new shell.
- **macOS/Linux:** run `source ~/.bashrc` (or `~/.zshrc`), or open a new terminal.

That's the only install step, ever. From now on, `cushion` is available in every project on your machine.

---

## 4. Try it without touching anything (demo)

The fastest way to understand what Cushion does. **No git repo needed, no setup, no files created in your current directory.**

```bash
cushion demo
```

You'll see two verdicts printed:

```
🟢  Security strengthened — 15 points earned.
   Safety cushion: 0 → 0 / 50 (50 remaining)

🟡  Security weakened — 40 points spent.
   Safety cushion: 0 → 40 / 50 (10 remaining)

   Changes:
   − [auth_coverage]   Login protection removed (app.py line 6)
   − [crypto_strength] TLS verification disabled (app.py line 10)
```

The demo writes to your OS temp directory, so it never touches your real project.

**To see the dashboard,** follow the path the demo prints:

```bash
# On Windows:
cd %TEMP%\cushion-demo
cushion ui

# On macOS/Linux:
cd /tmp/cushion-demo
cushion ui
```

Open **http://localhost:7878** in your browser. You'll see a **yellow ring at 20%** — this is what a non-engineer sees. **Ctrl+C** in the terminal to stop.

---

## 5. Use it on your own project

### Step 1 — Go to the project

```bash
cd path/to/your-project
```

### Step 2 — Confirm it's a git repo with at least one commit

```bash
git status
```

**If it's not a git repo yet:**

```bash
git init
git add .
git commit -m "baseline"
```

**If it's a repo but has no commits:** commit something first. Cushion compares *working tree vs the last commit* — with no commits, it sees nothing.

### Step 3 — Initialize the cushion

```bash
cushion init --budget 1000
```

Creates two files:

```
.cushion.toml              ← your budget config
.cushion/ledger.jsonl      ← the shared ledger
```

### Step 4 — Commit the setup

```bash
git add .cushion.toml .cushion/
git commit -m "add cushion security budget"
```

**Do not skip this.** The ledger must be in git so the whole team shares one budget. If you gitignore it, every developer has a private counter and the tool is useless.

### Step 5 — Test it

Open any `.py` file. Add `import pickle` at the top. Save.

```bash
cushion check --dry-run
```

You should see a red verdict flagging the added pickle import. **Ctrl+Z** to revert the change, save, and you're done.

---

## 6. Reading the dashboard (for non-engineers)

This is the part that's designed for people who don't work in security.

Open the dashboard in any project that's been set up:

```bash
cushion ui
```

Navigate to **http://localhost:7878**.

You'll see three things:

### The ring

A big percentage in the middle. This is **how much security cushion is left**:

| Ring | Means |
|---|---|
| 🟢 **Green, 60–100%** | Safe to make changes |
| 🟡 **Yellow, 15–60%** | Running low — worth a conversation |
| 🔴 **Red, below 15%** | Needs attention before more changes |

### The verdict

One sentence explaining the current state:

- *"You're safe to make changes"*
- *"Getting close to your limit"*
- *"Needs attention before more changes"*

### The change history

A plain-English list of recent changes:

```
⚠️  Security weakened — 40 points spent.    (Oct 3, 2:14 PM)   −40
✅  Security strengthened — 15 points earned. (Oct 2, 5:20 PM)   +15
```

**What to do with this information:**

You don't need to know what "TLS verification" is. You just need to notice when the ring is **yellow or red** and ask the engineering team:

> *"The security cushion is at 20% — do we need to schedule some cleanup work?"*

That's the whole conversation. The number gives you a shared vocabulary.

---

## 7. The daily workflow

Once a project is set up, this is the loop:

```bash
# 1. Make changes to a .py file
#    (remove @login_required, add verify=False, etc.)

# 2. Preview the cost (safe — does NOT record)
cushion check --dry-run

# 3. If you accept the cost, record it
cushion check

# 4. Commit
git add .
git commit -m "your commit message"
```

### ⚠️ The one rule you must not break

**Run `cushion check` (without `--dry-run`) exactly once per commit.**

| What you did | Result |
|---|---|
| `cushion check` → `git commit` | ✅ Correct |
| `cushion check --dry-run` → `git commit` | ✅ Correct — preview only, nothing recorded |
| `cushion check` → `cushion check` → `git commit` | ❌ **Double-recorded** |
| Edit → `git commit` (no check) | ❌ **Never recorded** — cushion stays at 100% forever |

**Rule of thumb:** use `--dry-run` freely while developing. Run `cushion check` once, right before you commit.

---

## 8. Understanding the verdicts

Every `cushion check` returns one of three verdicts, and a matching exit code.

| Verdict | Meaning | Exit code | Blocks commit? |
|---|---|---|---|
| 🟢 **Green** | Safe — plenty of cushion left | 0 | No |
| 🟡 **Yellow** | Running low — allowed, but worth a look | 0 | No |
| 🔴 **Red** | Too low — change rejected | 1 | Yes |

The **exit code** matters for CI: any script or pipeline can check it without parsing text.

```bash
cushion check
if [ $? -ne 0 ]; then
  echo "Blocked — cushion too low"
  exit 1
fi
```

The thresholds (when green turns yellow, when yellow turns red) are configurable — see [Configuration](#9-configuration).

---

## 9. Configuration

`.cushion.toml` looks like this:

```toml
[budget]
total = 1000
window_days = 30

[thresholds]
warn = 0.40   # warn when < 40% remaining
block = 0.15  # block when < 15% remaining
```

### What you can tune

**`budget.total`** — the starting cushion. Increase this if your team is hitting the block threshold too often.

- Small project: `500`
- Typical project: `1000`
- Large / production-critical project: `5000`

**`thresholds.warn`** — when the ring turns yellow. `0.40` means "warn when less than 40% remains."

**`thresholds.block`** — when `cushion check` starts returning exit code 1. `0.15` means "block when less than 15% remains."

### How to reset the cushion

If the cushion has drifted for reasons that aren't real (e.g. you were testing):

```bash
rm .cushion/ledger.jsonl
touch .cushion/ledger.jsonl
git add .cushion/ && git commit -m "reset cushion ledger"
```

This is a **team decision** — it should be reviewed, not done silently. The whole point of the ledger is that it's tamper-evident.

---

## 10. Team setup

The single most important thing for a team:

> **`.cushion/` and `.cushion.toml` must be committed to git.**

The ledger is the shared budget. If it's gitignored, everyone has their own private cushion, and the concept collapses.

### `.gitignore` for your project

```gitignore
# Cushion — keep these tracked, not ignored
# .cushion/          ← do NOT add this line
# .cushion.toml      ← do NOT add this line

# But do ignore these:
.cushion/*.tmp
.cushion/*.swp
```

### Recommended team flow

1. **One person** runs `cushion init` and commits `.cushion.toml` + `.cushion/ledger.jsonl`
2. **Everyone else** pulls — they now have the same shared cushion
3. **Every change** runs `cushion check` before commit (see [CI integration](#11-put-it-in-your-pipeline-ci) to automate this)
4. **The cushion** is visible to the whole team via `cushion ui`

### Optional — pre-commit hook (never forget again)

If you have Python installed:

```bash
pip install pre-commit
```

Create `.pre-commit-config.yaml` in your project:

```yaml
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

Then:

```bash
pre-commit install
```

Now every `git commit` runs `cushion check` automatically. If the change is too costly, the commit is rejected before it happens.

---

## 11. Put it in your pipeline (CI)

### GitHub Actions

Add `.github/workflows/cushion.yml` to your project:

```yaml
name: Cushion
on: [pull_request]

jobs:
  cushion:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: dtolnay/rust-toolchain@stable

      - name: Install Cushion
        run: cargo install --path path/to/cushion
        # (or publish to crates.io and use `cargo install cushion`)

      - name: Check security cushion
        run: cushion check --dry-run
```

PRs that would drain the cushion below the block threshold turn red on the PR page — visible to reviewers before merge.

**Why `--dry-run` on PRs?** Because PRs shouldn't commit to the ledger — only merges to `main` should. The ledger grows on `main` only, and every PR sees the same shared budget.

### Any other CI (GitLab, CircleCI, Jenkins)

Same principle — install `cushion`, then:

```bash
cushion check --dry-run
```

The exit code tells CI everything: `0` = pass, `1` = fail.

---

## 12. Troubleshooting

### "Missing .cushion.toml. Run `cushion init` first."

You're running `cushion check` in a directory that hasn't been set up. Either:

- Run `cushion init --budget 1000` here, **or**
- `cushion demo` to see how it works without touching this directory

### "Not in a git repository."

Cushion compares against git history. Either `git init` and commit a baseline, or run `cushion demo` for a self-contained example.

### `cushion check` says "no changes" but I definitely edited a file

Three likely causes:

1. **You edited a non-Python file.** v0.1 only checks `.py` files.
2. **You already committed the change** without running `cushion check` — the cost was never recorded, and it now looks like it never happened.
3. **You edited a file that isn't tracked by git.** Run `git add` first.

### The cost was recorded twice

You ran `cushion check` twice before committing. Fix: revert one entry from the ledger by editing `.cushion/ledger.jsonl` and removing the last line — but be aware the hash chain will break, so you'll also need to reset if you want to stay tamper-evident.

**Prevention:** run `cushion check --dry-run` while developing, and `cushion check` only once, right before commit.

### The ring is red but I don't think it should be

Two fixes:

1. **Raise the budget** in `.cushion.toml`:
   ```toml
   [budget]
   total = 2000   # was 1000
   ```
2. **Loosen the thresholds:**
   ```toml
   [thresholds]
   warn = 0.25
   block = 0.10
   ```

Cushion is a **signal**, not a jail. Tune it so it fits your team's reality.

### `cushion ui` won't start — port 7878 is in use

```bash
cushion ui --port 9000
```

Then open http://localhost:9000.

### `cushion` is not recognized (Windows)

Close and reopen PowerShell/CMD — PATH changes only apply to new shells. If it still fails, run:

```powershell
$env:PATH -split ';' | Select-String 'cargo'
```

You should see `C:\Users\<you>\.cargo\bin` in the list. If you don't, reinstall Rust or add that path manually.

---

## 13. FAQ

**Is this a replacement for Semgrep / Snyk / Gitleaks?**

No. Those tools find bugs. Cushion tracks *how much security your repo has spent over time*. They answer different questions and should coexist.

**What languages does it support?**

Python only, in v0.1. Adding TypeScript, Rust, or Go is a matter of adding a new tree-sitter grammar and extending `src/posture.rs` — the architecture is language-agnostic.

**Does it check dependencies for CVEs?**

No. That's a different job. Use `osv-scanner`, `Trivy`, or `cargo-audit` alongside Cushion.

**Does it find hardcoded secrets?**

No. Use `gitleaks` or `trufflehog` for that. Cushion measures posture, not secret presence.

**What happens when the cushion runs out completely?**

`cushion check` returns exit code 1, blocking commits (if you have the hook installed) or failing CI. The only way forward is to refund points (fix security debt) or raise the budget (a team decision).

**Can I have different budgets for different branches?**

Not built-in yet. Workaround: separate `.cushion.toml` per branch, or run `cushion init` fresh on long-lived feature branches.

**Can I see the ledger history?**

Yes:

```bash
cat .cushion/ledger.jsonl | jq
```

Or open `cushion ui` and look at the "Recent changes" section. Full history is in the raw file.

**Does the ledger ever grow too large?**

The web UI only reads the last 10 entries, so it stays fast. The file grows linearly (~200 bytes per merge). After 10,000 merges it's ~2 MB — negligible.

---

## 14. What Cushion does *not* do

Being honest so you don't expect magic:

| Cushion does | Cushion does **not** |
|---|---|
| Track cumulative security spend | Find CVEs in dependencies |
| Price 9 security-relevant Python patterns | Detect hardcoded secrets |
| Give a plain-English dashboard | Scan TypeScript, Rust, Go, etc. |
| Block commits when cushion is low | Replace Semgrep / Snyk / Gitleaks |
| Tamper-evident ledger | Analyze code logic for bugs |
| Expose a queryable budget for agents | Run static analysis beyond its 9 rules |

**Cushion is a resource management layer for security.** It's the number a team checks when asking "how are we doing?" — not the tool that finds the bugs.

---

## Summary — the whole tool on one screen

```bash
# Install once
cargo install --path .

# Set up any project
cd path/to/project
cushion init --budget 1000
git add .cushion.toml .cushion/
git commit -m "add cushion"

# Daily loop
cushion check --dry-run     # preview
cushion check               # record
git commit -am "..."        # consume

# Dashboard
cushion ui                  # http://localhost:7878

# Try without setting up anything
cushion demo
```

That's Cushion. If the project is Python and you make security-relevant changes, it works. If it isn't, it's a v0.2 problem — and the architecture is ready for it.