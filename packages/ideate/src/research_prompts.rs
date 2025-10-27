// ABOUTME: AI prompts for competitor analysis and research tools
// ABOUTME: Structured prompts for extracting features, patterns, and insights from competitors

/// System prompt for all research analysis tasks
pub const RESEARCH_SYSTEM_PROMPT: &str = r#"You are an expert product researcher and competitive analyst.

Your role is to:
- Analyze competitor products and extract key features
- Identify strengths, weaknesses, and gaps in competitor offerings
- Extract UI/UX patterns and design decisions
- Provide actionable insights for product differentiation
- Focus on facts and observable patterns, not speculation

Always respond in valid JSON format matching the requested structure."#;

/// Analyze competitor from scraped HTML content
pub fn competitor_analysis_prompt(
    project_description: &str,
    html_content: &str,
    url: &str,
) -> String {
    format!(
        r#"You are analyzing a competitor for this project:

Project Description:
{}

Competitor URL: {}

Scraped HTML Content (truncated to first 10000 chars):
{}

Based on the content, extract competitor information in this JSON format:

{{
  "name": "Competitor name (extract from content or infer)",
  "url": "{}",
  "strengths": ["List of 3-5 key strengths or standout features"],
  "gaps": ["List of 3-5 areas where they could improve"],
  "features": ["List of 5-10 main features or capabilities"]
}}

Focus on:
1. Core functionality and features
2. Value propositions mentioned
3. User experience observations
4. Technical capabilities evident from the page
5. Gaps that the user's project could fill

Be specific and actionable."#,
        project_description,
        url,
        &html_content.chars().take(10000).collect::<String>(),
        url
    )
}

/// Extract features from HTML content
pub fn feature_extraction_prompt(html_content: &str) -> String {
    format!(
        r#"Extract the main features and capabilities from this webpage content:

{}

Return JSON in this format:

{{
  "features": ["List of 5-15 distinct features or capabilities mentioned"]
}}

Look for:
- Feature lists or sections
- Capability descriptions
- Benefits or value propositions
- Technical specifications
- User-facing functionality

Be concise and specific."#,
        &html_content.chars().take(8000).collect::<String>()
    )
}

/// Extract UI/UX patterns from content and structure
pub fn ui_pattern_prompt(project_description: &str, html_structure: &str) -> String {
    format!(
        r#"You are analyzing UI/UX patterns for this project:

{}

HTML Structure Summary:
{}

Extract observable UI/UX patterns in this JSON format:

{{
  "patterns": [
    {{
      "type": "layout|navigation|interaction|visual|content",
      "name": "Pattern name",
      "description": "What this pattern does",
      "benefits": "Why this pattern might be effective",
      "adoptionNotes": "How this could be adapted for our project"
    }}
  ]
}}

Focus on:
1. Layout and information architecture
2. Navigation patterns
3. Interactive elements
4. Visual design choices
5. Content organization
6. User flow patterns

Identify 5-8 notable patterns."#,
        project_description,
        &html_structure.chars().take(8000).collect::<String>()
    )
}

/// Compare features across competitors
pub fn gap_analysis_prompt(
    project_description: &str,
    your_features: &[String],
    competitor_features: &[&(String, Vec<String>)],
) -> String {
    let competitor_list = competitor_features
        .iter()
        .map(|(name, features)| format!("{}: {}", name, features.join(", ")))
        .collect::<Vec<_>>()
        .join("\n");

    let your_features_str = your_features.join(", ");

    format!(
        r#"Perform a competitive gap analysis for this project:

Project Description:
{}

Your Planned Features:
{}

Competitor Features:
{}

Identify opportunities in this JSON format:

{{
  "opportunities": [
    {{
      "type": "differentiation|improvement|gap",
      "title": "Opportunity title",
      "description": "What makes this a valuable opportunity",
      "competitorContext": "What competitors are doing (or not doing)",
      "recommendation": "How to capitalize on this opportunity"
    }}
  ],
  "summary": "2-3 sentence strategic summary"
}}

Look for:
1. Features competitors have that you're missing (gaps)
2. Features you have that competitors lack (differentiation)
3. Common patterns to improve upon (improvements)
4. Unmet user needs evident from competitor limitations

Identify 4-6 key opportunities."#,
        project_description, your_features_str, competitor_list
    )
}

/// Extract lessons from similar projects
pub fn lessons_learned_prompt(
    project_description: &str,
    similar_project_name: &str,
    positive_aspects: &[String],
    negative_aspects: &[String],
) -> String {
    format!(
        r#"Extract actionable lessons from a similar project for this project:

Your Project:
{}

Similar Project: {}

Positive Aspects Observed:
{}

Negative Aspects Observed:
{}

Generate insights in this JSON format:

{{
  "lessons": [
    {{
      "category": "design|implementation|feature|ux|technical",
      "insight": "What we learned",
      "application": "How to apply this to our project",
      "priority": "high|medium|low"
    }}
  ]
}}

Focus on:
1. What to adopt from their successes
2. What to avoid from their mistakes
3. How to adapt their patterns for your context
4. Priority of implementation

Identify 4-6 actionable lessons."#,
        project_description,
        similar_project_name,
        positive_aspects.join("\n- "),
        negative_aspects.join("\n- ")
    )
}

/// Synthesize research findings
pub fn research_synthesis_prompt(
    project_description: &str,
    competitors: &[(String, Vec<String>, Vec<String>)], // (name, strengths, gaps)
    similar_projects_count: usize,
) -> String {
    let competitor_summary = competitors
        .iter()
        .map(|(name, strengths, gaps)| {
            format!(
                "{}: Strengths: {}. Gaps: {}",
                name,
                strengths.join(", "),
                gaps.join(", ")
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"Synthesize research findings for this project:

Project Description:
{}

Analyzed Competitors:
{}

Similar Projects Reviewed: {}

Generate a research summary in this JSON format:

{{
  "keyFindings": [
    "3-5 most important discoveries from research"
  ],
  "marketPosition": "How this project fits in the competitive landscape",
  "differentiators": [
    "3-5 ways this project can stand out"
  ],
  "risks": [
    "3-4 competitive or market risks to be aware of"
  ],
  "recommendations": [
    "4-6 actionable recommendations based on research"
  ]
}}

Provide strategic, actionable insights."#,
        project_description, competitor_summary, similar_projects_count
    )
}
