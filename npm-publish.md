# Publishing ktme to NPM

## Prerequisites
1. Node.js installed
2. NPM account
3. Rust toolchain (for building)

## Option 1: Using NAPI-RS (Recommended)

### Setup
```bash
# Install NAPI CLI
npm install -g @napi-rs/cli

# Add NAPI dependencies to your Rust project
cargo add napi@2
cargo add napi-derive@2

# Build for all platforms
npm run build
```

### Configure Cargo.toml for NAPI
Add to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "2", default-features = false, features = ["napi4"] }
napi-derive = "2"

[build-dependencies]
napi-build = "2"
```

### Create build.rs
```rust
extern crate napi_build;

fn main() {
    napi_build::setup();
}
```

### Publish to NPM
```bash
# Login to npm
npm login

# Publish
npm publish
```

## Option 2: Simple Binary Distribution

### Step 1: Build Binaries
```bash
# Build for your current platform
cargo build --release

# For cross-platform builds, use GitHub Actions (see .github/workflows/release.yml)
```

### Step 2: Update package.json
Use the `npm-scripts.json` file as your `package.json` and adjust paths as needed.

### Step 3: Create GitHub Release
1. Tag your release: `git tag v0.1.0 && git push origin v0.1.0`
2. GitHub Actions will build and upload binaries
3. Update URLs in `install.js` to match your release

### Step 4: Publish to NPM
```bash
# Copy the simple package.json
cp npm-scripts.json package.json

# Install dependencies
npm install

# Publish
npm publish
```

## Option 3: Using PKG (Node.js Binary)

```bash
# Install pkg
npm install -g pkg

# Package your Node.js wrapper
pkg index.js --targets node16-macos-x64,node16-linux-x64,node16-win-x64 --output ktme

# This creates standalone binaries that don't need Node.js installed
```

## Installation for Users

Once published, users can install with:
```bash
# Global install
npm install -g ktme

# Local install
npm install ktme

# npx (one-time use)
npx ktme --help
```

## Development Workflow

```bash
# For development, you can still use the Rust binary directly:
make dev  # or cargo build --release

# For npm package development:
npm run build:debug
npm test
npm pack  # Test the package before publishing
```