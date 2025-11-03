# AI Usage Tracking Implementation Plan

## Overview
Implement comprehensive AI usage tracking for Orkee, capturing all AI SDK calls including tool invocations, token usage, costs, and performance metrics. The goal is to ensure the Usage tab displays accurate, real-time data for all AI operations.

## Current State Analysis

### Problems Identified
1. **Frontend AI SDK calls are not tracked** - Dashboard makes AI SDK calls but doesn't send telemetry to backend
2. **Tool calls are not tracked** - No infrastructure to capture which tools are called or how often
3. **Duration tracking is broken** - Always shows 0ms (TODO in `ai_handlers.rs:414`)
4. **No centralized telemetry** - Each endpoint manually logs usage, easy to miss calls
5. **Split architecture** - Rust backend uses direct Anthropic SDK, TypeScript frontend uses AI SDK

### Existing Infrastructure

#### Database Schema (`ai_usage_logs` table)
```sql
- id INTEGER PRIMARY KEY
- project_id INTEGER (nullable, FK to projects)
- request_id TEXT
- operation TEXT NOT NULL
- model TEXT NOT NULL
- provider TEXT NOT NULL
- input_tokens INTEGER DEFAULT 0
- output_tokens INTEGER DEFAULT 0
- total_tokens INTEGER DEFAULT 0
- estimated_cost REAL DEFAULT 0.0
- duration_ms INTEGER DEFAULT 0
- error TEXT (nullable)
- created_at TEXT DEFAULT CURRENT_TIMESTAMP
```

#### Current Tracking Locations
- **Rust Backend**: `packages/api/src/ai_handlers.rs` (lines 399-423)
- **TypeScript Frontend**: Manual extraction in various files:
  - `packages/dashboard/src/lib/ai/services.ts`
  - `packages/dashboard/src/services/chat-ai.ts`
  - `packages/dashboard/src/services/ai-brainstorm.ts`

## Implementation Phases

### Phase 1: Database Schema Enhancement ✅ COMPLETED
- [x] Create migration file - Modified `packages/storage/migrations/001_initial_schema.sql` directly
- [x] Add tool tracking columns:
  ```sql
  tool_calls_count INTEGER DEFAULT 0
  tool_calls_json TEXT
  response_metadata TEXT
  ```
- [x] Update Rust types in `packages/ai/src/usage_logs/types.rs` to include new fields
- [x] Update storage layer in `packages/ai/src/usage_logs/storage.rs` with new columns
- [x] Update all AiUsageLog initializers in `packages/api/src/ai_handlers.rs`
- [x] Build verified - all code compiles successfully

### Phase 2: Backend API Endpoint ✅ COMPLETED
- [x] Create `POST /api/ai/usage` endpoint in `packages/api/src/ai_usage_log_handlers.rs`
- [x] Add request/response types:
  ```rust
  #[derive(Deserialize)]
  pub struct CreateLogRequest {
      pub project_id: Option<String>,
      pub request_id: Option<String>,
      pub operation: String,
      pub model: String,
      pub provider: String,
      pub input_tokens: i32,
      pub output_tokens: i32,
      pub total_tokens: i32,
      pub estimated_cost: f64,
      pub duration_ms: i32,
      pub tool_calls_count: Option<i32>,
      pub tool_calls_json: Option<String>,
      pub response_metadata: Option<String>,
      pub error: Option<String>,
  }
  ```
- [x] Implement validation and storage logic with comprehensive validation:
  - Required field validation (operation, model, provider)
  - Non-negative value validation (tokens, cost, duration)
  - JSON validation for tool_calls_json and response_metadata
  - Foreign key constraint handling for project_id
- [x] Add endpoint to router in `packages/api/src/lib.rs`
- [x] Build verified - compiles successfully
- [x] Endpoint functional at `POST /api/ai-usage`
  - **Note**: Requires valid project_id due to foreign key constraint (phase 1 schema)
  - Returns proper error messages for validation failures
  - Returns 201 Created with log ID on success

### Phase 3: Frontend Telemetry Infrastructure ✅ COMPLETED

#### 3.1: Core Telemetry Module ✅ COMPLETED
- [x] Create `packages/dashboard/src/lib/ai/telemetry.ts`:
  ```typescript
  interface AITelemetryData {
    operation: string;
    projectId?: string | null;
    model: string;
    provider: string;
    inputTokens: number;
    outputTokens: number;
    totalTokens: number;
    estimatedCost: number;
    durationMs: number;
    toolCallsCount: number;
    toolCallsJson?: string;
    responseMetadata?: string;
    error?: string;
  }

  interface ToolCall {
    name: string;
    arguments: Record<string, any>;
    result?: any;
    durationMs?: number;
    error?: string;
  }

  // Main wrapper function
  export async function trackAIOperation<T>(
    operation: string,
    projectId: string | null,
    aiFunction: () => Promise<T>
  ): Promise<T>

  // Tool call extraction
  export function extractToolCalls(response: any): ToolCall[]

  // Send telemetry to backend
  async function sendTelemetry(data: AITelemetryData): Promise<void>
  ```

