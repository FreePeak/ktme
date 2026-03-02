---
name: documentation
version: "1.0.0"
description: Standards and workflows for creating and maintaining high-quality documentation
---

# Documentation Standards

Comprehensive guidelines for writing clear, maintainable, and useful documentation.

## When to Use

Use this skill when:
- Writing README files
- Creating API documentation
- Writing user guides
- Documenting architecture decisions
- Creating inline code documentation
- Writing changelogs and release notes

## Documentation Types

### 1. Code Documentation (Inline)

**Purpose**: Explain how code works to developers

#### Rust Documentation

```rust
/// Calculates the total price for a list of items.
///
/// # Arguments
///
/// * `items` - A slice of items to calculate the total for
///
/// # Returns
///
/// The total price as a floating-point number
///
/// # Examples
///
/// ```
/// let items = vec![
///     Item { name: "Widget", price: 10.0 },
///     Item { name: "Gadget", price: 20.0 },
/// ];
/// assert_eq!(calculate_total(&items), 30.0);
/// ```
///
/// # Panics
///
/// Panics if any item has a negative price.
pub fn calculate_total(items: &[Item]) -> f64 {
    items.iter().map(|item| item.price).sum()
}

/// User entity representing a system user.
///
/// This struct stores basic user information and provides
/// methods for user management.
///
/// # Examples
///
/// ```
/// let user = User::new("john@example.com", "John Doe");
/// println!("User: {}", user.name);
/// ```
#[derive(Debug, Clone)]
pub struct User {
    /// User's email address (unique identifier)
    pub email: String,
    /// User's display name
    pub name: String,
    /// Account creation timestamp
    pub created_at: DateTime<Utc>,
}

// Module documentation
//! # My Crate
//!
//! `my_crate` is a collection of utilities for doing X, Y, and Z.
//!
//! ## Features
//!
//! - Feature A
//! - Feature B
//!
//! ## Quick Start
//!
//! ```rust
//! use my_crate::do_something;
//! do_something();
//! ```
```

#### TypeScript/JavaScript Documentation

```typescript
/**
 * Parses a configuration file and returns a validated config object.
 *
 * @param filePath - Path to the configuration file
 * @param options - Optional parsing options
 * @returns Promise resolving to the parsed configuration
 * @throws {ConfigError} If the file cannot be read or parsed
 *
 * @example
 * ```typescript
 * const config = await parseConfig('./config.toml');
 * console.log(config.database.url);
 * ```
 */
export async function parseConfig(
  filePath: string,
  options?: ParseOptions
): Promise<Config> {
  // Implementation
}

/**
 * User role enum defining permission levels.
 */
export enum UserRole {
  /** Can view content */
  Viewer = 'viewer',
  /** Can edit content */
  Editor = 'editor',
  /** Full access to all features */
  Admin = 'admin',
}

/**
 * Configuration options for the API client.
 *
 * @example
 * ```typescript
 * const options: ApiClientOptions = {
 *   baseUrl: 'https://api.example.com',
 *   timeout: 5000,
 *   retries: 3,
 * };
 * ```
 */
export interface ApiClientOptions {
  /** Base URL for API requests */
  baseUrl: string;
  /** Request timeout in milliseconds */
  timeout?: number;
  /** Number of retry attempts for failed requests */
  retries?: number;
}
```

### 2. README Files

**Purpose**: Quick start and project overview

#### README Template

```markdown
# Project Name

Brief description of what the project does and who it's for.

## Features

- Feature 1
- Feature 2
- Feature 3

## Quick Start

### Prerequisites

- Requirement 1
- Requirement 2

### Installation

```bash
# Clone the repository
git clone https://github.com/user/project.git

# Install dependencies
cd project
npm install

# Build
npm run build
```

### Usage

```bash
# Basic usage
npm start

# With options
npm start -- --option value
```

## Configuration

Explain configuration options here.

| Variable | Description | Default |
|----------|-------------|---------|
| `OPTION_1` | Description | `default_value` |
| `OPTION_2` | Description | `default_value` |

## Examples

### Example 1: Basic Usage

```bash
# Command
kilo process --input data.json --output result.json

# Output
Processing complete: 100 records processed
```

### Example 2: Advanced Usage

[More complex example]

## API Reference

[Link to API docs or brief API overview]

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

This project is licensed under the MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- Acknowledgment 1
- Acknowledgment 2
```

### 3. API Documentation

**Purpose**: Document public APIs for consumers

#### REST API Documentation

```markdown
# API Reference

## Authentication

All API requests require authentication via Bearer token:

```http
Authorization: Bearer <your-token>
```

## Endpoints

### List Users

Retrieves a paginated list of users.

**Request**

```http
GET /api/v1/users
```

**Query Parameters**

