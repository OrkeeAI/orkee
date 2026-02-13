// ABOUTME: Entry point for the agent runner subprocess.
// ABOUTME: Parses CLI args, validates configuration, and starts the iteration loop.

import { existsSync } from "node:fs";
import { resolve } from "node:path";
import { parseArgs } from "node:util";
import { runLoop } from "./loop.js";
import { emit, log } from "./events.js";
import { readPrd } from "./prd.js";
import type { RunConfig } from "./types.js";

let currentRunId: string | undefined;

function validatePath(path: string, label: string): void {
  if (!existsSync(path)) {
    log(`Error: ${label} does not exist: ${path}`);
    process.exit(1);
  }
}

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
  currentRunId = runId;
  const maxIterations = parseInt(values["max-iterations"] ?? "10", 10);
  const systemPromptPath = values["system-prompt"]
    ? resolve(values["system-prompt"])
    : undefined;

  // Validate paths exist
  validatePath(projectDir, "project directory");
  validatePath(prdPath, "PRD file");
  if (systemPromptPath) validatePath(systemPromptPath, "system prompt file");
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

  // Validate PRD contents before starting
  const prd = readPrd(prdPath);
  if (!prd.userStories?.length) {
    emit({ type: "run_failed", run_id: runId, error: "PRD contains no user stories" });
    process.exit(1);
  }

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

function handleTermination(signal: string): void {
  if (currentRunId) {
    emit({ type: "run_failed", run_id: currentRunId, error: `Process terminated by ${signal}` });
  }
  process.exit(1);
}

process.on("SIGTERM", () => handleTermination("SIGTERM"));
process.on("SIGINT", () => handleTermination("SIGINT"));
process.on("uncaughtException", (err) => {
  log(`Uncaught exception: ${err.message}`);
  if (currentRunId) {
    emit({ type: "run_failed", run_id: currentRunId, error: `Uncaught exception: ${err.message}` });
  }
  process.exit(1);
});

main();
