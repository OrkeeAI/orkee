// ABOUTME: AI-powered research analysis services using AI SDK
// ABOUTME: Handles competitor analysis, gap analysis, UI pattern extraction, lesson extraction, and research synthesis

import { generateObject } from 'ai';
import { getModelInstance, calculateCost } from '@/lib/ai/config';
import { getModelForTask } from './model-preferences';
import { trackAIOperationWithCost } from '@/lib/ai/telemetry';
import { z } from 'zod';
import type { Competitor, SimilarProject } from './ideate';

/**
 * Schema for competitor analysis
 */
const CompetitorSchema = z.object({
  name: z.string(),
  url: z.string(),
  strengths: z.array(z.string()),
  gaps: z.array(z.string()),
  features: z.array(z.string()),
});

/**
 * Schema for UI/UX pattern
 */
const UIPatternSchema = z.object({
  pattern_type: z.enum(['layout', 'navigation', 'interaction', 'visual', 'content']),
  name: z.string(),
  description: z.string(),
  benefits: z.string(),
  adoption_notes: z.string(),
});

/**
 * Schema for gap analysis opportunity
 */
const OpportunitySchema = z.object({
  opportunity_type: z.enum(['differentiation', 'improvement', 'gap']),
  title: z.string(),
  description: z.string(),
  competitor_context: z.string(),
  recommendation: z.string(),
});

/**
 * Schema for gap analysis result
 */
const GapAnalysisSchema = z.object({
  opportunities: z.array(OpportunitySchema),
  summary: z.string(),
});

/**
 * Schema for lesson learned
 */
const LessonSchema = z.object({
  category: z.enum(['design', 'implementation', 'feature', 'ux', 'technical']),
  insight: z.string(),
  application: z.string(),
  priority: z.enum(['high', 'medium', 'low']),
});

/**
 * Schema for research synthesis
 */
const ResearchSynthesisSchema = z.object({
  key_findings: z.array(z.string()),
  market_position: z.string(),
  differentiators: z.array(z.string()),
  risks: z.array(z.string()),
  recommendations: z.array(z.string()),
});

/**
 * Analyze a competitor URL
 * Replaces Rust: analyze_competitor()
 */
