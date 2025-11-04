// ABOUTME: Integration tests for PRD generation AI service
// ABOUTME: Validates PRD generation, section generation, and template regeneration without making real AI calls

import { describe, it, expect, vi, beforeEach } from 'vitest';
import type { AggregatedPRDData } from './ideate';

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
  generateCompletePRD,
  generateSection,
  generateFromSession,
  fillSkippedSections,
  generateSectionWithContext,
  regenerateWithTemplate,
  regenerateWithTemplateStream,
  buildContextFromAggregated,
} from './prd-ai';

describe('PRD Generation AI Service', () => {
  const mockCompletePRD = {
    overview: {
      problem_statement: 'Users need better project management',
      target_audience: 'Development teams',
      value_proposition: 'Streamlined workflow',
      one_line_pitch: 'The best project management tool',
    },
    ux: {
      personas: [
        {
          name: 'Developer Dave',
          role: 'Senior Developer',
          goals: ['Ship features fast', 'Maintain code quality'],
          pain_points: ['Too many tools', 'Poor coordination'],
        },
      ],
      user_flows: [
        {
          name: 'Create Project',
          steps: [
            {
              action: 'Click new project',
              screen: 'Dashboard',
              notes: 'Main action button',
            },
          ],
          touchpoints: ['Dashboard', 'Project Form'],
        },
      ],
      ui_considerations: 'Clean, minimal interface',
      ux_principles: 'User-first design',
    },
    technical: {
      components: [
        {
          name: 'API Gateway',
          purpose: 'Route requests',
          technology: 'Node.js',
        },
      ],
      data_models: [
        {
          name: 'Project',
          fields: [
            { name: 'id', field_type: 'string', required: true },
            { name: 'name', field_type: 'string', required: true },
          ],
        },
      ],
      apis: [
        {
          name: 'Projects API',
          purpose: 'CRUD operations',
          endpoints: ['/api/projects', '/api/projects/:id'],
        },
      ],
      infrastructure: {
        hosting: 'AWS',
        database: 'PostgreSQL',
        caching: 'Redis',
        file_storage: 'S3',
      },
      tech_stack_quick: 'React, Node.js, PostgreSQL',
    },
    roadmap: {
      mvp_scope: ['User authentication', 'Project creation', 'Basic dashboard'],
      future_phases: [
        {
          name: 'Phase 2',
          features: ['Team collaboration', 'Real-time updates'],
          goals: ['Improve team coordination'],
        },
      ],
    },
    dependencies: {
      foundation_features: ['User authentication', 'Database setup'],
      visible_features: ['Project dashboard', 'Task management'],
      enhancement_features: ['Notifications', 'Mobile app'],
      dependency_graph: { 'feature-1': ['feature-2'] },
    },
    risks: {
      technical_risks: [
        {
          description: 'Scalability concerns',
          severity: 'high',
          probability: 'medium',
        },
      ],
      mvp_scoping_risks: [
        {
          description: 'Feature creep',
          severity: 'medium',
          probability: 'high',
        },
      ],
      resource_risks: [
        {
          description: 'Limited development time',
          severity: 'high',
          probability: 'medium',
        },
      ],
      mitigations: [
        {
          risk: 'Scalability concerns',
          strategy: 'Use cloud auto-scaling',
          owner: 'DevOps team',
        },
      ],
    },
    research: {
      competitors: [
        {
          name: 'Competitor A',
          url: 'https://example.com',
          strengths: ['Good UI', 'Fast performance'],
          gaps: ['Poor mobile support'],
          features: ['Kanban boards', 'Time tracking'],
        },
      ],
      similar_projects: [
        {
          name: 'Similar Project',
          url: 'https://github.com/example',
          positive_aspects: ['Clean code', 'Good docs'],
          negative_aspects: ['Limited features'],
          patterns_to_adopt: ['API design pattern'],
        },
      ],
      research_findings: 'Market analysis shows strong demand',
      technical_specs: 'REST API with GraphQL support',
      reference_links: ['https://docs.example.com'],
    },
  };

  const mockSessionData: AggregatedPRDData = {
    session: {
      id: 'session-1',
      initial_description: 'Build a project management tool',
      mode: 'quick',
      status: 'active',
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    },
    overview: mockCompletePRD.overview,
    ux: mockCompletePRD.ux,
    technical: mockCompletePRD.technical,
    roadmap: mockCompletePRD.roadmap,
    dependencies: mockCompletePRD.dependencies,
    risks: mockCompletePRD.risks,
    research: mockCompletePRD.research,
    roundtable_insights: [
      {
        id: 'insight-1',
        category: 'technical',
        content: 'Consider microservices architecture',
      },
    ],
    completeness: {
      completed_sections: 7,
      total_sections: 7,
      completion_percentage: 100,
    },
    skipped_sections: [],
  } as any;

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('generateCompletePRD', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: mockCompletePRD,
        usage: {
          promptTokens: 500,
          completionTokens: 1500,
          totalTokens: 2000,
        },
      });
    });

    it('should generate a complete PRD from description', async () => {
      const result = await generateCompletePRD('Build a project management tool');

      expect(result).toHaveProperty('overview');
      expect(result).toHaveProperty('ux');
      expect(result).toHaveProperty('technical');
      expect(result).toHaveProperty('roadmap');
      expect(result).toHaveProperty('dependencies');
      expect(result).toHaveProperty('risks');
      expect(result).toHaveProperty('research');
      expect(result.overview.problem_statement).toBe('Users need better project management');
    });

    it('should call generateObject with correct schema', async () => {
      await generateCompletePRD('Build a project management tool');

      expect(mockGenerateObject).toHaveBeenCalledTimes(1);
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.model).toBe(mockAnthropicModel);
      expect(call.schema).toBeDefined();
      expect(call.prompt).toContain('Build a project management tool');
      expect(call.temperature).toBe(0.5);
      expect(call.maxTokens).toBe(16000);
    });

    it('should use custom model preferences when provided', async () => {
      const customPrefs = { provider: 'openai' as const, model: 'gpt-4-turbo' };

      await generateCompletePRD('Build a tool', undefined, undefined, customPrefs);

      const { getModelInstance } = await import('@/lib/ai/config');
      expect(getModelInstance).toHaveBeenCalledWith('openai', 'gpt-4-turbo');
    });

    it('should track telemetry for PRD generation', async () => {
      await generateCompletePRD('Build a tool', undefined, undefined, undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'generate_complete_prd',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should handle AI service errors gracefully', async () => {
      mockGenerateObject.mockRejectedValueOnce(new Error('API error'));

      await expect(generateCompletePRD('Build a tool')).rejects.toThrow('API error');
    });
  });

  describe('generateSection', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: mockCompletePRD.overview,
        usage: {
          promptTokens: 200,
          completionTokens: 150,
          totalTokens: 350,
        },
      });
    });

    it('should generate a specific section', async () => {
      const result = await generateSection('overview', 'Build a project management tool');

      expect(result).toHaveProperty('problem_statement');
      expect(result).toHaveProperty('target_audience');
      expect(result).toHaveProperty('value_proposition');
      expect(result).toHaveProperty('one_line_pitch');
    });

    it('should include context in prompt when provided', async () => {
      await generateSection('overview', 'Build a tool', 'Additional context about users');

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('Additional context about users');
    });

    it('should throw error for unknown section', async () => {
      await expect(generateSection('unknown_section', 'Description')).rejects.toThrow('Unknown section');
    });

    it('should generate each section type correctly', async () => {
      const sections = ['overview', 'ux', 'technical', 'roadmap', 'dependencies', 'risks', 'research'];

      for (const section of sections) {
        mockGenerateObject.mockClear();
        mockGenerateObject.mockResolvedValue({
          object: (mockCompletePRD as any)[section],
          usage: { promptTokens: 100, completionTokens: 100, totalTokens: 200 },
        });

        await generateSection(section, 'Description');

        expect(mockGenerateObject).toHaveBeenCalledTimes(1);
        const call = mockGenerateObject.mock.calls[0][0];
        expect(call.prompt).toContain(`"${section}"`);
      }
    });

    it('should track telemetry for section generation', async () => {
      await generateSection('overview', 'Description', undefined, undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'generate_section',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });
  });

  describe('generateFromSession', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: mockCompletePRD,
        usage: {
          promptTokens: 800,
          completionTokens: 1200,
          totalTokens: 2000,
        },
      });
    });

    it('should generate PRD from aggregated session data', async () => {
      const result = await generateFromSession(mockSessionData);

      expect(result).toHaveProperty('overview');
      expect(result).toHaveProperty('ux');
      expect(result).toHaveProperty('technical');
    });

    it('should build context from session data in prompt', async () => {
      await generateFromSession(mockSessionData);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('PROJECT DESCRIPTION:');
      expect(call.prompt).toContain('Build a project management tool');
    });

    it('should use lower temperature for session-based generation', async () => {
      await generateFromSession(mockSessionData);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.temperature).toBe(0.4);
    });

    it('should track telemetry for session generation', async () => {
      await generateFromSession(mockSessionData, undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'generate_from_session',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });
  });

  describe('fillSkippedSections', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: mockCompletePRD.overview,
        usage: {
          promptTokens: 200,
          completionTokens: 150,
          totalTokens: 350,
        },
      });
    });

    it('should fill multiple skipped sections', async () => {
      const sectionsToFill = ['overview', 'ux'];
      const result = await fillSkippedSections(sectionsToFill, mockSessionData);

      expect(result).toHaveProperty('overview');
      expect(result).toHaveProperty('ux');
      expect(mockGenerateObject).toHaveBeenCalledTimes(2);
    });

    it('should continue filling sections even if one fails', async () => {
      mockGenerateObject
        .mockRejectedValueOnce(new Error('First section failed'))
        .mockResolvedValueOnce({
          object: mockCompletePRD.ux,
          usage: { promptTokens: 200, completionTokens: 150, totalTokens: 350 },
        });

      const result = await fillSkippedSections(['overview', 'ux'], mockSessionData);

      expect(result).not.toHaveProperty('overview');
      expect(result).toHaveProperty('ux');
      expect(mockGenerateObject).toHaveBeenCalledTimes(2);
    });

    it('should return empty object when no sections provided', async () => {
      const result = await fillSkippedSections([], mockSessionData);

      expect(result).toEqual({});
      expect(mockGenerateObject).not.toHaveBeenCalled();
    });
  });

  describe('generateSectionWithContext', () => {
    beforeEach(() => {
      mockGenerateObject.mockResolvedValue({
        object: mockCompletePRD.overview,
        usage: {
          promptTokens: 200,
          completionTokens: 150,
          totalTokens: 350,
        },
      });
    });

    it('should generate section with full session context', async () => {
      const result = await generateSectionWithContext('overview', 'Description', mockSessionData);

      expect(result).toHaveProperty('problem_statement');
      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('PROJECT DESCRIPTION:');
      expect(call.prompt).toContain('Build a project management tool');
    });

    it('should include roundtable insights in context', async () => {
      await generateSectionWithContext('technical', 'Description', mockSessionData);

      const call = mockGenerateObject.mock.calls[0][0];
      expect(call.prompt).toContain('EXPERT INSIGHTS:');
      expect(call.prompt).toContain('Consider microservices architecture');
    });
  });

  describe('regenerateWithTemplate', () => {
    beforeEach(() => {
      const mockStream = {
        textStream: (async function* () {
          yield 'Generated ';
          yield 'PRD ';
          yield 'content';
        })(),
        finishReason: Promise.resolve('stop'),
        usage: Promise.resolve({
          promptTokens: 300,
          completionTokens: 200,
          totalTokens: 500,
        }),
      };
      mockStreamText.mockResolvedValue(mockStream);
    });

    it('should regenerate PRD using template', async () => {
      const template = '# PRD\n\n## Overview\n{{overview}}';
      const result = await regenerateWithTemplate(mockSessionData, template);

      expect(typeof result).toBe('string');
      expect(result).toBe('Generated PRD content');
      expect(mockStreamText).toHaveBeenCalledTimes(1);
    });

    it('should include template in prompt', async () => {
      const template = '# Custom Template\n{{sections}}';
      await regenerateWithTemplate(mockSessionData, template);

      const call = mockStreamText.mock.calls[0][0];
      expect(call.prompt).toContain('TEMPLATE:');
      expect(call.prompt).toContain('# Custom Template');
    });

    it('should use custom provider when specified', async () => {
      await regenerateWithTemplate(mockSessionData, 'Template', 'openai', 'gpt-4-turbo');

      const { getModelInstance } = await import('@/lib/ai/config');
      expect(getModelInstance).toHaveBeenCalledWith('openai', 'gpt-4-turbo');
    });

    it('should track telemetry for template regeneration', async () => {
      await regenerateWithTemplate(mockSessionData, 'Template', undefined, undefined, undefined, 'project-1');

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'regenerate_with_template',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });

    it('should consume entire stream before returning', async () => {
      let chunkCount = 0;
      const mockStream = {
        textStream: (async function* () {
          chunkCount++;
          yield 'Chunk 1';
          chunkCount++;
          yield 'Chunk 2';
        })(),
        finishReason: Promise.resolve('stop'),
        usage: Promise.resolve({ promptTokens: 100, completionTokens: 100, totalTokens: 200 }),
      };
      mockStreamText.mockResolvedValue(mockStream);

      await regenerateWithTemplate(mockSessionData, 'Template');

      expect(chunkCount).toBe(2);
    });
  });

  describe('regenerateWithTemplateStream', () => {
    beforeEach(() => {
      const mockStream = {
        textStream: (async function* () {
          yield 'Generated ';
          yield 'PRD ';
          yield 'content';
        })(),
        finishReason: Promise.resolve('stop'),
        usage: Promise.resolve({
          promptTokens: 300,
          completionTokens: 200,
          totalTokens: 500,
        }),
        onFinish: vi.fn(),
      };
      mockStreamText.mockResolvedValue(mockStream);
    });

    it('should stream PRD regeneration with chunks', async () => {
      const chunks: string[] = [];
      let fullText = '';

      await regenerateWithTemplateStream(
        mockSessionData,
        'Template',
        (chunk) => chunks.push(chunk),
        (full) => {
          fullText = full;
        },
        (error) => {
          throw error;
        }
      );

      expect(chunks).toEqual(['Generated ', 'PRD ', 'content']);
      expect(fullText).toBe('Generated PRD content');
    });

    it('should call onComplete with full text', async () => {
      const onComplete = vi.fn();

      await regenerateWithTemplateStream(
        mockSessionData,
        'Template',
        () => {},
        onComplete,
        () => {}
      );

      expect(onComplete).toHaveBeenCalledWith('Generated PRD content');
    });

    it('should handle streaming errors', async () => {
      mockStreamText.mockRejectedValueOnce(new Error('Stream error'));

      const onError = vi.fn();

      await regenerateWithTemplateStream(
        mockSessionData,
        'Template',
        () => {},
        () => {},
        onError
      );

      expect(onError).toHaveBeenCalledWith(expect.objectContaining({ message: 'Stream error' }));
    });

    it('should trigger onFinish for telemetry', async () => {
      const mockOnFinish = vi.fn();
      const mockStream = {
        textStream: (async function* () {
          yield 'content';
        })(),
        finishReason: Promise.resolve('stop'),
        usage: Promise.resolve({ promptTokens: 100, completionTokens: 100, totalTokens: 200 }),
        onFinish: mockOnFinish,
      };
      mockStreamText.mockResolvedValue(mockStream);

      await regenerateWithTemplateStream(
        mockSessionData,
        'Template',
        () => {},
        () => {},
        () => {}
      );

      expect(mockOnFinish).toHaveBeenCalledWith({
        finishReason: 'stop',
        usage: { promptTokens: 100, completionTokens: 100, totalTokens: 200 },
      });
    });

    it('should track telemetry for stream regeneration', async () => {
      await regenerateWithTemplateStream(
        mockSessionData,
        'Template',
        () => {},
        () => {},
        () => {},
        undefined,
        undefined,
        undefined,
        'project-1'
      );

      expect(mockTrackAIOperationWithCost).toHaveBeenCalledWith(
        'regenerate_with_template_stream',
        'project-1',
        'claude-sonnet-4-5-20250929',
        'anthropic',
        expect.any(Function),
        expect.any(Function)
      );
    });
  });

  describe('buildContextFromAggregated', () => {
    it('should build comprehensive context from session data', () => {
      const context = buildContextFromAggregated(mockSessionData);

      expect(context).toContain('PROJECT DESCRIPTION:');
      expect(context).toContain('Build a project management tool');
      expect(context).toContain('OVERVIEW:');
      expect(context).toContain('UX DETAILS:');
      expect(context).toContain('TECHNICAL ARCHITECTURE:');
      expect(context).toContain('ROADMAP:');
      expect(context).toContain('DEPENDENCIES:');
      expect(context).toContain('RISKS:');
      expect(context).toContain('RESEARCH:');
      expect(context).toContain('EXPERT INSIGHTS:');
      expect(context).toContain('COMPLETENESS: 100%');
    });

    it('should handle missing sections gracefully', () => {
      const minimalData: AggregatedPRDData = {
        session: {
          id: 'session-1',
          initial_description: 'Minimal project',
          mode: 'quick',
          status: 'active',
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
        },
        overview: null,
        ux: null,
        technical: null,
        roadmap: null,
        dependencies: null,
        risks: null,
        research: null,
        roundtable_insights: [],
        completeness: {
          completed_sections: 0,
          total_sections: 7,
          completion_percentage: 0,
        },
        skipped_sections: ['overview', 'ux'],
      } as any;

      const context = buildContextFromAggregated(minimalData);

      expect(context).toContain('PROJECT DESCRIPTION:');
      expect(context).toContain('Minimal project');
      expect(context).not.toContain('OVERVIEW:');
      expect(context).toContain('COMPLETENESS: 0%');
      expect(context).toContain('Skipped Sections: overview, ux');
    });

    it('should include all section details when present', () => {
      const context = buildContextFromAggregated(mockSessionData);

      // Overview details
      expect(context).toContain('Problem: Users need better project management');
      expect(context).toContain('Target Audience: Development teams');

      // UX details
      expect(context).toContain('Personas:');
      expect(context).toContain('Developer Dave');

      // Technical details
      expect(context).toContain('Tech Stack: React, Node.js, PostgreSQL');

      // Roadmap
      expect(context).toContain('MVP Scope: User authentication, Project creation, Basic dashboard');

      // Dependencies
      expect(context).toContain('Foundation: User authentication, Database setup');

      // Risks
      expect(context).toContain('Technical Risks:');
      expect(context).toContain('Scalability concerns');

      // Research
      expect(context).toContain('Findings: Market analysis shows strong demand');

      // Roundtable
      expect(context).toContain('[technical] Consider microservices architecture');
    });
  });

  describe('Integration', () => {
    it('should work end-to-end: generate -> sections -> template', async () => {
      // First generate complete PRD
      mockGenerateObject.mockResolvedValueOnce({
        object: mockCompletePRD,
        usage: { promptTokens: 500, completionTokens: 1500, totalTokens: 2000 },
      });

      const completePRD = await generateCompletePRD('Build a tool');

      // Then generate a specific section
      mockGenerateObject.mockResolvedValueOnce({
        object: mockCompletePRD.overview,
        usage: { promptTokens: 200, completionTokens: 150, totalTokens: 350 },
      });

      const overviewSection = await generateSection('overview', 'Build a tool');

      // Finally regenerate with template
      const mockStream = {
        textStream: (async function* () {
          yield 'Generated content';
        })(),
        finishReason: Promise.resolve('stop'),
        usage: Promise.resolve({ promptTokens: 300, completionTokens: 200, totalTokens: 500 }),
      };
      mockStreamText.mockResolvedValueOnce(mockStream);

      const regenerated = await regenerateWithTemplate(mockSessionData, 'Template');

      expect(completePRD).toHaveProperty('overview');
      expect(overviewSection).toHaveProperty('problem_statement');
      expect(regenerated).toBe('Generated content');
      expect(mockGenerateObject).toHaveBeenCalledTimes(2);
      expect(mockStreamText).toHaveBeenCalledTimes(1);
    });
  });
});
