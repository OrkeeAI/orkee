// ABOUTME: AI-powered PRD generation services using AI SDK
// ABOUTME: Handles PRD generation, section generation, template regeneration with streaming support

import { generateObject, streamText } from 'ai';
import { getModelInstance, calculateCost } from '@/lib/ai/config';
import { getModelForTask } from './model-preferences';
import { trackAIOperationWithCost } from '@/lib/ai/telemetry';
import { z } from 'zod';
import type { AggregatedPRDData } from './ideate';

/**
 * Schema for complete PRD generation
 */
const CompletePRDSchema = z.object({
  overview: z.object({
    problem_statement: z.string(),
    target_audience: z.string(),
    value_proposition: z.string(),
    one_line_pitch: z.string(),
  }),
  ux: z.object({
    personas: z.array(
      z.object({
        name: z.string(),
        role: z.string(),
        goals: z.array(z.string()),
        pain_points: z.array(z.string()),
      })
    ),
    user_flows: z.array(
      z.object({
        name: z.string(),
        steps: z.array(
          z.object({
            action: z.string(),
            screen: z.string(),
            notes: z.string().nullable(),
          })
        ),
        touchpoints: z.array(z.string()),
      })
    ),
    ui_considerations: z.string().nullable(),
    ux_principles: z.string().nullable(),
  }),
  technical: z.object({
    components: z.array(
      z.object({
        name: z.string(),
        purpose: z.string(),
        technology: z.string().nullable(),
      })
    ),
    data_models: z.array(
      z.object({
        name: z.string(),
        fields: z.array(
          z.object({
            name: z.string(),
            field_type: z.string(),
            required: z.boolean(),
          })
        ),
      })
    ),
    apis: z.array(
      z.object({
        name: z.string(),
        purpose: z.string(),
        endpoints: z.array(z.string()),
      })
    ),
    infrastructure: z
      .object({
        hosting: z.string().nullable(),
        database: z.string().nullable(),
        caching: z.string().nullable(),
        file_storage: z.string().nullable(),
      })
      .nullable(),
    tech_stack_quick: z.string().nullable(),
  }),
  roadmap: z.object({
    mvp_scope: z.array(z.string()),
    future_phases: z.array(
      z.object({
        name: z.string(),
        features: z.array(z.string()),
        goals: z.array(z.string()),
      })
    ),
  }),
  dependencies: z.object({
    foundation_features: z.array(z.string()),
    visible_features: z.array(z.string()),
    enhancement_features: z.array(z.string()),
    dependency_graph: z.record(z.unknown()).nullable(),
  }),
  risks: z.object({
    technical_risks: z.array(
      z.object({
        description: z.string(),
        severity: z.string(),
        probability: z.string(),
      })
    ),
    mvp_scoping_risks: z.array(
      z.object({
        description: z.string(),
        severity: z.string(),
        probability: z.string(),
      })
    ),
    resource_risks: z.array(
      z.object({
        description: z.string(),
        severity: z.string(),
        probability: z.string(),
      })
    ),
    mitigations: z.array(
      z.object({
        risk: z.string(),
        strategy: z.string(),
        owner: z.string().nullable(),
      })
    ),
  }),
  research: z.object({
    competitors: z.array(
      z.object({
        name: z.string(),
        url: z.string(),
        strengths: z.array(z.string()),
        gaps: z.array(z.string()),
        features: z.array(z.string()),
      })
    ),
    similar_projects: z.array(
      z.object({
        name: z.string(),
        url: z.string(),
        positive_aspects: z.array(z.string()),
        negative_aspects: z.array(z.string()),
        patterns_to_adopt: z.array(z.string()),
      })
    ),
    research_findings: z.string().nullable(),
    technical_specs: z.string().nullable(),
    reference_links: z.array(z.string()),
  }),
});

/**
 * Generate a complete PRD from a description
 * Replaces Rust: generate_complete_prd_with_model()
 */
