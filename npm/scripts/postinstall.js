#!/usr/bin/env node

const fs = require('fs');
const path = require('path');
const https = require('https');
const { exec } = require('child_process');
const { promisify } = require('util');

const execAsync = promisify(exec);

// Configuration
const GITHUB_REPO = 'OrkeeAI/orkee';
const BINARY_NAME = 'orkee';

// Detect platform and architecture
function getPlatform() {
  const platform = process.platform;
  const arch = process.arch;
  
  // Map Node.js platform/arch to Rust target triples
  const platformMap = {
    'darwin-x64': 'x86_64-apple-darwin',
    'darwin-arm64': 'aarch64-apple-darwin',
    'linux-x64': 'x86_64-unknown-linux-gnu',
    'linux-arm64': 'aarch64-unknown-linux-gnu',
    'win32-x64': 'x86_64-pc-windows-msvc',
  };
  
  const key = `${platform}-${arch}`;
  const target = platformMap[key];
  
  if (!target) {
    throw new Error(`Unsupported platform: ${platform} ${arch}`);
  }
  
  return { platform, arch, target };
}

// Download binary from GitHub releases
async function downloadBinary(version) {
  const { platform, target } = getPlatform();
  const ext = platform === 'win32' ? '.exe' : '';
  const binaryName = `${BINARY_NAME}${ext}`;
  
  // Construct download URL
  const isWindows = platform === 'win32';
  const fileName = isWindows ? `orkee-${target}.zip` : `orkee-${target}.tar.gz`;
  const downloadUrl = `https://github.com/${GITHUB_REPO}/releases/download/v${version}/${fileName}`;
  
  console.log(`Downloading Orkee ${version} for ${target}...`);
  console.log(`URL: ${downloadUrl}`);
  
  const binDir = path.join(__dirname, '..', 'bin');
  const binPath = path.join(binDir, binaryName);
  const tempFile = path.join(binDir, `${fileName}.tmp`);
  
  // Create bin directory
  if (!fs.existsSync(binDir)) {
    fs.mkdirSync(binDir, { recursive: true });
  }
  
  return new Promise((resolve, reject) => {
    const file = fs.createWriteStream(tempFile);
    
    https.get(downloadUrl, (response) => {
      if (response.statusCode === 302 || response.statusCode === 301) {
        // Follow redirect
        https.get(response.headers.location, (redirectResponse) => {
          handleResponse(redirectResponse);
        }).on('error', reject);
      } else {
        handleResponse(response);
      }
      
      function handleResponse(res) {
        if (res.statusCode !== 200) {
          reject(new Error(`Failed to download: ${res.statusCode}`));
          return;
        }
        
        const totalSize = parseInt(res.headers['content-length'], 10);
        let downloadedSize = 0;
        
        res.on('data', (chunk) => {
          downloadedSize += chunk.length;
          const percent = Math.round((downloadedSize / totalSize) * 100);
          process.stdout.write(`\rDownloading... ${percent}%`);
        });
        
        res.pipe(file);
        
        file.on('finish', async () => {
          file.close();
          console.log('\nExtracting binary...');
          
          try {
            // Extract archive based on platform
            if (isWindows) {
              // For Windows, use PowerShell to extract .zip
              await execAsync(`powershell -command "Expand-Archive -Path '${tempFile}' -DestinationPath '${binDir}' -Force"`);
            } else {
              // For Unix systems, use tar
              await execAsync(`tar -xzf ${tempFile} -C ${binDir}`);
            }
            
            // Make binary executable (Unix systems only)
            if (!isWindows) {
              fs.chmodSync(binPath, 0o755);
            }
            
            // Clean up temp file
            fs.unlinkSync(tempFile);
            
            console.log('✅ Orkee installed successfully!');
            console.log(`Binary location: ${binPath}`);
            resolve(binPath);
          } catch (error) {
            reject(error);
          }
        });
      }
    }).on('error', reject);
  });
}

// Main installation function
async function install() {
  try {
    const packageJson = require('../package.json');
    const version = packageJson.version;
    
    await downloadBinary(version);
    
    console.log('\n🎉 Installation complete!');
    console.log('Run "orkee --help" to get started.');
  } catch (error) {
    console.error('\n❌ Installation failed:', error.message);
    console.error('\nYou can try:');
    console.error('1. Building from source: https://github.com/OrkeeAI/orkee#building-from-source');
    console.error('2. Downloading pre-built binaries: https://github.com/OrkeeAI/orkee/releases');
    process.exit(1);
  }
}

// Run installation
install();