#### 3.2: AI SDK Response Parsers ✅ COMPLETED
- [x] Create parsers for different AI SDK response types:
  - [x] `parseGenerateTextResponse()` - Integrated into trackAIOperation
  - [x] `parseStreamTextResponse()` - Integrated into trackAIOperation
  - [x] `parseGenerateObjectResponse()` - Integrated into trackAIOperation
  - [x] Extract tool calls from `experimental_toolCalls` or response content
- [x] Handle streaming responses with `onFinish` callback
- [x] Capture accurate timing with `performance.now()`

#### 3.3: Provider Detection ✅ COMPLETED
- [x] Implement `detectProvider(model: string)` function:
  ```typescript
  function detectProvider(model: string): string {
    if (model.includes('gpt')) return 'openai';
    if (model.includes('claude')) return 'anthropic';
    if (model.includes('gemini')) return 'google';
    if (model.includes('llama')) return 'meta';
    if (model.includes('grok')) return 'xai';
    return 'unknown';
  }
  ```

#### 3.4: Cost Calculation ✅ COMPLETED
- [x] Update `calculateCost()` function to handle tool usage
  - Note: Tool usage is already included in token counts from providers
  - Cost calculation remains based on input/output tokens
- [x] Add tool invocation costs (if applicable)
  - Note: Not needed - providers bill tool usage as part of token counts
- [x] Consider different pricing for tool-enabled models
  - Note: Current pricing model is sufficient

### Phase 4: Refactor Existing AI Calls

#### 4.1: Update Core AI Services
- [ ] `packages/dashboard/src/lib/ai/services.ts`:
  - [ ] Wrap `generateProjectIdeas()`
  - [ ] Wrap `generateTasks()`
  - [ ] Wrap `generateProjectDescription()`
  - [ ] Wrap `generateReadme()`
  - [ ] Update all other AI functions

#### 4.2: Update Chat Services
- [ ] `packages/dashboard/src/services/chat-ai.ts`:
  - [ ] Wrap `sendMessage()`
  - [ ] Wrap streaming responses
  - [ ] Track tool calls in chat context

#### 4.3: Update Brainstorm Services
- [ ] `packages/dashboard/src/services/ai-brainstorm.ts`:
  - [ ] Wrap all brainstorm functions
  - [ ] Track ideation operations

#### 4.4: Update Component Hooks
- [ ] `packages/dashboard/src/pages/ai-chat/hooks.ts`:
  - [ ] Update `useAIChat()` hook
  - [ ] Ensure telemetry in all AI operations

#### 4.5: Search for Additional AI SDK Usage
- [ ] Grep for `import.*from.*'ai'` to find all usage
- [ ] Grep for `generateText`, `streamText`, `generateObject`
- [ ] Ensure no AI calls are missed

### Phase 5: Enhanced Usage Dashboard

#### 5.1: Update API Endpoints
- [ ] Create `GET /api/ai/usage/summary` for dashboard data
- [ ] Add tool usage statistics endpoint
- [ ] Add time-series data endpoint for charts

#### 5.2: Update Dashboard Components
- [ ] `packages/dashboard/src/pages/usage/index.tsx`:
  - [ ] Add tool calls metrics card
  - [ ] Show tool breakdown chart
  - [ ] Display most used tools
  - [ ] Add tool success/failure rates

#### 5.3: New Visualizations
- [ ] Tool usage pie chart
- [ ] Tool calls over time line chart
- [ ] Tool performance metrics (avg duration)
- [ ] Failed tool calls analysis

### Phase 6: Testing & Validation

#### 6.1: Unit Tests
- [ ] Test telemetry wrapper functions
- [ ] Test tool call extraction
- [ ] Test cost calculation with tools
- [ ] Test API endpoint

#### 6.2: Integration Tests
- [ ] Create test project
- [ ] Execute various AI operations:
  - [ ] Generate project ideas
  - [ ] Send chat messages
  - [ ] Generate tasks
  - [ ] Brainstorm features
- [ ] Verify database entries
- [ ] Check tool call tracking
- [ ] Validate dashboard display

#### 6.3: Performance Tests
- [ ] Ensure telemetry doesn't slow down AI operations
- [ ] Test with high-volume usage
- [ ] Verify no memory leaks

### Phase 7: Documentation & Cleanup

#### 7.1: Documentation
- [ ] Update README with telemetry information
- [ ] Document telemetry wrapper usage
- [ ] Add examples for new developers
- [ ] Document tool call data structure

#### 7.2: Code Cleanup
- [ ] Remove TODO comments for duration tracking
- [ ] Remove manual usage extraction code
- [ ] Consolidate duplicate telemetry logic
- [ ] Add proper error handling

#### 7.3: Monitoring
- [ ] Add logging for telemetry failures
- [ ] Create alerts for tracking issues
- [ ] Monitor telemetry endpoint performance

## Implementation Details

