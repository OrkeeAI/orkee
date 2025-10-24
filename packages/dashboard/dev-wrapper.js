#!/usr/bin/env node
// ABOUTME: Cross-platform wrapper for Vite dev server that ensures proper cleanup
// ABOUTME: when Tauri is killed (works on macOS, Linux, and Windows)

/**
 * Dev Server Wrapper
 *
 * WHY THIS EXISTS:
 * When Tauri dev mode is killed (Ctrl+C), the spawned Vite dev server can become
 * orphaned, continuing to run and hold ports (5173+). This wrapper ensures complete
 * process tree cleanup using tree-kill library for cross-platform reliability.
 *
 * ARCHITECTURE:
 *
 * Path Resolution:
 *   - Tauri runs beforeDevCommand from src-tauri/ directory (documented behavior)
 *   - tauri.conf.json uses "node ../dev-wrapper.js" to reference this script
 *   - This wrapper uses cwd: __dirname to spawn processes from packages/dashboard/
 *   - This separation ensures correct execution regardless of invocation directory
 *
 * Shutdown Flow:
 *   1. Signal received (SIGINT/SIGTERM from Ctrl+C)
 *   2. cleanup() called with isCleaningUp flag to prevent double execution
 *   3. Try graceful shutdown: treeKill(pid, 'SIGTERM')
 *   4. If SIGTERM fails → fallback to treeKill(pid, 'SIGKILL')
 *   5. 5-second timeout as safety net → force SIGKILL if process hangs
 *      (Note: timeout MUST keep process alive - no .unref() - to guarantee execution)
 *   6. Exit with preserved exit code from Vite
 *
 * Cross-Platform Support:
 *   - Windows: Uses shell: true to find bun.exe via PATH, tree-kill handles cmd.exe chain
 *   - macOS/Linux: Uses detached process groups for clean tree termination
 *   - tree-kill library handles platform-specific process tree killing (taskkill /T on Windows)
 *
 * Exit Code Preservation:
 *   - Vite's exit code is captured in viteExitCode variable
 *   - cleanup() uses this code to exit, maintaining crash information
 *   - Signal handlers (Ctrl+C) exit with 0 (clean shutdown)
 */

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
    }, 5000);  // Keep process alive to ensure timeout fires if SIGTERM hangs
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
