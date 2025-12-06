# 🧪 Testing Guide for ktme

## Overview

ktme has a comprehensive testing framework with multiple layers of testing:

1. **Unit Tests** - Test individual components in isolation
2. **Integration Tests** - Test complete CLI workflows
3. **End-to-End Tests** - Test with real AI providers
4. **Manual Testing** - Quick verification of functionality

## 🚀 Quick Start

### 1. Run All Tests
```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run AI component tests
cargo test ai::tests
```

### 2. Manual Testing
```bash
# Test CLI help
cargo run -- --help

# Test Git extraction (working)
cargo run -- extract --commit HEAD --output /tmp/test_diff.json

# Test AI generation (requires API key)
export OPENAI_API_KEY="your-api-key"
cargo run -- generate --commit HEAD --service "test" --doc-type changelog
```

## 🧪 Test Categories

### 1. Unit Tests (`src/*/tests.rs`)

**What they test:**
- Individual functions and methods
- Data serialization/deserialization
- Configuration loading
- Error handling

**Examples:**
```rust
#[test]
fn test_ai_client_fails_without_key() {
    let result = AIClient::new();
    assert!(result.is_err());
}

#[test]
fn test_extracted_diff_serialization() {
    let diff = ExtractedDiff { /* ... */ };
    let json = serde_json::to_string(&diff)?;
    let parsed: ExtractedDiff = serde_json::from_str(&json)?;
    assert_eq!(parsed.identifier, "test-commit");
}
```

**Run:**
```bash
cargo test test_ai_client_fails_without_key
cargo test test_extracted_diff_serialization
```

### 2. Integration Tests (`tests/integration_tests.rs`)

**What they test:**
- Complete CLI workflows
- File I/O operations
- Command-line argument parsing
- Error propagation through the system

**Test Categories:**

#### ✅ Working Tests
```bash
cargo test integration_tests::test_cli_help          # CLI help command
cargo test integration_tests::test_generate_command_without_ai_key  # AI key validation
cargo test integration_tests::test_generate_command_with_input_file   # File input processing
```

#### 🔄 Git-Dependent Tests (May fail in CI)
```bash
cargo test integration_tests::test_extract_command                 # Git extraction
cargo test integration_tests::test_extract_output_to_file           # File output
cargo test integration_tests::test_extract_and_generate_pipeline     # Full workflow
```

### 3. AI Provider Tests

**What they test:**
- AI client initialization
- Provider configuration
- Network error handling
- Provider-specific functionality

**Without API Keys:**
```bash
cargo test ai::tests::test_ai_client_fails_without_key
cargo test ai::tests::test_openai_config_defaults
cargo test ai::tests::test_claude_config_creation
cargo test ai::tests::test_openai_provider_creation
```

**With API Keys:**
```bash
export OPENAI_API_KEY="sk-..."
export ANTHROPIC_API_KEY="sk-ant-..."
cargo test ai::tests::test_openai_provider_with_network
```

## 🔧 Testing Strategies

### 1. Test Without AI Keys (Recommended for Development)

Most functionality can be tested without actual API keys:

```bash
# Test the complete pipeline structure
cargo run -- extract --commit HEAD --output /tmp/diff.json

# Create a test input file
cat > /tmp/test_diff.json << 'EOF'
{
  "source": "test",
  "identifier": "test-commit",
  "timestamp": "2025-12-06T00:00:00Z",
  "author": "test@example.com",
  "message": "Test commit",
  "files": [],
  "summary": {
    "total_files": 0,
    "total_additions": 0,
    "total_deletions": 0
  }
}
EOF

# Test generate command with input (will fail at AI call, but pipeline works)
cargo run -- generate --input /tmp/test_diff.json --service "test" --doc-type general
```

### 2. Mock AI Responses (For CI/CD)

Create mock responses for testing:
```bash
# Create a mock response file
cat > /tmp/mock_ai_response.txt << 'EOF'
# Documentation for test

## Overview
This is a test response from the AI provider.

## Changes
- Added new functionality
- Fixed bugs
- Improved performance

## Files Modified
- src/main.rs: Updated main function
- Cargo.toml: Updated dependencies
EOF

# Test with template processing
cargo run -- generate --input /tmp/test_diff.json --service "test" --template /path/to/template.txt
```

### 3. End-to-End Testing with Real AI

For full end-to-end testing with real AI providers:

```bash
# Set up OpenAI
export OPENAI_API_KEY="sk-..."
export OPENAI_MODEL="gpt-4"
export OPENAI_MAX_TOKENS="2048"

# Test complete workflow
cargo run -- extract --commit HEAD --output /tmp/latest_changes.json
cargo run -- generate --input /tmp/latest_changes.json --service "ktme" --doc-type changelog --output /tmp/changelog.md

# Verify output
cat /tmp/changelog.md
```

