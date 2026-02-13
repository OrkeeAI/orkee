// ABOUTME: NDJSON event emitter for structured communication with the Rust backend.
// ABOUTME: Writes one JSON object per line to stdout; the backend parses these for SSE relay.

import type { RunEvent } from "./types.js";

/**
 * Emit a structured event as a single NDJSON line to stdout.
 * Uses process.stdout.write directly to avoid console.log's trailing newline ambiguity.
 */
export function emit(event: RunEvent): void {
  process.stdout.write(JSON.stringify(event) + "\n");
}

/**
 * Write a debug/log message to stderr so it doesn't interfere with the NDJSON protocol.
 */
export function log(message: string): void {
  process.stderr.write(`[agent-runner] ${message}\n`);
}
