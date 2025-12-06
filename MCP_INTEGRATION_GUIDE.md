# 🚀 MCP Server Integration Guide for Claude Code

## Overview

ktme includes a complete MCP (Model Context Protocol) server that allows you to integrate automated documentation generation directly with Claude Code and other AI assistants.

## 🎯 Document Provider Modes

### 1. **Local Markdown Mode** (✅ Implemented)
- **What**: Saves documentation as Markdown files on your local filesystem
- **Where**: `./docs/{service}_{type}.md`
- **Use case**: Perfect for personal projects, local development, version-controlled documentation
- **Advantages**:
  - ✅ No external dependencies
  - ✅ Files tracked in Git
  - ✅ Full control over content
  - ✅ Easy to edit and maintain

### 2. **Cloud Confluence Mode** (🔄 In Progress)
- **What**: Publishes documentation directly to Confluence pages
- **Where**: Your Confluence workspace
- **Use case**: Enterprise teams, knowledge management, shared documentation
- **Advantages**:
  - ✅ Enterprise-ready knowledge base
  - ✅ Team collaboration features
  - ✅ Access controls and permissions
  - ✅ Rich formatting and attachments

## 🚀 Quick Start

### 1. Build and Install ktme

```bash
# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

### 2. Start the MCP Server

#### **Option A: Daemon Mode** (Recommended for Claude Code)

```bash
# Start HTTP daemon on port 3000
ktme mcp start --daemon

# Output:
# 🚀 ktme MCP server started in daemon mode on http://localhost:3000
# 💡 Add to Claude Code: mcp://localhost:3000
```

#### **Option B: STDIO Mode** (For direct connections)

```bash
# Start in STDIO mode
ktme mcp start

# Output:
# 🚀 ktme MCP server started in STDIO mode
# 💡 Ready for Claude Code integration
```

### 3. Configure AI Provider

The MCP server automatically detects AI providers from environment variables:

#### **For OpenAI:**
```bash
export OPENAI_API_KEY="sk-your-openai-key"
export OPENAI_MODEL="gpt-4"  # Optional
```

#### **For Claude:**
```bash
export ANTHROPIC_API_KEY="sk-ant-your-claude-key"
export CLAUDE_MODEL="claude-3-sonnet-20240229"  # Optional
```

### 4. Test the Server

```bash
# Check server status
ktme mcp status

# Stop the server
ktme mcp stop
```

## 🔗 Claude Code Integration

### Method 1: HTTP Transport (Recommended)

1. **Start the MCP server in daemon mode:**
   ```bash
   ktme mcp start --daemon
   ```

2. **Add to Claude Code:**
   - Open Claude Code settings
   - Go to MCP Servers
   - Add new server: `mcp://localhost:3000`
   - Server will auto-discover available tools

### Method 2: STDIO Transport

1. **Add to Claude Code:**
   - Open Claude Code settings
   - Go to MCP Servers
   - Add new server: `/path/to/ktme mcp start`
   - Use full path to ktme binary

### Method 3: Configuration File

Create or edit `~/.config/claude-code/mcp_servers.json`:

```json
{
  "ktme": {
    "command": "/path/to/ktme",
    "args": ["mcp", "start", "--config", "/path/to/config.toml"],
    "env": {
      "OPENAI_API_KEY": "your-api-key"
    }
  }
}
```

## 🛠️ Available MCP Tools

The ktme MCP server provides these tools that you can use directly in Claude Code:

### 1. `extract_changes`
Extract code changes from Git commits, PRs, or staged changes.

```json
{
  "name": "extract_changes",
  "description": "Extract code changes from Git",
  "parameters": {
    "source": "commit|pr|staged",
    "identifier": "commit-hash|pr-number|staged",
    "provider": "github|gitlab|bitbucket"
  }
}
```

**Example Usage:**
```
Extract the changes from the latest commit
```

### 2. `generate_documentation`
Generate documentation from code changes using AI.

```json
{
  "name": "generate_documentation",
  "description": "Generate AI-powered documentation",
  "parameters": {
    "service": "service-name",
    "doc_type": "changelog|api-doc|readme|general",
    "provider": "markdown|confluence",
    "format": "markdown|json"
  }
}
```

**Example Usage:**
```
Generate documentation for the authentication service as a changelog in markdown format
```

### 3. `read_changes`
Read previously extracted changes from a file.

```json
{
  "name": "read_changes",
  "description": "Read extracted changes from file",
  "parameters": {
    "file_path": "/path/to/diff.json"
  }
}
```

**Example Usage:**
```
Read the changes from /tmp/latest_changes.json
```

## 📝 Example Workflows in Claude Code

### Workflow 1: Automatic Changelog Generation

```
Claude: Extract the changes from the latest commit and generate a changelog for the user-service
```

