// ABOUTME: Core iteration loop that drives the autonomous agent.
// ABOUTME: Reads PRD, picks stories, invokes the Agent SDK, tracks progress, and emits events.

import { query } from "@anthropic-ai/claude-agent-sdk";
import type { RunConfig } from "./types.js";
import { emit, log } from "./events.js";
import { readPrd, pickNextStory, storyProgress } from "./prd.js";
import { buildSystemPrompt, buildIterationPrompt } from "./prompt.js";
import { postToolUseHook, resetToolCounts, getToolCounts } from "./hooks.js";

export async function runLoop(config: RunConfig): Promise<void> {
  let prd = readPrd(config.prdPath);
  const { completed, total } = storyProgress(prd);

  emit({
    type: "run_started",
    run_id: config.runId,
    total_stories: total,
    completed_stories: completed,
  });

  const systemPrompt = buildSystemPrompt(config.projectDir, prd, config.systemPromptPath);
  let totalCost = 0;
  let storiesCompletedThisRun = 0;
  const runStart = Date.now();

  for (let i = 0; i < config.maxIterations; i++) {
    // Re-read PRD each iteration (agent may have updated it)
    prd = readPrd(config.prdPath);
    const nextStory = pickNextStory(prd);

    if (!nextStory) {
      log("All stories complete!");
      break;
    }

    const iterationStart = Date.now();
    resetToolCounts();

    emit({
      type: "iteration_started",
      iteration: i + 1,
      story_id: nextStory.id,
      story_title: nextStory.title,
    });

    log(`Iteration ${i + 1}/${config.maxIterations}: ${nextStory.id} "${nextStory.title}"`);

    try {
      let iterationCost = 0;

      const conversation = query({
        prompt: buildIterationPrompt(nextStory, prd),
        options: {
          systemPrompt: systemPrompt,
          allowedTools: [
            "Read",
            "Edit",
            "Write",
            "Bash",
            "Glob",
            "Grep",
            "Task",
            "WebFetch",
            "WebSearch",
            "NotebookEdit",
          ],
          permissionMode: "bypassPermissions",
          cwd: config.projectDir,
          settingSources: ["project"],
          hooks: {
            PostToolUse: [{ hooks: [postToolUseHook] }],
          },
        },
      });

      for await (const message of conversation) {
        switch (message.type) {
          case "assistant": {
            // Extract text blocks and emit them
            if (message.message?.content) {
              for (const block of message.message.content) {
                if ("text" in block && block.text) {
                  emit({ type: "agent_text", text: block.text });
                }
              }
            }
            break;
          }
          case "result": {
            if ("total_cost_usd" in message) {
              iterationCost = message.total_cost_usd ?? 0;
            }
            if (message.subtype !== "success") {
              const errors =
                "errors" in message ? (message.errors as string[]).join("; ") : "Unknown error";
              log(`Iteration failed: ${errors}`);
              emit({
                type: "iteration_failed",
                iteration: i + 1,
                story_id: nextStory.id,
                error: errors,
              });
            }
            break;
          }
          // system, user, stream_event - no action needed
        }
      }

      totalCost += iterationCost;
      const durationSecs = Math.round((Date.now() - iterationStart) / 1000);

      emit({
        type: "iteration_completed",
        iteration: i + 1,
        story_id: nextStory.id,
        cost: iterationCost,
        duration_secs: durationSecs,
        tools: getToolCounts(),
      });

      // Re-read PRD to check if story was marked complete
      prd = readPrd(config.prdPath);
      const updatedStory = prd.userStories.find((s) => s.id === nextStory.id);
      if (updatedStory?.passes) {
        storiesCompletedThisRun++;
        const progress = storyProgress(prd);
        emit({
          type: "story_completed",
          story_id: nextStory.id,
          passed: progress.completed,
          total: progress.total,
        });
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : String(error);
      log(`Iteration error: ${errorMessage}`);
      emit({
        type: "iteration_failed",
        iteration: i + 1,
        story_id: nextStory.id,
        error: errorMessage,
      });
    }
  }

  const totalDuration = Math.round((Date.now() - runStart) / 1000);
  const finalProgress = storyProgress(prd);

  if (finalProgress.completed === finalProgress.total) {
    log(`Run complete! All ${finalProgress.total} stories done.`);
  } else {
    log(
      `Run finished after ${config.maxIterations} iterations. ` +
        `${finalProgress.completed}/${finalProgress.total} stories complete.`,
    );
  }

  emit({
    type: "run_completed",
    run_id: config.runId,
    total_cost: totalCost,
    stories_completed: finalProgress.completed,
    duration_secs: totalDuration,
  });
}
