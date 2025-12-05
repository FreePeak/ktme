# ktme - Knowledge Transfer Me

[![MIT License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![MCP](https://img.shields.io/badge/MCP-Compatible-green.svg)](https://modelcontextprotocol.io/)

A Rust-based CLI tool and MCP server for automated documentation generation from code changes. Integrates with AI coding assistants like **Claude Code**, **Cursor**, **Windsurf**, and other MCP-compatible clients.

## Table of Contents

- [Overview](#overview)
- [Core Concepts](#core-concepts)
- [Features](#features)
- [Installation](#installation)
- [AI Agent Integration](#ai-agent-integration)
- [Quick Start](#quick-start)
- [Configuration](#configuration)
- [MCP Tools](#mcp-tools)
- [Knowledge Search (RAG)](#knowledge-search-rag)
- [Usage Examples](#usage-examples)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

## Overview

**ktme** (Knowledge Transfer Me) is a CLI tool and MCP server that bridges code changes with documentation. It uses the [Model Context Protocol (MCP)](https://modelcontextprotocol.io/) to communicate with AI coding assistants, enabling them to:

- Read and analyze code changes from Git
- Generate documentation based on diffs
- Update existing docs intelligently
- Publish to Markdown files or Confluence

```mermaid
graph LR
    subgraph AI Assistants
        Claude[Claude Code]
        Cursor[Cursor]
        Windsurf[Windsurf]
    end

    subgraph ktme MCP Server
        Server[MCP Server]
        Tools[Tools]
    end

    Git[Git Repository] --> Tools
    Claude --> Server
    Cursor --> Server
    Windsurf --> Server
    Server --> Tools
    Tools --> Docs[Documentation]
```

### How It Works

1. **You configure** ktme as an MCP server in your AI assistant (Claude Code, Cursor, etc.)
2. **AI assistant connects** to ktme via the Model Context Protocol
3. **AI uses ktme tools** to read code changes, query mappings, and generate docs
4. **Documentation is created** in your preferred format (Markdown, Confluence)

## Core Concepts

### Multi-Source Change Extraction

ktme can extract code changes from multiple sources:

```bash
# From commits
ktme extract --commit abc123

# From staged changes
ktme extract --staged

# From pull requests
ktme extract --pr 456 --provider github

# From commit ranges
ktme extract --commit v1.0.0..v1.1.0
```

### Service-to-Documentation Mapping

Map your services to their documentation locations:

```toml
# ~/.config/ktme/mappings.toml
[[services]]
name = "api-gateway"
path = "my-company/api-gateway"
docs = [
  { type = "markdown", location = "my-company/api-gateway/README.md" },
  { type = "confluence", location = "https://confluence.example.com/display/DEV/APIGateway" }
]
```

### MCP Integration

ktme exposes tools through the Model Context Protocol, enabling AI assistants to:

- Read extracted code changes
- Query service-document mappings
- Generate documentation based on changes
- Update existing documentation intelligently

## Features

| Feature | Description |
|---------|-------------|
| Multi-Source Extraction | Extract from commits, staged changes, PRs, or commit ranges |
| AI-Powered Generation | Generate documentation using MCP-connected AI models |
| Multi-Format Output | Support for Markdown files and Confluence pages |
| Service Mapping | Local storage for service-to-documentation mappings |
| Git Platform Support | GitHub, GitLab, and Bitbucket integration |
| Template System | Customizable documentation templates |
| Flexible Configuration | TOML-based config with environment variable support |
| **Knowledge Search** | Cross-team documentation search with RAG capabilities |
| **Confluence Sync** | Incremental sync from Confluence to local cache |
| **Hybrid Search** | Combines FTS5 keyword search with semantic embeddings |

## Installation

### Prerequisites

- Rust 1.70 or higher
- Git 2.30 or higher
- Access to an AI model API (Claude, GPT, etc.)

### From Source

```bash
git clone https://github.com/FreePeak/ktme.git
cd ktme
cargo build --release
```

### Using Cargo

```bash
cargo install --path .
```

### Verify Installation

```bash
ktme --version
```

## AI Agent Integration

ktme uses the [Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk) to implement the Model Context Protocol, enabling seamless communication with AI coding assistants.

### Claude Code

Add to your Claude Code MCP settings (`~/.claude/mcp_settings.json`):

```json
{
  "mcpServers": {
    "ktme": {
      "command": "ktme",
      "args": ["mcp", "start"],
      "env": {
        "KTME_MCP_API_KEY": "your-api-key"
      }
    }
  }
}
```

### Cursor

Add to Cursor's MCP configuration (`.cursor/mcp.json`):

```json
{
  "servers": {
    "ktme": {
      "command": "ktme",
      "args": ["mcp", "start", "--stdio"]
    }
  }
}
```

### Windsurf

Add to Windsurf's MCP settings:

```json
{
  "mcp": {
    "servers": {
      "ktme": {
        "command": "ktme",
        "args": ["mcp", "start", "--stdio"]
      }
    }
  }
}
```

### SSE Mode (HTTP)

For HTTP-based connections, start ktme in SSE mode:

```bash
ktme mcp start --sse --host 127.0.0.1 --port 8080
```

Then configure your AI assistant to connect to `http://127.0.0.1:8080`.

## Quick Start

### 1. Initialize Configuration

```bash
ktme config init
```

### 2. Set Up API Key

```bash
export KTME_MCP_API_KEY="your-api-key"
```

### 3. Map Your Service

```bash
ktme mapping add my-service --file ~/projects/my-service/README.md
```

### 4. Generate Documentation

```bash
cd ~/projects/my-service
ktme generate --commit HEAD --service my-service
```

## Configuration

### Configuration File

Location: `~/.config/ktme/config.toml`

```toml
[general]
default_directory = "~/work/my-company"
log_level = "info"

[git]
default_branch = "main"
include_merge_commits = false

[mcp]
model = "claude-3-5-sonnet-20241022"
max_tokens = 4096
temperature = 0.7

[documentation]
default_format = "markdown"
template_directory = "~/.config/ktme/templates"

[confluence]
base_url = "https://your-domain.atlassian.net"
auth_type = "token"
space_key = "DEV"
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `KTME_MCP_API_KEY` | AI model API key |
| `KTME_MCP_MODEL` | Model identifier |
| `CONFLUENCE_API_TOKEN` | Confluence authentication token |
| `CONFLUENCE_USERNAME` | Confluence username |
| `KTME_LOG_LEVEL` | Logging level (debug, info, warn, error) |
| `OPENAI_API_KEY` | OpenAI API key for embeddings (optional) |

### Knowledge Search Configuration

```toml
[knowledge]
# Cache location (default: ~/.config/ktme/knowledge.db)
cache_path = "~/.config/ktme/knowledge.db"

# Embedding provider: "openai" or "local"
embedding_provider = "local"

# Confluence spaces to sync
sync_spaces = ["MOBILE", "BACKEND", "SHARED"]

# Auto-sync interval in hours (0 = manual only)
auto_sync_interval = 0

[knowledge.chunking]
# Chunk size for RAG (tokens)
chunk_size = 512
chunk_overlap = 50
```

## MCP Tools

ktme exposes the following tools through MCP:

### Documentation Tools

| Tool | Description |
|------|-------------|
| `read_changes` | Read extracted diff from a file or Git |
| `generate_documentation` | Create new documentation from changes |
| `update_documentation` | Modify existing documentation |

### Mapping Tools

| Tool | Description |
|------|-------------|
| `get_service_mapping` | Retrieve document URL for a service |
| `list_services` | List all mapped services |
| `add_mapping` | Add a new service-to-doc mapping |

### Git Tools

| Tool | Description |
|------|-------------|
| `extract_commit` | Extract changes from a specific commit |
| `extract_pr` | Extract changes from a pull request |
| `list_commits` | List commits in a range |

### Knowledge Search Tools

| Tool | Description |
|------|-------------|
| `search_knowledge` | Search documentation using natural language queries |
| `get_document` | Retrieve full document content by ID or URL |
| `list_documents` | List documents filtered by team, tags, or source |
| `sync_documents` | Trigger sync from Confluence or other sources |

### Feature Mapping Tools

| Tool | Description |
|------|-------------|
| `get_feature` | Get a feature/screen with all related documentation |
| `map_feature_document` | Link a feature to a documentation page |
| `list_features` | List all features filtered by team |

## Knowledge Search (RAG)

ktme includes a powerful knowledge search system that enables teams to search documentation across Confluence and local files using natural language queries through MCP-connected AI assistants.

### Architecture

```mermaid
graph TB
    subgraph "Users (via Cursor/Claude)"
        Backend[Backend Team]
        Mobile[Mobile Team]
    end

    subgraph "MCP Server"
        SearchTool[search_knowledge]
        GetDocTool[get_document]
        SyncTool[sync_documents]
    end

    subgraph "Local Cache"
        SQLite[(SQLite)]
        FTS[FTS5 Index]
        Embeddings[Embeddings]
    end

    subgraph "Source of Truth"
        Confluence[(Confluence)]
    end

    Backend --> SearchTool
    Mobile --> SearchTool
    SearchTool --> FTS
    SearchTool --> Embeddings
    GetDocTool --> SQLite
    Confluence -->|"Incremental Sync"| SQLite
    SQLite --> FTS
    SQLite --> Embeddings
```

### How It Works

1. **Confluence is the source of truth** - All documentation lives in Confluence
2. **Local SQLite cache** - Documents are synced to a local cache for fast searching
3. **Hybrid search** - Combines keyword matching (FTS5) with semantic search (embeddings)
4. **Incremental sync** - Only fetches documents modified since last sync

### Knowledge Search Tools

| Tool | Description |
|------|-------------|
| `search_knowledge` | Search documentation using natural language |
| `get_document` | Retrieve full document content by ID |
| `list_documents` | List documents by team, tags, or source |
| `sync_documents` | Sync documents from Confluence |

### Syncing Documents

```bash
# Initial full sync from Confluence
ktme sync --space MOBILE --full
ktme sync --space BACKEND --full

# Incremental sync (only changed documents)
ktme sync --space MOBILE

# Sync all configured spaces
ktme sync --all
```

### Searching Knowledge

```bash
# CLI search
ktme search "food home list resto"
ktme search "payment integration" --team mobile
ktme search "authentication flow" --tag "feature:auth"

# Via MCP (AI assistant uses these tools)
# search_knowledge("food home list resto")
# search_knowledge("how does the order API work", team="backend")
```

### Tagging Documents

Documents can be tagged for better organization:

```bash
# Add tags to indexed documents
ktme tag DOC_ID --team mobile --tag "screen:food_home"
ktme tag DOC_ID --tag "feature:restaurant_list"

# Search by tags
ktme search --tag "screen:*" --team mobile
```

### Cache Location

The knowledge cache is stored locally per user:

```
~/.config/ktme/
  config.toml           # Configuration
  ktme.db               # Service mappings
  knowledge.db          # Knowledge search cache
    documents           # Cached document content
    documents_fts       # Full-text search index
    document_chunks     # RAG chunks for context
    embeddings          # Vector embeddings
    sync_state          # Last sync timestamps
```

The cache is ephemeral and can be regenerated by re-syncing from Confluence.

### Feature Mapping

Features represent logical units like mobile screens, components, or business flows. They can be mapped to documentation and services.

```bash
# Add a feature (mobile screen)
ktme feature add food_home_screen \
    --team mobile \
    --display-name "Food Home Screen" \
    --aliases "food home,resto list,home screen"

# Map feature to documentation
ktme feature map food_home_screen \
    --doc-url "https://confluence.company.com/display/MOBILE/Food+Home"

# Link feature to backend service
ktme feature link food_home_screen --service restaurant-api

# Get feature with all related docs
ktme feature get food_home_screen
```

**Feature-Service-Document Relationships:**

```
Feature (Mobile)          Service (Backend)       Document (Confluence)
----------------          -----------------       ---------------------
food_home_screen    <---> restaurant-api    ---> "Food Home Design Doc"
                    <---> order-api         ---> "Restaurant API Reference"
                                            ---> "Order Flow Doc"
```

### Example: Mobile Team Searching Documentation

```
Mobile Dev in Cursor: "Find docs about the food home list resto screen"

AI Assistant calls: search_knowledge("food home list resto screen")

System searches:
1. Features FTS: Matches "food_home_screen" via alias "food home"
2. Documents FTS: Matches documents containing "resto list"
3. Returns merged results with feature->document mappings

Results returned:
1. "Food Home Screen - Restaurant List"
   URL: https://confluence.company.com/display/MOBILE/Food+Home
   Team: mobile
   Feature: food_home_screen
   Related Services: [restaurant-api, order-api]
   Summary: "The food home screen displays a list of nearby restaurants..."

2. "Restaurant API Integration"
   URL: https://confluence.company.com/display/BACKEND/Restaurant+API
   Team: backend
   Service: restaurant-api
   Related Features: [food_home_screen, resto_list]
   Summary: "API endpoints for fetching restaurant data..."
```

## Usage Examples

### Document a Feature Branch

```bash
# Extract changes from feature branch
ktme extract --commit main..feature/new-api --output /tmp/feature.json

# Generate documentation
ktme generate --input /tmp/feature.json --service api-gateway --type api-doc
```

### Update Changelog from PR

```bash
# Extract PR changes
ktme extract --pr 456 --provider github

# Update changelog section
ktme update --pr 456 --service user-service --section "Changelog"
```

### Publish to Confluence

```bash
# Configure Confluence
export CONFLUENCE_API_TOKEN="your-token"
ktme config set confluence.base_url "https://company.atlassian.net"

# Map and generate
ktme mapping add auth-service --url "https://company.atlassian.net/wiki/spaces/DEV/pages/12345"
ktme generate --commit abc123 --service auth-service --format confluence
```

### Document Staged Changes

```bash
# Stage your changes
git add src/main.rs

# Generate documentation before committing
ktme generate --staged --service my-service

# Commit both code and docs
git add .
git commit -m "feat: Add new feature with documentation"
```

## Architecture

```mermaid
graph TB
    subgraph CLI Layer
        CLI[CLI Interface]
        Config[Config Manager]
    end

    subgraph Core Layer
        Git[Git Reader]
        Extractor[Diff Extractor]
        Generator[Doc Generator]
        Storage[Mapping Storage]
    end

    subgraph MCP Layer
        Server[MCP Server]
        Tools[MCP Tools]
    end

    subgraph Output Layer
        MD[Markdown Writer]
        Confluence[Confluence Writer]
    end

    CLI --> Git
    CLI --> Config
    Config --> Storage
    Git --> Extractor
    Extractor --> Server
    Server --> Tools
    Tools --> Generator
    Storage --> Generator
    Generator --> MD
    Generator --> Confluence
```

### Component Overview

| Component | Purpose |
|-----------|---------|
| CLI Interface | Command parsing and user interaction |
| Git Reader | Extract changes from Git repositories |
| Diff Extractor | Parse and structure Git diffs |
| MCP Server | Model Context Protocol communication |
| Doc Generator | Transform AI output to documentation |
| Mapping Storage | Service-to-document mappings |
| Writers | Output to Markdown or Confluence |

## Supported Platforms

| Platform | PR Extraction | Commit Extraction |
|----------|---------------|-------------------|
| GitHub | Yes | Yes |
| GitLab | Yes | Yes |
| Bitbucket | Yes | Yes |
| Local Git | N/A | Yes |

## Troubleshooting

### Common Issues

**MCP server connection failed**
```bash
ktme mcp status
ktme mcp start
```

**Service mapping not found**
```bash
ktme mapping list
ktme mapping add my-service --file ~/path/to/docs/README.md
```

**Enable debug logging**
```bash
export KTME_LOG_LEVEL="debug"
ktme --verbose generate --commit abc123 --service my-service
```

## Contributing

Contributions are welcome!

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development

```bash
# Build
cargo build

# Run tests
cargo test

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [Model Context Protocol](https://modelcontextprotocol.io/) for the MCP specification
- [Rust MCP SDK](https://github.com/modelcontextprotocol/rust-sdk) for MCP server implementation
- [git2-rs](https://github.com/rust-lang/git2-rs) for Git integration
- [clap](https://github.com/clap-rs/clap) for CLI parsing

---

**Built with Rust. Powered by AI. Documentation made simple.**
