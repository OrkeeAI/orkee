# AI Integration Guide for OpenSpec

This document explains how to use the AI service layer that was implemented in Week 5.

## Overview

The AI integration provides the following capabilities:
- **PRD Analysis**: Extract capabilities, requirements, and scenarios from PRD documents
- **Spec Generation**: Generate OpenSpec specifications from requirements
- **Task Suggestions**: Generate implementation tasks from specs
- **Orphan Task Analysis**: Suggest where unlinked tasks fit in the spec
- **Task Validation**: Validate task completion against spec scenarios
- **Spec Refinement**: Improve specs based on feedback
- **Markdown Generation**: Generate formatted OpenSpec markdown documents

## Architecture

```
┌─────────────────────────────────────────────────┐
│ Frontend (packages/dashboard/src/lib/ai/)      │
│                                                 │
│  ┌──────────────┐  ┌──────────────┐           │
│  │  Providers   │  │   Schemas    │           │
│  │  (OpenAI,    │  │   (Zod)      │           │
│  │  Anthropic)  │  │              │           │
│  └──────────────┘  └──────────────┘           │
│         │                  │                    │
│         └─────────┬────────┘                    │
│                   │                             │
│          ┌────────▼────────┐                   │
│          │   AI Service    │                   │
│          │   (services.ts) │                   │
│          └────────┬────────┘                   │
│                   │                             │
│          ┌────────▼────────┐                   │
│          │   Workflows     │                   │
│          │  (PRD→Spec→Task)│                   │
│          └─────────────────┘                   │
└─────────────────────────────────────────────────┘
```

## Setup

### 1. Install Dependencies (✅ Complete)

```bash
cd packages/dashboard
bun add ai @ai-sdk/openai @ai-sdk/anthropic @ai-sdk/react @ai-sdk/ui-utils zod zod-to-json-schema
```

### 2. Configure Environment Variables

Create a `.env` file in `packages/dashboard/`:

```bash
# OpenAI Configuration (Optional)
VITE_OPENAI_API_KEY=sk-...

# Anthropic Configuration (Recommended for code analysis)
VITE_ANTHROPIC_API_KEY=sk-ant-...

# Vercel AI Gateway (Optional - for observability)
VITE_VERCEL_AI_GATEWAY_URL=https://gateway.vercel.sh
VITE_VERCEL_AI_GATEWAY_KEY=your-gateway-key
```

**Note**: At least one provider (OpenAI or Anthropic) must be configured.

### 3. Provider Priority

The system prefers Anthropic Claude (better for code/spec analysis) but will fall back to OpenAI if only that is configured:

```typescript
import { getPreferredProvider, isProviderConfigured } from '@/lib/ai';

// Check if any provider is configured
if (!isProviderConfigured('openai') && !isProviderConfigured('anthropic')) {
  console.error('No AI provider configured!');
}
```

## Usage Examples

### Example 1: Analyze a PRD

```typescript
import { aiSpecService } from '@/lib/ai';

async function analyzePRD(prdContent: string) {
  try {
    const result = await aiSpecService.analyzePRD(prdContent);

    console.log('Analysis:', result.data);
    console.log('Cost:', result.cost.estimatedCost, 'USD');
    console.log('Tokens used:', result.usage.totalTokens);

    // result.data.capabilities: Array of extracted capabilities
    // result.data.suggestedTasks: Optional task suggestions
    // result.data.dependencies: External dependencies identified
  } catch (error) {
    console.error('PRD analysis failed:', error);
  }
}
```

### Example 2: Generate a Spec

```typescript
import { aiSpecService } from '@/lib/ai';

async function generateSpec() {
  const result = await aiSpecService.generateSpec(
    'User Authentication',
    'Allow users to securely sign in and manage their accounts',
    [
      'Users should be able to register with email',
      'Users should be able to log in with credentials',
      'Users should be able to reset passwords',
    ]
  );

  console.log('Generated spec:', result.data);
  // result.data.id: kebab-case ID (e.g., "user-authentication")
  // result.data.requirements: Array with scenarios
}
```

### Example 3: Complete PRD → Spec → Task Workflow

```typescript
import { createSpecWorkflow } from '@/lib/ai';

async function processNewPRD(prdId: string, prdContent: string, projectId: string) {
  const workflow = createSpecWorkflow((step, progress) => {
    console.log(`${step} (${progress}%)`);
  });

  const result = await workflow.processNewPRD(prdId, prdContent, projectId);

  console.log('Total cost:', result.totalCost, 'USD');
  console.log('Capabilities created:', result.capabilities.length);

  // Now save to database
  for (const { capability, specMarkdown } of result.capabilities) {
    await saveCapabilityToDatabase(capability, specMarkdown);
  }
}
```

### Example 4: Analyze Orphan Tasks

```typescript
import { aiSpecService } from '@/lib/ai';

async function analyzeOrphanTask(task, existingCapabilities) {
  const result = await aiSpecService.analyzeOrphanTask(task, existingCapabilities);

  if (result.data.suggestedCapability.existing) {
    console.log('Link to existing capability:', result.data.suggestedCapability.capabilityId);
  } else {
    console.log('Create new capability:', result.data.suggestedCapability.capabilityName);
  }

  console.log('Suggested requirement:', result.data.suggestedRequirement);
  console.log('Confidence:', result.data.confidence);
}
```

### Example 5: Validate Task Completion

