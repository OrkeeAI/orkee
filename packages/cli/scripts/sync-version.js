#!/usr/bin/env node

const fs = require('fs');
const path = require('path');

// Read workspace root Cargo.toml (since CLI uses workspace inheritance)
const cargoPath = path.join(__dirname, '..', '..', '..', 'Cargo.toml');
const cargoContent = fs.readFileSync(cargoPath, 'utf8');

// Extract version from [workspace.package] section
const workspaceMatch = cargoContent.match(/\[workspace\.package\]([\s\S]*?)(?=\n\[|$)/);
if (!workspaceMatch) {
  console.error('Could not find [workspace.package] section in root Cargo.toml');
  process.exit(1);
}

const versionMatch = workspaceMatch[1].match(/version\s*=\s*"([^"]+)"/);
if (!versionMatch) {
  console.error('Could not find version in [workspace.package]');
  process.exit(1);
}

const version = versionMatch[1];
console.log(`Found workspace version: ${version}`);

// Read package.json
const packagePath = path.join(__dirname, '..', 'package.json');
const packageJson = JSON.parse(fs.readFileSync(packagePath, 'utf8'));

// Update version in package.json
const oldVersion = packageJson.version;
packageJson.version = version;

// Write updated package.json
fs.writeFileSync(packagePath, JSON.stringify(packageJson, null, 2) + '\n');

console.log(`Updated package.json version from ${oldVersion} to ${version}`);