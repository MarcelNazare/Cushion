# Cushion — your code's security budget, at a glance
#
# Usage: make <target>      (run `make` alone to see all targets)

BINARY := cushion
CARGO  := cargo
TARGET := target/release/$(BINARY)

.PHONY: all build release test fmt fmt-check clippy verify demo ui \
        install uninstall publish publish-dry clean watch help ci

.DEFAULT_GOAL := help

all: release

# ── Build ─────────────────────────────────────────────────────────────

build: ## Build the debug binary
	$(CARGO) build

release: ## Build the optimized release binary
	$(CARGO) build --release

# ── Test & verify ─────────────────────────────────────────────────────

test: ## Run all unit tests
	$(CARGO) test

fmt: ## Format code with rustfmt
	$(CARGO) fmt

fmt-check: ## Verify code is formatted (no changes)
	$(CARGO) fmt --check

clippy: ## Lint with clippy, treating warnings as errors
	$(CARGO) clippy --all-targets -- -D warnings

verify: fmt-check clippy test ## Full verification: fmt + clippy + tests
	@echo ""
	@echo "✓ All checks passed."

ci: verify demo ## Reproduce the GitHub Actions pipeline locally

# ── Run ───────────────────────────────────────────────────────────────

demo: release ## Run the self-contained demo (no git repo needed)
	./$(TARGET) demo

ui: release ## Launch the friendly web dashboard on :7878
	./$(TARGET) ui

watch: ## Watch sources and re-run the demo (requires cargo-watch)
	@command -v cargo-watch >/dev/null 2>&1 || { \
		echo "cargo-watch not installed. Run: cargo install cargo-watch"; \
		exit 1; \
	}
	$(CARGO) watch -x "run -- demo"

# ── Install ───────────────────────────────────────────────────────────

install: ## Install the binary into ~/.cargo/bin
	$(CARGO) install --path .

uninstall: ## Remove the installed binary
	$(CARGO) uninstall $(BINARY)

# ── Publish ───────────────────────────────────────────────────────────

publish-dry: ## Dry-run the crates.io publish
	$(CARGO) publish --dry-run

publish: ## Publish to crates.io (requires `cargo login` first)
	$(CARGO) publish

# ── Cleanup ───────────────────────────────────────────────────────────

clean: ## Remove build artifacts and the demo directory
	$(CARGO) clean
	rm -rf /tmp/$(BINARY)-demo

# ── Help ──────────────────────────────────────────────────────────────

help: ## Show this help
	@echo "Cushion — your code's security budget, at a glance"
	@echo ""
	@echo "Usage:"
	@echo "  make <target>"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; \
		     {printf "  \033[36m%-14s\033[0m %s\n", $$1, $$2}'