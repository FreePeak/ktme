# Quick Release Guide for KTME v0.2.0

## ⚠️ Important: Rust Project (Not npm)

KTME is a **Rust project** that publishes to **crates.io** (Rust's package registry), not npm.

- ✅ Use: `cargo publish` (for Rust/crates.io)
- ❌ Not: `npm publish` (for Node.js/npm)

## 🚀 Steps to Publish v0.2.0

### 1. Login to crates.io (One-time setup)

```bash
# Get your token from: https://crates.io/settings/tokens
# Click "New Token", name it "KTME publishing", copy the token

cargo login
# Paste your token when prompted
```

### 2. Run the Release Script

```bash
cd /Users/linh.doan/work/harvey/freepeak/ktme
./release.sh
```

This will:
1. ✅ Commit version changes (0.1.0 → 0.2.0)
2. ✅ Push to GitHub
3. ✅ Create git tag v0.2.0
4. ✅ Push tag to GitHub
5. ✅ Publish to crates.io
6. ✅ Install v0.2.0 on your macOS

### 3. Manual Alternative (If script fails)

```bash
cd /Users/linh.doan/work/harvey/freepeak/ktme

# Commit
git add Cargo.toml Cargo.lock
git commit -m "chore: release v0.2.0"

# Push
git push origin main

# Tag
git tag -a v0.2.0 -m "Release v0.2.0"
git push origin v0.2.0

# Publish to crates.io
cargo publish

# Install on macOS
cargo install --path . --force
```

## ✅ Current Status

- Version bumped: 0.1.0 → **0.2.0** ✅
- Tests passed: 34/34 ✅
- Code formatted: ✅
- Ready to publish: ✅ (after crates.io login)

## 📦 What's New in v0.2.0

- Template engine with variable substitution
- Smart documentation merging by section
- GitHub PR extraction
- GitLab MR extraction
- Confluence integration
- Enhanced Markdown writer
- Release automation (Makefile)
- Professional documentation

## 🔍 Verify Installation

After publishing:

```bash
# Check installed version
ktme --version
# Should show: ktme 0.2.0

# Search on crates.io
cargo search ktme

# Install from crates.io on any machine
cargo install ktme
```

## 📝 After Publishing

1. Create GitHub Release: https://github.com/FreePeak/ktme/releases/new?tag=v0.2.0
2. Add release notes (copy from git tag)
3. Announce on your channels

## 🆘 Troubleshooting

**"not logged in to crates.io"**
```bash
cargo login
# Paste token from https://crates.io/settings/tokens
```

**"crate already published"**
- Each version can only be published once
- Bump version again: `cargo set-version --bump patch`

**"uncommitted changes"**
```bash
git status
git add .
git commit -m "fix: commit remaining changes"
```

## 📚 Resources

- Crates.io: https://crates.io/
- Your package: https://crates.io/crates/ktme
- Publishing docs: https://doc.rust-lang.org/cargo/reference/publishing.html
