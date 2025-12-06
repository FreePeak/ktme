use crate::git::diff::ExtractedDiff;
use crate::error::Result;

pub struct PromptTemplates;

impl PromptTemplates {
    pub fn generate_documentation_prompt(
        diff: &ExtractedDiff,
        doc_type: &str,
        context: Option<&str>,
    ) -> Result<String> {
        let base_prompt = match doc_type {
            "changelog" => Self::changelog_prompt(),
            "api-doc" => Self::api_doc_prompt(),
            "readme" => Self::readme_prompt(),
            "commit-message" => Self::commit_message_prompt(),
            _ => Self::general_prompt(),
        };

        let diff_summary = Self::format_diff_summary(diff);
        let context_section = context.map(|c| format!("\nAdditional Context:\n{}\n", c)).unwrap_or_default();

        Ok(format!(
            "{}\n\n{}{}\n\nChanges:\n{}",
            base_prompt, context_section, diff_summary, Self::format_diff_content(diff)
        ))
    }

    fn changelog_prompt() -> String {
        r#"You are a technical writer generating changelog entries. Based on the provided Git diff, create a clear, concise changelog entry following this format:

## [Version] - [Date]

### Added
- New features and functionality

### Changed
- Modifications to existing functionality

### Fixed
- Bug fixes

### Removed
- Deprecated or removed features

Guidelines:
- Use present tense ("Adds support for..." not "Added support for...")
- Focus on user impact, not implementation details
- Group related changes together
- Keep entries brief but informative
- Use bullet points with hyphens"#.to_string()
    }

    fn api_doc_prompt() -> String {
        r#"You are a technical writer documenting API changes. Based on the provided Git diff, update the API documentation following this format:

# API Documentation

## Overview
[Brief description of what this API does]

## Changes
[Document what changed in this version]

## Endpoints
### [Method] [Path]
**Description**: [What this endpoint does]

**Request Parameters:**
- `param1` (type): Description
- `param2` (type): Description

**Response:**
```json
{
  "field": "description"
}
```

**Example:**
```bash
curl -X METHOD url \
  -H "Header: value" \
  -d '{"key": "value"}'
```

Guidelines:
- Document all new or modified endpoints
- Include request/response schemas
- Provide practical examples
- Note any breaking changes
- Use clear, consistent formatting"#.to_string()
    }

    fn readme_prompt() -> String {
        r#"You are a technical writer updating README documentation. Based on the provided Git diff, update the README content to reflect the changes.

Focus on:
1. **Feature descriptions** - What does this do?
2. **Installation/Setup** - Any new requirements?
3. **Usage examples** - How to use the new functionality
4. **Configuration** - Any new settings?
5. **Migration notes** - Breaking changes?

Format:
# Project Name

## Description
[Brief project description]

## Features
- Existing features
- [NEW] New features from this diff

## Installation
[Installation instructions]

## Usage
```bash
# Example commands
```

## Configuration
[Configuration details]

Guidelines:
- Keep it user-friendly
- Include practical examples
- Highlight new features clearly
- Maintain consistent formatting"#.to_string()
    }

    fn commit_message_prompt() -> String {
        r#"You are helping write better commit messages. Based on the provided Git diff, generate a conventional commit message.

Format:
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]

Types:
- feat: A new feature
- fix: A bug fix
- docs: Documentation only changes
- style: Changes that do not affect the meaning of the code
- refactor: A code change that neither fixes a bug nor adds a feature
- perf: A code change that improves performance
- test: Adding missing tests or correcting existing tests
- chore: Changes to the build process or auxiliary tools

Guidelines:
- Use the imperative mood ("add" not "added")
- Keep the first line under 50 characters
- Wrap the body at 72 characters
- Explain what and why, not how"#.to_string()
    }

    fn general_prompt() -> String {
        r#"You are a technical documentation specialist. Based on the provided Git diff, generate clear, comprehensive documentation that explains:

1. What changed in the code
2. Why these changes were made
3. How the changes affect users or other developers
4. Any important implementation details

Requirements:
- Be clear and concise
- Focus on the user/developer perspective
- Include relevant code examples
- Explain any breaking changes
- Use proper formatting with headers and code blocks

Generate documentation in Markdown format."#.to_string()
    }

    fn format_diff_summary(diff: &ExtractedDiff) -> String {
        format!(
            "Commit: {}\nAuthor: {}\nTimestamp: {}\nMessage: {}\nFiles changed: {} (+{}/-{})",
            diff.identifier,
            diff.author,
            diff.timestamp,
            diff.message.trim(),
            diff.summary.total_files,
            diff.summary.total_additions,
            diff.summary.total_deletions
        )
    }

    pub fn format_diff_content(diff: &ExtractedDiff) -> String {
        let mut content = String::new();

        for file in &diff.files {
            content.push_str(&format!(
                "\n## File: {} ({})\n```\n{}\n```\n",
                file.path,
                file.status,
                file.diff
            ));
        }

        content
    }
}