# KTME - Knowledge Transfer Me

> Automated documentation generation from Git changes using AI

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ktme.svg)](https://crates.io/crates/ktme)

KTME is a CLI tool and MCP server that automatically generates and maintains documentation from Git changes. It integrates with GitHub, GitLab, and Confluence, using AI to create meaningful documentation from code commits and pull requests.

## Features

- **Auto-Initialization** - Automatically creates documentation structure and knowledge graph on first use
- **Smart Documentation Generation** - AI-powered documentation from Git diffs, commits, and PRs
- **Knowledge Graph** - Tracks features, relationships, and documentation across services
- **Multiple Integrations** - GitHub, GitLab, and Confluence support
- **Template System** - Customizable Markdown templates with variable substitution
- **MCP Server** - Model Context Protocol server for AI agent integration
- **Dual Storage** - TOML and SQLite backends for flexibility
- **Service Mapping** - Organize documentation by service/project

## Quick Start

### Installation

```bash
# Install via npm (recommended - easiest)
npm install -g ktme-cli

# Install from crates.io
cargo install ktme

# Or build from source
cargo build --release
cargo install --path .
```

### Basic Usage

```bash
# Initialize project documentation and knowledge graph
ktme init --service my-service

# Or initialize with auto-detection
ktme init

# Generate docs from staged changes (auto-initializes if needed)
ktme generate --service my-service --staged

# Extract GitHub PR and generate docs
ktme extract --pr 123 --provider github
ktme generate --service my-service --commit HEAD

# Update existing documentation
ktme update --service my-service --staged --section "API Changes"

# Map service to documentation location
ktme mapping add my-service --file docs/api.md
```

### Configuration

Create `~/.config/ktme/config.toml`:

```toml
[git]
github_token = "ghp_xxxxx"
gitlab_token = "glpat_xxxxx"

[confluence]
base_url = "https://your-company.atlassian.net/wiki"
api_token = "your-api-token"
space_key = "DOCS"

[ai]
provider = "openai"
api_key = "sk-xxxxx"
model = "gpt-4"
```

## Documentation

- **[Quick Start Guide](docs/PUBLISH.md)** - Get started in 5 minutes
- **[Release Workflow](docs/RELEASE.md)** - Publishing and version management
- **[Architecture](docs/architecture.md)** - System design and components
- **[Development Guide](docs/DEVELOPMENT.md)** - Contributing and development setup

## Core Capabilities

### 1. Initialization & Setup

Initialize your project documentation and knowledge graph:
```bash
# Initialize in current directory
ktme init

# Initialize with specific service name
ktme init --service my-api-service

# Initialize in a different directory
ktme init --path /path/to/project --service my-service

# Force re-initialization
ktme init --service my-service --force
```

What `ktme init` creates:
- **Documentation structure** - `docs/` directory with README, architecture, API docs, and changelog
- **Knowledge graph** - Service entry in SQLite database for tracking features and documentation
- **Subdirectories** - `docs/api/`, `docs/guides/`, `docs/examples/` for organized documentation

### 2. Git Integration

Extract changes from various sources:
- Staged changes (`--staged`)
- Specific commits (`--commit abc123`)
- Commit ranges (`--range main..feature`)
- Pull/Merge requests (`--pr 123`)

### 3. Documentation Generation

Generate documentation with templates:
```bash
# Use custom template
ktme generate --service api --template api-docs

# Generate changelog
ktme generate --service api --type changelog

# Output to specific file
ktme generate --service api --output docs/changelog.md
```

The `generate` command automatically:
- Initializes the knowledge graph if not already done
- Creates feature entries for significant code changes
- Tracks documentation history and relationships

### 4. Smart Updates

Update existing documentation intelligently:
```bash
# Update specific section
ktme update --service api --section "Breaking Changes"

# Smart merge with existing content
ktme update --service api --staged
```

### 5. MCP Server

Run as MCP server for AI agents:
```bash
# Start server
ktme mcp start

# Available tools:
# - ktme_generate_documentation
# - ktme_update_documentation
# - ktme_list_services
# - ktme_search_features
```

## Development

### Quick Commands

```bash
# Test changes (fast - only new modules)
make test-changes

# Run all checks (format, lint, tests)
make pre-release

# Development cycle
make dev
```

### Publishing

```bash
# Automated release workflow
make release
```

See [docs/RELEASE.md](docs/RELEASE.md) for complete release documentation.

## Architecture

```
┌─────────────┐     ┌──────────────┐     ┌───────────────┐
│   Git CLI   │────▶│  Extractors  │────▶│  Generators   │
│  GitHub API │     │ (Diff/PR/MR) │     │  (Templates)  │
│  GitLab API │     └──────────────┘     └───────────────┘
└─────────────┘              │                     │
                             ▼                     ▼
                     ┌──────────────┐     ┌───────────────┐
                     │   Storage    │     │    Writers    │
                     │ (TOML/SQLite)│     │ (MD/Confluence)│
                     └──────────────┘     └───────────────┘
```

## Recent Updates (v0.1.0)

### New Features
- ✅ Template engine with variable substitution
- ✅ Smart documentation merging by section
- ✅ GitHub PR extraction and integration
- ✅ GitLab MR extraction and integration
- ✅ Confluence writer with Markdown conversion
- ✅ Enhanced Markdown writer with section updates

### Implementation Details
- **34 new tests** (all passing)
- **10 files modified** with new functionality
- **Zero compilation errors** after strict linting

See [CHANGELOG.md](CHANGELOG.md) for complete version history.

## Contributing

We welcome contributions! Please see our [Development Guide](docs/DEVELOPMENT.md).

```bash
# Setup development environment
git clone https://github.com/FreePeak/ktme.git
cd ktme
make setup

# Run tests
make test-changes

# Submit PR
make pre-release
git push
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Support

- **Issues**: [GitHub Issues](https://github.com/FreePeak/ktme/issues)
- **Discussions**: [GitHub Discussions](https://github.com/FreePeak/ktme/discussions)
- **Documentation**: [docs/](docs/)

---

**Built with** ❤️ **using Rust**
