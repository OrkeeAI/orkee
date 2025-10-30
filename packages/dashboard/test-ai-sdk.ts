// ABOUTME: Test script to debug Vercel AI SDK with proxy
// ABOUTME: Tests generateObject with Anthropic provider through our proxy

import { createAnthropic } from '@ai-sdk/anthropic';
import { generateObject } from 'ai';
import { z } from 'zod';

const TestSchema = z.object({
  greeting: z.string().describe('A greeting message'),
  words: z.number().describe('Number of words in the greeting'),
});

async function testAISDK() {
  console.log('Testing Vercel AI SDK with proxy...\n');

  // Configure Anthropic provider with proxy
  const apiBaseUrl = 'http://localhost:4001';
  const anthropic = createAnthropic({
    apiKey: 'dummy-key-proxy-will-handle',
    baseURL: `${apiBaseUrl}/api/ai/anthropic/v1`,
  });

  console.log(`Using proxy at: ${apiBaseUrl}/api/ai/anthropic/v1\n`);

  try {
    console.log('Calling generateObject...');
    const result = await generateObject({
      model: anthropic('claude-haiku-4-5-20251001'),
      schema: TestSchema,
      prompt: 'Say hello in exactly 3 words',
      maxTokens: 1024,
    });

    console.log('\n✅ Success!');
    console.log('Generated object:', result.object);
    console.log('Usage:', result.usage);
  } catch (error) {
    console.error('\n❌ Error:', error);
    if (error instanceof Error) {
      console.error('Message:', error.message);
      console.error('Stack:', error.stack);
    }
  }
}

testAISDK();
