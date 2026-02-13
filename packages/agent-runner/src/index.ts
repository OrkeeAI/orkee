// ABOUTME: Entry point for the agent runner subprocess.
// ABOUTME: Parses CLI args, validates configuration, and starts the iteration loop.

import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { parseArgs } from "node:util";
import { runLoop } from "./loop.js";
import { emit, log } from "./events.js";
import type { RunConfig } from "./types.js";

function printUsage(): void {
  log(
    "Usage: bun run src/index.ts " +
      "--project-dir <path> --prd <path> --run-id <uuid> " +
      "[--max-iterations <n>] [--system-prompt <path>]",
  );
}

async function main(): Promise<void> {
  const { values } = parseArgs({
    options: {
      "project-dir": { type: "string" },
      prd: { type: "string" },
      "run-id": { type: "string" },
      "max-iterations": { type: "string", default: "10" },
      "system-prompt": { type: "string" },
      help: { type: "boolean", short: "h" },
    },
    strict: true,
  });

  if (values.help) {
    printUsage();
    process.exit(0);
  }

  // Validate required args
  if (!values["project-dir"] || !values.prd || !values["run-id"]) {
    log("Error: --project-dir, --prd, and --run-id are required");
    printUsage();
    process.exit(1);
  }

  const projectDir = resolve(values["project-dir"]);
  const prdPath = resolve(values.prd);
  const runId = values["run-id"];
  const maxIterations = parseInt(values["max-iterations"] ?? "10", 10);
  const systemPromptPath = values["system-prompt"]
    ? resolve(values["system-prompt"])
    : undefined;

  // Validate paths exist
  if (!existsSync(projectDir)) {
    log(`Error: project directory does not exist: ${projectDir}`);
    process.exit(1);
  }
  if (!existsSync(prdPath)) {
    log(`Error: PRD file does not exist: ${prdPath}`);
    process.exit(1);
  }
  if (systemPromptPath && !existsSync(systemPromptPath)) {
    log(`Error: system prompt file does not exist: ${systemPromptPath}`);
    process.exit(1);
  }
  if (isNaN(maxIterations) || maxIterations < 1) {
    log("Error: --max-iterations must be a positive integer");
    process.exit(1);
  }

  // Check for auth token
  if (!process.env.CLAUDE_CODE_OAUTH_TOKEN && !process.env.ANTHROPIC_API_KEY) {
    log(
      "Error: No authentication found. " +
        "Set CLAUDE_CODE_OAUTH_TOKEN or ANTHROPIC_API_KEY environment variable.",
    );
    process.exit(1);
  }

  const config: RunConfig = {
    runId,
    projectDir,
    prdPath,
    maxIterations,
    systemPromptPath,
  };

  log(`Starting agent runner: ${JSON.stringify({ runId, projectDir, prdPath, maxIterations })}`);

  try {
    await runLoop(config);
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error);
    log(`Fatal error: ${errorMessage}`);
    emit({
      type: "run_failed",
      run_id: runId,
      error: errorMessage,
    });
    process.exit(1);
  }
}

main();