### Telemetry Wrapper Example
```typescript
// packages/dashboard/src/lib/ai/telemetry.ts

import { performance } from 'perf_hooks';

export async function trackAIOperation<T>(
  operation: string,
  projectId: string | null,
  aiFunction: () => Promise<T>
): Promise<T> {
  const startTime = performance.now();

  try {
    const result = await aiFunction();

    // Handle different response types
    let telemetryData: AITelemetryData;

    if (isStreamResponse(result)) {
      // For streaming, use onFinish callback
      result.onFinish = async (finalResult) => {
        telemetryData = extractTelemetryData(finalResult);
        telemetryData.durationMs = performance.now() - startTime;
        await sendTelemetry(telemetryData);
      };
    } else {
      // For non-streaming, send immediately
      telemetryData = extractTelemetryData(result);
      telemetryData.durationMs = performance.now() - startTime;
      await sendTelemetry(telemetryData);
    }

    return result;
  } catch (error) {
    // Log failed attempt
    await sendTelemetry({
      operation,
      projectId,
      model: 'unknown',
      provider: 'unknown',
      inputTokens: 0,
      outputTokens: 0,
      totalTokens: 0,
      estimatedCost: 0,
      durationMs: performance.now() - startTime,
      toolCallsCount: 0,
      error: error.message
    });
    throw error;
  }
}

function extractTelemetryData(response: any): AITelemetryData {
  const usage = response.usage || {};
  const toolCalls = extractToolCalls(response);

  return {
    operation: response.operation || 'unknown',
    model: response.model || 'unknown',
    provider: detectProvider(response.model),
    inputTokens: usage.promptTokens || 0,
    outputTokens: usage.completionTokens || 0,
    totalTokens: usage.totalTokens || 0,
    estimatedCost: calculateCost(usage, response.model),
    durationMs: 0, // Set by caller
    toolCallsCount: toolCalls.length,
    toolCallsJson: toolCalls.length > 0 ? JSON.stringify(toolCalls) : undefined,
    responseMetadata: JSON.stringify({
      finishReason: response.finishReason,
      id: response.id,
      ...response.experimental_providerMetadata
    })
  };
}

export function extractToolCalls(response: any): ToolCall[] {
  const toolCalls: ToolCall[] = [];

  // Check various possible locations for tool calls
  if (response.experimental_toolCalls) {
    response.experimental_toolCalls.forEach((call: any) => {
      toolCalls.push({
        name: call.toolName,
        arguments: call.args,
        result: call.result
      });
    });
  }

  // Check for tool calls in response content (for older AI SDK versions)
  if (response.toolCalls) {
    response.toolCalls.forEach((call: any) => {
      toolCalls.push({
        name: call.name || call.tool,
        arguments: call.arguments || call.input,
        result: call.output
      });
    });
  }

  return toolCalls;
}
```

### Usage Pattern Example
```typescript
// Before (no tracking)
const result = await generateText({
  model: anthropic('claude-3-opus'),
  prompt: 'Generate project ideas',
  tools: { search, calculate }
});

// After (with tracking)
const result = await trackAIOperation(
  'generate_project_ideas',
  projectId,
  () => generateText({
    model: anthropic('claude-3-opus'),
    prompt: 'Generate project ideas',
    tools: { search, calculate }
  })
);
```

### Tool Call JSON Structure
```json
{
  "toolCallsCount": 3,
  "toolCallsJson": [
    {
      "name": "search",
      "arguments": {
        "query": "latest web frameworks 2024"
      },
      "result": {
        "results": ["Next.js 14", "Remix", "SvelteKit"]
      },
      "durationMs": 150
    },
    {
      "name": "calculate",
      "arguments": {
        "expression": "1024 * 1024"
      },
      "result": {
        "value": 1048576
      },
      "durationMs": 5
    }
  ]
}
```

## Success Metrics
1. **All AI operations tracked** - 100% coverage of AI SDK calls
2. **Tool visibility** - Complete tracking of tool invocations
3. **Accurate costs** - Token usage matches provider billing
4. **Performance data** - Real duration measurements (not 0ms)
5. **Dashboard accuracy** - Usage tab shows real-time data
6. **Zero manual tracking** - Automatic telemetry for all calls

## Rollback Plan
If issues arise:
1. Feature flag to disable telemetry (`ORKEE_AI_TELEMETRY_ENABLED`)
2. Telemetry failures don't break AI operations (fail silently with logging)
3. Keep existing manual tracking as fallback
4. Database changes are backward compatible

## Timeline Estimate
- Phase 1 (Database): 1-2 hours
- Phase 2 (Backend API): 2-3 hours
- Phase 3 (Frontend Infrastructure): 4-6 hours
- Phase 4 (Refactoring): 3-4 hours
- Phase 5 (Dashboard): 3-4 hours
- Phase 6 (Testing): 2-3 hours
- Phase 7 (Documentation): 1-2 hours

**Total: 16-24 hours of development**

## Notes
- Priority is accurate tracking over UI features
- Tool call tracking is essential for understanding AI costs
- Performance impact must be minimal (<5ms overhead)
- All telemetry is async to not block AI operations
- Consider adding OpenTelemetry support in future