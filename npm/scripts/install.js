#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { execSync } = require('child_process');

const VERSION = require('../package.json').version;
const REPO = 'anvanster/memrl';

// Platform mapping
const PLATFORMS = {
  'darwin-x64': 'x86_64-apple-darwin',
  'darwin-arm64': 'aarch64-apple-darwin',
  'linux-x64': 'x86_64-unknown-linux-gnu',
  'linux-arm64': 'aarch64-unknown-linux-gnu',
  'win32-x64': 'x86_64-pc-windows-msvc',
};

function getPlatformKey() {
  const platform = process.platform;
  const arch = process.arch;
  return `${platform}-${arch}`;
}

function getDownloadUrl(binary) {
  const platformKey = getPlatformKey();
  const target = PLATFORMS[platformKey];

  if (!target) {
    throw new Error(`Unsupported platform: ${platformKey}`);
  }

  const ext = process.platform === 'win32' ? '.exe' : '';
  const archiveExt = process.platform === 'win32' ? '.zip' : '.tar.gz';

  return `https://github.com/${REPO}/releases/download/v${VERSION}/memrl-${target}${archiveExt}`;
}

function downloadFile(url, dest) {
  return new Promise((resolve, reject) => {
    console.log(`Downloading: ${url}`);

    const file = fs.createWriteStream(dest);

    const request = (url) => {
      https.get(url, (response) => {
        if (response.statusCode === 302 || response.statusCode === 301) {
          // Follow redirect
          request(response.headers.location);
          return;
        }

        if (response.statusCode !== 200) {
          reject(new Error(`Failed to download: ${response.statusCode}`));
          return;
        }

        response.pipe(file);
        file.on('finish', () => {
          file.close();
          resolve();
        });
      }).on('error', (err) => {
        fs.unlink(dest, () => {});
        reject(err);
      });
    };

    request(url);
  });
}

function extractArchive(archive, destDir) {
  if (process.platform === 'win32') {
    // Use PowerShell to extract zip on Windows
    execSync(`powershell -Command "Expand-Archive -Force '${archive}' '${destDir}'"`, { stdio: 'inherit' });
  } else {
    // Use tar on Unix
    execSync(`tar -xzf "${archive}" -C "${destDir}"`, { stdio: 'inherit' });
  }
}

async function install() {
  const binariesDir = path.join(__dirname, '..', 'binaries');
  const tmpDir = path.join(__dirname, '..', 'tmp');

  // Create directories
  fs.mkdirSync(binariesDir, { recursive: true });
  fs.mkdirSync(tmpDir, { recursive: true });

  const platformKey = getPlatformKey();
  const target = PLATFORMS[platformKey];

  if (!target) {
    console.error(`Unsupported platform: ${platformKey}`);
    console.error('Please build from source: cargo install memrl');
    process.exit(1);
  }

  const archiveExt = process.platform === 'win32' ? '.zip' : '.tar.gz';
  const archivePath = path.join(tmpDir, `memrl${archiveExt}`);

  try {
    const url = getDownloadUrl();
    await downloadFile(url, archivePath);

    // Extract archive
    console.log('Extracting...');
    extractArchive(archivePath, tmpDir);

    // Move binaries to binaries directory
    const ext = process.platform === 'win32' ? '.exe' : '';
    const binaries = ['memrl', 'memrl-mcp'];

    for (const binary of binaries) {
      const src = path.join(tmpDir, `${binary}${ext}`);
      const dest = path.join(binariesDir, `${binary}${ext}`);

      if (fs.existsSync(src)) {
        fs.renameSync(src, dest);
        if (process.platform !== 'win32') {
          fs.chmodSync(dest, 0o755);
        }
        console.log(`Installed: ${binary}`);
      }
    }

    // Cleanup
    fs.rmSync(tmpDir, { recursive: true, force: true });

    console.log('Installation complete!');
  } catch (err) {
    console.error('Failed to install memrl:', err.message);
    console.error('');
    console.error('You can try installing from source:');
    console.error('  cargo install memrl');
    console.error('');
    console.error('Or download manually from:');
    console.error(`  https://github.com/${REPO}/releases`);
    process.exit(1);
  }
}

install();