| Parameter | Type   | Required | Description                          |
|-----------|--------|----------|--------------------------------------|
| `page`    | int    | No       | Page number (default: 1)            |
| `limit`   | int    | No       | Items per page (default: 20, max: 100) |
| `search`  | string | No       | Search query for filtering          |

**Response**

```json
{
  "data": [
    {
      "id": "123",
      "email": "user@example.com",
      "name": "John Doe",
      "created_at": "2024-01-15T10:30:00Z"
    }
  ],
  "pagination": {
    "page": 1,
    "limit": 20,
    "total": 100,
    "total_pages": 5
  }
}
```

**Status Codes**

| Code | Description |
|------|-------------|
| 200  | Success |
| 401  | Unauthorized - invalid or missing token |
| 403  | Forbidden - insufficient permissions |
| 500  | Internal server error |

**Example**

```bash
curl -H "Authorization: Bearer $TOKEN" \
  "https://api.example.com/api/v1/users?page=1&limit=10"
```

### Create User

Creates a new user.

**Request**

```http
POST /api/v1/users
Content-Type: application/json
```

**Body**

```json
{
  "email": "newuser@example.com",
  "name": "Jane Doe",
  "role": "editor"
}
```

**Response**

```json
{
  "id": "124",
  "email": "newuser@example.com",
  "name": "Jane Doe",
  "role": "editor",
  "created_at": "2024-01-15T11:00:00Z"
}
```

**Status Codes**

| Code | Description |
|------|-------------|
| 201  | Created successfully |
| 400  | Bad request - invalid input |
| 401  | Unauthorized |
| 409  | Conflict - email already exists |

## Error Responses

All error responses follow this format:

```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input",
    "details": [
      {
        "field": "email",
        "message": "Invalid email format"
      }
    ]
  }
}
```

## Rate Limiting

- Limit: 100 requests per minute
- Headers: `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset`
```

### 4. Architecture Documentation

**Purpose**: Document design decisions and system architecture

#### Architecture Decision Records (ADRs)

```markdown
# ADR-001: Use SQLite for Local Storage

## Status

Accepted

## Context

We need a local storage solution for caching user data and settings.
Options considered:
- SQLite
- JSON files
- RocksDB

## Decision

We will use SQLite for the following reasons:

1. **ACID compliance**: Ensures data integrity
2. **Query capabilities**: Complex queries with SQL
3. **Performance**: Fast for our use case (< 10K records)
4. **Tooling**: Excellent Rust support via `rusqlite`
5. **Simplicity**: Single file, no server setup

## Consequences

### Positive
- Reliable data storage
- Easy to inspect with standard SQLite tools
- Good performance for read-heavy workloads

### Negative
- Not suitable for large-scale data (> 1M records)
- Requires schema migrations

### Neutral
- Adds SQLite as a dependency
- Need to handle database migrations

## Implementation

Database location: `~/.config/ktme/data.db`

Schema defined in `migrations/` directory.
```

#### System Architecture

```markdown
# System Architecture

## Overview

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│  (command parsing, argument validation, output formatting)   │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────┴──────────────────────────────────────┐
│                    Application Layer                         │
│  (business logic, orchestration, AI integration)             │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐  ┌──────────┐    │
│  │   AI     │  │   Git    │  │   Doc    │  │  Config  │    │
│  │ Provider │  │ Handler  │  │Generator │  │ Manager  │    │
│  └──────────┘  └──────────┘  └──────────┘  └──────────┘    │
└──────────────────────┬──────────────────────────────────────┘
                       │
┌──────────────────────┴──────────────────────────────────────┐
│                   Infrastructure Layer                       │
│  (storage, external APIs, file system)                       │
│                                                              │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
│  │  SQLite  │  │   HTTP   │  │   File   │                  │
│  │   DB     │  │  Client  │  │  System  │                  │
│  └──────────┘  └──────────┘  └──────────┘                  │
└─────────────────────────────────────────────────────────────┘
```

## Components

### CLI Layer

- **Purpose**: Handle user input and output
- **Key Files**: `src/cli/`
- **Dependencies**: `clap`, `tracing`

### Application Layer

- **AI Provider**: Manages AI API calls (OpenAI, Claude)
- **Git Handler**: Analyzes git history and diffs
- **Doc Generator**: Creates documentation from templates
- **Config Manager**: Loads and validates configuration

### Infrastructure Layer

- **SQLite DB**: Persistent storage for services and mappings
- **HTTP Client**: API calls with retry logic
- **File System**: Reads/writes configuration and documents

## Data Flow

1. User runs CLI command
2. CLI layer validates arguments
3. Application layer executes business logic
4. Infrastructure layer persists/retrieves data
5. Results flow back to user

## Key Design Decisions

See ADRs:
- [ADR-001: Use SQLite for Local Storage](./adr/001-sqlite-storage.md)
- [ADR-002: Modular AI Provider System](./adr/002-ai-providers.md)
- [ADR-003: Async-First Architecture](./adr/003-async-architecture.md)
```

