// ABOUTME: AI prompts for PRD section generation
// ABOUTME: Structured prompts that guide Claude to generate each PRD section

/// System prompt for all PRD generation tasks
pub const SYSTEM_PROMPT: &str = r#"You are an expert product manager helping to create detailed, actionable PRDs (Product Requirement Documents).

Your responses should be:
- Structured and well-organized
- Specific and actionable
- Based on the user's project description
- Free of timeline commitments (focus on scope, not dates)
- Emphasizing logical build order and dependencies

Always respond in valid JSON format matching the requested structure."#;

/// Generate the overview section (Problem, Target, Value)
pub fn overview_prompt(description: &str) -> String {
    format!(
        r#"Based on this project description:

{}

Generate a comprehensive project overview with the following structure:

{{
  "problemStatement": "Clear description of the problem being solved",
  "targetAudience": "Who are the primary users? Be specific about demographics, roles, or use cases",
  "valueProposition": "What unique value does this solution provide? Why is it better than alternatives?",
  "oneLinePitch": "A single sentence that captures the essence of the project"
}}

Make the overview compelling and specific to this project."#,
        description
    )
}

/// Generate core features section
pub fn features_prompt(description: &str) -> String {
    format!(
        r#"Based on this project description:

{}

Generate 5-8 core features for an MVP. For each feature, provide:

{{
  "features": [
    {{
      "name": "Feature name",
      "what": "What does this feature do?",
      "why": "Why is this important for the MVP?",
      "how": "High-level approach for implementation",
      "dependsOn": ["List of feature names this depends on"],
      "enables": ["List of features this unlocks"],
      "buildPhase": 1-3 (1=foundation, 2=visible, 3=enhancement),
      "isVisible": true/false (does this give users something to see/use?)
    }}
  ]
}}

Focus on:
1. Foundation features that must be built first
2. Visible features that users can interact with quickly
3. Enhancement features that build on the foundation

Ensure dependencies are logical and create a clear build order."#,
        description
    )
}

/// Generate UX section (Personas, Flows)
pub fn ux_prompt(description: &str) -> String {
    format!(
        r#"Based on this project description:

{}

Generate a UX analysis including personas and user flows:

{{
  "personas": [
    {{
      "name": "Persona name",
      "role": "Their role or job title",
      "goals": ["What they want to accomplish"],
      "painPoints": ["Current frustrations or challenges"]
    }}
  ],
  "userFlows": [
    {{
      "name": "Flow name (e.g., 'User onboarding')",
      "steps": [
        {{
          "action": "User action",
          "screen": "Screen or interface",
          "notes": "Optional context or variations"
        }}
      ],
      "touchpoints": ["Key interactions or decision points"]
    }}
  ],
  "uiConsiderations": "Key UI design considerations",
  "uxPrinciples": "Core UX principles to follow"
}}

Create 2-3 personas and 3-5 critical user flows."#,
        description
    )
}

/// Generate technical architecture section
pub fn technical_prompt(description: &str) -> String {
    format!(
        r#"Based on this project description:

{}

Generate a technical architecture plan:

{{
  "components": [
    {{
      "name": "Component name",
      "purpose": "What this component does",
      "technology": "Suggested technology/framework"
    }}
  ],
  "dataModels": [
    {{
      "name": "Model name",
      "fields": [
        {{
          "name": "Field name",
          "type": "Data type",
          "required": true/false
        }}
      ]
    }}
  ],
  "apis": [
    {{
      "name": "API name",
      "purpose": "What this API does",
      "endpoints": ["List of endpoint patterns"]
    }}
  ],
  "infrastructure": {{
    "hosting": "Recommended hosting approach",
    "database": "Database technology and why",
    "caching": "Caching strategy if needed",
    "fileStorage": "File storage approach if needed"
  }},
  "techStackQuick": "One-line summary of the tech stack"
}}

Be practical and consider:
- Scalability needs
- Development complexity
- Maintenance requirements
- Cost efficiency"#,
        description
    )
}