export async function analyzeCompetitor(
  projectDescription: string,
  url: string,
  textContent: string,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<Competitor> {
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const prompt = `Analyze this competitor website to understand their product offering and competitive position.

PROJECT DESCRIPTION:
${projectDescription}

COMPETITOR URL: ${url}

WEBSITE CONTENT:
${textContent.substring(0, 8000)} ${textContent.length > 8000 ? '...(truncated)' : ''}

Provide a comprehensive analysis of:
1. **Company Name** - The company or product name
2. **Strengths** - What they do well, their competitive advantages
3. **Gaps** - Weaknesses, missing features, or areas where they fall short
4. **Features** - Key features and capabilities they offer

Be specific and extract concrete details from the content.`;

  const result = await trackAIOperationWithCost(
    'analyze_competitor',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model,
        schema: CompetitorSchema,
        prompt,
        temperature: 0.3,
        maxTokens: 2000,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object;
}

/**
 * Perform gap analysis across competitors
 * Replaces Rust: analyze_gaps()
 */
export async function analyzeGaps(
  projectDescription: string,
  yourFeatures: string[],
  competitors: Competitor[],
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<{ opportunities: Array<z.infer<typeof OpportunitySchema>>; summary: string }> {
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  if (competitors.length === 0) {
    return {
      opportunities: [],
      summary: 'No competitors analyzed yet.',
    };
  }

  const competitorInfo = competitors
    .map(
      (c) => `**${c.name}**:
- Features: ${c.features.join(', ')}
- Strengths: ${c.strengths.join(', ')}
- Gaps: ${c.gaps.join(', ')}`
    )
    .join('\n\n');

  const prompt = `Perform a competitive gap analysis to identify opportunities for differentiation and improvement.

YOUR PROJECT:
${projectDescription}

YOUR PLANNED FEATURES:
${yourFeatures.map((f, i) => `${i + 1}. ${f}`).join('\n')}

COMPETITOR ANALYSIS:
${competitorInfo}

Identify opportunities in three categories:
1. **Differentiation** - Features you can build that competitors don't have
2. **Improvement** - Areas where competitors are weak that you can do better
3. **Gap** - Important features competitors have that you're missing

For each opportunity, provide:
- Type (differentiation/improvement/gap)
- Clear title
- Detailed description
- Competitor context (who has what)
- Specific recommendation for your product

Also provide an overall summary of the competitive landscape.`;

  const result = await trackAIOperationWithCost(
    'analyze_gaps',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model,
        schema: GapAnalysisSchema,
        prompt,
        temperature: 0.4,
        maxTokens: 3000,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object;
}

/**
 * Extract UI/UX patterns from a website
 * Replaces Rust: extract_ui_patterns()
 */
export async function extractUIPatterns(
  projectDescription: string,
  url: string,
  textContent: string,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<Array<z.infer<typeof UIPatternSchema>>> {
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const prompt = `Extract UI/UX patterns from this website that could be useful for the project.

PROJECT DESCRIPTION:
${projectDescription}

WEBSITE URL: ${url}

WEBSITE CONTENT:
${textContent.substring(0, 8000)} ${textContent.length > 8000 ? '...(truncated)' : ''}

Identify 5-10 notable UI/UX patterns in these categories:
- **Layout** - Page structure, grid systems, responsive patterns
- **Navigation** - Menu styles, navigation patterns, breadcrumbs
- **Interaction** - Buttons, forms, modals, animations
- **Visual** - Color schemes, typography, imagery style
- **Content** - Content organization, information architecture

For each pattern provide:
- Pattern type (layout/navigation/interaction/visual/content)
- Name of the pattern
- Description of how it works
- Benefits of this pattern
- Notes on how to adopt it for your project`;

  const PatternResponseSchema = z.object({
    patterns: z.array(UIPatternSchema),
  });

  const result = await trackAIOperationWithCost(
    'extract_ui_patterns',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model,
        schema: PatternResponseSchema,
        prompt,
        temperature: 0.3,
        maxTokens: 3000,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object.patterns;
}

/**
 * Extract lessons from a similar project
 * Replaces Rust: extract_lessons()
 */
export async function extractLessons(
  projectDescription: string,
  similarProject: SimilarProject,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<Array<z.infer<typeof LessonSchema>>> {
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const prompt = `Extract actionable lessons from this similar project for our own development.

OUR PROJECT:
${projectDescription}

SIMILAR PROJECT: ${similarProject.name}
URL: ${similarProject.url}

POSITIVE ASPECTS:
${similarProject.positive_aspects.map((a, i) => `${i + 1}. ${a}`).join('\n')}

NEGATIVE ASPECTS:
${similarProject.negative_aspects.map((a, i) => `${i + 1}. ${a}`).join('\n')}

PATTERNS TO ADOPT:
${similarProject.patterns_to_adopt.map((p, i) => `${i + 1}. ${p}`).join('\n')}

Extract 5-10 lessons across these categories:
- **Design** - UI/UX design lessons
- **Implementation** - Technical implementation approaches
- **Feature** - Feature scope and prioritization
- **UX** - User experience and interaction design
- **Technical** - Architecture and technology choices

For each lesson provide:
- Category (design/implementation/feature/ux/technical)
- The insight or lesson learned
- How to apply it to our project
- Priority level (high/medium/low)`;

  const LessonResponseSchema = z.object({
    lessons: z.array(LessonSchema),
  });

  const result = await trackAIOperationWithCost(
    'extract_lessons',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model,
        schema: LessonResponseSchema,
        prompt,
        temperature: 0.3,
        maxTokens: 2500,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object.lessons;
}

/**
 * Synthesize all research findings
 * Replaces Rust: synthesize_research()
 */
export async function synthesizeResearch(
  projectDescription: string,
  competitors: Competitor[],
  similarProjects: SimilarProject[],
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<z.infer<typeof ResearchSynthesisSchema>> {
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const competitorSummary = competitors
    .map(
      (c) => `**${c.name}**:
- Strengths: ${c.strengths.join(', ')}
- Gaps: ${c.gaps.join(', ')}`
    )
    .join('\n\n');

  const prompt = `Synthesize all research findings into strategic insights and recommendations.

PROJECT DESCRIPTION:
${projectDescription}

COMPETITOR ANALYSIS (${competitors.length} analyzed):
${competitorSummary}

SIMILAR PROJECTS REVIEWED: ${similarProjects.length}

Provide a comprehensive synthesis including:

1. **Key Findings** - 5-7 most important insights from the research
2. **Market Position** - Where your product fits in the competitive landscape
3. **Differentiators** - 3-5 unique aspects that set you apart
4. **Risks** - 3-5 competitive or market risks to be aware of
5. **Recommendations** - 5-7 strategic recommendations based on the research

Be specific and actionable. Base recommendations on concrete findings from the competitor and project analysis.`;

  const result = await trackAIOperationWithCost(
    'synthesize_research',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model,
        schema: ResearchSynthesisSchema,
        prompt,
        temperature: 0.4,
        maxTokens: 3000,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object;
}
