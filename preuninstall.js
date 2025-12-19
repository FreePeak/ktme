#!/usr/bin/env node
const fs = require('fs');
const path = require('path');

// Clean up downloaded binaries
const binDir = path.join(__dirname, 'bin');

const filesToRemove = [
    'ktme',
    'ktme.exe',
    'ktme-binary'
];

filesToRemove.forEach(file => {
    const filePath = path.join(binDir, file);
    if (fs.existsSync(filePath)) {
        try {
            fs.unlinkSync(filePath);
            console.log(`Removed ${file}`);
        } catch (err) {
            // Ignore errors during uninstall
        }
    }
});

console.log('ktme-cli uninstalled successfully!');