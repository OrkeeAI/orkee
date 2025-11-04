// ABOUTME: AI-powered dependency analysis services using AI SDK
// ABOUTME: Handles automatic detection of feature dependencies and build order suggestions

import { generateObject } from 'ai';
import { getModelInstance, calculateCost } from '@/lib/ai/config';
import { getModelForTask } from './model-preferences';
import { trackAIOperationWithCost } from '@/lib/ai/telemetry';
import { z } from 'zod';
import type { IdeateFeature, DependencyType, DependencyStrength, FeatureDependency } from './ideate';
import { ideateService } from './ideate';

/**
 * Schema for detected dependency
 */
const DetectedDependencySchema = z.object({
  from_feature_id: z.string(),
  to_feature_id: z.string(),
  dependency_type: z.enum(['technical', 'logical', 'business']),
  strength: z.enum(['required', 'recommended', 'optional']),
  reason: z.string(),
  confidence_score: z.number().min(0).max(1),
});

/**
 * Schema for dependency analysis result
 */
const DependencyAnalysisSchema = z.object({
  dependencies: z.array(DetectedDependencySchema),
  analysis_summary: z.string(),
  recommendations: z.array(z.string()),
});

/**
 * Analyze features and detect dependencies using AI
 */
export async function analyzeDependencies(
  sessionId: string,
  features: IdeateFeature[],
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<{
  detected_dependencies: FeatureDependency[];
  analysis_summary: string;
  recommendations: string[];
}> {
  // Use preferences or fall back to default
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  // Build feature context
  const featuresText = features
    .map(
      (f, i) => `${i + 1}. ${f.name} (ID: ${f.id})
   Category: ${f.category}
   Description: ${f.description}
   Priority: ${f.priority}
   Complexity: ${f.complexity}
   ${f.technical_details ? `Technical: ${f.technical_details}` : ''}`
    )
    .join('\n\n');

  const prompt = `Analyze these feature descriptions and identify dependencies between them:

${featuresText}

Identify dependencies where one feature must or should be built before another. Consider:

1. **Technical Dependencies** (e.g., API before UI, database before CRUD operations)
   - Infrastructure or technical foundation requirements
   - Technology stack prerequisites

2. **Logical Dependencies** (e.g., data models before CRUD operations)
   - Business logic flow requirements
   - Data flow dependencies

3. **Business Dependencies** (e.g., MVP features before enhancements)
   - Priority-based ordering
   - Risk mitigation requirements

For each dependency provide:
- From which feature ID to which feature ID
- Type of dependency (technical/logical/business)
- Strength (required/recommended/optional)
- Clear reason explaining why the dependency exists
- Confidence score (0-1) indicating how certain you are

Also provide:
- An overall analysis summary
- Recommendations for implementation order

Only suggest dependencies that are clearly justified. Avoid creating unnecessary dependencies.`;

  const result = await trackAIOperationWithCost(
    'analyze_dependencies',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () => generateObject({
      model,
      schema: DependencyAnalysisSchema,
      prompt,
      temperature: 0.3,
      experimental_telemetry: { isEnabled: true },
    })
  );

  const analysis = result.object;

  // Convert detected dependencies to FeatureDependency format and save to backend
  const savedDependencies: FeatureDependency[] = [];

  for (const dep of analysis.dependencies) {
    // Only save dependencies with confidence > 0.7
    if (dep.confidence_score > 0.7) {
      const saved = await ideateService.createFeatureDependency(sessionId, {
        fromFeatureId: dep.from_feature_id,
        toFeatureId: dep.to_feature_id,
        dependencyType: dep.dependency_type,
        strength: dep.strength,
        reason: dep.reason,
      });

      if (saved) {
        savedDependencies.push(saved);
      }
    }
  }

  return {
    detected_dependencies: savedDependencies,
    analysis_summary: analysis.analysis_summary,
    recommendations: analysis.recommendations,
  };
}

/**
 * Schema for build order suggestion
 */
const BuildOrderSuggestionSchema = z.object({
  suggested_order: z.array(z.string()),
  rationale: z.string(),
  parallel_groups: z.array(
    z.object({
      features: z.array(z.string()),
      description: z.string(),
    })
  ),
  critical_path: z.array(z.string()),
  risks: z.array(z.string()),
});

/**
 * Suggest build order based on dependencies using AI
 */
export async function suggestBuildOrder(
  sessionId: string,
  features: IdeateFeature[],
  dependencies: FeatureDependency[],
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<{
  suggested_order: string[];
  rationale: string;
  parallel_groups: Array<{ features: string[]; description: string }>;
  critical_path: string[];
  risks: string[];
}> {
  // Use preferences or fall back to default
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  // Build context
  const featuresMap = new Map(features.map((f) => [f.id, f]));

  const featuresText = features
    .map((f) => `- ${f.name} (${f.id}): ${f.description} [Priority: ${f.priority}, Complexity: ${f.complexity}]`)
    .join('\n');

  const dependenciesText = dependencies
    .map((d) => {
      const from = featuresMap.get(d.from_feature_id)?.name || d.from_feature_id;
      const to = featuresMap.get(d.to_feature_id)?.name || d.to_feature_id;
      return `- ${from} → ${to} (${d.dependency_type}, ${d.strength}): ${d.reason || 'No reason provided'}`;
    })
    .join('\n');

  const prompt = `Suggest an optimal build order for these features based on their dependencies:

FEATURES:
${featuresText}

DEPENDENCIES:
${dependenciesText}

Provide:
1. **Suggested Order**: Complete list of feature IDs in recommended build order
2. **Rationale**: Explanation of why this order is optimal
3. **Parallel Groups**: Sets of features that can be built in parallel (no dependencies between them)
4. **Critical Path**: Sequence of features that must be built in order (longest dependency chain)
5. **Risks**: Potential issues with this build order

Optimize for:
- Respecting all required dependencies
- Maximizing parallel work opportunities
- Building foundation features early
- Completing high-priority features as soon as their dependencies allow`;

  const result = await trackAIOperationWithCost(
    'suggest_build_order',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () => generateObject({
      model,
      schema: BuildOrderSuggestionSchema,
      prompt,
      temperature: 0.2,
      experimental_telemetry: { isEnabled: true },
    })
  );

  return result.object;
}

/**
 * Schema for quick win suggestion
 */
const QuickWinSchema = z.object({
  feature_id: z.string(),
  name: z.string(),
  rationale: z.string(),
  estimated_effort: z.number().min(1).max(10),
  expected_impact: z.string(),
});

/**
 * Schema for quick wins analysis
 */
const QuickWinsAnalysisSchema = z.object({
  quick_wins: z.array(QuickWinSchema),
  explanation: z.string(),
});

/**
 * Suggest quick wins - features with no dependencies that can be started immediately
 */
export async function suggestQuickWins(
  sessionId: string,
  features: IdeateFeature[],
  dependencies: FeatureDependency[],
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<{
  quick_wins: Array<{
    feature_id: string;
    name: string;
    rationale: string;
    estimated_effort: number;
    expected_impact: string;
  }>;
  explanation: string;
}> {
  // Use preferences or fall back to default
  const modelConfig = modelPreferences || { provider: 'anthropic' as const, model: 'claude-sonnet-4-5-20250929' };
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  // Identify features with no dependencies
  const dependentFeatureIds = new Set(dependencies.map((d) => d.from_feature_id));
  const independentFeatures = features.filter((f) => !dependentFeatureIds.has(f.id));

  if (independentFeatures.length === 0) {
    return {
      quick_wins: [],
      explanation: 'No independent features found. All features have dependencies that should be resolved first.',
    };
  }

  const featuresText = independentFeatures
    .map(
      (f) => `${f.id}: ${f.name}
   Description: ${f.description}
   Priority: ${f.priority}
   Complexity: ${f.complexity}
   Category: ${f.category}`
    )
    .join('\n\n');

  const prompt = `Identify quick win features from this list of independent features (no dependencies):

${featuresText}

Quick wins are features that:
1. Can be started immediately (no dependencies)
2. Provide clear value quickly
3. Have reasonable effort/complexity
4. Build momentum for the project
5. Help validate assumptions or technical choices

For each quick win, provide:
- Feature ID
- Feature name
- Rationale for why it's a quick win
- Estimated effort (1-10, where 1 is trivial and 10 is complex)
- Expected impact on the project

Suggest 3-5 quick wins ordered by priority.`;

  const result = await trackAIOperationWithCost(
    'suggest_quick_wins',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(modelConfig.provider, modelConfig.model, inputTokens, outputTokens),
    () => generateObject({
      model,
      schema: QuickWinsAnalysisSchema,
      prompt,
      temperature: 0.3,
      experimental_telemetry: { isEnabled: true },
    })
  );

  return result.object;
}
