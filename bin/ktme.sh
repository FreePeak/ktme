#!/usr/bin/env node

const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Determine the platform-specific binary name
function getBinaryName() {
  const platform = process.platform;
  return platform === 'win32' ? 'ktme.exe' : 'ktme';
}

// Find the binary in the package
function getBinaryPath() {
  const binaryName = getBinaryName();
  const binaryPath = path.join(__dirname, binaryName);

  if (!fs.existsSync(binaryPath)) {
    console.error(`Error: ktme binary not found at ${binaryPath}`);
    console.error('Please reinstall the package: npm install -g ktme-cli');
    process.exit(1);
  }

  return binaryPath;
}

// Main function to run ktme
const binaryPath = getBinaryPath();
const args = process.argv.slice(2);

const child = spawn(binaryPath, args, {
  stdio: 'inherit',
  cwd: process.cwd(),
  env: { ...process.env }
});

child.on('exit', (code) => {
  process.exit(code || 0);
});

child.on('error', (err) => {
  console.error('Failed to start ktme:', err.message);
  console.error('Please ensure ktme is properly installed.');
  process.exit(1);
});
