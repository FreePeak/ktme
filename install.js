const https = require('https');
const fs = require('fs');
const path = require('path');
const os = require('os');

function getPlatform() {
  const platform = os.platform();
  const arch = os.arch();

  switch (platform) {
    case 'darwin':
      return arch === 'arm64' ? 'darwin-arm64' : 'darwin-x64';
    case 'linux':
      return arch === 'arm64' ? 'linux-arm64' : 'linux-x64';
    case 'win32':
      return arch === 'x64' ? 'windows-x64.exe' : 'windows-ia32.exe';
    default:
      throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }
}

function downloadBinary() {
  return new Promise((resolve, reject) => {
    const platform = getPlatform();
    const version = require('./package.json').version;
    const url = `https://github.com/FreePeak/ktme/releases/download/v${version}/ktme-${platform}`;

    const binDir = path.join(__dirname, 'bin');
    if (!fs.existsSync(binDir)) {
      fs.mkdirSync(binDir, { recursive: true });
    }

    const binaryPath = path.join(binDir, platform.includes('windows') ? 'ktme.exe' : 'ktme');
    const file = fs.createWriteStream(binaryPath);

    console.log(`Downloading ktme for ${platform}...`);

    https.get(url, (response) => {
      if (response.statusCode === 302) {
        // Follow redirect
        https.get(response.headers.location, (redirectResponse) => {
          redirectResponse.pipe(file);
          file.on('finish', () => {
            file.close();
            fs.chmodSync(binaryPath, '755');
            console.log('Downloaded ktme successfully!');
            resolve();
          });
        }).on('error', reject);
      } else {
        response.pipe(file);
        file.on('finish', () => {
          file.close();
          fs.chmodSync(binaryPath, '755');
          console.log('Downloaded ktme successfully!');
          resolve();
        });
      }
    }).on('error', reject);
  });
}

// Only download if we're in production (not when developing)
if (process.env.NODE_ENV !== 'development') {
  downloadBinary().catch(console.error);
}