# .kilo Directory

Configuration and skills directory for Kilo AI coding assistant.

## Directory Structure

```
.kilo/
├── AGENTS.md           # Global Kilo configuration
├── skills/             # Specialized skills
│   ├── README.md       # Skills documentation
│   ├── SKILL.md        # Core Kilo skill
│   ├── systematic-debugging/
│   ├── code-review/
│   ├── test-driven-development/
│   ├── git-worktree/
│   ├── documentation/
│   └── template/
└── rules/              # Reserved for future rule-based configurations
```

## What's Here

### AGENTS.md
Global configuration file that defines:
- Personal preferences
- Development workflow
- Zero tolerance rules
- Language-specific configurations
- Documentation standards
- Git safety protocols

### skills/
Directory containing specialized skills that enhance Kilo's capabilities:
- **Core skill** (`SKILL.md`): Base configuration and principles
- **Workflow skills**: Debugging, TDD, code review
- **Utility skills**: Git worktree, documentation
- **Template**: Starting point for custom skills

### rules/
Reserved directory for future rule-based activation patterns (similar to Cursor's `.cursor/rules/`).

## How It Works

### Configuration Loading Order

```
1. ~/.kilo/AGENTS.md        (global preferences)
2. ./AGENTS.md              (project-specific rules)
3. ~/.kilo/skills/          (global skills)
4. ./.kilo/skills/          (project skills)
```

### Skill Activation

Skills are activated based on:
1. **AI detection**: Description field matches context
2. **Explicit reference**: User mentions skill name
3. **Always active**: Special skills that load every time

## Quick Start

### View Available Skills
```bash
cat .kilo/skills/README.md
ls .kilo/skills/
```

### Create Custom Skill
```bash
cp -r .kilo/skills/template .kilo/skills/my-skill
vim .kilo/skills/my-skill/SKILL.md
```

### Edit Global Config
```bash
vim .kilo/AGENTS.md
```

## Integration with Other Tools

This configuration is compatible with:

| Tool | Compatibility | Notes |
|------|--------------|-------|
| **Kilo** | ✅ Full | Primary target |
| **Claude Code** | ✅ Full | Follows Agent Skills standard |
| **Cursor** | ⚠️ Partial | Can reference skills, uses own rules |
| **OpenCode** | ✅ Full | Reads AGENTS.md and skills |
| **VS Code** | ⚠️ Indirect | Via extension integrations |

## Best Practices

### Keep Skills Focused
- One topic per skill
- Under 500 lines
- Clear descriptions

### Maintain Consistency
- Use consistent formatting
- Follow naming conventions
- Keep skills updated

### Version Control
- Commit `.kilo/` directory
- Track skill changes
- Document in commit messages

## Customization

### Personal Preferences
Edit `.kilo/AGENTS.md` for:
- Communication style
- Workflow preferences
- Language-specific settings

### Project-Specific Rules
Edit `./AGENTS.md` (project root) for:
- Build commands
- Test commands
- Code style guidelines
- Project architecture

### Custom Skills
Create in `.kilo/skills/` for:
- Reusable workflows
- Team standards
- Specialized processes

## Resources

- **Kilo Documentation**: https://kilo.ai/docs
- **Agent Skills Spec**: https://agentskills.io
- **Examples**: https://github.com/anthropics/skills

## Troubleshooting

### Skills Not Loading
```bash
# Check syntax
cat .kilo/skills/*/SKILL.md | head -20

# Verify structure
ls -la .kilo/skills/
```

### Configuration Not Applied
```bash
# Check file exists
cat .kilo/AGENTS.md

# Verify YAML in skills
grep -A 5 "^---" .kilo/skills/*/SKILL.md
```

### Clear Cache
If changes aren't reflected:
```bash
# Restart Kilo session
# Skills load fresh on each session
```

---

**Note**: The `.kilo/` directory follows the Agent Skills open standard, ensuring compatibility across multiple AI coding assistants.