#!/bin/sh
# KTME Binary Wrapper
# This script will download and run the appropriate ktme binary

set -e

# Get the directory where this script is located
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Check if binary exists, if not trigger download
if [ ! -f "$SCRIPT_DIR/ktme-binary" ]; then
    # The install script should have already downloaded the binary
    # But if not, we can try to trigger it
    node "$SCRIPT_DIR/../install.js"
fi

# Try different possible binary names
if [ -f "$SCRIPT_DIR/ktme" ]; then
    exec "$SCRIPT_DIR/ktme" "$@"
elif [ -f "$SCRIPT_DIR/ktme.exe" ]; then
    exec "$SCRIPT_DIR/ktme.exe" "$@"
elif [ -f "$SCRIPT_DIR/ktme-binary" ]; then
    exec "$SCRIPT_DIR/ktme-binary" "$@"
else
    echo "Error: ktme binary not found. Please run 'npm install ktme-cli' again."
    exit 1
fi