This will:
1. ✅ Extract latest Git changes
2. ✅ Generate AI-powered changelog
3. ✅ Save as `docs/user-service_changelog.md`

### Workflow 2: API Documentation

```
Claude: Generate API documentation for the payment service using the staged changes, save as markdown
```

This will:
1. ✅ Extract staged changes
2. ✅ Generate comprehensive API documentation
3. ✅ Save as `docs/payment-service_api-doc.md`

### Workflow 3: PR Documentation

```
Claude: Read changes from /tmp/pr_123_changes.json and create documentation for the analytics service
```

This will:
1. ✅ Load changes from file
2. ✅ Generate documentation for analytics service
3. ✅ Save as `docs/analytics-service_general.md`

## ⚙️ Configuration Options

### Server Configuration

You can create a custom configuration file:

```toml
# ktme-config.toml
[mcp]
server_name = "my-ktme-server"
transport = "http"  # or "stdio"
port = 3000

[ai]
# Auto-detected from environment variables
# provider = "openai"  # or "anthropic"

[documentation]
default_provider = "markdown"
default_format = "markdown"
base_path = "./docs"

[logging]
level = "info"
```

### Using Custom Configuration

```bash
# Start with custom config
ktme mcp start --config ./ktme-config.toml --daemon
```

## 🔍 Troubleshooting

### Server Won't Start

```bash
# Check if port is already in use
lsof -i :3000

# Kill existing process
kill -9 <pid>

# Try different port
ktme mcp start --daemon --config <(echo '[mcp]\nport = 3001')
```

### AI Provider Not Detected

```bash
# Check environment variables
env | grep -E "(OPENAI|ANTHROPIC)_API_KEY"

# Test AI client directly
ktme generate --service test --doc-type general
```

### Claude Code Connection Issues

1. **Check server is running:**
   ```bash
   ktme mcp status
   ```

2. **Test connectivity:**
   ```bash
   curl http://localhost:3000/status
   ```

3. **Check Claude Code logs for connection errors**

### Documentation Not Generated

1. **Check AI provider is configured**
2. **Check server has write permissions to docs directory**
3. **Look at server logs for error messages**

```bash
# Enable debug logging
export KTME_LOG_LEVEL=debug
ktme mcp start --daemon
```

## 🎯 Best Practices

### 1. **Local Development Setup**
```bash
# Start server with debug logging
export KTME_LOG_LEVEL=debug
export OPENAI_API_KEY="your-key"
ktme mcp start --daemon

# Monitor logs
tail -f ~/.local/share/ktme/logs/server.log
```

### 2. **Team Collaboration**
- Use Confluence provider for shared knowledge
- Set consistent naming conventions
- Configure proper access controls

### 3. **Performance Optimization**
- Use HTTP transport for multiple clients
- Cache generated documentation
- Batch process multiple changes

### 4. **Security**
- Store API keys in environment variables
- Use HTTPS for remote connections
- Rotate API keys regularly

## 📚 Advanced Usage

### Custom Templates

Create custom prompt templates in `~/.config/ktme/templates/`:

```markdown
<!-- custom_changelog.md -->
# {{SERVICE}} Changes

## What's New
{{CHANGE_SUMMARY}}

## Technical Details
{{TECHNICAL_DETAILS}}

## Impact
{{USER_IMPACT}}
```

### Automation Scripts

Create automation scripts for repetitive tasks:

```bash
#!/bin/bash
# auto-docs.sh

# Extract changes and generate docs for all services
for service in user-service payment-service auth-service; do
  echo "Generating docs for $service..."
  ktme generate --service "$service" --doc-type changelog
  ktme generate --service "$service" --doc_type api-doc
done

echo "Documentation generation complete!"
```

### CI/CD Integration

Add to your CI pipeline:

```yaml
# .github/workflows/docs.yml
name: Generate Documentation

on:
  push:
    branches: [main]

jobs:
  docs:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v2

    - name: Setup Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Build ktme
      run: cargo build --release

    - name: Generate Documentation
      env:
        OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
      run: |
        ./target/release/ktme generate --service myapp --doc-type changelog

    - name: Commit Documentation
      run: |
        git add docs/
        git commit -m "docs: auto-generated documentation"
        git push
```

## 🔮 Future Enhancements

- **Slack Integration**: Post documentation to Slack channels
- **Teams Integration**: Publish to Microsoft Teams
- **Wiki.js Support**: Alternative to Confluence
- **Analytics**: Documentation usage metrics
- **Multi-language Support**: Documentation in different languages
- **Version Management**: Track documentation versions

## 📞 Support

If you encounter issues:

1. Check this guide first
2. Look at the [GitHub Issues](https://github.com/your-org/ktme/issues)
3. Create detailed bug reports with logs
4. Join our [Discord Community](https://discord.gg/ktme)

---

**Happy documenting! 🎉**