/// Generate development roadmap (scope only, NO timelines)
pub fn roadmap_prompt(description: &str, features: &str) -> String {
    format!(
        r#"Based on this project description:

{}

And these features:
{}

Generate a development roadmap focused on SCOPE, not timelines:

{{
  "mvpScope": [
    "Feature or capability to include in MVP",
    "Another MVP feature"
  ],
  "futurePhases": [
    {{
      "name": "Phase name (e.g., 'Phase 2: Advanced Analytics')",
      "features": ["Features to add in this phase"],
      "goals": ["What this phase achieves"]
    }}
  ]
}}

The MVP should be the minimum viable version that provides real value.
Future phases should be logical extensions, not arbitrary chunks.
NO dates, NO timelines - focus on what to build and why."#,
        description, features
    )
}

/// Generate dependency chain section
pub fn dependencies_prompt(description: &str, features: &str) -> String {
    format!(
        r#"Based on this project description:

{}

And these features:
{}

Generate a logical dependency chain:

{{
  "foundationFeatures": [
    {{
      "id": "feature-id",
      "name": "Feature name",
      "rationale": "Why this must be built first",
      "blocks": ["feature-ids that depend on this"],
      "dependsOn": ["feature-ids this depends on"]
    }}
  ],
  "visibleFeatures": [
    {{
      "id": "feature-id",
      "name": "Feature name",
      "rationale": "Why this gives quick user value",
      "blocks": ["feature-ids that depend on this"],
      "dependsOn": ["feature-ids this depends on"]
    }}
  ],
  "enhancementFeatures": [
    {{
      "id": "feature-id",
      "name": "Feature name",
      "rationale": "Why this enhances the experience",
      "blocks": ["feature-ids that depend on this"],
      "dependsOn": ["feature-ids this depends on"]
    }}
  ],
  "dependencyGraph": {{
    "nodes": [
      {{
        "id": "feature-id",
        "label": "Feature name",
        "phase": 1-3
      }}
    ],
    "edges": [
      {{
        "from": "dependency-id",
        "to": "dependent-id"
      }}
    ]
  }}
}}

Focus on:
1. What MUST be built first (foundation)
2. What gets users to a usable state quickly (visible)
3. What enhances the experience (enhancement)

Create a clear path from "nothing" to "working product" to "polished product"."#,
        description, features
    )
}

/// Generate risks and mitigations section
pub fn risks_prompt(description: &str) -> String {
    format!(
        r#"Based on this project description:

{}

Identify potential risks and mitigation strategies:

{{
  "technicalRisks": [
    {{
      "description": "Technical risk",
      "severity": "low/medium/high/critical",
      "probability": "low/medium/high"
    }}
  ],
  "mvpScopingRisks": [
    {{
      "description": "Scope-related risk",
      "severity": "low/medium/high/critical",
      "probability": "low/medium/high"
    }}
  ],
  "resourceRisks": [
    {{
      "description": "Resource or capability risk",
      "severity": "low/medium/high/critical",
      "probability": "low/medium/high"
    }}
  ],
  "mitigations": [
    {{
      "risk": "Risk being mitigated",
      "strategy": "How to mitigate this risk",
      "owner": "Who should handle this (can be role)"
    }}
  ]
}}

Be realistic but not alarmist. Focus on actionable mitigations."#,
        description
    )
}

/// Generate research/appendix section
pub fn research_prompt(description: &str) -> String {
    format!(
        r#"Based on this project description:

{}

Generate research notes and references:

{{
  "competitors": [
    {{
      "name": "Competitor name",
      "url": "URL if known, or 'N/A'",
      "strengths": ["What they do well"],
      "gaps": ["Opportunities they're missing"],
      "features": ["Key features to be aware of"]
    }}
  ],
  "similarProjects": [
    {{
      "name": "Similar project name",
      "url": "URL or 'N/A'",
      "positiveAspects": ["What to learn from"],
      "negativeAspects": ["What to avoid"],
      "patternsToAdopt": ["Useful patterns or approaches"]
    }}
  ],
  "researchFindings": "Key research insights that informed this PRD",
  "technicalSpecs": "Technical specifications or standards to follow",
  "referenceLinks": [
    {{
      "title": "Reference title",
      "url": "URL",
      "notes": "Why this is relevant"
    }}
  ]
}}

Suggest 2-3 competitors and similar projects based on the description.
These can be real products or hypothetical ones in the same space."#,
        description
    )
}

