# KTME - Knowledge Transfer Me

> AI-powered knowledge tree map that saves tokens for coding agents

[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ktme.svg)](https://crates.io/crates/ktme)

KTME is a CLI tool and MCP server that serves as a **knowledge-based tree map system** for developers and AI agents. Its core mission is to **save tokens per AI agent context window** by providing structured, indexed, always-up-to-date knowledge about a codebase and its documentation.

KTME bridges local documentation (markdown files, knowledge graphs) with cloud documentation platforms (Notion, Confluence) via fetch-and-sync capabilities, ensuring documents are **always up-to-date** through the MCP server protocol.

## The Problem

### The Context Window Problem

AI coding agents (Copilot, Cursor, Claude, etc.) face a fundamental limitation: **context window size**. When working with large codebases, agents must repeatedly:

- Re-scan entire codebases to understand architecture
- Re-read documentation scattered across files, wikis, and cloud platforms
- Re-discover relationships between features, services, and modules
- Waste tokens on information that has already been processed

### The Documentation Fragmentation Problem

Teams maintain documentation across multiple platforms:
- **Local markdown** files in the repo (`docs/`, `README.md`)
- **Confluence** / Atlassian wiki for team knowledge
- **Notion** for product specs and design docs
- **GitHub/GitLab** wikis and issue trackers

These documents are often out of sync, not indexed for machine consumption, and scattered without a unified hierarchy.

## Features

- **Knowledge Tree Map** - Hierarchical representation of services, features, and relationships
- **Auto-Initialization** - Automatically creates documentation structure and knowledge graph
- **Smart Documentation Generation** - AI-powered documentation from Git diffs, commits, and PRs
- **Multiple Cloud Integrations** - Notion, Confluence, GitHub, and GitLab support
- **Cloud Sync Engine** - Bi-directional sync with conflict detection and resolution
- **Template System** - Customizable Markdown templates with variable substitution
- **MCP Server** - Model Context Protocol server with 16+ tools for AI agent integration
- **Dual Storage** - TOML and SQLite backends for flexibility
- **Service Mapping** - Organize documentation by service/project
- **Search** - Keyword and feature-based search with relevance scoring

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

# Scan codebase to auto-populate features and relationships
ktme scan --service my-service

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

### MCP Server

```bash
# Start MCP server for AI agents
ktme mcp start

# Available tools include:
# - ktme_get_knowledge_tree    # Get hierarchical knowledge map
# - ktme_get_feature_context   # Get context for specific feature
# - ktme_generate_documentation
# - ktme_update_documentation
# - ktme_list_services
# - ktme_search_services
# - ktme_search_by_feature
# - ktme_search_by_keyword
# - ktme_detect_service
# - ktme_scan_documentation
# - And more...
```

### Cloud Sync

```bash
# Sync Notion documents
ktme sync --provider notion --workspace "Product Specs"

# Sync Confluence pages
ktme sync --provider confluence --space DOCS

# Check sync status
ktme sync --status
```

### Configuration

Create `~/.config/ktme/config.toml`:

```toml
[ai]
provider = "openai"         # openai, anthropic, gemini, mock
api_key = "sk-xxxxx"        # or use KTME_AI_API_KEY env var
model = "gpt-4"
embedding_model = "text-embedding-3-small"

[notion]
api_key = "ntn_xxxxx"       # or use KTME_NOTION_API_KEY env var
default_workspace = "Product"

[confluence]
base_url = "https://your-company.atlassian.net/wiki"
api_token = "your-api-token"
space_key = "DOCS"

[git]
github_token = "ghp_xxxxx"
gitlab_token = "glpat_xxxxx"

[sync]
auto_sync = false
conflict_strategy = "timestamp"  # local_wins, remote_wins, timestamp, manual
sync_on_generate = true
```

## Documentation

- **[Quick Start Guide](docs/PUBLISH.md)** - Get started in 5 minutes
- **[Release Workflow](docs/RELEASE.md)** - Publishing and version management
- **[Architecture](docs/architecture.md)** - System design and components
- **[Development Guide](docs/DEVELOPMENT.md)** - Contributing and development setup

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
                     │ (TOML/SQLite)│     │(MD/Confluence)│
                     └──────────────┘     └───────────────┘
                             │
                             ▼
                     ┌──────────────┐     ┌───────────────┐
                     │ Knowledge Tree│     │ Cloud Sync    │
                     │    Engine     │     │   Engine      │
                     └──────────────┘     └───────────────┘
                                                  │
                          ┌──────────────┐        │
                          │  Notion API   │◀───────┘
                          │  Confluence   │
                          └──────────────┘
```

## Current Status (v0.3.0)

### Implemented Modules

| Module | Status | Description |
| --- | --- | --- |
| `cli/` | ✅ Implemented | 9 CLI commands |
| `mcp/` | ✅ Implemented | MCP server with 16+ tools |
| `storage/` | ✅ Implemented | SQLite backend |
| `git/` | ✅ Implemented | Git diff extraction |
| `doc/` | ✅ Implemented | Markdown + Confluence writers |
| `ai/` | ⚠️ Partial | OpenAI + mock only |
| `research/` | ⚠️ Basic | Reference finder, tech detector |
| `enhance/` | ⚠️ Stub | Basic sync, link enricher |
| `analysis/` | ⚠️ Basic | Doc parser, coverage analysis |
| `config/` | ✅ Implemented | TOML-based configuration |

### What's Implemented

- Service detection from Git repo, package files
- AI-powered documentation generation from Git diffs
- Knowledge graph models (Feature, FeatureRelation, KnowledgeNode)
- MCP server with stdio + HTTP/SSE transport
- Keyword and feature search with relevance scoring
- GitHub PR and GitLab MR extraction
- Confluence write support
- Markdown file generation with section updates
- Template engine with variable substitution

### What's NOT Yet Implemented

| Feature | Status |
| --- | --- |
| Knowledge Tree Map traversal engine | Models defined, no query API |
| Notion integration | Not started |
| Cloud Sync Engine | Stub only |
| Vector/Semantic Search | Text-only matching |
| Feature Relationship Engine | Models exist, no CRUD |
| Context Builder for AI agents | Models exist, no assembly |
| Multi-AI Provider Support | OpenAI + mock only |
| Confluence Fetch (Read) | Write-only |

## Roadmap

### Phase 1: Automated Discovery & Cold Start (Current Priority)
**Goal**: Populate the database with meaningful data without manual entry.
- [ ] CLI Command: Implement `ktme scan` to run heuristics and save to DB
- [ ] Relationship Detection: Enhance `CodebaseScanner` to map imports to feature dependencies
- [ ] Idempotency: Ensure scanning multiple times doesn't duplicate data

### Phase 2: Cloud Sync Engine & Notion Integration
**Goal**: Bi-directional sync with cloud documentation platforms.
- [ ] Sync Infrastructure: Create `cloud_sync_status` to track SHA-256 hashes
- [ ] Notion Provider: Implement page/database fetching and Markdown conversion
- [ ] Sync Workflow: Implement conflict detection (local-wins vs. remote-wins)

### Phase 3: AI Context Optimization & Semantic Search
**Goal**: Optimize token usage and improve relevance.
- [ ] Semantic Search: Implement embedding generation and vector search
- [ ] Context Builder: Intelligent assembly of "context packages" for AI agents
- [ ] Token-Aware Trimming: Summarize or truncate content to fit context windows

### Phase 4: Polish & Multi-Provider Support
**Goal**: Enterprise-readiness and extensibility.
- [ ] Anthropic/Gemini Providers: Add more LLM options
- [ ] Caching: LRU caching for expensive search results and knowledge graphs
- [ ] Validation: Deep link checking and documentation coverage analysis

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
