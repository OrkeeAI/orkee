// ABOUTME: Integration tests for roundtable AI service
// ABOUTME: Validates expert suggestions, response generation, insight extraction without making real AI calls

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { ExpertPersona, RoundtableMessage, SuggestExpertsRequest } from './ideate';

// Mock localStorage for rate limiter
global.localStorage = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
  length: 0,
  key: vi.fn(),
};

// Mock the AI SDK before importing services
const mockGenerateObject = vi.fn();
const mockStreamText = vi.fn();

vi.mock('ai', () => ({
  generateObject: (...args: any[]) => mockGenerateObject(...args),
  streamText: (...args: any[]) => mockStreamText(...args),
}));

// Mock the providers
const mockAnthropicModel = { id: 'anthropic-mock', provider: 'anthropic' };

vi.mock('@/lib/ai/config', () => ({
  getModelInstance: vi.fn(() => mockAnthropicModel),
  calculateCost: vi.fn(() => ({
    inputCost: 1.5,
    outputCost: 7.5,
    totalCost: 9.0,
  })),
}));

vi.mock('./model-preferences', () => ({
  getModelForTask: vi.fn(() => ({
    provider: 'anthropic',
    model: 'claude-sonnet-4-5-20250929',
  })),
}));

// Mock telemetry tracking
const mockTrackAIOperationWithCost = vi.fn((operationName, projectId, model, provider, costFn, operation) => {
  return operation();
});

vi.mock('@/lib/ai/telemetry', () => ({
  trackAIOperationWithCost: (...args: any[]) => mockTrackAIOperationWithCost(...args),
}));

// Import the service after mocks are set up
import {
  suggestExperts,
  generateExpertResponse,
  streamExpertResponse,
  extractInsights,
  formatConversationHistory,
  buildModeratorOpening,
  buildExpertSuggestionPrompt,
  buildInsightExtractionPrompt,
  selectNextExpert,
  shouldEndDiscussion,
} from './roundtable-ai';