## 🐛 Troubleshooting Tests

### Common Issues

#### 1. Git Reference Errors
```
Error: Git(Error { code: -3, klass: 4, message: "reference 'refs/tags/HEAD' not found" })
```

**Solution:** This happens when running tests outside a Git repository or in a fresh repo. Tests that need Git history should be run in a proper Git repository.

#### 2. AI Provider Not Found
```
Error: No AI provider configured. Set OPENAI_API_KEY or ANTHROPIC_API_KEY environment variable.
```

**Solution:** This is expected for tests without API keys. The functionality should still work up to the AI call.

#### 3. Network Timeouts
```
Error: Network error: operation timed out
```

**Solution:** Check internet connection and API key validity.

### Test Isolation

Each test should be independent:

```rust
// ✅ Good - Isolated test
#[test]
fn test_diff_serialization() {
    let diff = create_test_diff();
    let json = serde_json::to_string(&diff).unwrap();
    assert!(json.contains("identifier"));
}

// ❌ Bad - Depends on external state
#[test]
fn test_git_operations() {
    // This depends on being in a specific Git repository
    let reader = GitReader::new(None).unwrap();
}
```

## 📊 Test Coverage

### Currently Tested

- ✅ CLI help command parsing
- ✅ AI client initialization and error handling
- ✅ Configuration loading and validation
- ✅ Diff data serialization/deserialization
- ✅ File I/O operations
- ✅ Command argument validation

### Needs Testing

- 🔄 Git operations (tests exist but need proper Git environment)
- 🔄 AI provider network calls (require API keys)
- 🔄 MCP protocol implementation
- 🔄 Database operations
- 🔄 Confluence provider API calls

## 🎯 Testing Checklist

When adding new features, ensure you have tests for:

- [ ] **Happy path**: Basic functionality works
- [ ] **Error cases**: Invalid inputs handled gracefully
- [ ] **Edge cases**: Boundary conditions
- [ ] **Integration**: Works with other components
- [ ] **CLI**: Command-line interface works
- [ ] **Serialization**: Data can be saved/loaded

### Example Test for New Feature

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_new_feature_basic_functionality() {
        // Arrange
        let input = create_test_input();

        // Act
        let result = new_feature_function(input);

        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("expected content"));
    }

    #[test]
    fn test_new_feature_error_handling() {
        // Arrange
        let invalid_input = create_invalid_input();

        // Act
        let result = new_feature_function(invalid_input);

        // Assert
        assert!(result.is_err());
        match result.err().unwrap() {
            KtmeError::ValidationError(msg) => {
                assert!(msg.contains("expected error"));
            }
            _ => panic!("Expected validation error"),
        }
    }
}
```

## 🚀 Continuous Integration

For CI/CD pipelines:

```yaml
# .github/workflows/test.yml
name: Tests
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
    - uses: actions/checkout@v2

    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable

    - name: Run unit tests
      run: cargo test --lib

    - name: Run integration tests
      run: cargo test --test integration_tests

    # Optional: Run AI tests with secrets
    - name: Run AI tests
      env:
        OPENAI_API_KEY: ${{ secrets.OPENAI_API_KEY }}
      run: cargo test ai::tests
      if: secrets.OPENAI_API_KEY != ''
```

## 📝 Manual Testing Script

Create a comprehensive manual testing script:

```bash
#!/bin/bash
# test_ktme.sh

echo "🧪 ktme Manual Testing Script"

echo "1. Testing CLI help..."
cargo run -- --help

echo "2. Testing extract command..."
cargo run -- extract --commit HEAD --output /tmp/test_diff.json

echo "3. Testing extract output..."
ls -la /tmp/test_diff.json
head -5 /tmp/test_diff.json

echo "4. Testing generate without AI key (should fail)..."
cargo run -- generate --service "test" --doc-type general

echo "5. Testing with input file..."
cat > /tmp/test_input.json << 'EOF'
{
  "source": "manual-test",
  "identifier": "test-commit",
  "timestamp": "2025-12-06T00:00:00Z",
  "author": "test@example.com",
  "message": "Manual test commit",
  "files": [
    {
      "path": "src/main.rs",
      "status": "modified",
      "additions": 10,
      "deletions": 5,
      "diff": "+some new code\\n-removed code"
    }
  ],
  "summary": {
    "total_files": 1,
    "total_additions": 10,
    "total_deletions": 5
  }
}
EOF

cargo run -- generate --input /tmp/test_input.json --service "test" --doc-type general

echo "✅ Manual testing complete!"
```

Run it with:
```bash
chmod +x test_ktme.sh
./test_ktme.sh
```

This comprehensive testing approach ensures ktme works reliably across different environments and use cases!