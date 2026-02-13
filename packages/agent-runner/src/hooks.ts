// ABOUTME: Agent SDK hooks for tracking tool usage during iterations.
// ABOUTME: Emits NDJSON events for each tool call so the dashboard shows real-time activity.

import type { HookCallback } from "@anthropic-ai/claude-agent-sdk";
import { emit } from "./events.js";

/**
 * Tracks tool usage counts across an iteration.
 * Reset between iterations via resetToolCounts().
 */
const toolCounts: Record<string, number> = {};

export function resetToolCounts(): void {
  for (const key of Object.keys(toolCounts)) {
    delete toolCounts[key];
  }
}

export function getToolCounts(): Record<string, number> {
  return { ...toolCounts };
}

/**
 * Extract a human-readable detail string from tool input.
 */
function extractDetail(toolName: string, toolInput: Record<string, unknown>): string {
  switch (toolName) {
    case "Read":
    case "Write":
    case "Edit": {
      const filePath = toolInput.file_path as string | undefined;
      if (filePath) {
        // Show just the filename, not the full path
        return filePath.split("/").pop() ?? filePath;
      }
      return "";
    }
    case "Bash": {
      const cmd = toolInput.command as string | undefined;
      if (cmd) {
        return cmd.length > 80 ? cmd.slice(0, 77) + "..." : cmd;
      }
      return "";
    }
    case "Grep": {
      const pattern = toolInput.pattern as string | undefined;
      return pattern ? `/${pattern}/` : "";
    }
    case "Glob": {
      const pattern = toolInput.pattern as string | undefined;
      return pattern ?? "";
    }
    case "Task": {
      const desc = toolInput.description as string | undefined;
      return desc ?? "";
    }
    default:
      return "";
  }
}

/**
 * PostToolUse hook callback for the Agent SDK.
 * Emits an agent_tool event and tracks tool counts.
 */
export const postToolUseHook: HookCallback = async (input) => {
  if (input.hook_event_name !== "PostToolUse") return {};

  const toolName = "tool_name" in input && typeof input.tool_name === "string" ? input.tool_name : "unknown";
  const toolInput = "tool_input" in input && typeof input.tool_input === "object" && input.tool_input !== null
    ? (input.tool_input as Record<string, unknown>)
    : {};

  // Track count
  toolCounts[toolName] = (toolCounts[toolName] ?? 0) + 1;

  // Emit event
  const detail = extractDetail(toolName, toolInput);
  emit({ type: "agent_tool", tool: toolName, detail });

  return {};
};
