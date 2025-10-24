#!/usr/bin/env node
// ABOUTME: Cross-platform wrapper for Vite dev server that ensures proper cleanup
// ABOUTME: when Tauri is killed (works on macOS, Linux, and Windows)

import { spawn } from 'child_process';
import treeKill from 'tree-kill';

let viteProcess = null;
let isCleaningUp = false;

// Cleanup function that kills entire process tree
function cleanup(exitCode = 0) {
  if (isCleaningUp) return;
  isCleaningUp = true;

  if (viteProcess && viteProcess.pid) {
    console.log('Cleaning up dev server...');
    treeKill(viteProcess.pid, 'SIGTERM', (err) => {
      if (err && err.code !== 'ESRCH') {  // ESRCH = process doesn't exist
        console.error('Error killing process tree:', err);
        process.exit(1);
      }
      process.exit(exitCode);
    });
  } else {
    process.exit(exitCode);
  }
}

// Register cleanup handlers
process.on('SIGINT', () => cleanup(0));
process.on('SIGTERM', () => cleanup(0));
// Exit handler is a no-op since cleanup is handled by signal handlers
process.on('exit', () => {
  // Cleanup already handled by signal handlers
});

// Start Vite dev server
const isWindows = process.platform === 'win32';
const shell = isWindows ? true : false;
const command = isWindows ? 'bun.exe' : 'bun';

viteProcess = spawn(command, ['run', 'dev'], {
  stdio: 'inherit',
  shell,
  detached: !isWindows, // Process group on Unix
});

viteProcess.on('error', (err) => {
  console.error('Failed to start dev server:', err);
  process.exit(1);
});

viteProcess.on('exit', (code) => {
  cleanup(code || 0);
});
