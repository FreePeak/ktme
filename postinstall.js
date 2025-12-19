#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

// Create a simple symlink or copy for direct usage
const binDir = path.join(__dirname, 'bin');
const platform = process.platform;
const isWindows = platform === 'win32';

// Find the downloaded binary
let binaryName = 'ktme';
if (isWindows) {
    binaryName = 'ktme.exe';
}

const binaryPath = path.join(binDir, binaryName);
const wrapperPath = path.join(binDir, 'ktme.sh');

// Make sure the shell script is executable on Unix systems
if (!isWindows && fs.existsSync(wrapperPath)) {
    fs.chmodSync(wrapperPath, '755');
}

// Also make the binary executable if it exists
if (!isWindows && fs.existsSync(binaryPath)) {
    fs.chmodSync(binaryPath, '755');
}

console.log('ktme-cli installed successfully!');