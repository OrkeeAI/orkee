# Streaming PRD Generation Example

This document demonstrates how to use the streaming PRD generation feature with AI SDK.

## Overview

The streaming implementation allows real-time updates as the AI generates PRD content, providing immediate feedback to users instead of waiting for the entire response.

## Architecture

```
Frontend (React) → ideateService.quickGenerateStreaming()
                 → AIService.generateCompletePRDStreaming()
                 → AI SDK streamObject()
                 → Anthropic API (direct, no proxy)
```

## React Component Example

```typescript
import { useState } from 'react';
import { ideateService } from '@/services/ideate';

function PRDGenerator({ sessionId }: { sessionId: string }) {
  const [isGenerating, setIsGenerating] = useState(false);
  const [partialPRD, setPartialPRD] = useState<any>(null);
  const [finalPRD, setFinalPRD] = useState<any>(null);

  const handleGenerateWithStreaming = async () => {
    setIsGenerating(true);
    setPartialPRD(null);
    setFinalPRD(null);

    try {
      // Call streaming version with callback for partial updates
      const result = await ideateService.quickGenerateStreaming(
        sessionId,
        // Callback receives partial data as it streams in
        (partial) => {
          console.log('[Stream Update]', partial);
          setPartialPRD(partial);
        },
        // Optional: specify model
        { model: 'claude-sonnet-4-20250514' }
      );

      // Final complete result
      setFinalPRD(result);
      console.log('[Generation Complete]', result);
    } catch (error) {
      console.error('[Generation Error]', error);
    } finally {
      setIsGenerating(false);
    }
  };

  return (
    <div>
      <button
        onClick={handleGenerateWithStreaming}
        disabled={isGenerating}
      >
        {isGenerating ? 'Generating...' : 'Generate PRD'}
      </button>

      {/* Show partial results while streaming */}
      {isGenerating && partialPRD && (
        <div className="partial-results">
          <h3>Generating (Partial)...</h3>
          <pre>{JSON.stringify(partialPRD, null, 2)}</pre>
        </div>
      )}

      {/* Show final results */}
      {finalPRD && !isGenerating && (
        <div className="final-results">
          <h3>Final PRD</h3>
          <pre>{finalPRD.content}</pre>
        </div>
      )}
    </div>
  );
}

export default PRDGenerator;
```

## Usage in Ideate Service

```typescript
// Import the service
import { ideateService } from '@/services/ideate';

// Use streaming version
const result = await ideateService.quickGenerateStreaming(
  sessionId,
  (partial) => {
    // Handle partial updates
    console.log('Partial:', partial);
    // Update UI with partial data
    updateUI(partial);
  },
  { model: 'claude-sonnet-4-20250514' }
);

// Use non-streaming version (backwards compatible)
const result = await ideateService.quickGenerate(
  sessionId,
  { model: 'claude-sonnet-4-20250514' }
);
```

## Benefits of Streaming

1. **Real-time Feedback**: Users see content being generated in real-time
2. **Better UX**: No long waiting periods with loading spinners
3. **Progressive Enhancement**: UI can show partial content as it arrives
4. **Same Backend**: Both streaming and non-streaming save to database

## Implementation Details

### AI Service Methods

All generation methods have streaming versions:

- `generateCompletePRDStreaming()` - Full PRD with all sections
- `generateOverviewStreaming()` - Overview section only
- `generateFeaturesStreaming()` - Features section only
- `generateUXStreaming()` - UX section only
- `generateTechnicalStreaming()` - Technical architecture section
- `generateRoadmapStreaming()` - Roadmap section
- `generateDependenciesStreaming()` - Dependencies section
- `generateRisksStreaming()` - Risks section
- `generateResearchStreaming()` - Research section

### Direct Usage Example

```typescript
import { createAIService } from '@/services/ai/service';
import { usersService } from '@/services/users';

// Get API key from backend
const apiKey = await usersService.getAnthropicApiKey();

// Create AI service
const aiService = createAIService({
  apiKey,
  model: 'claude-sonnet-4-20250514',
  maxTokens: 64000,
  temperature: 0.7,
});

// Generate with streaming
const streamResult = await aiService.generateOverviewStreaming(description);

// Consume the stream
for await (const partial of streamResult.partialObjectStream) {
  console.log('Partial overview:', partial);
  // Update UI with partial.title, partial.description, etc.
}

// Get final result
const final = await streamResult.object;
const usage = await streamResult.usage;

console.log('Final overview:', final);
console.log('Token usage:', usage);
```

## Performance Considerations

- **Streaming is async**: The callback runs in parallel with the main promise
- **Partial data structure**: Partial objects may have incomplete fields (use optional chaining)
- **UI updates**: Throttle UI updates if needed to avoid excessive re-renders

## Error Handling

```typescript
try {
  const result = await ideateService.quickGenerateStreaming(
    sessionId,
    (partial) => {
      try {
        // Handle partial update safely
        updateUI(partial);
      } catch (error) {
        console.error('Partial update error:', error);
      }
    }
  );
} catch (error) {
  if (error instanceof AIGenerationError) {
    switch (error.code) {
      case 'NO_API_KEY':
        // Redirect to settings
        break;
      case 'API_ERROR':
        // Show error message
        break;
      case 'TIMEOUT':
        // Allow retry
        break;
    }
  }
}
```

## Testing

The implementation can be tested without modifying the backend:

```bash
# Run dev server
cd packages/dashboard
bun run dev

# Open browser and use streaming generation
# Watch console for stream updates
```

## Next Steps

To fully integrate streaming into the UI:

1. **Update Generation Buttons**: Add option to use streaming vs non-streaming
2. **Real-time Display**: Show partial content as it generates
3. **Progress Indicators**: Show which sections are being generated
4. **Animations**: Add smooth transitions as content appears
5. **Cancel Support**: Add ability to cancel mid-generation (future enhancement)

## API SDK Resources

- [Vercel AI SDK Documentation](https://sdk.vercel.ai/docs/introduction)
- [AI SDK Anthropic Provider](https://sdk.vercel.ai/providers/ai-sdk-providers/anthropic)
- [streamObject API Reference](https://sdk.vercel.ai/docs/reference/ai-sdk-core/stream-object)
