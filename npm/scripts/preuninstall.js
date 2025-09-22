#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

const binDir = path.join(__dirname, '..', 'bin');

// Clean up binary files on uninstall
if (fs.existsSync(binDir)) {
  console.log('Cleaning up Orkee binary...');
  try {
    fs.rmSync(binDir, { recursive: true, force: true });
    console.log('✅ Cleanup complete');
  } catch (error) {
    console.error('Warning: Could not clean up binary files:', error.message);
  }
}