// ABOUTME: Reads and writes prd.json files, and selects the next story to execute.
// ABOUTME: Mirrors the schema from the /ralph skill's prd.json format.

import { readFileSync, writeFileSync } from "node:fs";
import type { PrdJson, UserStory } from "./types.js";
import { log } from "./events.js";

/**
 * Read and parse prd.json from disk.
 */
export function readPrd(path: string): PrdJson {
  const raw = readFileSync(path, "utf-8");
  try {
    return JSON.parse(raw) as PrdJson;
  } catch (err) {
    const message = err instanceof Error ? err.message : String(err);
    throw new Error(`Failed to parse PRD at ${path}: ${message}`);
  }
}

/**
 * Write prd.json back to disk (after updating story status).
 */
export function writePrd(path: string, prd: PrdJson): void {
  writeFileSync(path, JSON.stringify(prd, null, 2) + "\n", "utf-8");
}

/**
 * Pick the next story to work on: lowest priority where passes === false.
 * Returns null if all stories are complete.
 */
export function pickNextStory(prd: PrdJson): UserStory | null {
  const pending = prd.userStories
    .filter((s) => !s.passes)
    .sort((a, b) => a.priority - b.priority);

  if (pending.length === 0) {
    return null;
  }

  log(`Next story: ${pending[0].id} "${pending[0].title}" (priority ${pending[0].priority})`);
  return pending[0];
}

/**
 * Count completed vs total stories.
 */
export function storyProgress(prd: PrdJson): { completed: number; total: number } {
  const total = prd.userStories.length;
  const completed = prd.userStories.filter((s) => s.passes).length;
  return { completed, total };
}
