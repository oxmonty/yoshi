.PHONY: help bootstrap run headless build release fmt lint test deny ci gacp

# Default target - show help
.DEFAULT_GOAL := help

## Help:
help: ## Show this help message
	@printf "\n\033[1mUsage:\033[0m make \033[36m<target>\033[0m\n"
	@awk 'BEGIN {FS = ":.*##"; section=""} \
		/^## [A-Za-z]/ { section=substr($$0, 4); next } \
		/^[a-zA-Z_-]+:.*##/ { \
			if (section != "") { printf "\n\033[1m%s\033[0m\n", section; section="" } \
			printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 \
		}' $(MAKEFILE_LIST)
	@printf "\n"

## Dev:
bootstrap: ## Install toolchain components, cargo-deny/nextest, and the ipykernel venv
	script/bootstrap

run: ## Run the yoshi app (kernel prewarms on launch)
	cargo run -p yoshi-app

headless: ## Kernel round-trip check without a window (exit 0 = pass)
	cargo run -p yoshi-app -- --headless

## Build:
build: ## Debug build of the whole workspace
	cargo build --workspace

release: ## Optimized release build
	cargo build --workspace --release

## Quality:
fmt: ## Format all Rust code
	cargo fmt

lint: ## Clippy with warnings denied (matches CI)
	cargo clippy --workspace --all-targets -- -D warnings

test: ## Run the test suite via nextest
	cargo nextest run --workspace --no-tests=warn

deny: ## Check dependency licenses against the allowlist
	cargo deny check licenses

ci: ## Run every CI quality gate locally (fmt, clippy, deny, tests, headless)
	cargo fmt --check
	cargo clippy --workspace --all-targets -- -D warnings
	cargo deny check licenses
	cargo nextest run --workspace --no-tests=warn
	cargo run -q -p yoshi-app -- --headless

## Git:
gacp: ## Git add, commit, push (Usage: make gacp M="type(scope): message")
	git add -A && git commit -m "$(M)" && git push
