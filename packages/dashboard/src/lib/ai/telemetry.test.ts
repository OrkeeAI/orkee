// ABOUTME: Tests for AI telemetry tracking infrastructure
// ABOUTME: Validates tool extraction, provider detection, and tracking wrapper functions

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  extractToolCalls,
  detectProvider,
  extractTelemetryData,
  trackAIOperation,
  trackAIOperationWithCost,
  sendAIResultTelemetry,
  type AIResponse,
  type ToolCall,
} from './telemetry';

describe('extractToolCalls', () => {
  it('should extract tool calls from experimental_toolCalls', () => {
    const response = {
      experimental_toolCalls: [
        {
          toolName: 'search',
          args: { query: 'test' },
          result: { results: ['a', 'b'] },
        },
        {
          toolName: 'calculate',
          args: { expression: '2+2' },
          result: { value: 4 },
        },
      ],
    };

    const toolCalls = extractToolCalls(response);

    expect(toolCalls).toHaveLength(2);
    expect(toolCalls[0]).toEqual({
      name: 'search',
      arguments: { query: 'test' },
      result: { results: ['a', 'b'] },
    });
    expect(toolCalls[1]).toEqual({
      name: 'calculate',
      arguments: { expression: '2+2' },
      result: { value: 4 },
    });
  });

  it('should extract tool calls from toolCalls (legacy format)', () => {
    const response = {
      toolCalls: [
        {
          name: 'search',
          arguments: { query: 'legacy' },
          output: { results: ['x'] },
        },
      ],
    };

    const toolCalls = extractToolCalls(response);

    expect(toolCalls).toHaveLength(1);
    expect(toolCalls[0]).toEqual({
      name: 'search',
      arguments: { query: 'legacy' },
      result: { results: ['x'] },
    });
  });

  it('should handle response with no tool calls', () => {
    const response = {
      text: 'Just text, no tools',
    };

    const toolCalls = extractToolCalls(response);

    expect(toolCalls).toHaveLength(0);
  });

  it('should handle malformed tool calls gracefully', () => {
    const response = {
      experimental_toolCalls: [
        {
          // Missing toolName
          args: { query: 'test' },
        },
        {
          toolName: 'valid',
          // Missing args
          result: {},
        },
      ],
    };

    const toolCalls = extractToolCalls(response);

    expect(toolCalls).toHaveLength(2);
    expect(toolCalls[0].name).toBeUndefined();
    expect(toolCalls[0].arguments).toEqual({ query: 'test' });
    expect(toolCalls[1].name).toBe('valid');
    expect(toolCalls[1].arguments).toEqual({});
  });

  it('should prefer toolName over name', () => {
    const response = {
      experimental_toolCalls: [
        {
          toolName: 'correct',
          name: 'wrong',
          args: {},
        },
      ],
    };

    const toolCalls = extractToolCalls(response);

    expect(toolCalls[0].name).toBe('correct');
  });

  it('should prefer args over arguments', () => {
    const response = {
      experimental_toolCalls: [
        {
          toolName: 'test',
          args: { correct: true },
          arguments: { wrong: true },
        },
      ],
    };

    const toolCalls = extractToolCalls(response);

    expect(toolCalls[0].arguments).toEqual({ correct: true });
  });
});

describe('detectProvider', () => {
  it('should detect OpenAI models', () => {
    expect(detectProvider('gpt-4')).toBe('openai');
    expect(detectProvider('gpt-3.5-turbo')).toBe('openai');
    expect(detectProvider('GPT-4o')).toBe('openai');
    expect(detectProvider('openai/gpt-4')).toBe('openai');
  });

  it('should detect Anthropic models', () => {
    expect(detectProvider('claude-3-opus')).toBe('anthropic');
    expect(detectProvider('claude-sonnet-4-5-20250929')).toBe('anthropic');
    expect(detectProvider('CLAUDE-3-HAIKU')).toBe('anthropic');
    expect(detectProvider('anthropic/claude-3-opus')).toBe('anthropic');
  });

  it('should detect Google models', () => {
    expect(detectProvider('gemini-pro')).toBe('google');
    expect(detectProvider('gemini-1.5-flash')).toBe('google');
    expect(detectProvider('google/gemini-pro')).toBe('google');
  });

  it('should detect Meta models', () => {
    expect(detectProvider('llama-3-70b')).toBe('meta');
    expect(detectProvider('meta/llama-2')).toBe('meta');
  });

  it('should detect xAI models', () => {
    expect(detectProvider('grok-1')).toBe('xai');
    expect(detectProvider('xai/grok-2')).toBe('xai');
  });

  it('should return unknown for unrecognized models', () => {
    expect(detectProvider('mystery-model')).toBe('unknown');
    expect(detectProvider('custom-llm-v1')).toBe('unknown');
    expect(detectProvider('')).toBe('unknown');
  });

  it('should be case insensitive', () => {
    expect(detectProvider('GPT-4')).toBe('openai');
    expect(detectProvider('CLAUDE-3')).toBe('anthropic');
    expect(detectProvider('GEMINI-PRO')).toBe('google');
  });
});

