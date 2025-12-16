const { spawn } = require('child_process');
const path = require('path');
const fs = require('fs');

// Determine the platform-specific binary name
function getBinaryName() {
  const platform = process.platform;
  const arch = process.arch;

  let binaryName = 'ktme';

  if (platform === 'win32') {
    binaryName = 'ktme.exe';
  }

  return binaryName;
}

// Find the binary in the package
function getBinaryPath() {
  const binaryName = getBinaryName();

  // Try to find the binary in various locations
  const possiblePaths = [
    path.join(__dirname, 'bin', binaryName),
    path.join(__dirname, 'dist', binaryName),
    path.join(__dirname, 'target', 'release', binaryName),
  ];

  for (const binaryPath of possiblePaths) {
    if (fs.existsSync(binaryPath)) {
      return binaryPath;
    }
  }

  // Fallback for development
  return path.join(__dirname, '..', 'target', 'release', binaryName);
}

// Main function to run ktme
function runKtme() {
  const binaryPath = getBinaryPath();
  const args = process.argv.slice(2);

  const child = spawn(binaryPath, args, {
    stdio: 'inherit',
    cwd: process.cwd(),
    env: { ...process.env }
  });

  child.on('exit', (code) => {
    process.exit(code);
  });

  child.on('error', (err) => {
    console.error('Failed to start ktme:', err.message);
    console.error('Please ensure ktme is properly installed.');
    process.exit(1);
  });
}

// Run if this file is executed directly
if (require.main === module) {
  runKtme();
}

module.exports = { runKtme };