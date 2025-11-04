// ABOUTME: Integration tests for research analysis AI service
// ABOUTME: Validates competitor analysis, gap analysis, UI pattern extraction, lesson extraction without making real AI calls

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { Competitor, SimilarProject } from './ideate';

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

vi.mock('ai', () => ({
  generateObject: (...args: any[]) => mockGenerateObject(...args),
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
  analyzeCompetitor,
  analyzeGaps,
  extractUIPatterns,
  extractLessons,
  synthesizeResearch,
} from './research-ai';

describe('Research Analysis AI Service', () => {
  const mockCompetitor: Competitor = {
    id: 'comp-1',
    session_id: 'session-1',
    name: 'Competitor A',
    url: 'https://competitor-a.com',
    strengths: ['Strong brand', 'Large user base', 'Advanced features'],
    gaps: ['Poor mobile experience', 'Slow loading times', 'Limited integrations'],
    features: ['Feature tracking', 'Team collaboration', 'Analytics dashboard'],
    analysis_summary: 'Leading player with some weaknesses',
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };

  const mockSimilarProject: SimilarProject = {
    id: 'proj-1',
    session_id: 'session-1',
    name: 'Similar Project',
    url: 'https://github.com/example/similar-project',
    positive_aspects: ['Clean code', 'Good documentation', 'Active community'],
    negative_aspects: ['Limited features', 'Performance issues', 'Outdated dependencies'],
    patterns_to_adopt: ['API design pattern', 'Component structure', 'Testing approach'],
    created_at: new Date().toISOString(),
    updated_at: new Date().toISOString(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('analyzeCompetitor', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          name: 'Competitor A',
          url: 'https://competitor-a.com',
          strengths: ['Strong brand', 'Large user base', 'Advanced features'],
          gaps: ['Poor mobile experience', 'Slow loading times', 'Limited integrations'],
          features: ['Feature tracking', 'Team collaboration', 'Analytics dashboard'],
        },
        usage: {
          promptTokens: 300,
          completionTokens: 200,
          totalTokens: 500,
        },
      });
    });

    it('should analyze a competitor from URL and content', async () => {
      const result = await analyzeCompetitor(
        'Build a project management tool',
        'https://competitor-a.com',
        'Website content about the product...'
      );

      expect(result).toHaveProperty('name', 'Competitor A');
      expect(result).toHaveProperty('url');
      expect(result).toHaveProperty('strengths');
      expect(result).toHaveProperty('gaps');
      expect(result).toHaveProperty('features');
      expect(Array.isArray(result.strengths)).toBe(true);
      expect(Array.isArray(result.gaps)).toBe(true);
      expect(Array.isArray(result.features)).toBe(true);
    });

    it('should include project description and URL in prompt', async () => {
      await analyzeCompetitor('Build a tool', 'https://example.com', 'Content here');

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('PROJECT DESCRIPTION:');
      expect(call.prompt).toContain('Build a tool');
      expect(call.prompt).toContain('COMPETITOR URL: https://example.com');
      expect(call.prompt).toContain('WEBSITE CONTENT:');
    });

    it('should truncate long content in prompt', async () => {
      const longContent = 'a'.repeat(10000);
      await analyzeCompetitor('Description', 'https://example.com', longContent);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('...(truncated)');
    });

    it('should use correct schema and temperature', async () => {
      await analyzeCompetitor('Description', 'https://example.com', 'Content');

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockAnthropicModel);
      expect(call.schema).toBeDefined();
      expect(call.temperature).toBe(0.3);
      expect(call.maxTokens).toBe(2000);
    });

    it('should track telemetry for competitor analysis', async () => {
      await analyzeCompetitor('Description', 'https://example.com', 'Content', undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'analyze_competitor',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should handle AI service errors gracefully', async () => {
      mockGenerateObject.mockRejectedValueOnce(new Error('Analysis failed'));

      await expect(analyzeCompetitor('Description', 'https://example.com', 'Content')).rejects.toThrow(
        'Analysis failed'
      );
    });
  });

  describe('analyzeGaps', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          opportunities: [
            {
              opportunity_type: 'differentiation',
              title: 'Real-time collaboration',
              description: 'Competitors lack real-time features',
              competitor_context: 'None of the competitors offer live editing',
              recommendation: 'Build WebSocket-based collaboration',
            },
            {
              opportunity_type: 'improvement',
              title: 'Mobile experience',
              description: 'Competitors have poor mobile apps',
              competitor_context: 'All competitors have low mobile ratings',
              recommendation: 'Focus on mobile-first design',
            },
            {
              opportunity_type: 'gap',
              title: 'Integration ecosystem',
              description: 'Missing integrations with key tools',
              competitor_context: 'Competitor A has 50+ integrations',
              recommendation: 'Build integration platform',
            },
          ],
          summary: 'Strong opportunities for differentiation in real-time features and mobile experience',
        },
        usage: {
          promptTokens: 400,
          completionTokens: 300,
          totalTokens: 700,
        },
      });
    });

    it('should perform gap analysis across competitors', async () => {
      const competitors = [mockCompetitor];
      const features = ['User authentication', 'Project management', 'Real-time updates'];

      const result = await analyzeGaps('Build a tool', features, competitors);

      expect(result).toHaveProperty('opportunities');
      expect(result).toHaveProperty('summary');
      expect(Array.isArray(result.opportunities)).toBe(true);
      expect(result.opportunities.length).toBe(3);
      expect(result.opportunities[0].opportunity_type).toBe('differentiation');
    });

    it('should return empty result when no competitors provided', async () => {
      const result = await analyzeGaps('Build a tool', ['Feature 1'], []);

      expect(result.opportunities).toEqual([]);
      expect(result.summary).toContain('No competitors analyzed yet');
      expect(mockGenerateObject).not.toHaveBeenCalled();
    });

    it('should include project description and features in prompt', async () => {
      const features = ['Feature A', 'Feature B'];
      await analyzeGaps('Build a tool', features, [mockCompetitor]);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('YOUR PROJECT:');
      expect(call.prompt).toContain('Build a tool');
      expect(call.prompt).toContain('YOUR PLANNED FEATURES:');
      expect(call.prompt).toContain('1. Feature A');
      expect(call.prompt).toContain('2. Feature B');
    });

    it('should include competitor analysis in prompt', async () => {
      await analyzeGaps('Description', [], [mockCompetitor]);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('COMPETITOR ANALYSIS:');
      expect(call.prompt).toContain('**Competitor A**:');
      expect(call.prompt).toContain('Features: Feature tracking, Team collaboration, Analytics dashboard');
    });

    it('should track telemetry for gap analysis', async () => {
      await analyzeGaps('Description', ['Feature 1'], [mockCompetitor], undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'analyze_gaps',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should identify all three opportunity types', async () => {
      const result = await analyzeGaps('Description', ['Feature 1'], [mockCompetitor]);

      const types = result.opportunities.map((o) => o.opportunity_type);
      expect(types).toContain('differentiation');
      expect(types).toContain('improvement');
      expect(types).toContain('gap');
    });
  });

  describe('extractUIPatterns', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          patterns: [
            {
              pattern_type: 'layout',
              name: 'Responsive Grid',
              description: '12-column grid with breakpoints',
              benefits: 'Clean, organized layout that works on all devices',
              adoption_notes: 'Use CSS Grid with mobile-first approach',
            },
            {
              pattern_type: 'navigation',
              name: 'Sidebar Navigation',
              description: 'Collapsible sidebar with nested items',
              benefits: 'Easy access to all sections',
              adoption_notes: 'Implement with React context for state',
            },
            {
              pattern_type: 'interaction',
              name: 'Drag and Drop',
              description: 'Intuitive drag-drop interface',
              benefits: 'Natural user interaction',
              adoption_notes: 'Use react-dnd library',
            },
          ],
        },
        usage: {
          promptTokens: 350,
          completionTokens: 250,
          totalTokens: 600,
        },
      });
    });

    it('should extract UI patterns from website content', async () => {
      const result = await extractUIPatterns('Build a tool', 'https://example.com', 'Website content...');

      expect(Array.isArray(result)).toBe(true);
      expect(result.length).toBe(3);
      expect(result[0]).toHaveProperty('pattern_type', 'layout');
      expect(result[0]).toHaveProperty('name');
      expect(result[0]).toHaveProperty('description');
      expect(result[0]).toHaveProperty('benefits');
      expect(result[0]).toHaveProperty('adoption_notes');
    });

    it('should identify patterns in all categories', async () => {
      const result = await extractUIPatterns('Description', 'https://example.com', 'Content');

      const types = result.map((p) => p.pattern_type);
      expect(types).toContain('layout');
      expect(types).toContain('navigation');
      expect(types).toContain('interaction');
    });

    it('should include project description and URL in prompt', async () => {
      await extractUIPatterns('Build a tool', 'https://example.com', 'Content');

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('PROJECT DESCRIPTION:');
      expect(call.prompt).toContain('Build a tool');
      expect(call.prompt).toContain('WEBSITE URL: https://example.com');
      expect(call.prompt).toContain('WEBSITE CONTENT:');
    });

    it('should truncate long content', async () => {
      const longContent = 'b'.repeat(10000);
      await extractUIPatterns('Description', 'https://example.com', longContent);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('...(truncated)');
    });

    it('should track telemetry for pattern extraction', async () => {
      await extractUIPatterns('Description', 'https://example.com', 'Content', undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'extract_ui_patterns',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should use correct temperature for pattern extraction', async () => {
      await extractUIPatterns('Description', 'https://example.com', 'Content');

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.temperature).toBe(0.3);
      expect(call.maxTokens).toBe(3000);
    });
  });

  describe('extractLessons', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          lessons: [
            {
              category: 'design',
              insight: 'Clean code architecture leads to better maintainability',
              application: 'Use SOLID principles and design patterns',
              priority: 'high',
            },
            {
              category: 'implementation',
              insight: 'Good documentation saves time',
              application: 'Write comprehensive API docs from the start',
              priority: 'high',
            },
            {
              category: 'feature',
              insight: 'Start with MVP features',
              application: 'Focus on core features first, iterate later',
              priority: 'medium',
            },
          ],
        },
        usage: {
          promptTokens: 300,
          completionTokens: 200,
          totalTokens: 500,
        },
      });
    });

    it('should extract lessons from similar project', async () => {
      const result = await extractLessons('Build a tool', mockSimilarProject);

      expect(Array.isArray(result)).toBe(true);
      expect(result.length).toBe(3);
      expect(result[0]).toHaveProperty('category');
      expect(result[0]).toHaveProperty('insight');
      expect(result[0]).toHaveProperty('application');
      expect(result[0]).toHaveProperty('priority');
    });

    it('should include all lesson categories', async () => {
      const result = await extractLessons('Description', mockSimilarProject);

      const categories = result.map((l) => l.category);
      expect(categories).toContain('design');
      expect(categories).toContain('implementation');
      expect(categories).toContain('feature');
    });

    it('should include project details in prompt', async () => {
      await extractLessons('Build a tool', mockSimilarProject);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('OUR PROJECT:');
      expect(call.prompt).toContain('Build a tool');
      expect(call.prompt).toContain('SIMILAR PROJECT: Similar Project');
      expect(call.prompt).toContain('URL: https://github.com/example/similar-project');
    });

    it('should include positive and negative aspects in prompt', async () => {
      await extractLessons('Description', mockSimilarProject);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('POSITIVE ASPECTS:');
      expect(call.prompt).toContain('1. Clean code');
      expect(call.prompt).toContain('NEGATIVE ASPECTS:');
      expect(call.prompt).toContain('1. Limited features');
      expect(call.prompt).toContain('PATTERNS TO ADOPT:');
      expect(call.prompt).toContain('1. API design pattern');
    });

    it('should track telemetry for lesson extraction', async () => {
      await extractLessons('Description', mockSimilarProject, undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'extract_lessons',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should assign priority levels to lessons', async () => {
      const result = await extractLessons('Description', mockSimilarProject);

      const priorities = result.map((l) => l.priority);
      expect(priorities).toContain('high');
      expect(priorities).toContain('medium');
    });

    it('should use correct temperature for lesson extraction', async () => {
      await extractLessons('Description', mockSimilarProject);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.temperature).toBe(0.3);
      expect(call.maxTokens).toBe(2500);
    });
  });

  describe('synthesizeResearch', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: {
          key_findings: [
            'Market is highly competitive',
            'Users demand mobile-first experience',
            'Real-time features are becoming standard',
            'Integration ecosystem is critical',
            'Performance is a key differentiator',
          ],
          market_position: 'Position as a modern, mobile-first alternative with superior real-time capabilities',
          differentiators: [
            'Real-time collaboration',
            'Superior mobile experience',
            'Open API architecture',
            'Performance optimization',
          ],
          risks: ['Established competitors have strong brand', 'Feature parity will take time', 'Market saturation'],
          recommendations: [
            'Focus on mobile experience from day one',
            'Build real-time features as core, not add-on',
            'Create open integration platform',
            'Invest in performance optimization',
            'Start with narrow use case for differentiation',
          ],
        },
        usage: {
          promptTokens: 500,
          completionTokens: 400,
          totalTokens: 900,
        },
      });
    });

    it('should synthesize research findings', async () => {
      const competitors = [mockCompetitor];
      const similarProjects = [mockSimilarProject];

      const result = await synthesizeResearch('Build a tool', competitors, similarProjects);

      expect(result).toHaveProperty('key_findings');
      expect(result).toHaveProperty('market_position');
      expect(result).toHaveProperty('differentiators');
      expect(result).toHaveProperty('risks');
      expect(result).toHaveProperty('recommendations');
      expect(Array.isArray(result.key_findings)).toBe(true);
      expect(Array.isArray(result.differentiators)).toBe(true);
      expect(Array.isArray(result.risks)).toBe(true);
      expect(Array.isArray(result.recommendations)).toBe(true);
    });

    it('should include project description in prompt', async () => {
      await synthesizeResearch('Build a tool', [mockCompetitor], [mockSimilarProject]);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('PROJECT DESCRIPTION:');
      expect(call.prompt).toContain('Build a tool');
    });

    it('should include competitor summary in prompt', async () => {
      await synthesizeResearch('Description', [mockCompetitor], [mockSimilarProject]);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('COMPETITOR ANALYSIS (1 analyzed):');
      expect(call.prompt).toContain('**Competitor A**:');
      expect(call.prompt).toContain('Strengths: Strong brand, Large user base, Advanced features');
    });

    it('should include similar projects count in prompt', async () => {
      await synthesizeResearch('Description', [mockCompetitor], [mockSimilarProject]);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('SIMILAR PROJECTS REVIEWED: 1');
    });

    it('should work with multiple competitors', async () => {
      const competitor2 = { ...mockCompetitor, id: 'comp-2', name: 'Competitor B' };
      await synthesizeResearch('Description', [mockCompetitor, competitor2], []);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('COMPETITOR ANALYSIS (2 analyzed):');
      expect(call.prompt).toContain('**Competitor A**:');
      expect(call.prompt).toContain('**Competitor B**:');
    });

    it('should track telemetry for research synthesis', async () => {
      await synthesizeResearch('Description', [mockCompetitor], [mockSimilarProject], undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'synthesize_research',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should provide strategic insights', async () => {
      const result = await synthesizeResearch('Description', [mockCompetitor], [mockSimilarProject]);

      expect(result.key_findings.length).toBeGreaterThan(0);
      expect(result.market_position).toBeTruthy();
      expect(result.differentiators.length).toBeGreaterThan(0);
      expect(result.risks.length).toBeGreaterThan(0);
      expect(result.recommendations.length).toBeGreaterThan(0);
    });

    it('should use correct temperature for synthesis', async () => {
      await synthesizeResearch('Description', [mockCompetitor], [mockSimilarProject]);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.temperature).toBe(0.4);
      expect(call.maxTokens).toBe(3000);
    });
  });

  describe('Integration', () => {
    it('should work end-to-end: analyze -> gaps -> patterns -> lessons -> synthesize', async () => {
      // Analyze competitor
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          name: 'Competitor A',
          url: 'https://competitor-a.com',
          strengths: ['Strong brand'],
          gaps: ['Poor mobile'],
          features: ['Feature tracking'],
        },
        usage: { promptTokens: 300, completionTokens: 200, totalTokens: 500 },
      });

      const competitor = await analyzeCompetitor('Build a tool', 'https://competitor-a.com', 'Content');

      // Analyze gaps
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          opportunities: [
            {
              opportunity_type: 'differentiation',
              title: 'Mobile focus',
              description: 'Better mobile',
              competitor_context: 'Competitors lack mobile',
              recommendation: 'Build mobile-first',
            },
          ],
          summary: 'Mobile opportunity',
        },
        usage: { promptTokens: 400, completionTokens: 300, totalTokens: 700 },
      });

      const gaps = await analyzeGaps('Build a tool', ['Feature 1'], [competitor]);

      // Extract UI patterns
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          patterns: [
            {
              pattern_type: 'layout',
              name: 'Grid',
              description: 'Responsive grid',
              benefits: 'Clean layout',
              adoption_notes: 'Use CSS Grid',
            },
          ],
        },
        usage: { promptTokens: 350, completionTokens: 250, totalTokens: 600 },
      });

      const patterns = await extractUIPatterns('Build a tool', 'https://example.com', 'Content');

      // Extract lessons
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          lessons: [
            {
              category: 'design',
              insight: 'Clean code',
              application: 'Use SOLID',
              priority: 'high',
            },
          ],
        },
        usage: { promptTokens: 300, completionTokens: 200, totalTokens: 500 },
      });

      const lessons = await extractLessons('Build a tool', mockSimilarProject);

      // Synthesize research
      mockGenerateObject.mockResolvedValueOnce({
        object: {
          key_findings: ['Finding 1'],
          market_position: 'Position',
          differentiators: ['Diff 1'],
          risks: ['Risk 1'],
          recommendations: ['Recommendation 1'],
        },
        usage: { promptTokens: 500, completionTokens: 400, totalTokens: 900 },
      });

      const synthesis = await synthesizeResearch('Build a tool', [competitor], [mockSimilarProject]);

      expect(competitor.name).toBe('Competitor A');
      expect(gaps.opportunities).toHaveLength(1);
      expect(patterns).toHaveLength(1);
      expect(lessons).toHaveLength(1);
      expect(synthesis.key_findings).toHaveLength(1);
      expect(mockGenerateObject).toHaveBeenCalledTimes(5);
    });
  });
});