```typescript
import { aiSpecService } from '@/lib/ai';

async function validateTask(task, scenarios) {
  const result = await aiSpecService.validateTaskCompletion(task, scenarios);

  console.log('Overall passed:', result.data.overallPassed);

  for (const scenarioResult of result.data.scenarioResults) {
    console.log(`${scenarioResult.scenarioName}: ${scenarioResult.passed ? '✅' : '❌'}`);
    console.log(`  Confidence: ${(scenarioResult.confidence * 100).toFixed(0)}%`);
  }

  if (result.data.recommendations) {
    console.log('Recommendations:', result.data.recommendations);
  }
}
```

## Integration with Backend

The backend has placeholder AI endpoints at:
- `POST /api/ai/analyze-prd`
- `POST /api/ai/generate-spec`
- `POST /api/ai/suggest-tasks`
- `POST /api/ai/refine-spec`
- `POST /api/ai/validate-completion`

### Option 1: Client-Side AI (Development)

For development, call the AI service directly from React components:

```typescript
import { aiSpecService } from '@/lib/ai';
import { useMutation } from '@tanstack/react-query';

function MyComponent() {
  const analyzeMutation = useMutation({
    mutationFn: (prdContent: string) => aiSpecService.analyzePRD(prdContent),
    onSuccess: (result) => {
      console.log('Analysis complete:', result.data);
    },
  });

  return (
    <button onClick={() => analyzeMutation.mutate(prdContent)}>
      Analyze PRD
    </button>
  );
}
```

**⚠️ Security Warning**: This exposes API keys in the browser. Only use for development.

### Option 2: Server-Side Proxy (Production)

For production, create a backend proxy that:
1. Receives requests from frontend
2. Calls AI providers server-side (keeps API keys secure)
3. Returns results to frontend

```typescript
// Backend proxy (to be implemented)
async fn analyze_prd(Json(request): Json<AnalyzePRDRequest>) -> impl IntoResponse {
    // Call Node.js AI service or implement in Rust
    let result = call_ai_service("analyze-prd", request).await?;
    Ok(Json(ApiResponse::success(result)))
}
```

## Cost Management

All AI operations return cost estimates:

```typescript
const result = await aiSpecService.analyzePRD(prdContent);

console.log('Cost breakdown:');
console.log('- Input tokens:', result.cost.inputTokens);
console.log('- Output tokens:', result.cost.outputTokens);
console.log('- Total cost:', result.cost.estimatedCost, 'USD');
console.log('- Model:', result.cost.model);
console.log('- Provider:', result.cost.provider);
```

Cost calculation is based on the model pricing in `config.ts`:

```typescript
import { calculateCost } from '@/lib/ai';

const cost = calculateCost('anthropic', 'claude-3-5-sonnet-20241022', 1000, 500);
// Returns cost in USD
```

## Rate Limiting

Rate limiting is configured in `config.ts`:

```typescript
export const AI_CONFIG = {
  rateLimits: {
    requestsPerMinute: 60,
    tokensPerMinute: 100000,
  },
};
```

To implement rate limiting:

```typescript
// TODO: Implement rate limiting middleware
// See Week 5 tasks
```

## Caching

To implement caching for repeated queries:

```typescript
// TODO: Implement caching layer
// Cache key: hash(operation + parameters)
// TTL: 1 hour for static content, 5 minutes for dynamic
// See Week 5 tasks
```

## Streaming Support

For real-time AI responses:

```typescript
// TODO: Implement streaming support
// Use streamObject() and streamText() from AI SDK
// See Week 5 tasks
```

## Testing

To write tests:

```typescript
// TODO: Implement tests
// - Mock AI responses
// - Test schema validation
// - Test cost calculation
// - Test workflow orchestration
// See Week 5 tasks
```

## Troubleshooting

### "No AI provider configured"

Solution: Set at least one of `VITE_OPENAI_API_KEY` or `VITE_ANTHROPIC_API_KEY` in your `.env` file.

### "API key not configured"

Solution: Ensure environment variables are prefixed with `VITE_` and restart the dev server.

### "Model not found in configuration"

Solution: The requested model isn't in `config.ts`. Either add it or use a different model.

### High costs

Solution:
1. Reduce `maxTokens` in config
2. Use cheaper models (gpt-3.5-turbo, claude-3-haiku)
3. Implement caching for repeated queries
4. Enable rate limiting

## Next Steps (Week 5 Remaining Tasks)

- [ ] **Streaming Support**: Add real-time streaming for long AI operations
- [ ] **Cost Tracking UI**: Build dashboard to monitor AI usage and costs
- [ ] **Rate Limiting**: Implement client-side rate limiting
- [ ] **Caching Layer**: Add response caching to reduce API calls
- [ ] **Integration Tests**: Test complete workflows end-to-end
- [ ] **Backend Proxy**: Implement secure server-side AI proxy
- [ ] **Error Handling**: Add retry logic and better error messages
- [ ] **Telemetry**: Track AI operations for analytics

## File Structure

```
packages/dashboard/src/lib/ai/
├── config.ts           # AI provider configuration
├── providers.ts        # OpenAI & Anthropic provider setup
├── schemas.ts          # Zod schemas for validation
├── services.ts         # Main AI service methods
└── index.ts            # Public API exports

packages/dashboard/src/lib/workflows/
└── spec-workflow.ts    # High-level workflow orchestration
```

## Models Available

### OpenAI
- `gpt-4-turbo` (default)
- `gpt-4`
- `gpt-3.5-turbo` (cheapest)

### Anthropic
- `claude-3-5-sonnet-20241022` (default, recommended)
- `claude-3-opus-20240229` (most capable)
- `claude-3-sonnet-20240229`
- `claude-3-haiku-20240307` (cheapest)

## Support

For questions or issues with the AI integration, refer to:
- Vercel AI SDK docs: https://sdk.vercel.ai/docs
- OpenAI API docs: https://platform.openai.com/docs
- Anthropic API docs: https://docs.anthropic.com/