describe('extractTelemetryData', () => {
  it('should extract basic telemetry data', () => {
    const response = {
      usage: {
        promptTokens: 100,
        completionTokens: 50,
        totalTokens: 150,
      },
      finishReason: 'stop',
      id: 'test-id-123',
    };

    const data = extractTelemetryData(
      response,
      'test_operation',
      'gpt-4',
      'openai',
      0.05
    );

    expect(data).toEqual({
      operation: 'test_operation',
      model: 'gpt-4',
      provider: 'openai',
      inputTokens: 100,
      outputTokens: 50,
      totalTokens: 150,
      estimatedCost: 0.05,
      toolCallsCount: 0,
      responseMetadata: JSON.stringify({
        finishReason: 'stop',
        id: 'test-id-123',
      }),
    });
  });

  it('should extract tool calls and count them', () => {
    const response = {
      usage: {
        promptTokens: 100,
        completionTokens: 50,
        totalTokens: 150,
      },
      experimental_toolCalls: [
        { toolName: 'search', args: {}, result: {} },
        { toolName: 'calculate', args: {}, result: {} },
      ],
    };

    const data = extractTelemetryData(response, 'test', 'gpt-4', 'openai', 0.05);

    expect(data.toolCallsCount).toBe(2);
    expect(data.toolCallsJson).toBeDefined();

    const parsedToolCalls = JSON.parse(data.toolCallsJson!);
    expect(parsedToolCalls).toHaveLength(2);
    expect(parsedToolCalls[0].name).toBe('search');
    expect(parsedToolCalls[1].name).toBe('calculate');
  });

  it('should handle missing usage data', () => {
    const response = {};

    const data = extractTelemetryData(response, 'test', 'gpt-4', 'openai', 0);

    expect(data.inputTokens).toBe(0);
    expect(data.outputTokens).toBe(0);
    expect(data.totalTokens).toBe(0);
  });

  it('should include provider metadata', () => {
    const response = {
      usage: { totalTokens: 100 },
      experimental_providerMetadata: {
        customField: 'value',
        anotherField: 123,
      },
    };

    const data = extractTelemetryData(response, 'test', 'gpt-4', 'openai', 0);

    const metadata = JSON.parse(data.responseMetadata!);
    expect(metadata.customField).toBe('value');
    expect(metadata.anotherField).toBe(123);
  });

  it('should not include toolCallsJson if no tools used', () => {
    const response = {
      usage: { totalTokens: 100 },
    };

    const data = extractTelemetryData(response, 'test', 'gpt-4', 'openai', 0);

    expect(data.toolCallsCount).toBe(0);
    expect(data.toolCallsJson).toBeUndefined();
  });
});

