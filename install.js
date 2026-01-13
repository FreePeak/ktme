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
      return 'linux-x64';
    case 'win32':
      return 'windows-x64.exe';
    default:
      throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }
}

function followRedirects(url, maxRedirects = 5) {
  return new Promise((resolve, reject) => {
    if (maxRedirects === 0) {
      reject(new Error('Too many redirects'));
      return;
    }

    https.get(url, (response) => {
      if (response.statusCode === 301 || response.statusCode === 302) {
        followRedirects(response.headers.location, maxRedirects - 1)
          .then(resolve)
          .catch(reject);
      } else {
        resolve(response);
      }
    }).on('error', reject);
  });
}

function downloadBinary() {
  return new Promise(async (resolve, reject) => {
    try {
      const platform = getPlatform();
      const version = require('./package.json').version;
      const url = `https://github.com/FreePeak/ktme/releases/download/v${version}/ktme-${platform}`;

      const binDir = path.join(__dirname, 'bin');
      if (!fs.existsSync(binDir)) {
        fs.mkdirSync(binDir, { recursive: true });
      }

      const binaryPath = path.join(binDir, platform.includes('windows') ? 'ktme.exe' : 'ktme');
      const file = fs.createWriteStream(binaryPath);

      console.log(`Downloading ktme v${version} for ${platform}...`);
      console.log(`URL: ${url}`);

      const response = await followRedirects(url);
      
      response.pipe(file);
      file.on('finish', () => {
        file.close();
        if (!platform.includes('windows')) {
          fs.chmodSync(binaryPath, '755');
        }
        console.log('✓ Downloaded ktme successfully!');
        resolve();
      });

      file.on('error', (err) => {
        fs.unlink(binaryPath, () => {});
        reject(err);
      });
    } catch (err) {
      reject(err);
    }
  });
}

// Only download if we're in production (not when developing)
if (process.env.NODE_ENV !== 'development') {
  downloadBinary().catch((err) => {
    console.error('Failed to download ktme binary:', err.message);
    console.error('Please try installing manually from: https://github.com/FreePeak/ktme/releases');
    process.exit(1);
  });
}