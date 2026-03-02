---
name: git-worktree
version: "1.0.0"
description: Manage Git worktrees for parallel development workflows without switching branches
---

# Git Worktree Management

Efficiently work on multiple branches simultaneously using Git worktrees.

## When to Use

Use this skill when:
- Working on multiple features simultaneously
- Reviewing PRs while working on a feature
- Need to test something on another branch
- Hotfixing production while developing
- Comparing branches side-by-side

## What Are Worktrees?

Git worktrees allow you to checkout multiple branches at the same time in different directories.

```
main project/
├── .git (repository)
├── src/ (main branch)
└── ...

../project-feature-a/ (worktree for feature-a branch)
├── .git (file pointing to main repo)
├── src/
└── ...

../project-hotfix/ (worktree for hotfix branch)
├── .git (file pointing to main repo)
├── src/
└── ...
```

**Benefits**:
- No need to stash or commit incomplete work
- Instant branch switching (just cd to directory)
- Each worktree has its own build artifacts
- Can run different versions simultaneously

## Basic Commands

### Create Worktree

```bash
# Create worktree for new branch
git worktree add ../project-feature-a -b feature-a

# Create worktree for existing branch
git worktree add ../project-hotfix hotfix-branch

# Create worktree at specific commit
git worktree add ../project-debug abc123

# Create worktree with detached HEAD
git worktree add --detach ../project-temp
```

### List Worktrees

```bash
# List all worktrees
git worktree list

# Output:
# /path/to/main           abc123 [main]
# /path/to/feature-a      def456 [feature-a]
# /path/to/hotfix         ghi789 [hotfix-branch]
```

### Remove Worktree

```bash
# Remove worktree after merging branch
git worktree remove ../project-feature-a

# Force remove (if untracked files exist)
git worktree remove --force ../project-feature-a

# Prune deleted worktrees from .git/worktrees
git worktree prune
```

### Move Worktree

```bash
# Move worktree to new location
git worktree move ../project-feature-a ../new-location
```

## Workflow Patterns

### Pattern 1: Feature Development

```bash
# Start working on main branch
cd ~/projects/myproject
git checkout main

# Need to start feature-a while main build is running
git worktree add ../myproject-feature-a -b feature-a

# Work on feature-a
cd ../myproject-feature-a
# Make changes, commit, test...

# Meanwhile, go back to main for something else
cd ~/projects/myproject
# Main branch is exactly as you left it

# When feature-a is done
cd ../myproject-feature-a
git push origin feature-a

# Create PR, get approved, merge

# Clean up
cd ~/projects/myproject
git pull
git worktree remove ../myproject-feature-a
```

### Pattern 2: PR Review

```bash
# Working on feature branch
cd ~/projects/myproject-feature-a

# Need to review PR #123
git worktree add ../myproject-pr-123 pr-123

# Review and test
cd ../myproject-pr-123
# Run tests, check code...

# Post review
cd ~/projects/myproject-feature-a

# Clean up after PR merged
git worktree remove ../myproject-pr-123
```

### Pattern 3: Hotfix During Development

```bash
# Working on long-running feature
cd ~/projects/myproject-feature-complex

# Production issue! Need to hotfix
git worktree add ../myproject-hotfix -b hotfix-urgent origin/main

# Fix and test
cd ../myproject-hotfix
# Fix the issue...
git commit -m "Fix critical production issue"
git push origin hotfix-urgent

# Deploy hotfix, continue feature work
cd ~/projects/myproject-feature-complex
# Feature work still intact, no stashing needed

# After hotfix merged
git worktree remove ../myproject-hotfix
```

### Pattern 4: Comparing Branches

```bash
# Create worktrees for comparison
git worktree add ../myproject-v1 v1.0
git worktree add ../myproject-v2 v2.0

# Run both versions side-by-side
cd ../myproject-v1
./run-server.sh --port 8001 &

cd ../myproject-v2
./run-server.sh --port 8002 &

# Compare behavior...

# Clean up
git worktree remove ../myproject-v1
git worktree remove ../myproject-v2
```

## Advanced Usage

### Shared Configuration

Worktrees share the same `.git` directory, which means:

✅ **Shared**:
- Git configuration
- Hooks (pre-commit, etc.)
- Refs (branches, tags)

❌ **Not Shared**:
- Working directory files
- Index (staging area)
- Build artifacts
- IDE settings

### IDE Setup

Each worktree can have its own IDE instance:

```bash
# VSCode
cd ../myproject-feature-a
code . --user-data-dir=/tmp/vscode-feature-a

# JetBrains IDEs
cd ../myproject-feature-a
idea .  # Opens separate instance
```

### Build Artifacts

Build artifacts don't conflict:

