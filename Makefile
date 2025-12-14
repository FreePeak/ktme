# ktme Makefile
# A Rust-based CLI tool and MCP server for automated documentation generation

.PHONY: help build build-release install install-dev test fmt lint clean run-mcp stop-mcp status-mcp

# Default target
help:
	@echo "ktme - Knowledge Transfer Me"
	@echo ""
	@echo "Available targets:"
	@echo "  build          - Build in debug mode"
	@echo "  build-release  - Build in release mode (recommended)"
	@echo "  install        - Install globally using cargo"
	@echo "  install-dev    - Install development version with force flag"
	@echo "  test           - Run tests"
	@echo "  fmt            - Format code"
	@echo "  lint           - Run clippy lints"
	@echo "  clean          - Clean build artifacts"
	@echo "  run-mcp        - Start MCP server in daemon mode"
	@echo "  stop-mcp       - Stop running MCP server"
	@echo "  status-mcp     - Check MCP server status"
	@echo "  dev            - Quick dev cycle (build-release + install-dev)"
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
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy

clean:
	cargo clean

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

# CI/CD helpers
ci: fmt lint test
	@echo "CI checks completed"
