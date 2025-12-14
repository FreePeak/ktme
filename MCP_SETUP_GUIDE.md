# MCP Server Setup Guide

This guide shows how to configure ktme's MCP server with Claude Code, Cursor, and Windsurf.

## Prerequisites

1. Install ktme:
```bash
cargo install --path /path/to/ktme
# or build from source:
cargo build --release
```

2. Initialize configuration:
```bash
ktme config init
```

## Configuration

### Claude Code

Edit your MCP settings file at `~/.claude/mcp_settings.json`:

```json
{
  "mcpServers": {
    "ktme": {
      "command": "ktme",
      "args": ["mcp", "start", "--stdio"],
      "env": {
        "KTME_LOG_LEVEL": "info"
      }
    }
  }
}
```

### Cursor

Create or edit `.cursor/mcp.json` in your project directory:

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

Add to your MCP settings (location varies by installation):

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

## Available MCP Tools

The ktme MCP server provides the following tools:

### 1. `read_changes`
Read extracted code changes from Git.

**Parameters:**
- `source` (string, required): Commit hash, "staged", or file path

**Example:**
```python
# Read changes from a specific commit
result = mcp.call_tool("read_changes", {"source": "abc123"})

# Read staged changes
result = mcp.call_tool("read_changes", {"source": "staged"})
```

### 2. `get_service_mapping`
Get documentation location for a service.

**Parameters:**
- `service` (string, required): Service name

**Example:**
```python
result = mcp.call_tool("get_service_mapping", {"service": "api-gateway"})
```

### 3. `list_services`
List all mapped services.

**Parameters:** None

**Example:**
```python
result = mcp.call_tool("list_services", {})
```

### 4. `generate_documentation`
Generate documentation from code changes.

**Parameters:**
- `service` (string, required): Service name
- `changes` (string, required): JSON string of extracted changes
- `format` (string, optional): Output format ("markdown" or "json")

**Example:**
```python
changes = json.dumps(extracted_changes)
result = mcp.call_tool("generate_documentation", {
    "service": "my-service",
    "changes": changes,
    "format": "markdown"
})
```

### 5. `update_documentation`
Update existing documentation.

**Parameters:**
- `service` (string, required): Service name
- `doc_path` (string, required): Path to documentation file
- `content` (string, required): Content to append/update

**Example:**
```python
result = mcp.call_tool("update_documentation", {
    "service": "my-service",
    "doc_path": "/path/to/docs.md",
    "content": "## New Section\n\nThis is new content."
})
```

## Usage Examples

### Example 1: Document Staged Changes

```python
# In Claude Code, Cursor, or Windsurf:

# 1. Read staged changes
changes = mcp.call_tool("read_changes", {"source": "staged"})

# 2. Get service mapping
mapping = mcp.call_tool("get_service_mapping", {"service": "my-service"})

# 3. Generate documentation
docs = mcp.call_tool("generate_documentation", {
    "service": "my-service",
    "changes": changes,
    "format": "markdown"
})
```

### Example 2: Update Documentation

```python
# 1. Read changes from a commit
changes = mcp.call_tool("read_changes", {"source": "abc123"})

# 2. Generate update content
update = mcp.call_tool("generate_documentation", {
    "service": "api-gateway",
    "changes": changes,
    "format": "markdown"
})

# 3. Update the documentation file
mcp.call_tool("update_documentation", {
    "service": "api-gateway",
    "doc_path": "/path/to/api-docs.md",
    "content": update
})
```

## Setting Up Service Mappings

Before using the MCP tools effectively, you need to map your services:

```bash
# Add a service with a local markdown file
ktme mapping add my-service --file /path/to/docs/README.md

# Add a service with a Confluence URL
ktme mapping add my-service --url https://confluence.company.com/display/SERVICE/Home

# List all mappings
ktme mapping list

# Get a specific mapping
ktme mapping get my-service
```

## Environment Variables

Optional environment variables you can set:

- `KTME_LOG_LEVEL`: Logging level (debug, info, warn, error)
- `OPENAI_API_KEY`: For OpenAI AI provider
- `ANTHROPIC_API_KEY`: For Claude AI provider
- `KTME_MCP_API_KEY`: Alternative API key configuration

## Troubleshooting

### Server Not Starting

1. Check if ktme is in your PATH
2. Verify installation: `ktme --version`
3. Check logs: Set `KTME_LOG_LEVEL=debug`

### Tools Not Working

1. Ensure service mappings exist: `ktme mapping list`
2. Check if the git repository is initialized
3. Verify file paths are correct

### Permission Issues

Make sure the binary has execute permissions:
```bash
chmod +x ./target/release/ktme
```

## Testing the MCP Server

You can test the MCP server manually:

```bash
# Start in STDIO mode (for Claude Code/Cursor/Windsurf)
ktme mcp start --stdio

# Start in daemon mode (for HTTP connections)
ktme mcp start --daemon

# Check status
ktme mcp status

# Stop daemon
ktme mcp stop
```

## Next Steps

1. Map your services using `ktme mapping add`
2. Configure your AI assistant with the MCP server
3. Start using the MCP tools to automate documentation generation

For more information, see the [main README](README.md).