/// Generate a complete PRD from a one-liner description
pub fn complete_prd_prompt(description: &str) -> String {
    format!(
        r#"Generate a complete, comprehensive PRD for this project:

{}

Provide a structured JSON response with ALL sections:

{{
  "overview": {{
    "problemStatement": "...",
    "targetAudience": "...",
    "valueProposition": "...",
    "oneLinePitch": "..."
  }},
  "features": [...],
  "ux": {{
    "personas": [...],
    "userFlows": [...],
    "uiConsiderations": "...",
    "uxPrinciples": "..."
  }},
  "technical": {{
    "components": [...],
    "dataModels": [...],
    "apis": [...],
    "infrastructure": {{...}},
    "techStackQuick": "..."
  }},
  "roadmap": {{
    "mvpScope": [...],
    "futurePhases": [...]
  }},
  "dependencies": {{
    "foundationFeatures": [
      {{
        "id": "feature-id",
        "name": "Feature name",
        "rationale": "Why this must be built first",
        "blocks": ["feature-ids that depend on this"],
        "dependsOn": ["feature-ids this depends on"]
      }}
    ],
    "visibleFeatures": [
      {{
        "id": "feature-id",
        "name": "Feature name",
        "rationale": "Why this gives quick user value",
        "blocks": ["feature-ids that depend on this"],
        "dependsOn": ["feature-ids this depends on"]
      }}
    ],
    "enhancementFeatures": [
      {{
        "id": "feature-id",
        "name": "Feature name",
        "rationale": "Why this enhances the experience",
        "blocks": ["feature-ids that depend on this"],
        "dependsOn": ["feature-ids this depends on"]
      }}
    ],
    "dependencyGraph": {{
      "nodes": [
        {{
          "id": "feature-id",
          "label": "Feature name",
          "phase": 1-3
        }}
      ],
      "edges": [
        {{
          "from": "dependency-id",
          "to": "dependent-id"
        }}
      ]
    }}
  }},
  "risks": {{
    "technicalRisks": [...],
    "mvpScopingRisks": [...],
    "resourceRisks": [...],
    "mitigations": [...]
  }},
  "research": {{
    "competitors": [...],
    "similarProjects": [...],
    "researchFindings": "...",
    "technicalSpecs": "...",
    "referenceLinks": [...]
  }}
}}

Make this PRD:
1. Comprehensive and detailed
2. Actionable and specific
3. Focused on MVP scope
4. Free of timeline commitments
5. Emphasizing logical build order"#,
        description
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overview_prompt_includes_description() {
        let desc = "A mobile app for tracking water intake";
        let prompt = overview_prompt(desc);
        assert!(prompt.contains(desc));
        assert!(prompt.contains("problemStatement"));
        assert!(prompt.contains("targetAudience"));
    }

    #[test]
    fn test_features_prompt_requests_json() {
        let prompt = features_prompt("Test project");
        assert!(prompt.contains("features"));
        assert!(prompt.contains("buildPhase"));
        assert!(prompt.contains("dependsOn"));
    }

    #[test]
    fn test_complete_prd_prompt_has_all_sections() {
        let prompt = complete_prd_prompt("Test");
        assert!(prompt.contains("overview"));
        assert!(prompt.contains("features"));
        assert!(prompt.contains("ux"));
        assert!(prompt.contains("technical"));
        assert!(prompt.contains("roadmap"));
        assert!(prompt.contains("dependencies"));
        assert!(prompt.contains("risks"));
        assert!(prompt.contains("research"));
    }
}
