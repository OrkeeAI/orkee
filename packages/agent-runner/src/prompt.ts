// ABOUTME: Builds the system prompt and per-iteration prompts for the Agent SDK.
// ABOUTME: Composes agent instructions, project CLAUDE.md, PRD context, and story details.

import { readFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import type { PrdJson, UserStory } from "./types.js";
import { log } from "./events.js";

const __dirname = dirname(fileURLToPath(import.meta.url));
const PROMPTS_DIR = join(__dirname, "..", "prompts");

function loadPromptFile(name: string): string {
  const path = join(PROMPTS_DIR, name);
  if (!existsSync(path)) {
    log(`Warning: prompt file not found: ${path}`);
    return "";
  }
  return readFileSync(path, "utf-8");
}

/**
 * Build the system prompt for the Agent SDK.
 * Composes agent instructions + project CLAUDE.md + PRD overview.
 */
export function buildSystemPrompt(
  projectDir: string,
  prd: PrdJson,
  customSystemPromptPath?: string,
): string {
  const parts: string[] = [];

  // 1. Agent instructions (bundled from /ralph skill)
  if (customSystemPromptPath && existsSync(customSystemPromptPath)) {
    parts.push(readFileSync(customSystemPromptPath, "utf-8"));
  } else {
    parts.push(loadPromptFile("agent-instructions.md"));
  }

  // 2. Project CLAUDE.md (if it exists in the target project)
  const claudeMdPath = join(projectDir, "CLAUDE.md");
  if (existsSync(claudeMdPath)) {
    parts.push("\n---\n\n# Project Conventions (CLAUDE.md)\n");
    parts.push(readFileSync(claudeMdPath, "utf-8"));
  }

  // 3. PRD overview - what the project is and progress so far
  const completed = prd.userStories.filter((s) => s.passes).length;
  const total = prd.userStories.length;
  parts.push("\n---\n\n# PRD Context\n");
  parts.push(`**Project:** ${prd.project}`);
  parts.push(`**Description:** ${prd.description}`);
  parts.push(`**Progress:** ${completed}/${total} stories completed`);
  parts.push(`**Branch:** ${prd.branchName}\n`);

  // List all stories with status
  parts.push("## All User Stories\n");
  for (const story of prd.userStories) {
    const status = story.passes ? "DONE" : "TODO";
    parts.push(`- [${status}] ${story.id}: ${story.title} (priority ${story.priority})`);
    if (story.passes && story.notes) {
      parts.push(`  Notes: ${story.notes}`);
    }
  }

  return parts.join("\n");
}

/**
 * Build the per-iteration prompt for a specific story.
 * This is the `prompt` argument to query(), not the system prompt.
 */
export function buildIterationPrompt(story: UserStory, prd: PrdJson): string {
  const parts: string[] = [];

  parts.push(`# Your Assignment: ${story.id} - ${story.title}\n`);
  parts.push(`**Description:** ${story.description}\n`);
  parts.push("**Acceptance Criteria:**");
  for (const criterion of story.acceptanceCriteria) {
    parts.push(`- [ ] ${criterion}`);
  }
  parts.push(`\n**Epic:** ${story.epic}`);
  parts.push(`**Priority:** ${story.priority}\n`);

  // Context from completed stories that might be relevant
  const completedStories = prd.userStories.filter((s) => s.passes && s.notes);
  if (completedStories.length > 0) {
    parts.push("## Context from Previously Completed Stories\n");
    for (const s of completedStories) {
      parts.push(`- **${s.id} (${s.title}):** ${s.notes}`);
    }
  }

  parts.push(
    "\n## Instructions\n" +
      "Complete this story following the TDD workflow in your system prompt. " +
      "Create a feature branch, write tests first, implement, verify all checks pass, " +
      "then create and merge a PR. Ensure `git status` is clean when done.",
  );

  return parts.join("\n");
}
