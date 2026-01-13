# ktme Makefile
# A Rust-based CLI tool and MCP server for automated documentation generation

.PHONY: help build build-release install install-dev test test-new fmt lint clean run-mcp stop-mcp status-mcp
.PHONY: test-changes ci pre-release version-patch version-minor version-major publish release

# Default target
help:
	@echo "ktme - Knowledge Transfer Me"
	@echo ""
	@echo "Available targets:"
	@echo ""
	@echo "Build & Development:"
	@echo "  build          - Build in debug mode"
	@echo "  build-release  - Build in release mode (recommended)"
	@echo "  install        - Install globally using cargo"
	@echo "  install-dev    - Install development version with force flag"
	@echo "  dev            - Quick dev cycle (build-release + install-dev)"
	@echo ""
	@echo "Testing & Quality:"
	@echo "  test           - Run all tests"
	@echo "  test-new       - Run tests for newly implemented modules"
	@echo "  test-changes   - Run tests for recent changes (templates, generator, providers, writers)"
	@echo "  fmt            - Format code"
	@echo "  lint           - Run clippy lints"
	@echo "  ci             - Run all checks (fmt + lint + test)"
	@echo ""
	@echo "MCP Server:"
	@echo "  run-mcp        - Start MCP server in daemon mode"
	@echo "  stop-mcp       - Stop running MCP server"
	@echo "  status-mcp     - Check MCP server status"
	@echo ""
	@echo "Release & Publish:"
	@echo "  version-patch  - Bump patch version (0.1.0 -> 0.1.1)"
	@echo "  version-minor  - Bump minor version (0.1.0 -> 0.2.0)"
	@echo "  version-major  - Bump major version (0.1.0 -> 1.0.0)"
	@echo "  pre-release    - Run all checks before release"
	@echo "  release        - Full release workflow: commit, push, tag, publish to crates.io"
	@echo "  publish        - Publish to crates.io (requires authentication)"
	@echo ""
	@echo "Utilities:"
	@echo "  clean          - Clean build artifacts"
	@echo "  help           - Show this help message"

# Build targets
build:
	cargo build

build-release:
	cargo build --release

# Installation targets
install:
	cargo install --path .

install-dev:
	cargo install --path . --force

# Development targets
dev: build-release install-dev
	@echo "Development cycle complete: built and installed ktme"

# Testing and code quality
test:
	@echo "Running all tests..."
	cargo test

test-new:
	@echo "Running tests for newly implemented modules..."
	@echo "Testing templates..."
	@cargo test --lib templates 2>&1 | grep -E "(test result|FAILED)" || true
	@echo ""
	@echo "Testing generator..."
	@cargo test --lib generator 2>&1 | grep -E "(test result|FAILED)" || true
	@echo ""
	@echo "Testing git providers..."
	@cargo test --lib 'git::providers' 2>&1 | grep -E "(test result|FAILED)" || true
	@echo ""
	@echo "Testing confluence..."
	@cargo test --lib confluence 2>&1 | grep -E "(test result|FAILED)" || true
	@echo ""
	@echo "Testing markdown writer..."
	@cargo test --lib 'writers::markdown' 2>&1 | grep -E "(test result|FAILED)" || true

test-changes: test-new
	@echo ""
	@echo "✓ All new module tests completed"

fmt:
	@echo "Formatting code..."
	cargo fmt

fmt-check:
	@echo "Checking code formatting..."
	cargo fmt -- --check

lint:
	@echo "Running clippy..."
	cargo clippy

lint-strict:
	@echo "Running clippy with strict warnings..."
	cargo clippy -- -D warnings

clean:
	cargo clean

# CI/CD helpers
ci: fmt-check lint-strict test
	@echo ""
	@echo "✓ All CI checks passed"

pre-release: ci
	@echo ""
	@echo "✓ Pre-release checks passed"
	@echo "Ready to publish!"

# Version management
version-patch:
	@echo "Bumping patch version..."
	@cargo set-version --bump patch 2>/dev/null || { \
		echo "cargo-edit not installed. Installing..."; \
		cargo install cargo-edit; \
		cargo set-version --bump patch; \
	}
	@NEW_VERSION=$$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version'); \
	echo "New version: $$NEW_VERSION"

version-minor:
	@echo "Bumping minor version..."
	@cargo set-version --bump minor 2>/dev/null || { \
		echo "cargo-edit not installed. Installing..."; \
		cargo install cargo-edit; \
		cargo set-version --bump minor; \
	}
	@NEW_VERSION=$$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version'); \
	echo "New version: $$NEW_VERSION"

version-major:
	@echo "Bumping major version..."
	@cargo set-version --bump major 2>/dev/null || { \
		echo "cargo-edit not installed. Installing..."; \
		cargo install cargo-edit; \
		cargo set-version --bump major; \
	}
	@NEW_VERSION=$$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version'); \
	echo "New version: $$NEW_VERSION"

# Publishing workflow
publish: pre-release
	@echo ""
	@echo "Publishing to crates.io..."
	@echo "NOTE: You must be logged in to crates.io (run 'cargo login' first)"
	cargo publish

# Full release workflow: commit -> push -> tag -> push tag -> publish to crates.io
release:
	@echo "Starting release workflow..."
	@echo ""
	@echo "Step 1: Running pre-release checks..."
	@$(MAKE) pre-release
	@echo ""
	@echo "Step 2: Getting version information..."
	@VERSION=$$(cargo metadata --format-version 1 --no-deps | jq -r '.packages[0].version'); \
	echo "Current version: $$VERSION"; \
	echo ""; \
	read -p "Proceed with release v$$VERSION? (y/N) " -n 1 -r; \
	echo ""; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		echo "Step 3: Committing changes..."; \
		git add Cargo.toml Cargo.lock; \
		git commit -m "chore: release v$$VERSION" || echo "No changes to commit"; \
		echo ""; \
		echo "Step 4: Pushing to origin..."; \
		git push origin $$(git branch --show-current); \
		echo ""; \
		echo "Step 5: Creating and pushing tag..."; \
		git tag -a "v$$VERSION" -m "Release v$$VERSION"; \
		git push origin "v$$VERSION"; \
		echo ""; \
		echo "Step 6: Publishing to crates.io..."; \
		cargo publish; \
		echo ""; \
		echo "✓ Release v$$VERSION completed successfully!"; \
		echo ""; \
		echo "Published:"; \
		echo "  - Git tag: v$$VERSION"; \
		echo "  - Crates.io: https://crates.io/crates/ktme/$$VERSION"; \
		echo "  - GitHub: https://github.com/FreePeak/ktme/releases/tag/v$$VERSION"; \
	else \
		echo "Release cancelled."; \
		exit 1; \
	fi

# MCP server management
run-mcp:
	@echo "Starting ktme MCP server in daemon mode..."
	./target/release/ktme mcp start --daemon

stop-mcp:
	@echo "Stopping ktme MCP server..."
	./target/release/ktme mcp stop || true

status-mcp:
	@echo "Checking ktme MCP server status..."
	./target/release/ktme mcp status || true

# Quick commands
version:
	./target/release/ktme --version || echo "Build the project first"

# Development workflow
quick-run: build-release
	@echo "Running ktme from target/release..."
	./target/release/ktme --version

# Setup for first-time developers
setup:
	@echo "Setting up ktme development environment..."
	@echo "1. Building in release mode..."
	cargo build --release
	@echo "2. Installing globally..."
	cargo install --path --force .
	@echo "3. Verifying installation..."
	ktme --version
	@echo "Setup complete!"
