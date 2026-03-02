# Kilo Skills Directory

This directory contains specialized skills that enhance Kilo's capabilities for specific workflows and tasks.

## What Are Skills?

Skills are reusable instruction packages that teach Kilo how to perform specialized tasks consistently. Each skill is a self-contained directory with a `SKILL.md` file containing:

- **Metadata**: YAML frontmatter with name, version, description
- **Instructions**: Structured guidance for the specific workflow
- **Examples**: Real-world usage patterns
- **Best Practices**: Do's and don'ts

## Available Skills

### Core Skills

| Skill | Description | When to Use |
|-------|-------------|-------------|
| **SKILL.md** (root) | Core Kilo configuration | Always loaded, defines base behavior |
| **systematic-debugging** | 4-phase debugging framework | Debugging issues, investigating bugs |
| **code-review** | Comprehensive review checklist | Reviewing PRs, evaluating changes |
| **test-driven-development** | Red-Green-Refactor TDD | Writing tests first, TDD workflow |
| **git-worktree** | Parallel branch management | Working on multiple features |
| **documentation** | Documentation standards | Writing docs, API references |
| **template** | Skill creation template | Creating new skills |

### Skill Categories

#### Development Workflow
- `systematic-debugging` - Structured debugging approach
- `test-driven-development` - TDD methodology
- `git-worktree` - Parallel development

#### Quality Assurance
- `code-review` - Review framework with security, performance, maintainability checks
- `documentation` - Documentation standards and best practices

## Using Skills

### Automatic Loading

Skills are automatically loaded based on:

1. **Description matching**: AI reads description and loads when relevant
2. **Explicit reference**: Mention skill name in request
3. **Context detection**: AI detects relevant context

### Explicit Reference

```
Use the systematic-debugging skill to investigate this null pointer exception
```

### Multiple Skills

```
Use code-review and documentation skills to review and document these changes
```

## Creating Custom Skills

### Quick Start

1. Copy the template:
```bash
cp -r .kilo/skills/template .kilo/skills/my-skill
```

2. Edit the skill:
```bash
vim .kilo/skills/my-skill/SKILL.md
```

3. Update frontmatter:
```yaml
---
name: my-skill
version: "1.0.0"
description: Clear description of when to use this skill
---
```

4. Add your instructions in markdown

### Skill Structure

```
my-skill/
├── SKILL.md          # Required: Main skill file
├── templates/        # Optional: Template files
├── examples/         # Optional: Example files
└── resources/        # Optional: Additional resources
```

### Best Practices

✅ **Do**:
- Keep skills focused (one topic per skill)
- Use clear, descriptive names
- Write detailed descriptions for AI detection
- Include examples and templates
- Keep under 500 lines

❌ **Don't**:
- Create overly broad skills
- Duplicate content across skills
- Skip the description field
- Add unnecessary complexity

### Frontmatter Fields

```yaml
---
name: skill-name              # Required: unique identifier
version: "1.0.0"              # Required: semantic version
description: When to use      # Required: AI uses this for auto-loading
author: Your Name             # Optional: skill author
tags: [debugging, testing]    # Optional: categorization
---
```

## Skill Loading Priority

Skills load in this order (later overrides earlier):

1. Global skills (`~/.kilo/skills/`)
2. Project skills (`.kilo/skills/`)
3. Explicitly requested skills

## Skill Activation Modes

### Mode 1: Automatic (AI-Detect)
AI reads description and loads when context matches.

```yaml
description: "Use when debugging null pointer exceptions"
```

### Mode 2: Explicit Reference
Loaded only when explicitly mentioned.

```
Use the my-custom-skill to process this data
```

### Mode 3: Always Active
Loaded in every session (use sparingly).

```yaml
---
name: core-workflow
description: "Core workflow that applies to all tasks"
alwaysApply: true
---
```

## Cross-Platform Compatibility

Skills written here follow the Agent Skills standard and work with:

- **Kilo** (primary)
- **Claude Code** (Anthropic)
- **Cursor** (IDE integration)
- **OpenCode** (terminal-based)
- Other Agent Skills-compatible tools

## Skill Discovery

To see what skills are available:

```bash
ls -la .kilo/skills/
cat .kilo/skills/*/SKILL.md | grep "^name:"
```

## Troubleshooting

### Skill Not Loading

1. Check YAML syntax in frontmatter
2. Verify `name` and `description` fields exist
3. Ensure file is named `SKILL.md` (case-sensitive)
4. Check skill directory structure

### Conflicting Skills

If multiple skills apply:
1. Explicitly request specific skill
2. More specific description wins
3. Later loaded skill takes precedence

### Skills Directory Not Found

```bash
mkdir -p .kilo/skills
```

## Contributing Skills

To add skills to this project:

1. Create skill directory in `.kilo/skills/`
2. Add `SKILL.md` with proper frontmatter
3. Test with Kilo
4. Document in this README

## Resources

- **Agent Skills Spec**: https://agentskills.io
- **Kilo Documentation**: https://kilo.ai/docs
- **Template**: `.kilo/skills/template/SKILL.md`

## Examples from Other Projects

For inspiration, see:
- [Anthropic's Skills Repository](https://github.com/anthropics/skills)
- [Cursor Rules Gallery](https://cursor.com/rules)
- [OpenCode Examples](https://opencode.ai/examples)

---

**Note**: This skills system is compatible with the Agent Skills open standard, ensuring your customizations work across multiple AI coding assistants.