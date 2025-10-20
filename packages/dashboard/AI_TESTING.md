# AI Integration Testing Guide

## Quick Start

### 1. Configure API Keys

Create or update `packages/dashboard/.env`:

```bash
# Option 1: Anthropic (Recommended - better for code/specs)
VITE_ANTHROPIC_API_KEY=sk-ant-your-key-here

# Option 2: OpenAI
VITE_OPENAI_API_KEY=sk-your-key-here

# Optional: Vercel AI Gateway (for observability)
VITE_VERCEL_AI_GATEWAY_URL=https://gateway.vercel.sh
VITE_VERCEL_AI_GATEWAY_KEY=your-gateway-key
```

**Note**: You only need ONE provider (OpenAI or Anthropic). The system will prefer Anthropic if both are configured.

### 2. Restart Dev Server

```bash
cd packages/dashboard
bun run dev
```

### 3. Test the AI Integration

We've created a test dialog component that you can add to any page to test AI functionality.

#### Quick Test in Browser Console

Open your browser console and run:

```javascript
import { aiSpecService } from '@/lib/ai';

// Test PRD analysis
const result = await aiSpecService.analyzePRD(`
# User Authentication
Users need to be able to log in securely.
`);

console.log('Analysis:', result.data);
console.log('Cost:', result.cost.estimatedCost, 'USD');
```

#### Using the Test Dialog Component

Add the `AITestDialog` to any page:

```typescript
import { AITestDialog } from '@/components/AITestDialog';
import { useState } from 'react';

function MyPage() {
  const [showAITest, setShowAITest] = useState(false);

  return (
    <>
      <button onClick={() => setShowAITest(true)}>Test AI</button>
      <AITestDialog open={showAITest} onOpenChange={setShowAITest} />
    </>
  );
}
```

#### Using the React Hooks

```typescript
import { useAnalyzePRD } from '@/hooks/useAI';

function MyComponent() {
  const analyzeMutation = useAnalyzePRD();

  const handleAnalyze = () => {
    analyzeMutation.mutate(prdContent, {
      onSuccess: (result) => {
        console.log('Capabilities:', result.data.capabilities);
        console.log('Cost:', result.cost.estimatedCost, 'USD');
        console.log('Tokens:', result.usage.totalTokens);
      },
      onError: (error) => {
        console.error('Analysis failed:', error);
      },
    });
  };

  return (
    <button onClick={handleAnalyze} disabled={analyzeMutation.isPending}>
      {analyzeMutation.isPending ? 'Analyzing...' : 'Analyze PRD'}
    </button>
  );
}
```

## Available Hooks

All hooks are available in `@/hooks/useAI`:

### 1. `useAnalyzePRD()`
Analyze a PRD document and extract capabilities.

```typescript
const analyzeMutation = useAnalyzePRD();
analyzeMutation.mutate(prdContent);
```

### 2. `useGenerateSpec()`
Generate a spec from requirements.

```typescript
const generateMutation = useGenerateSpec();
generateMutation.mutate({
  capabilityName: 'User Authentication',
  purpose: 'Secure user login',
  requirements: ['Email login', 'Password reset'],
});
```

### 3. `useSuggestTasks()`
Generate task suggestions from a capability.

```typescript
const suggestMutation = useSuggestTasks();
suggestMutation.mutate({
  capability: myCapability,
  existingTasks: ['Task 1', 'Task 2'],
});
```

### 4. `useAnalyzeOrphanTask()`
Analyze where an orphan task fits.

```typescript
const analyzeMutation = useAnalyzeOrphanTask();
analyzeMutation.mutate({
  task: { title: 'Add OAuth', description: 'Implement OAuth login' },
  existingCapabilities: capabilities,
});
```

### 5. `useValidateTask()`
Validate task completion against scenarios.

```typescript
const validateMutation = useValidateTask();
validateMutation.mutate({
  task: { title: 'Login', description: 'User login feature', implementation: '...' },
  scenarios: [{ name: 'Valid login', when: '...', then: '...' }],
});
```

### 6. `useRefineSpec()`
Refine a spec based on feedback.

```typescript
const refineMutation = useRefineSpec();
refineMutation.mutate({
  capability: myCapability,
  feedback: 'Add error handling scenarios',
});
```

### 7. `usePRDWorkflow()`
Complete PRD → Spec → Task workflow with progress tracking.

```typescript
const workflowMutation = usePRDWorkflow();
workflowMutation.mutate({
  prdId: 'prd-123',
  prdContent: '...',
  projectId: 'proj-456',
  onProgress: (step, progress) => {
    console.log(`${step} (${progress}%)`);
  },
});
```

### 8. `useAIConfiguration()`
Check if AI is configured.

```typescript
const { data: config } = useAIConfiguration();

if (config?.isConfigured) {
  console.log('Provider:', config.preferredProvider);
} else {
  console.log('AI not configured');
}
```

## Testing Checklist

- [ ] Verify API key is set correctly (check browser Network tab for 401 errors)
- [ ] Test PRD analysis with sample content
- [ ] Verify cost tracking is working
- [ ] Check token usage is reasonable
- [ ] Test error handling (invalid API key, network error, etc.)
- [ ] Verify progress callbacks work in workflows
- [ ] Test with both OpenAI and Anthropic (if you have both keys)

## Cost Estimates

Approximate costs per operation (with Claude 3.5 Sonnet):

- **PRD Analysis** (1000 word PRD): ~$0.02 - $0.05
- **Spec Generation** (1 capability): ~$0.01 - $0.03
- **Task Suggestions** (1 capability): ~$0.01 - $0.02
- **Full Workflow** (1 PRD with 3 capabilities): ~$0.10 - $0.20

OpenAI GPT-4 Turbo costs are similar. GPT-3.5 Turbo is ~10x cheaper but lower quality.

## Troubleshooting

### "No AI provider configured"
**Solution**: Set `VITE_OPENAI_API_KEY` or `VITE_ANTHROPIC_API_KEY` in your `.env` file and restart the dev server.

### "API key not configured"
**Solution**: Make sure the environment variable starts with `VITE_` (required for Vite).

### High costs
**Solution**:
1. Use cheaper models (set in `lib/ai/config.ts`)
2. Reduce `maxTokens` in config
3. Cache results for repeated queries

### API Rate Limits
**Solution**:
1. Add delays between requests
2. Implement proper rate limiting (TODO in Week 5)
3. Use Vercel AI Gateway for better rate limit handling

### Network Errors
**Solution**:
1. Check your internet connection
2. Verify API keys are valid
3. Check for firewall/proxy issues

## Next Steps

Once basic testing is working:

1. **Integrate into PRDUploadDialog**: Replace placeholder with real AI analysis
2. **Add to SpecBuilderWizard**: Use AI suggestions for requirements
3. **Backend Proxy**: Move AI calls server-side for production security
4. **Cost Dashboard**: Build UI to track and monitor AI usage
5. **Caching Layer**: Cache results to reduce API calls
6. **Rate Limiting**: Implement client-side rate limiting

## Security Note ⚠️

**Current implementation exposes API keys in the browser**. This is acceptable for development but NOT for production.

For production:
1. Move AI service to backend
2. Proxy all AI calls through your server
3. Never send API keys to the browser
4. Implement server-side rate limiting and cost controls

See `AI_INTEGRATION.md` for details on backend integration.
