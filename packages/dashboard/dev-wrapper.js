#!/usr/bin/env node
// ABOUTME: Cross-platform wrapper for Vite dev server that ensures proper cleanup
// ABOUTME: when Tauri is killed (works on macOS, Linux, and Windows)

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import treeKill from 'tree-kill';

// Get the directory containing this script
const __dirname = dirname(fileURLToPath(import.meta.url));

let viteProcess = null;
let isCleaningUp = false;
let viteExitCode = 0;

// Cleanup function that kills entire process tree
function cleanup(exitCode = viteExitCode) {
  if (isCleaningUp) return;
  isCleaningUp = true;

  if (viteProcess && viteProcess.pid) {
    console.log('Cleaning up dev server...');

    // Try graceful shutdown with SIGTERM
    treeKill(viteProcess.pid, 'SIGTERM', (err) => {
      if (err && err.code !== 'ESRCH') {  // ESRCH = process doesn't exist
        // If graceful shutdown fails, force kill
        console.warn('Graceful shutdown failed, forcing kill...');
        treeKill(viteProcess.pid, 'SIGKILL', () => {
          process.exit(exitCode);
        });
      } else {
        process.exit(exitCode);
      }
    });

    // Force kill after 5 seconds if process is still hanging
    setTimeout(() => {
      console.warn('Cleanup timeout (5s), forcing shutdown...');
      treeKill(viteProcess.pid, 'SIGKILL', () => {
        process.exit(exitCode);
      });
    }, 5000).unref();  // Don't keep process alive just for this timeout
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
  cwd: __dirname, // Run from the directory containing this script (packages/dashboard)
});

viteProcess.on('error', (err) => {
  console.error('Failed to start dev server:', err);
  process.exit(1);
});

viteProcess.on('exit', (code) => {
  viteExitCode = code || 0;
  cleanup(viteExitCode);
});