describe('Roundtable AI Service', () => {
  const mockExpert: ExpertPersona = {
    id: 'expert-1',
    discussion_id: 'discussion-1',
    name: 'Dr. Sarah Chen',
    role: 'Senior Software Architect',
    expertise_area: 'Distributed Systems',
    system_prompt: 'You are Dr. Sarah Chen, an expert in distributed systems...',
    avatar_url: null,
    relevance_score: 0.95,
    created_at: new Date().toISOString(),
  };

  const mockExpert2: ExpertPersona = {
    id: 'expert-2',
    discussion_id: 'discussion-1',
    name: 'Alex Morgan',
    role: 'UX Designer',
    expertise_area: 'User Experience',
    system_prompt: 'You are Alex Morgan, a UX design expert...',
    avatar_url: null,
    relevance_score: 0.90,
    created_at: new Date().toISOString(),
  };

  const mockMessages: RoundtableMessage[] = [
    {
      id: 'msg-1',
      discussion_id: 'discussion-1',
      role: 'moderator',
      content: 'Welcome everyone to our discussion about project architecture.',
      expert_id: null,
      expert_name: null,
      created_at: new Date().toISOString(),
    },
    {
      id: 'msg-2',
      discussion_id: 'discussion-1',
      role: 'expert',
      content: 'I think we should focus on microservices architecture.',
      expert_id: 'expert-1',
      expert_name: 'Dr. Sarah Chen',
      created_at: new Date().toISOString(),
    },
    {
      id: 'msg-3',
      discussion_id: 'discussion-1',
      role: 'user',
      content: 'What about scalability concerns?',
      expert_id: null,
      expert_name: null,
      created_at: new Date().toISOString(),
    },
    {
      id: 'msg-4',
      discussion_id: 'discussion-1',
      role: 'expert',
      content: 'From a UX perspective, we need to ensure consistency across services.',
      expert_id: 'expert-2',
      expert_name: 'Alex Morgan',
      created_at: new Date().toISOString(),
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('suggestExperts', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          suggestions: [
            {
              name: 'Dr. Sarah Chen',
              role: 'Senior Software Architect',
              expertise_area: 'Distributed Systems',
              reason: 'Expert in scalable architectures',
              relevance_score: 0.95,
            },
            {
              name: 'Alex Morgan',
              role: 'UX Designer',
              expertise_area: 'User Experience',
              reason: 'Provides user-centric perspective',
              relevance_score: 0.90,
            },
            {
              name: 'Jordan Lee',
              role: 'DevOps Engineer',
              expertise_area: 'Infrastructure',
              reason: 'Deployment and operations expertise',
              relevance_score: 0.85,
            },
          ],
        },
        usage: {
          promptTokens: 300,
          completionTokens: 250,
          totalTokens: 550,
        },
      });
    });

    it('should suggest experts based on project description', async () => {
      const request: SuggestExpertsRequest = {
        projectDescription: 'Build a scalable microservices platform',
        numExperts: 3,
      };

      const result = await suggestExperts(request);

      expect(Array.isArray(result)).toBe(true);
      expect(result.length).toBe(3);
      expect(result[0]).toHaveProperty('name', 'Dr. Sarah Chen');
      expect(result[0]).toHaveProperty('role');
      expect(result[0]).toHaveProperty('expertise_area');
      expect(result[0]).toHaveProperty('reason');
      expect(result[0]).toHaveProperty('relevance_score');
      expect(result[0].relevance_score).toBeGreaterThanOrEqual(0);
      expect(result[0].relevance_score).toBeLessThanOrEqual(1);
    });

    it('should include project description in prompt', async () => {
      const request: SuggestExpertsRequest = {
        projectDescription: 'Build a tool',
        numExperts: 3,
      };

      await suggestExperts(request);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('Project Description:');
      expect(call.prompt).toContain('Build a tool');
      expect(call.prompt).toContain('Suggest 3 expert personas');
    });

    it('should include existing content when provided', async () => {
      const request: SuggestExpertsRequest = {
        projectDescription: 'Build a tool',
        existingContent: 'We already have authentication and user management',
        numExperts: 3,
      };

      await suggestExperts(request);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('Existing Content:');
      expect(call.prompt).toContain('We already have authentication');
    });

    it('should use expert suggestion system prompt', async () => {
      const request: SuggestExpertsRequest = {
        projectDescription: 'Description',
        numExperts: 3,
      };

      await suggestExperts(request);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.system).toContain('assembling high-quality advisory panels');
    });

    it('should track telemetry for expert suggestions', async () => {
      const request: SuggestExpertsRequest = {
        projectDescription: 'Description',
        numExperts: 3,
      };

      await suggestExperts(request, undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'suggest_experts',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should handle AI service errors gracefully', async () => {
      mockGenerateObject.mockRejectedValueOnce(new Error('Suggestion failed'));

      const request: SuggestExpertsRequest = {
        projectDescription: 'Description',
        numExperts: 3,
      };

      await expect(suggestExperts(request)).rejects.toThrow('Suggestion failed');
    });
  });

  describe('generateExpertResponse', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          response:
            'I recommend using a microservices architecture with proper service boundaries. This will allow for better scalability and independent deployment of services.',
        },
        usage: {
          promptTokens: 250,
          completionTokens: 100,
          totalTokens: 350,
        },
      });
    });

    it('should generate expert response for discussion', async () => {
      const result = await generateExpertResponse(
        mockExpert,
        'Project architecture decisions',
        mockMessages,
        [mockExpert, mockExpert2]
      );

      expect(typeof result).toBe('string');
      expect(result).toContain('microservices');
      expect(result.length).toBeGreaterThan(0);
    });

    it('should include topic and conversation context in prompt', async () => {
      await generateExpertResponse(mockExpert, 'Architecture', mockMessages, [mockExpert]);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('Topic: Architecture');
      expect(call.prompt).toContain('Previous discussion:');
      expect(call.prompt).toContain('Dr. Sarah Chen: I think we should focus on microservices');
    });

    it('should use expert-specific system prompt', async () => {
      await generateExpertResponse(mockExpert, 'Topic', mockMessages, [mockExpert]);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.system).toContain(mockExpert.system_prompt);
      expect(call.system).toContain('You are participating in an expert roundtable');
    });

    it('should use correct temperature for natural responses', async () => {
      await generateExpertResponse(mockExpert, 'Topic', mockMessages, [mockExpert]);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.temperature).toBe(0.7);
      expect(call.maxTokens).toBe(500);
    });

    it('should track telemetry for expert response', async () => {
      await generateExpertResponse(mockExpert, 'Topic', mockMessages, [mockExpert], undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'generate_expert_response',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should generate response with expert name in prompt', async () => {
      await generateExpertResponse(mockExpert, 'Topic', mockMessages, [mockExpert]);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('As Dr. Sarah Chen');
    });
  });

  describe('streamExpertResponse', () => {
    beforeEach(() => {
      const mockStream = {
        textStream: (async function* () {
          yield 'I recommend ';
          yield 'using microservices ';
          yield 'architecture.';
        })(),
      };
      mockStreamText.mockResolvedValue(mockStream);
    });

    it('should stream expert response with chunks', async () => {
      const chunks: string[] = [];
      let fullText = '';

      await streamExpertResponse(
        mockExpert,
        'Architecture',
        mockMessages,
        [mockExpert],
        (chunk) => chunks.push(chunk),
        (full) => {
          fullText = full;
        },
        (error) => {
          throw error;
        }
      );

      expect(chunks).toEqual(['I recommend ', 'using microservices ', 'architecture.']);
      expect(fullText).toBe('I recommend using microservices architecture.');
    });

    it('should call onComplete with full text', async () => {
      const onComplete = vi.fn();

      await streamExpertResponse(
        mockExpert,
        'Topic',
        mockMessages,
        [mockExpert],
        () => {},
        onComplete,
        () => {}
      );

      expect(onComplete).toHaveBeenCalledWith('I recommend using microservices architecture.');
    });

    it('should handle streaming errors', async () => {
      mockStreamText.mockRejectedValueOnce(new Error('Stream error'));

      const onError = vi.fn();

      await streamExpertResponse(
        mockExpert,
        'Topic',
        mockMessages,
        [mockExpert],
        () => {},
        () => {},
        onError
      );

      expect(onError).toHaveBeenCalledWith(expect.objectContaining({ message: 'Stream error' }));
    });

    it('should respect abort signal', async () => {
      const abortController = new AbortController();
      const chunks: string[] = [];

      // Create a stream that yields chunks and checks abort status
      const mockStream = {
        textStream: (async function* () {
          yield 'Chunk 1 ';
          // Abort is checked AFTER yield, so we need to abort before second yield
          if (abortController.signal.aborted) {
            return;
          }
          yield 'Chunk 2 ';
        })(),
      };
      mockStreamText.mockResolvedValue(mockStream);

      // Start the stream
      const promise = streamExpertResponse(
        mockExpert,
        'Topic',
        mockMessages,
        [mockExpert],
        (chunk) => chunks.push(chunk),
        () => {},
        () => {},
        abortController.signal
      );

      // Abort immediately after starting
      abortController.abort();

      await promise;

      // May receive 1 or 2 chunks depending on timing, but should not get all 3
      expect(chunks.length).toBeLessThanOrEqual(2);
    });

    it('should track telemetry for stream response', async () => {
      await streamExpertResponse(
        mockExpert,
        'Topic',
        mockMessages,
        [mockExpert],
        () => {},
        () => {},
        () => {},
        undefined,
        undefined,
        'project-1'
      );

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'stream_expert_response',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });
  });

  describe('extractInsights', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          insights: [
            {
              insight_text: 'Microservices architecture provides better scalability',
              category: 'Technical',
              priority: 'high',
              source_experts: ['Dr. Sarah Chen'],
            },
            {
              insight_text: 'Consistency across services is critical for UX',
              category: 'UX',
              priority: 'high',
              source_experts: ['Alex Morgan'],
            },
            {
              insight_text: 'Consider service mesh for inter-service communication',
              category: 'Technical',
              priority: 'medium',
              source_experts: ['Dr. Sarah Chen'],
            },
          ],
          summary:
            'The discussion highlighted the importance of microservices architecture with strong emphasis on scalability and user experience consistency.',
        },
        usage: {
          promptTokens: 400,
          completionTokens: 300,
          totalTokens: 700,
        },
      });
    });

    it('should extract insights from discussion', async () => {
      const result = await extractInsights(mockMessages);

      expect(result).toHaveProperty('insights');
      expect(result).toHaveProperty('summary');
      expect(Array.isArray(result.insights)).toBe(true);
      expect(result.insights.length).toBe(3);
      expect(result.insights[0]).toHaveProperty('insight_text');
      expect(result.insights[0]).toHaveProperty('category');
      expect(result.insights[0]).toHaveProperty('priority');
      expect(result.insights[0]).toHaveProperty('source_experts');
    });

    it('should return empty result when no messages provided', async () => {
      const result = await extractInsights([]);

      expect(result.insights).toEqual([]);
      expect(result.summary).toContain('No messages to analyze');
      expect(mockGenerateObject).not.toHaveBeenCalled();
    });

    it('should include conversation history in prompt', async () => {
      await extractInsights(mockMessages);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('Discussion:');
      expect(call.prompt).toContain('Moderator: Welcome everyone');
      expect(call.prompt).toContain('Dr. Sarah Chen: I think we should focus on microservices');
    });

    it('should organize insights by categories when provided', async () => {
      const categories = ['Technical', 'UX', 'Business'];
      await extractInsights(mockMessages, categories);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('Organize insights into these categories: Technical, UX, Business');
    });

    it('should use insight extraction system prompt', async () => {
      await extractInsights(mockMessages);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.system).toContain('discussion analyst');
      expect(call.system).toContain('extracting actionable insights');
    });

    it('should assign priority levels to insights', async () => {
      const result = await extractInsights(mockMessages);

      const priorities = result.insights.map((i) => i.priority);
      expect(priorities).toContain('high');
      expect(priorities).toContain('medium');
    });

    it('should attribute insights to source experts', async () => {
      const result = await extractInsights(mockMessages);

      expect(result.insights[0].source_experts).toContain('Dr. Sarah Chen');
      expect(result.insights[1].source_experts).toContain('Alex Morgan');
    });

    it('should track telemetry for insight extraction', async () => {
      await extractInsights(mockMessages, undefined, undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'extract_insights',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should use correct temperature for analysis', async () => {
      await extractInsights(mockMessages);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.temperature).toBe(0.3);
      expect(call.maxTokens).toBe(3000);
    });
  });

  describe('Utility Functions', () => {
    describe('formatConversationHistory', () => {
      it('should format messages into conversation string', () => {
        const result = formatConversationHistory(mockMessages);

        expect(result).toContain('Moderator: Welcome everyone');
        expect(result).toContain('Dr. Sarah Chen: I think we should focus on microservices');
        expect(result).toContain('User: What about scalability concerns?');
        expect(result).toContain('Alex Morgan: From a UX perspective');
      });

      it('should skip system messages', () => {
        const messagesWithSystem: RoundtableMessage[] = [
          ...mockMessages,
          {
            id: 'msg-5',
            discussion_id: 'discussion-1',
            role: 'system',
            content: 'System notification',
            expert_id: null,
            expert_name: null,
            created_at: new Date().toISOString(),
          },
        ];

        const result = formatConversationHistory(messagesWithSystem);

        expect(result).not.toContain('System notification');
      });

      it('should handle empty message list', () => {
        const result = formatConversationHistory([]);

        expect(result).toBe('');
      });
    });

    describe('buildModeratorOpening', () => {
      it('should build opening statement with topic and experts', () => {
        const result = buildModeratorOpening('Project architecture decisions', [mockExpert, mockExpert2]);

        expect(result).toContain('Project architecture decisions');
        expect(result).toContain('Dr. Sarah Chen (Senior Software Architect)');
        expect(result).toContain('Alex Morgan (UX Designer)');
        expect(result).toContain('Welcome everyone!');
      });

      it('should list all expert names and roles', () => {
        const result = buildModeratorOpening('Topic', [mockExpert, mockExpert2]);

        expect(result).toContain('Dr. Sarah Chen');
        expect(result).toContain('Alex Morgan');
      });
    });

    describe('buildExpertSuggestionPrompt', () => {
      it('should build prompt with project description', () => {
        const request: SuggestExpertsRequest = {
          projectDescription: 'Build a tool',
          numExperts: 3,
        };

        const result = buildExpertSuggestionPrompt(request);

        expect(result).toContain('Project Description:');
        expect(result).toContain('Build a tool');
        expect(result).toContain('Suggest 3 expert personas');
      });

      it('should include existing content when provided', () => {
        const request: SuggestExpertsRequest = {
          projectDescription: 'Description',
          existingContent: 'Existing work',
          numExperts: 5,
        };

        const result = buildExpertSuggestionPrompt(request);

        expect(result).toContain('Existing Content:');
        expect(result).toContain('Existing work');
      });

      it('should default to 3 experts when numExperts not provided', () => {
        const request: SuggestExpertsRequest = {
          projectDescription: 'Description',
        };

        const result = buildExpertSuggestionPrompt(request);

        expect(result).toContain('Suggest 3 expert personas');
      });
    });

    describe('buildInsightExtractionPrompt', () => {
      it('should build prompt with conversation history', () => {
        const result = buildInsightExtractionPrompt(mockMessages);

        expect(result).toContain('Discussion:');
        expect(result).toContain('Dr. Sarah Chen: I think we should focus on microservices');
        expect(result).toContain('Extract key insights');
      });

      it('should include categories when provided', () => {
        const categories = ['Technical', 'UX', 'Business'];
        const result = buildInsightExtractionPrompt(mockMessages, categories);

        expect(result).toContain('Organize insights into these categories: Technical, UX, Business');
      });

      it('should not mention categories when not provided', () => {
        const result = buildInsightExtractionPrompt(mockMessages);

        expect(result).not.toContain('Organize insights into these categories');
      });
    });

    describe('selectNextExpert', () => {
      it('should select expert who has spoken the least', () => {
        const result = selectNextExpert([mockExpert, mockExpert2], mockMessages);

        // mockExpert spoke once, mockExpert2 spoke once, should return the first
        expect(result).toBe(mockExpert);
      });

      it('should return expert who has not spoken yet', () => {
        const messagesWithOneSpeaker: RoundtableMessage[] = [
          {
            id: 'msg-1',
            discussion_id: 'discussion-1',
            role: 'expert',
            content: 'First message',
            expert_id: 'expert-1',
            expert_name: 'Dr. Sarah Chen',
            created_at: new Date().toISOString(),
          },
          {
            id: 'msg-2',
            discussion_id: 'discussion-1',
            role: 'expert',
            content: 'Second message',
            expert_id: 'expert-1',
            expert_name: 'Dr. Sarah Chen',
            created_at: new Date().toISOString(),
          },
        ];

        const result = selectNextExpert([mockExpert, mockExpert2], messagesWithOneSpeaker);

        // mockExpert2 has not spoken, should be selected
        expect(result).toBe(mockExpert2);
      });

      it('should return null when no participants', () => {
        const result = selectNextExpert([], mockMessages);

        expect(result).toBeNull();
      });

      it('should handle empty message list', () => {
        const result = selectNextExpert([mockExpert, mockExpert2], []);

        // Both have 0 speaks, should return first expert
        expect(result).toBe(mockExpert);
      });
    });

    describe('shouldEndDiscussion', () => {
      it('should return true when enough expert messages', () => {
        const manyExpertMessages: RoundtableMessage[] = [];
        for (let i = 0; i < 12; i++) {
          manyExpertMessages.push({
            id: `msg-${i}`,
            discussion_id: 'discussion-1',
            role: 'expert',
            content: `Message ${i}`,
            expert_id: 'expert-1',
            expert_name: 'Expert',
            created_at: new Date().toISOString(),
          });
        }

        const result = shouldEndDiscussion(manyExpertMessages, 10);

        expect(result).toBe(true);
      });

      it('should return false when not enough expert messages', () => {
        const result = shouldEndDiscussion(mockMessages, 10);

        // Only 2 expert messages in mockMessages
        expect(result).toBe(false);
      });

      it('should use custom minMessages threshold', () => {
        const result = shouldEndDiscussion(mockMessages, 2);

        // 2 expert messages, threshold is 2, should be true
        expect(result).toBe(true);
      });

      it('should only count expert messages', () => {
        const mixedMessages: RoundtableMessage[] = [
          ...mockMessages,
          {
            id: 'msg-5',
            discussion_id: 'discussion-1',
            role: 'user',
            content: 'User message',
            expert_id: null,
            expert_name: null,
            created_at: new Date().toISOString(),
          },
          {
            id: 'msg-6',
            discussion_id: 'discussion-1',
            role: 'moderator',
            content: 'Moderator message',
            expert_id: null,
            expert_name: null,
            created_at: new Date().toISOString(),
          },
        ];

        const result = shouldEndDiscussion(mixedMessages, 3);

        // Still only 2 expert messages
        expect(result).toBe(false);
      });
    });
  });

  describe('Integration', () => {
    it('should work end-to-end: suggest -> generate -> extract', async () => {
      // Suggest experts
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          suggestions: [
            {
              name: 'Dr. Sarah Chen',
              role: 'Senior Software Architect',
              expertise_area: 'Distributed Systems',
              reason: 'Expert in scalable architectures',
              relevance_score: 0.95,
            },
          ],
        },
        usage: { promptTokens: 300, completionTokens: 250, totalTokens: 550 },
      });

      const request: SuggestExpertsRequest = {
        projectDescription: 'Build a tool',
        numExperts: 1,
      };

      const experts = await suggestExperts(request);

      // Generate expert response
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          response: 'I recommend using microservices architecture.',
        },
        usage: { promptTokens: 250, completionTokens: 100, totalTokens: 350 },
      });

      const response = await generateExpertResponse(mockExpert, 'Architecture', mockMessages, [mockExpert]);

      // Extract insights
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          insights: [
            {
              insight_text: 'Microservices is the way to go',
              category: 'Technical',
              priority: 'high',
              source_experts: ['Dr. Sarah Chen'],
            },
          ],
          summary: 'Discussion focused on microservices architecture.',
        },
        usage: { promptTokens: 400, completionTokens: 300, totalTokens: 700 },
      });

      const insights = await extractInsights(mockMessages);

      expect(experts).toHaveLength(1);
      expect(experts[0].name).toBe('Dr. Sarah Chen');
      expect(response).toContain('microservices');
      expect(insights.insights).toHaveLength(1);
      expect(insights.insights[0].category).toBe('Technical');
      expect(mockGenerateObject).toHaveBeenCalledTimes(3);
    });
  });
});
