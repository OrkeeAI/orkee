#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const { execSync } = require('child_process');
const https = require('https');
const { createGunzip } = require('zlib');
const { pipeline } = require('stream');
const { promisify } = require('util');
const tar = require('tar-stream');

const pipelineAsync = promisify(pipeline);

// Configuration
const PACKAGE_NAME = 'orkee';
const GITHUB_REPO = 'OrkeeAI/orkee';
const VERSION = require('../package.json').version;

// Platform mapping
const PLATFORM_MAP = {
  'darwin-x64': 'x86_64-apple-darwin',
  'darwin-arm64': 'aarch64-apple-darwin',
  'linux-x64': 'x86_64-unknown-linux-gnu',
  'linux-arm64': 'aarch64-unknown-linux-gnu',
  'win32-x64': 'x86_64-pc-windows-msvc'
};

function getPlatform() {
  const platform = process.platform;
  const arch = process.arch === 'x64' ? 'x64' : process.arch === 'arm64' ? 'arm64' : null;
  
  if (!arch) {
    throw new Error(`Unsupported architecture: ${process.arch}`);
  }
  
  const key = `${platform}-${arch}`;
  const rustTarget = PLATFORM_MAP[key];
  
  if (!rustTarget) {
    throw new Error(`Unsupported platform: ${platform}-${arch}`);
  }
  
  return {
    key,
    rustTarget,
    isWindows: platform === 'win32'
  };
}

function downloadFile(url) {
  return new Promise((resolve, reject) => {
    https.get(url, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        // Follow redirect
        return downloadFile(response.headers.location).then(resolve).catch(reject);
      }
      
      if (response.statusCode !== 200) {
        reject(new Error(`Failed to download: ${response.statusCode}`));
        return;
      }
      
      const chunks = [];
      response.on('data', chunk => chunks.push(chunk));
      response.on('end', () => resolve(Buffer.concat(chunks)));
      response.on('error', reject);
    }).on('error', reject);
  });
}

async function extractTarGz(buffer, outputPath) {
  const extract = tar.extract();
  
  return new Promise((resolve, reject) => {
    extract.on('entry', (header, stream, next) => {
      if (header.name === PACKAGE_NAME || header.name === `${PACKAGE_NAME}.exe`) {
        const chunks = [];
        stream.on('data', chunk => chunks.push(chunk));
        stream.on('end', () => {
          fs.writeFileSync(outputPath, Buffer.concat(chunks), { mode: 0o755 });
          next();
        });
        stream.on('error', reject);
      } else {
        stream.on('end', next);
        stream.resume();
      }
    });
    
    extract.on('finish', resolve);
    extract.on('error', reject);
    
    const gunzip = createGunzip();
    gunzip.end(buffer);
    gunzip.pipe(extract);
  });
}

async function extractZip(buffer, outputPath) {
  // For Windows, we'll use PowerShell to extract
  const tempZip = path.join(process.cwd(), 'orkee-temp.zip');
  fs.writeFileSync(tempZip, buffer);
  
  try {
    execSync(`powershell -command "Expand-Archive -Path '${tempZip}' -DestinationPath '${path.dirname(outputPath)}' -Force"`, {
      stdio: 'inherit'
    });
    
    // The zip contains orkee.exe, move it to the right location if needed
    const extractedExe = path.join(path.dirname(outputPath), 'orkee.exe');
    if (fs.existsSync(extractedExe) && extractedExe !== outputPath) {
      fs.renameSync(extractedExe, outputPath);
    }
  } finally {
    if (fs.existsSync(tempZip)) {
      fs.unlinkSync(tempZip);
    }
  }
}

async function install() {
  try {
    console.log(`Installing ${PACKAGE_NAME} v${VERSION}...`);
    
    const platform = getPlatform();
    const binaryName = platform.isWindows ? `${PACKAGE_NAME}.exe` : PACKAGE_NAME;
    const outputPath = path.join(__dirname, '..', binaryName);
    
    // Check if binary already exists
    if (fs.existsSync(outputPath)) {
      console.log(`Binary already exists at ${outputPath}, skipping download.`);
      return;
    }
    
    // Construct download URL
    const archiveName = platform.isWindows 
      ? `orkee-${platform.rustTarget}.zip`
      : `orkee-${platform.rustTarget}.tar.gz`;
    
    const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/v${VERSION}/${archiveName}`;
    
    console.log(`Downloading from: ${downloadUrl}`);
    console.log(`Platform: ${platform.key} (${platform.rustTarget})`);
    
    // Download the archive
    const buffer = await downloadFile(downloadUrl);
    console.log(`Downloaded ${buffer.length} bytes`);
    
    // Extract the binary
    if (platform.isWindows) {
      await extractZip(buffer, outputPath);
    } else {
      await extractTarGz(buffer, outputPath);
    }
    
    // Verify installation
    if (!fs.existsSync(outputPath)) {
      throw new Error(`Failed to extract binary to ${outputPath}`);
    }
    
    // Make executable on Unix
    if (!platform.isWindows) {
      fs.chmodSync(outputPath, 0o755);
    }
    
    console.log(`✅ Successfully installed ${PACKAGE_NAME} to ${outputPath}`);
    
  } catch (error) {
    console.error(`❌ Installation failed: ${error.message}`);
    
    // Provide fallback instructions
    console.log('\nYou can manually download the binary from:');
    console.log(`https://github.com/${GITHUB_REPO}/releases/tag/v${VERSION}`);
    console.log('\nOr install from source:');
    console.log('  cargo install orkee-cli');
    
    // Don't exit with error for now since binaries aren't published yet
    // This allows CI to continue
    console.log('\n⚠️  Continuing without binary (development mode)');
    process.exit(0);
  }
}

// Run installation
install().catch(console.error);