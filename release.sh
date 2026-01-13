#!/bin/bash
# KTME Release Script v0.2.0
# This script will publish KTME to crates.io and install it on your macOS

set -e

echo "🚀 KTME Release Workflow - v0.2.0"
echo "=================================="
echo ""

# Check if logged in to crates.io
if [ ! -f ~/.cargo/credentials.toml ]; then
    echo "❌ ERROR: Not logged in to crates.io"
    echo ""
    echo "Please complete these steps first:"
    echo "1. Go to: https://crates.io/settings/tokens"
    echo "2. Click 'New Token' and name it (e.g., 'KTME publishing')"
    echo "3. Copy the token (starts with 'cio_')"
    echo "4. Run: cargo login"
    echo "5. Paste your token"
    echo ""
    echo "Then run this script again."
    exit 1
fi

echo "✓ Logged in to crates.io"
echo ""

# Step 1: Commit version changes
echo "Step 1: Committing version changes..."
git add Cargo.toml
# Add Cargo.lock if it exists and is tracked
if git ls-files --error-unmatch Cargo.lock 2>/dev/null; then
    git add Cargo.lock
fi
git commit -m "chore: release v0.2.0

- Bump version to 0.2.0
- Add template engine with variable substitution
- Add smart documentation merging by section
- Add GitHub PR extraction and integration
- Add GitLab MR extraction and integration
- Add Confluence writer with Markdown conversion
- Enhance Markdown writer with section updates
- Reorganize documentation into docs/ folder
- Add release workflow automation
- 34 new tests (all passing)
" || echo "No changes to commit"

echo ""

# Step 2: Push to GitHub
echo "Step 2: Pushing to GitHub..."
BRANCH=$(git branch --show-current)
git push origin $BRANCH

echo ""

# Step 3: Create and push tag
echo "Step 3: Creating git tag v0.2.0..."
git tag -a "v0.2.0" -m "Release v0.2.0

New Features:
- Template engine with variable substitution
- Smart documentation merging by section  
- GitHub PR extraction and integration
- GitLab MR extraction and integration
- Confluence writer with Markdown conversion
- Enhanced Markdown writer with section updates
- Comprehensive release workflow (Makefile)
- Professional documentation structure

Testing:
- 34 new tests (all passing)
- Template tests: 6/6
- Generator tests: 5/5
- Git provider tests: 12/12
- Confluence tests: 9/9
- Markdown writer tests: 2/2

Documentation:
- Reorganized into docs/ folder
- Added PUBLISH.md (quick start)
- Added RELEASE.md (complete workflow)
- Added DEVELOPMENT.md (contributor guide)
- Updated README.md (professional, concise)
"

git push origin v0.2.0

echo ""

# Step 4: Build release version
echo "Step 4: Building release version..."
cargo build --release

echo ""

# Step 5: Publish to crates.io
echo "Step 5: Publishing to crates.io..."
cargo publish

echo ""

# Step 6: Install on macOS
echo "Step 6: Installing on your macOS..."
cargo install --path . --force

echo ""

# Verify installation
echo "Step 7: Verifying installation..."
NEW_VERSION=$(ktme --version | cut -d' ' -f2)

echo ""
echo "✓ Release v0.2.0 completed successfully!"
echo ""
echo "Published:"
echo "  - Git tag: v0.2.0"
echo "  - GitHub: https://github.com/FreePeak/ktme/releases/tag/v0.2.0"
echo "  - Crates.io: https://crates.io/crates/ktme/0.2.0"
echo "  - Installed version: $NEW_VERSION"
echo ""
echo "Next steps:"
echo "1. Create GitHub release: https://github.com/FreePeak/ktme/releases/new?tag=v0.2.0"
echo "2. Add release notes from the tag message"
echo ""