```bash
# Main worktree
cd ~/projects/myproject
cargo build --release
ls target/release/myproject  # Main build

# Feature worktree
cd ../myproject-feature-a
cargo build --release
ls target/release/myproject  # Feature build (different!)

# Both builds can exist simultaneously
```

### Gitignore for Worktrees

Add to `.gitignore`:

```gitignore
# Ignore other worktree directories
../project-*/*.log
../project-*/target/
../project-*/node_modules/
```

## Best Practices

### Naming Convention

```bash
# Good: Descriptive names
git worktree add ../myproject-feature-auth -b feature/auth
git worktree add ../myproject-fix-logging -b fix/logging
git worktree add ../myproject-review-pr-456 origin/pr/456

# Bad: Cryptic names
git worktree add ../temp1 -b x
git worktree add ../test -b y
```

### Directory Structure

```bash
# Option 1: Sibling directories (recommended)
~/projects/
├── myproject/           # main worktree
├── myproject-feature-a/ # feature worktree
└── myproject-hotfix/    # hotfix worktree

# Option 2: Subdirectory in main project
~/projects/myproject/
├── .git/
├── src/
├── worktrees/
│   ├── feature-a/
│   └── hotfix/
└── ...

# Option 3: Centralized worktree directory
~/worktrees/
├── myproject-main/
├── myproject-feature-a/
└── myproject-hotfix/
```

### Cleanup Schedule

```bash
# Regularly prune deleted worktrees
git worktree prune

# List worktrees with details
git worktree list --porcelain

# Script to clean up old worktrees
#!/bin/bash
for wt in $(git worktree list --porcelain | grep '^worktree' | cut -d' ' -f2); do
    if [ ! -d "$wt" ]; then
        echo "Pruning deleted worktree: $wt"
        git worktree prune
        break
    fi
done
```

## Common Scenarios

### Scenario: Multiple Active Features

```bash
# Feature A in progress
git worktree add ../project-feature-a -b feature-a

# Feature B needs to start
git worktree add ../project-feature-b -b feature-b

# Feature C hotfix for A
git worktree add ../project-fix-a -b fix/feature-a-a-issue

# Work on all three
cd ../project-feature-a    # Work on A
cd ../project-feature-b    # Work on B
cd ../project-fix-a        # Fix A's issue

# Merge order: fix-a → feature-a → main
cd ../project-feature-a
git merge fix/feature-a-a-issue
git push

# Clean up
git worktree remove ../project-fix-a
git worktree remove ../project-feature-a  # After merge
```

### Scenario: Benchmarking

```bash
# Create worktrees for old and new versions
git worktree add ../project-baseline v1.0
git worktree add ../project-optimized optimized-branch

# Run benchmarks
cd ../project-baseline
cargo bench --bench my_benchmark > baseline.txt

cd ../project-optimized
cargo bench --bench my_benchmark > optimized.txt

# Compare
diff baseline.txt optimized.txt

# Clean up
git worktree remove ../project-baseline
git worktree remove ../project-optimized
```

## Troubleshooting

### Worktree Already Exists

```bash
# Error: 'feature-a' is already checked out
git worktree add ../project-feature-a feature-a

# Solution: Remove existing worktree first
git worktree list
git worktree remove /path/to/existing/worktree
# OR just cd to it and continue working there
```

### Untracked Files in Worktree

```bash
# Error: cannot remove worktree: untracked files present

# Option 1: Commit or stash files
cd ../project-feature-a
git add .
git stash

# Option 2: Force remove
git worktree remove --force ../project-feature-a
```

### Branch Already Exists

```bash
# Error: a branch named 'feature-a' already exists

# Create worktree from existing branch
git worktree add ../project-feature-a feature-a

# OR use different branch name
git worktree add ../project-feature-a-v2 -b feature-a-v2
```

### Detached HEAD

```bash
# Worktree is in detached HEAD state
cd ../project-temp
git status
# HEAD detached at abc123

# Create branch from current state
git checkout -b new-branch-name

# OR reset to a branch
git checkout main
```

## Integration with Kilo

This skill integrates with Kilo's workflow:

1. **Parallel Work**: Switch between tasks without stashing
2. **Evidence**: Compare branches with file:line references
3. **Minimal Context**: Each worktree has focused scope
4. **Clean Workflow**: No incomplete commits just to switch

Use with other skills:
- **systematic-debugging**: Create worktree for debugging
- **code-review**: Review PRs in separate worktree
- **test-driven-development**: Test different implementations

## Quick Reference

```bash
# Create
git worktree add <path> <branch>
git worktree add <path> -b <new-branch>

# List
git worktree list
git worktree list --porcelain

# Remove
git worktree remove <path>
git worktree remove --force <path>
git worktree prune

# Move
git worktree move <old-path> <new-path>

# Lock (prevent pruning)
git worktree lock <path>
git worktree unlock <path>
```