export async function generateCompletePRD(
  description: string,
  provider?: 'anthropic' | 'openai' | 'google' | 'xai',
  model?: string,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<z.infer<typeof CompletePRDSchema>> {
  // Use explicit params > preferences > default
  const modelConfig =
    provider && model
      ? { provider, model }
      : modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };

  const modelInstance = getModelInstance(modelConfig.provider, modelConfig.model);

  const prompt = `Generate a comprehensive Product Requirements Document based on this description:

${description}

Create a structured PRD with:

1. **Overview**:
   - Clear problem statement
   - Target audience description
   - Value proposition
   - One-line pitch

2. **User Experience**:
   - User personas (2-3 with goals and pain points)
   - User flows (key journeys through the system)
   - UI considerations
   - UX principles

3. **Technical Architecture**:
   - System components and their purposes
   - Data models with fields
   - API specifications
   - Infrastructure requirements
   - Recommended tech stack

4. **Roadmap**:
   - MVP scope (essential features)
   - Future phases with features and goals

5. **Dependencies**:
   - Foundation features (must build first)
   - Visible features (core functionality)
   - Enhancement features (nice-to-have)

6. **Risks**:
   - Technical risks
   - MVP scoping risks
   - Resource risks
   - Mitigation strategies for each

7. **Research** (IMPORTANT - be thorough here):
   - Competitor analysis: identify 3-5 real competitors/alternatives, their strengths and gaps
   - Similar projects for reference: open-source or notable projects in the same space
   - Research findings: market context, user behavior patterns, industry trends
   - Technical specifications: relevant standards, protocols, or best practices
   - Reference links: real URLs to documentation, competitors, and resources

Be specific and actionable. Think through the technical implementation details.
For the research section, draw on your knowledge of the market to provide genuine competitive analysis rather than generic placeholders.`;

  const result = await trackAIOperationWithCost(
    'generate_complete_prd',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model: modelInstance,
        schema: CompletePRDSchema,
        prompt,
        temperature: 0.5,
        maxTokens: 16000,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object;
}

/**
 * Generate a specific section of the PRD
 * Replaces Rust: generate_section()
 */
export async function generateSection(
  section: string,
  description: string,
  context?: string,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<unknown> {
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  // Define schemas for different sections
  const sectionSchemas: Record<string, z.ZodType> = {
    overview: z.object({
      problem_statement: z.string(),
      target_audience: z.string(),
      value_proposition: z.string(),
      one_line_pitch: z.string(),
    }),
    ux: z.object({
      personas: z.array(
        z.object({
          name: z.string(),
          role: z.string(),
          goals: z.array(z.string()),
          pain_points: z.array(z.string()),
        })
      ),
      user_flows: z.array(
        z.object({
          name: z.string(),
          steps: z.array(
            z.object({
              action: z.string(),
              screen: z.string(),
              notes: z.string().nullable(),
            })
          ),
          touchpoints: z.array(z.string()),
        })
      ),
      ui_considerations: z.string().nullable(),
      ux_principles: z.string().nullable(),
    }),
    technical: z.object({
      components: z.array(
        z.object({
          name: z.string(),
          purpose: z.string(),
          technology: z.string().nullable(),
        })
      ),
      data_models: z.array(
        z.object({
          name: z.string(),
          fields: z.array(
            z.object({
              name: z.string(),
              field_type: z.string(),
              required: z.boolean(),
            })
          ),
        })
      ),
      apis: z.array(
        z.object({
          name: z.string(),
          purpose: z.string(),
          endpoints: z.array(z.string()),
        })
      ),
      infrastructure: z
        .object({
          hosting: z.string().nullable(),
          database: z.string().nullable(),
          caching: z.string().nullable(),
          file_storage: z.string().nullable(),
        })
        .nullable(),
      tech_stack_quick: z.string().nullable(),
    }),
    roadmap: z.object({
      mvp_scope: z.array(z.string()),
      future_phases: z.array(
        z.object({
          name: z.string(),
          features: z.array(z.string()),
          goals: z.array(z.string()),
        })
      ),
    }),
    dependencies: z.object({
      foundation_features: z.array(z.string()),
      visible_features: z.array(z.string()),
      enhancement_features: z.array(z.string()),
      dependency_graph: z.record(z.unknown()).nullable(),
    }),
    risks: z.object({
      technical_risks: z.array(
        z.object({
          description: z.string(),
          severity: z.string(),
          probability: z.string(),
        })
      ),
      mvp_scoping_risks: z.array(
        z.object({
          description: z.string(),
          severity: z.string(),
          probability: z.string(),
        })
      ),
      resource_risks: z.array(
        z.object({
          description: z.string(),
          severity: z.string(),
          probability: z.string(),
        })
      ),
      mitigations: z.array(
        z.object({
          risk: z.string(),
          strategy: z.string(),
          owner: z.string().nullable(),
        })
      ),
    }),
    research: z.object({
      competitors: z.array(
        z.object({
          name: z.string(),
          url: z.string(),
          strengths: z.array(z.string()),
          gaps: z.array(z.string()),
          features: z.array(z.string()),
        })
      ),
      similar_projects: z.array(
        z.object({
          name: z.string(),
          url: z.string(),
          positive_aspects: z.array(z.string()),
          negative_aspects: z.array(z.string()),
          patterns_to_adopt: z.array(z.string()),
        })
      ),
      research_findings: z.string().nullable(),
      technical_specs: z.string().nullable(),
      reference_links: z.array(z.string()),
    }),
  };

  const schema = sectionSchemas[section];
  if (!schema) {
    throw new Error(`Unknown section: ${section}`);
  }

  const contextPart = context ? `\n\nAdditional context:\n${context}` : '';

  const prompt = `Generate the "${section}" section for a Product Requirements Document.

Project description:
${description}${contextPart}

Provide detailed, actionable content for this section. Be specific about technical details where appropriate.`;

  const result = await trackAIOperationWithCost(
    'generate_section',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model,
        schema,
        prompt,
        temperature: 0.5,
        maxTokens: 4000,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object;
}

/**
 * Generate PRD from aggregated session data
 * Replaces Rust: generate_from_session()
 */
export async function generateFromSession(
  sessionData: AggregatedPRDData,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<z.infer<typeof CompletePRDSchema>> {
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  // Build context from aggregated data using helper function pattern
  const context = buildContextFromAggregated(sessionData);

  const prompt = `Generate a complete Product Requirements Document based on this collected session data:

${context}

Synthesize all the information provided to create a comprehensive, cohesive PRD. Fill in any gaps with reasonable defaults based on the context.`;

  const result = await trackAIOperationWithCost(
    'generate_from_session',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      generateObject({
        model,
        schema: CompletePRDSchema,
        prompt,
        temperature: 0.4,
        maxTokens: 16000,
        experimental_telemetry: { isEnabled: true },
      })
  );

  return result.object;
}

/**
 * Fill skipped sections with AI-generated content
 * Replaces Rust: fill_skipped_sections()
 */
export async function fillSkippedSections(
  sectionsToFill: string[],
  sessionData: AggregatedPRDData,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<Record<string, unknown>> {
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };

  const context = buildContextFromAggregated(sessionData);
  const filledSections: Record<string, unknown> = {};

  // Fill each section sequentially
  for (const section of sectionsToFill) {
    try {
      const sectionData = await generateSection(
        section,
        sessionData.session.initial_description,
        context,
        modelConfig,
        projectId
      );
      filledSections[section] = sectionData;
    } catch (error) {
      console.error(`Failed to fill section ${section}:`, error);
      // Continue with other sections
    }
  }

  return filledSections;
}

/**
 * Generate a specific section with full context from other sections
 * Replaces Rust: generate_section_with_context()
 */
export async function generateSectionWithContext(
  section: string,
  description: string,
  sessionData: AggregatedPRDData,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<unknown> {
  const context = buildContextFromAggregated(sessionData);
  return generateSection(section, description, context, modelPreferences, projectId);
}

/**
 * Regenerate PRD using a template (non-streaming)
 * Replaces Rust: regenerate_with_template()
 */
export async function regenerateWithTemplate(
  sessionData: AggregatedPRDData,
  templateContent: string,
  provider?: string,
  model?: string,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<string> {
  const modelConfig =
    provider && model
      ? { provider: provider as 'anthropic' | 'openai' | 'google' | 'xai', model }
      : modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };

  const modelInstance = getModelInstance(modelConfig.provider, modelConfig.model);

  const context = buildContextFromAggregated(sessionData);

  const prompt = `Regenerate this PRD using the provided template. Fill in the template sections with content based on the session data.

SESSION DATA:
${context}

TEMPLATE:
${templateContent}

Generate markdown content that follows the template structure exactly, replacing placeholders with appropriate content from the session data. Output ONLY the filled template, no explanations.`;

  const result = await trackAIOperationWithCost(
    'regenerate_with_template',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () =>
      streamText({
        model: modelInstance,
        prompt,
        temperature: 0.4,
        maxTokens: 16000,
        experimental_telemetry: { isEnabled: true },
      })
  );

  // Consume entire stream
  let fullText = '';
  for await (const chunk of result.textStream) {
    fullText += chunk;
  }

  // Wait for finalization
  await result.finishReason;
  await result.usage;

  return fullText;
}

/**
 * Regenerate PRD using a template (streaming version)
 * Replaces Rust: regenerate_with_template_stream()
 */
export async function regenerateWithTemplateStream(
  sessionData: AggregatedPRDData,
  templateContent: string,
  onChunk: (text: string) => void,
  onComplete: (fullText: string) => void,
  onError: (error: Error) => void,
  provider?: string,
  model?: string,
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<void> {
  try {
    const modelConfig =
      provider && model
        ? { provider: provider as 'anthropic' | 'openai' | 'google' | 'xai', model }
        : modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };

    const modelInstance = getModelInstance(modelConfig.provider, modelConfig.model);

    const context = buildContextFromAggregated(sessionData);

    const prompt = `Regenerate this PRD using the provided template. Fill in the template sections with content based on the session data.

SESSION DATA:
${context}

TEMPLATE:
${templateContent}

Generate markdown content that follows the template structure exactly, replacing placeholders with appropriate content from the session data. Output ONLY the filled template, no explanations.`;

    const result = await trackAIOperationWithCost(
      'regenerate_with_template_stream',
      projectId || null,
      modelConfig.model,
      modelConfig.provider,
      (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
      () =>
        streamText({
          model: modelInstance,
          prompt,
          temperature: 0.4,
          maxTokens: 16000,
          experimental_telemetry: { isEnabled: true },
        })
    );

    let fullText = '';

    // Consume stream and emit chunks
    for await (const chunk of result.textStream) {
      fullText += chunk;
      onChunk(chunk);
    }

    // Wait for finalization
    const finalReason = await result.finishReason;
    const finalUsage = await result.usage;

    console.log('[regenerateWithTemplateStream] Stream finalized:', { finalReason, finalUsage });

    // Manually trigger onFinish for telemetry
    if (result.onFinish) {
      await result.onFinish({
        finishReason: finalReason,
        usage: finalUsage,
      });
    }

    onComplete(fullText);
  } catch (error) {
    console.error('[regenerateWithTemplateStream] Streaming error:', error);
    const err = error instanceof Error ? error : new Error('Streaming failed');
    onError(err);
  }
}

/**
 * Build context string from aggregated session data
 * This is a public helper function matching the pattern in prd_generator.rs
 */
export function buildContextFromAggregated(data: AggregatedPRDData): string {
  const parts: string[] = [];

  // Session info
  parts.push(`PROJECT DESCRIPTION:\n${data.session.initial_description}\n`);

  // Overview
  if (data.overview) {
    parts.push(`\nOVERVIEW:`);
    if (data.overview.problem_statement) {
      parts.push(`Problem: ${data.overview.problem_statement}`);
    }
    if (data.overview.target_audience) {
      parts.push(`Target Audience: ${data.overview.target_audience}`);
    }
    if (data.overview.value_proposition) {
      parts.push(`Value Proposition: ${data.overview.value_proposition}`);
    }
    if (data.overview.one_line_pitch) {
      parts.push(`Pitch: ${data.overview.one_line_pitch}`);
    }
  }

  // UX
  if (data.ux) {
    parts.push(`\nUX DETAILS:`);
    if (data.ux.personas && data.ux.personas.length > 0) {
      parts.push(`Personas: ${JSON.stringify(data.ux.personas)}`);
    }
    if (data.ux.user_flows && data.ux.user_flows.length > 0) {
      parts.push(`User Flows: ${JSON.stringify(data.ux.user_flows)}`);
    }
    if (data.ux.ui_considerations) {
      parts.push(`UI Considerations: ${data.ux.ui_considerations}`);
    }
    if (data.ux.ux_principles) {
      parts.push(`UX Principles: ${data.ux.ux_principles}`);
    }
  }

  // Technical
  if (data.technical) {
    parts.push(`\nTECHNICAL ARCHITECTURE:`);
    if (data.technical.tech_stack_quick) {
      parts.push(`Tech Stack: ${data.technical.tech_stack_quick}`);
    }
    if (data.technical.components && data.technical.components.length > 0) {
      parts.push(`Components: ${JSON.stringify(data.technical.components)}`);
    }
    if (data.technical.data_models && data.technical.data_models.length > 0) {
      parts.push(`Data Models: ${JSON.stringify(data.technical.data_models)}`);
    }
    if (data.technical.infrastructure) {
      parts.push(`Infrastructure: ${JSON.stringify(data.technical.infrastructure)}`);
    }
  }

  // Roadmap
  if (data.roadmap) {
    parts.push(`\nROADMAP:`);
    if (data.roadmap.mvp_scope && data.roadmap.mvp_scope.length > 0) {
      parts.push(`MVP Scope: ${data.roadmap.mvp_scope.join(', ')}`);
    }
    if (data.roadmap.future_phases && data.roadmap.future_phases.length > 0) {
      parts.push(`Future Phases: ${JSON.stringify(data.roadmap.future_phases)}`);
    }
  }

  // Dependencies
  if (data.dependencies) {
    parts.push(`\nDEPENDENCIES:`);
    if (data.dependencies.foundation_features && data.dependencies.foundation_features.length > 0) {
      parts.push(`Foundation: ${data.dependencies.foundation_features.join(', ')}`);
    }
    if (data.dependencies.visible_features && data.dependencies.visible_features.length > 0) {
      parts.push(`Core Features: ${data.dependencies.visible_features.join(', ')}`);
    }
    if (data.dependencies.enhancement_features && data.dependencies.enhancement_features.length > 0) {
      parts.push(`Enhancements: ${data.dependencies.enhancement_features.join(', ')}`);
    }
  }

  // Risks
  if (data.risks) {
    parts.push(`\nRISKS:`);
    if (data.risks.technical_risks && data.risks.technical_risks.length > 0) {
      parts.push(`Technical Risks: ${JSON.stringify(data.risks.technical_risks)}`);
    }
    if (data.risks.mitigations && data.risks.mitigations.length > 0) {
      parts.push(`Mitigations: ${JSON.stringify(data.risks.mitigations)}`);
    }
  }

  // Research
  if (data.research) {
    parts.push(`\nRESEARCH:`);
    if (data.research.research_findings) {
      parts.push(`Findings: ${data.research.research_findings}`);
    }
    if (data.research.competitors && data.research.competitors.length > 0) {
      parts.push(`Competitors: ${JSON.stringify(data.research.competitors)}`);
    }
  }

  // Roundtable insights
  if (data.roundtable_insights && data.roundtable_insights.length > 0) {
    parts.push(`\nEXPERT INSIGHTS:`);
    data.roundtable_insights.forEach((insight) => {
      parts.push(`- [${insight.category}] ${insight.content}`);
    });
  }

  // Completeness info
  parts.push(
    `\nCOMPLETENESS: ${data.completeness.completion_percentage.toFixed(0)}% (${data.completeness.completed_sections}/${data.completeness.total_sections} sections)`
  );

  if (data.skipped_sections && data.skipped_sections.length > 0) {
    parts.push(`Skipped Sections: ${data.skipped_sections.join(', ')}`);
  }

  return parts.join('\n');
}
