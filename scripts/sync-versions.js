#!/usr/bin/env node

/**
 * ABOUTME: Synchronizes version numbers across all workspace packages
 * ABOUTME: Reads from root package.json and updates all package.json files in workspaces
 */

import { readFileSync, writeFileSync, readdirSync, statSync } from 'fs';
import { join } from 'path';

const rootDir = join(import.meta.dirname, '..');
const rootPackageJson = JSON.parse(readFileSync(join(rootDir, 'package.json'), 'utf8'));

if (!rootPackageJson.version) {
  console.error('❌ Error: Root package.json must have a "version" field');
  process.exit(1);
}

const targetVersion = rootPackageJson.version;
console.log(`📦 Syncing all packages to version ${targetVersion}\n`);

/**
 * Find all package.json files in workspace directories
 */
function findPackageJsonFiles(dir) {
  const packages = [];
  const entries = readdirSync(dir);

  for (const entry of entries) {
    const fullPath = join(dir, entry);
    const stat = statSync(fullPath);

    if (stat.isDirectory() && entry !== 'node_modules') {
      const pkgJsonPath = join(fullPath, 'package.json');
      try {
        const pkgJson = JSON.parse(readFileSync(pkgJsonPath, 'utf8'));
        packages.push({ path: pkgJsonPath, data: pkgJson, name: pkgJson.name });
      } catch (err) {
        // No package.json or invalid JSON, skip
      }
    }
  }

  return packages;
}

// Sync all workspace packages
const packagesDir = join(rootDir, 'packages');
const packages = findPackageJsonFiles(packagesDir);

let updatedCount = 0;
let skippedCount = 0;

for (const pkg of packages) {
  const currentVersion = pkg.data.version;

  if (currentVersion === targetVersion) {
    console.log(`⏭️  ${pkg.name} - already at ${targetVersion}`);
    skippedCount++;
    continue;
  }

  pkg.data.version = targetVersion;
  writeFileSync(pkg.path, JSON.stringify(pkg.data, null, 2) + '\n', 'utf8');
  console.log(`✅ ${pkg.name} - updated from ${currentVersion} to ${targetVersion}`);
  updatedCount++;
}

console.log(`\n📊 Summary:`);
console.log(`   Updated: ${updatedCount}`);
console.log(`   Skipped: ${skippedCount}`);
console.log(`   Total: ${packages.length}`);

if (updatedCount > 0) {
  console.log(`\n💡 Next steps:`);
  console.log(`   1. Review changes: git diff packages/*/package.json`);
  console.log(`   2. Commit: git add -A && git commit -m "chore: bump version to ${targetVersion}"`);
  console.log(`   3. Tag: git tag v${targetVersion}`);
  console.log(`   4. Push: git push origin main --tags`);
}
