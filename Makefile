.PHONY: help build release test fmt lint clean run install

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

build: ## Build the project in debug mode
	cargo build

release: ## Build the project in release mode (optimized)
	cargo build --release
	strip target/release/lookout

test: ## Run all tests
	cargo test

fmt: ## Format code using rustfmt
	cargo fmt

fmt-check: ## Check code formatting without modifying files
	cargo fmt -- --check

lint: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

check: fmt-check lint test ## Run all checks (format, lint, test)

clean: ## Clean build artifacts
	cargo clean

run: ## Run the application in debug mode
	cargo run

run-release: release ## Run the application in release mode
	./target/release/lookout

install: release ## Install the application to ~/.local/bin
	mkdir -p ~/.local/bin
	cp target/release/lookout ~/.local/bin/
	@echo "Installed to ~/.local/bin/lookout"
	@echo "Make sure ~/.local/bin is in your PATH"

dev: ## Development workflow: format, lint, test, build
	@$(MAKE) fmt
	@$(MAKE) lint
	@$(MAKE) test
	@$(MAKE) build