### 5. Changelogs

**Purpose**: Track changes between releases

```markdown
# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Feature description with PR/issue reference (#123)

### Changed
- Change description (#124)

### Fixed
- Bug fix description (#125)

## [1.2.0] - 2024-01-15

### Added
- New AI provider: Google Gemini support (#100)
- Configuration validation command: `kilo config validate` (#102)

### Changed
- Improved error messages for API failures (#105)
- Updated dependencies to latest versions (#107)

### Fixed
- Fixed memory leak in long-running sessions (#110)
- Fixed race condition in parallel processing (#112)

### Security
- Updated OpenSSL to address CVE-2024-XXXX (#115)

## [1.1.0] - 2024-01-01

[Previous release notes...]

## [1.0.0] - 2023-12-15

### Added
- Initial release
- Core features: X, Y, Z

[Unreleased]: https://github.com/user/project/compare/v1.2.0...HEAD
[1.2.0]: https://github.com/user/project/compare/v1.1.0...v1.2.0
[1.1.0]: https://github.com/user/project/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/user/project/releases/tag/v1.0.0
```

## Documentation Organization

### Project Structure

```
project/
├── README.md              # Quick start and overview
├── CHANGELOG.md           # Version history
├── CONTRIBUTING.md        # Contribution guidelines
├── docs/
│   ├── index.md           # Documentation home
│   ├── getting-started.md # Detailed setup guide
│   ├── api/               # API documentation
│   │   ├── endpoints.md
│   │   └── schemas.md
│   ├── guides/            # User guides
│   │   ├── basic-usage.md
│   │   └── advanced-usage.md
│   ├── architecture/      # Technical docs
│   │   ├── overview.md
│   │   └── adr/           # Architecture Decision Records
│   │       ├── 001-sqlite-storage.md
│   │       └── 002-ai-providers.md
│   └── reference/         # Reference material
│       ├── configuration.md
│       └── cli-commands.md
└── src/
    └── lib.rs             # Inline documentation (cargo doc)
```

## Writing Guidelines

### Be Clear and Concise

```markdown
# ❌ Bad: Verbose and unclear
In order to utilize this functionality for the purpose of achieving
your desired outcome, you must first ensure that you have properly
configured the system according to the specifications outlined in
the configuration section above.

# ✅ Good: Clear and direct
To use this feature, configure the system as described in Configuration.
```

### Show, Don't Just Tell

```markdown
# ❌ Bad: Abstract description
The function accepts various parameters and returns different types
based on the input.

# ✅ Good: Concrete examples
The function returns different types based on input:

```rust
parse("123")     // Returns: Number(123)
parse("hello")   // Returns: String("hello")
parse("true")    // Returns: Boolean(true)
```
```

### Keep Code Examples Updated

```markdown
# ❌ Bad: Outdated example
```rust
let user = User::new("John");  // Old API
```

# ✅ Good: Current example
```rust
let user = User::new("John", "john@example.com");  // Current API
```

### Use Consistent Formatting

```markdown
# ✅ Consistent structure for all endpoints

## Endpoint Name

Brief description.

### Request
[HTTP method and path]

### Parameters
[Table of parameters]

### Response
[Example response]

### Status Codes
[Table of status codes]

### Example
[curl command]
```

## Documentation as Code

### Generate from Tests

```rust
/// Examples in doc comments are tested by `cargo test`
///
/// ```
/// use mylib::Calculator;
///
/// let calc = Calculator::new();
/// assert_eq!(calc.add(2, 3), 5);
/// assert_eq!(calc.multiply(4, 5), 20);
/// ```
pub struct Calculator { }
```

### Generate API Docs

```bash
# Rust
cargo doc --open

# TypeScript
npm run docs  # With TypeDoc

# Generate OpenAPI spec
cargo run -- openapi-spec > openapi.json
```

## Anti-Patterns to Avoid

❌ **Don't**:
- Write documentation as an afterthought
- Let documentation become outdated
- Duplicate information in multiple places
- Assume knowledge the reader doesn't have
- Skip examples for complex features

✅ **Do**:
- Document as you develop
- Update docs with code changes
- Link to single source of truth
- Explain context and prerequisites
- Provide working examples

## Integration with Kilo

This skill integrates with Kilo's workflow:

1. **Evidence**: Documentation includes `file:line` references
2. **Minimal**: Keep docs focused and up-to-date
3. **Verification**: Test examples in documentation

Use with other skills:
- **code-review**: Review documentation quality
- **test-driven-development**: Test examples in docs