describe('trackAIOperation', () => {
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    // Mock fetch globally
    fetchMock = vi.fn();
    global.fetch = fetchMock;

    // Mock window object for tests
    (global as any).window = {
      location: {
        origin: 'http://localhost:5173',
      },
    };

    // Mock crypto.randomUUID
    global.crypto = {
      randomUUID: () => 'test-uuid-123',
    } as any;

    // Mock performance.now
    let time = 0;
    vi.spyOn(performance, 'now').mockImplementation(() => {
      time += 100;
      return time;
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should track non-streaming AI operation', async () => {
    const mockResponse: AIResponse = {
      text: 'Hello world',
      usage: {
        promptTokens: 10,
        completionTokens: 5,
        totalTokens: 15,
      },
      finishReason: 'stop',
      id: 'resp-123',
    };

    fetchMock.mockResolvedValueOnce({
      ok: true,
      status: 201,
    });

    const aiFunction = vi.fn().mockResolvedValue(mockResponse);

    const result = await trackAIOperation(
      'test_operation',
      'project-123',
      'gpt-4',
      'openai',
      aiFunction
    );

    expect(result).toBe(mockResponse);
    expect(aiFunction).toHaveBeenCalledOnce();

    // Verify telemetry was sent
    expect(fetchMock).toHaveBeenCalledOnce();
    const [url, options] = fetchMock.mock.calls[0];
    expect(url).toContain('/api/ai-usage');
    expect(options.method).toBe('POST');

    const body = JSON.parse(options.body);
    expect(body.operation).toBe('test_operation');
    expect(body.project_id).toBe('project-123');
    expect(body.model).toBe('gpt-4');
    expect(body.provider).toBe('openai');
    expect(body.input_tokens).toBe(10);
    expect(body.output_tokens).toBe(5);
    expect(body.total_tokens).toBe(15);
    expect(body.duration_ms).toBeGreaterThan(0);
  });

  it('should handle streaming responses with onFinish', async () => {
    const mockStreamResponse: AIResponse = {
      textStream: (async function* () {
        yield 'Hello';
        yield ' world';
      })(),
      onFinish: undefined,
    };

    fetchMock.mockResolvedValueOnce({ ok: true });

    const aiFunction = vi.fn().mockResolvedValue(mockStreamResponse);

    const result = await trackAIOperation(
      'stream_operation',
      'project-123',
      'gpt-4',
      'openai',
      aiFunction
    );

    expect(result.onFinish).toBeDefined();
    expect(result.textStream).toBeDefined();

    // Simulate onFinish callback
    const finalResult = {
      usage: {
        promptTokens: 20,
        completionTokens: 10,
        totalTokens: 30,
      },
      finishReason: 'stop',
    };

    await result.onFinish!(finalResult);

    // Verify telemetry was sent after stream finished
    expect(fetchMock).toHaveBeenCalledOnce();
    const body = JSON.parse(fetchMock.mock.calls[0][1].body);
    expect(body.input_tokens).toBe(20);
    expect(body.output_tokens).toBe(10);
  });

  it('should track errors and still send telemetry', async () => {
    const error = new Error('AI operation failed');

    fetchMock.mockResolvedValueOnce({ ok: true });

    const aiFunction = vi.fn().mockRejectedValue(error);

    await expect(
      trackAIOperation('failing_operation', null, 'gpt-4', 'openai', aiFunction)
    ).rejects.toThrow('AI operation failed');

    // Verify error telemetry was sent
    expect(fetchMock).toHaveBeenCalledOnce();
    const body = JSON.parse(fetchMock.mock.calls[0][1].body);
    expect(body.error).toBe('AI operation failed');
    expect(body.input_tokens).toBe(0);
    expect(body.output_tokens).toBe(0);
  });

  it('should handle telemetry send failures gracefully', async () => {
    const mockResponse: AIResponse = {
      text: 'Hello',
      usage: { totalTokens: 10 },
    };

    // Simulate telemetry endpoint failure
    fetchMock.mockRejectedValueOnce(new Error('Network error'));

    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});

    const aiFunction = vi.fn().mockResolvedValue(mockResponse);

    // Should not throw even if telemetry fails
    const result = await trackAIOperation(
      'test',
      null,
      'gpt-4',
      'openai',
      aiFunction
    );

    expect(result).toBe(mockResponse);
    expect(consoleSpy).toHaveBeenCalled();

    consoleSpy.mockRestore();
  });

  it('should handle null projectId', async () => {
    const mockResponse: AIResponse = {
      text: 'Hello',
      usage: { totalTokens: 10 },
    };

    fetchMock.mockResolvedValueOnce({ ok: true });

    const aiFunction = vi.fn().mockResolvedValue(mockResponse);

    await trackAIOperation('test', null, 'gpt-4', 'openai', aiFunction);

    const body = JSON.parse(fetchMock.mock.calls[0][1].body);
    expect(body.project_id).toBeNull();
  });
});

