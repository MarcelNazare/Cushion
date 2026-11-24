# Cushion

**Your code's security budget, at a glance.**

Cushion is not another vulnerability scanner. It measures something no other tool does: **how much security each change spends**.

Every repository starts with a **safety cushion** — a budget of, say, 1000 safety points. Each change is compiled into a **posture delta**: removing an auth decorator costs points, disabling TLS costs points, adding an auth check refunds points. The cushion is consumed over the repository's lifetime.

When an agent proposes a change, it can ask *"will this fit in the cushion?"* before writing code. When a human makes a change, the CLI blocks it if the cushion runs too low.

```
🟡  Security weakened — 40 points spent.
    Safety cushion: 40 / 50 (10 remaining)

    Changes:
    − [auth_coverage]   Login protection removed (line 6)
    − [crypto_strength] TLS verification disabled (line 10)

    ⚠️  Getting close to the cushion limit.
```

## Why

Scanners ask *"is there a vulnerability here?"* Cushion asks *"how much security did this change cost, and can we afford it?"*

That distinction matters for AI-assisted development, where agents make hundreds of small changes — each one shaving a bit of security away without ever tripping a scanner rule. The code looks clean. The cushion is nearly empty.

## Install

```bash
cargo install --path .
```

## Quick start

```bash
# 1. Try the self-contained demo (no git required)
cushion demo

# 2. See the friendly UI
cd /tmp/cushion-demo
cushion ui
# open http://localhost:7878

# 3. Use it on your own repo
cd your-project
cushion init --budget 1000
# ...make some changes to a .py file...
cushion check
```

## Commands

| Command | What it does |
|---|---|
| `cushion init [--budget N]` | Create `.cushion.toml` and `.cushion/ledger.jsonl` |
| `cushion check [--file F] [--dry-run]` | Evaluate current changes against the cushion |
| `cushion demo` | Self-contained example — no git needed |
| `cushion ui [--port P]` | Launch the friendly web dashboard |

## Config (`.cushion.toml`)

```toml
[budget]
total = 1000
window_days = 30

[thresholds]
warn = 0.40   # warn when < 40% remaining
block = 0.15  # block when < 15% remaining
```

## How it works

1. **AST diff** — parses old and new versions with tree-sitter, extracts security-relevant assertions (decorators, dangerous calls, config assignments), and computes the multiset diff.
2. **Cost mapping** — each assertion change is priced (e.g. removed `@login_required` = −15 points).
3. **Ledger** — the net cost is appended to a **hash-chained, tamper-evident ledger**. The cushion cannot be quietly refunded.
4. **Verdict** — green / yellow / red based on remaining cushion.

## Integrations

**GitHub Actions** — fails the build when the cushion runs low:
```yaml
- run: cargo install --path . && cushion check
```

**pre-commit** — blocks bad commits locally:
```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: cushion
        name: cushion
        entry: cushion check --dry-run
        language: system
        pass_filenames: false
```

## Status

v0.1 — Python only. The architecture is language-agnostic; add `tree-sitter-typescript` and extend `src/posture.rs` to support more.

## License

MIT