describe('trackAIOperationWithCost', () => {
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    fetchMock = vi.fn().mockResolvedValue({ ok: true });
    global.fetch = fetchMock;
    (global as any).window = {
      location: {
        origin: 'http://localhost:5173',
      },
    };
    global.crypto = { randomUUID: () => 'test-uuid' } as any;

    let time = 0;
    vi.spyOn(performance, 'now').mockImplementation(() => {
      time += 50;
      return time;
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should use custom cost calculation function', async () => {
    const mockResponse: AIResponse = {
      text: 'Result',
      usage: {
        promptTokens: 1000,
        completionTokens: 500,
        totalTokens: 1500,
      },
    };

    const calculateCost = vi.fn((input: number, output: number) => {
      return input * 0.00001 + output * 0.00003;
    });

    const aiFunction = vi.fn().mockResolvedValue(mockResponse);

    await trackAIOperationWithCost(
      'custom_cost',
      'project-123',
      'gpt-4',
      'openai',
      calculateCost,
      aiFunction
    );

    expect(calculateCost).toHaveBeenCalledWith(1000, 500);

    const body = JSON.parse(fetchMock.mock.calls[0][1].body);
    expect(body.estimated_cost).toBe(1000 * 0.00001 + 500 * 0.00003);
  });

  it('should work with streaming responses', async () => {
    const mockStreamResponse: AIResponse = {
      textStream: (async function* () {
        yield 'test';
      })(),
      onFinish: undefined,
    };

    const calculateCost = vi.fn((input: number, output: number) => {
      return (input + output) * 0.00002;
    });

    const aiFunction = vi.fn().mockResolvedValue(mockStreamResponse);

    const result = await trackAIOperationWithCost(
      'stream_with_cost',
      null,
      'claude-3-opus',
      'anthropic',
      calculateCost,
      aiFunction
    );

    const finalResult = {
      usage: { promptTokens: 100, completionTokens: 50, totalTokens: 150 },
    };

    await result.onFinish!(finalResult);

    expect(calculateCost).toHaveBeenCalledWith(100, 50);
  });
});

describe('sendAIResultTelemetry', () => {
  let fetchMock: ReturnType<typeof vi.fn>;

  beforeEach(() => {
    fetchMock = vi.fn().mockResolvedValue({ ok: true });
    global.fetch = fetchMock;
    (global as any).window = {
      location: {
        origin: 'http://localhost:5173',
      },
    };
    global.crypto = { randomUUID: () => 'test-uuid' } as any;
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('should send telemetry for AIResult pattern', async () => {
    await sendAIResultTelemetry(
      'analyze_prd',
      'project-123',
      'claude-sonnet-4-5-20250929',
      'anthropic',
      { inputTokens: 5000, outputTokens: 2000, totalTokens: 7000 },
      0.15,
      1500
    );

    expect(fetchMock).toHaveBeenCalledOnce();
    const body = JSON.parse(fetchMock.mock.calls[0][1].body);

    expect(body.operation).toBe('analyze_prd');
    expect(body.project_id).toBe('project-123');
    expect(body.model).toBe('claude-sonnet-4-5-20250929');
    expect(body.provider).toBe('anthropic');
    expect(body.input_tokens).toBe(5000);
    expect(body.output_tokens).toBe(2000);
    expect(body.total_tokens).toBe(7000);
    expect(body.estimated_cost).toBe(0.15);
    expect(body.duration_ms).toBe(1500);
    expect(body.tool_calls_count).toBe(0);
  });

  it('should include error if provided', async () => {
    await sendAIResultTelemetry(
      'failed_operation',
      null,
      'gpt-4',
      'openai',
      { inputTokens: 0, outputTokens: 0, totalTokens: 0 },
      0,
      100,
      'Operation timed out'
    );

    const body = JSON.parse(fetchMock.mock.calls[0][1].body);
    expect(body.error).toBe('Operation timed out');
  });

  it('should generate unique request IDs', async () => {
    const uuids = ['uuid-1', 'uuid-2', 'uuid-3'];
    let callCount = 0;
    global.crypto = {
      randomUUID: () => uuids[callCount++],
    } as any;

    await sendAIResultTelemetry(
      'op1',
      null,
      'gpt-4',
      'openai',
      { inputTokens: 10, outputTokens: 5, totalTokens: 15 },
      0.01,
      100
    );

    await sendAIResultTelemetry(
      'op2',
      null,
      'gpt-4',
      'openai',
      { inputTokens: 20, outputTokens: 10, totalTokens: 30 },
      0.02,
      200
    );

    expect(fetchMock).toHaveBeenCalledTimes(2);

    const body1 = JSON.parse(fetchMock.mock.calls[0][1].body);
    const body2 = JSON.parse(fetchMock.mock.calls[1][1].body);

    expect(body1.request_id).toBe('uuid-1');
    expect(body2.request_id).toBe('uuid-2');
  });
});
