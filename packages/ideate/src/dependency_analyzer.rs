// ABOUTME: AI-powered dependency analysis for feature relationships
// ABOUTME: Detects technical, logical, and business dependencies between features

use crate::error::{IdeateError, Result};
use crate::types::IdeateFeature;
use ai::AIService;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashSet;
use tracing::{debug, info, warn};

/// Type of dependency relationship
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    /// Technical dependency (e.g., API before UI, auth before features)
    Technical,
    /// Logical dependency (e.g., data model before CRUD)
    Logical,
    /// Business dependency (e.g., MVP features before enhancements)
    Business,
}

/// Strength of dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DependencyStrength {
    /// Must be built first
    Required,
    /// Should be built first for best results
    Recommended,
    /// Can be built first but not necessary
    Optional,
}

/// Feature dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureDependency {
    pub id: String,
    pub session_id: String,
    pub from_feature_id: String,
    pub to_feature_id: String,
    pub dependency_type: DependencyType,
    pub strength: DependencyStrength,
    pub reason: Option<String>,
    pub auto_detected: bool,
}

/// Result of dependency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub session_id: String,
    pub dependencies: Vec<FeatureDependency>,
    pub confidence_score: f32,
    pub model_version: String,
    pub analyzed_at: chrono::DateTime<Utc>,
}

/// Input for manual dependency creation
#[derive(Debug, Clone, Deserialize)]
pub struct CreateDependencyInput {
    pub from_feature_id: String,
    pub to_feature_id: String,
    pub dependency_type: DependencyType,
    pub strength: DependencyStrength,
    pub reason: Option<String>,
}

/// Dependency analyzer with AI integration
pub struct DependencyAnalyzer {
    db: SqlitePool,
    ai_service: AIService,
}

impl DependencyAnalyzer {
    pub fn new(db: SqlitePool, ai_service: AIService) -> Self {
        Self { db, ai_service }
    }

    /// Analyze features and auto-detect dependencies using AI
    pub async fn analyze_dependencies(&self, session_id: &str) -> Result<DependencyAnalysis> {
        info!("Analyzing dependencies for session: {}", session_id);

        // Get all features for this session
        let features = self.get_session_features(session_id).await?;

        if features.is_empty() {
            return Ok(DependencyAnalysis {
                session_id: session_id.to_string(),
                dependencies: vec![],
                confidence_score: 1.0,
                model_version: "none".to_string(),
                analyzed_at: Utc::now(),
            });
        }

        // Check cache first
        let features_hash = self.compute_features_hash(&features);
        if let Some(cached) = self.get_cached_analysis(session_id, &features_hash).await? {
            debug!("Using cached dependency analysis");
            return Ok(cached);
        }

        // Prepare prompt for AI analysis
        let prompt = self.build_analysis_prompt(&features);

        // Call AI service
        #[derive(Deserialize)]
        struct RawResponse {
            response: String,
        }

        let ai_response = self
            .ai_service
            .generate_structured::<RawResponse>(
                prompt,
                Some(DEPENDENCY_ANALYSIS_SYSTEM_PROMPT.to_string()),
            )
            .await
            .map_err(|e| {
                IdeateError::AIService(format!("Failed to analyze dependencies: {}", e))
            })?;

        let response = &ai_response.data.response;

        // Parse AI response
        let parsed = self.parse_dependency_response(&response, &features, session_id)?;

        // Store dependencies in database
        for dep in &parsed.dependencies {
            self.store_dependency(dep).await?;
        }

        // Cache the analysis
        self.cache_analysis(session_id, &features_hash, &parsed)
            .await?;

        info!(
            "Dependency analysis complete: {} dependencies found",
            parsed.dependencies.len()
        );

        Ok(parsed)
    }

    /// Get all features for a session
    async fn get_session_features(&self, session_id: &str) -> Result<Vec<IdeateFeature>> {
        let rows = sqlx::query(
            "SELECT id, session_id, feature_name, what_it_does, why_important, how_it_works,
                    depends_on, enables, build_phase, is_visible, created_at
             FROM ideate_features
             WHERE session_id = $1
             ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await?;

        let features = rows
            .into_iter()
            .map(|row| IdeateFeature {
                id: row.get("id"),
                session_id: row.get("session_id"),
                feature_name: row.get("feature_name"),
                what_it_does: row.get("what_it_does"),
                why_important: row.get("why_important"),
                how_it_works: row.get("how_it_works"),
                depends_on: row
                    .get::<Option<String>, _>("depends_on")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                enables: row
                    .get::<Option<String>, _>("enables")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                build_phase: row.get("build_phase"),
                is_visible: row.get::<i32, _>("is_visible") != 0,
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(features)
    }

    /// Build prompt for AI dependency analysis
    fn build_analysis_prompt(&self, features: &[IdeateFeature]) -> String {
        let mut prompt =
            String::from("Analyze the following features and identify dependencies:\n\n");

        for (idx, feature) in features.iter().enumerate() {
            prompt.push_str(&format!("Feature {}: {}\n", idx + 1, feature.feature_name));
            if let Some(what) = &feature.what_it_does {
                prompt.push_str(&format!("  What: {}\n", what));
            }
            if let Some(why) = &feature.why_important {
                prompt.push_str(&format!("  Why: {}\n", why));
            }
            if let Some(how) = &feature.how_it_works {
                prompt.push_str(&format!("  How: {}\n", how));
            }
            prompt.push_str(&format!("  ID: {}\n\n", feature.id));
        }

        prompt.push_str("\nFor each feature, identify which other features it depends on. ");
        prompt.push_str("Respond in JSON format with this structure:\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"dependencies\": [\n");
        prompt.push_str("    {\n");
        prompt.push_str("      \"from_feature_id\": \"<feature that depends>\",\n");
        prompt.push_str("      \"to_feature_id\": \"<feature depended upon>\",\n");
        prompt.push_str("      \"type\": \"technical|logical|business\",\n");
        prompt.push_str("      \"strength\": \"required|recommended|optional\",\n");
        prompt.push_str("      \"reason\": \"explanation\"\n");
        prompt.push_str("    }\n");
        prompt.push_str("  ],\n");
        prompt.push_str("  \"confidence\": 0.0-1.0\n");
        prompt.push_str("}\n");

        prompt
    }

    /// Parse AI response into dependency analysis
    fn parse_dependency_response(
        &self,
        response: &str,
        features: &[IdeateFeature],
        session_id: &str,
    ) -> Result<DependencyAnalysis> {
        // Extract JSON from response (handle markdown code blocks)
        let json_str = if response.contains("```json") {
            response
                .split("```json")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else if response.contains("```") {
            response
                .split("```")
                .nth(1)
                .and_then(|s| s.split("```").next())
                .unwrap_or(response)
                .trim()
        } else {
            response.trim()
        };

        #[derive(Deserialize)]
        struct AIResponse {
            dependencies: Vec<AIDependency>,
            confidence: Option<f32>,
        }

        #[derive(Deserialize)]
        struct AIDependency {
            from_feature_id: String,
            to_feature_id: String,
            #[serde(rename = "type")]
            dep_type: String,
            strength: String,
            reason: Option<String>,
        }

        let ai_response: AIResponse = serde_json::from_str(json_str).map_err(|e| {
            warn!("Failed to parse AI response: {}", e);
            warn!("Response was: {}", response);
            IdeateError::AIService(format!("Invalid JSON response: {}", e))
        })?;

        // Build feature ID map for validation
        let feature_ids: HashSet<String> = features.iter().map(|f| f.id.clone()).collect();

        let dependencies: Vec<FeatureDependency> = ai_response
            .dependencies
            .into_iter()
            .filter_map(|dep| {
                // Validate feature IDs
                if !feature_ids.contains(&dep.from_feature_id) {
                    warn!("Invalid from_feature_id: {}", dep.from_feature_id);
                    return None;
                }
                if !feature_ids.contains(&dep.to_feature_id) {
                    warn!("Invalid to_feature_id: {}", dep.to_feature_id);
                    return None;
                }

                // Parse dependency type
                let dep_type = match dep.dep_type.to_lowercase().as_str() {
                    "technical" => DependencyType::Technical,
                    "logical" => DependencyType::Logical,
                    "business" => DependencyType::Business,
                    _ => {
                        warn!("Invalid dependency type: {}", dep.dep_type);
                        return None;
                    }
                };

                // Parse strength
                let strength = match dep.strength.to_lowercase().as_str() {
                    "required" => DependencyStrength::Required,
                    "recommended" => DependencyStrength::Recommended,
                    "optional" => DependencyStrength::Optional,
                    _ => {
                        warn!("Invalid dependency strength: {}", dep.strength);
                        return None;
                    }
                };

                Some(FeatureDependency {
                    id: nanoid::nanoid!(8),
                    session_id: session_id.to_string(),
                    from_feature_id: dep.from_feature_id,
                    to_feature_id: dep.to_feature_id,
                    dependency_type: dep_type,
                    strength,
                    reason: dep.reason,
                    auto_detected: true,
                })
            })
            .collect();

        Ok(DependencyAnalysis {
            session_id: session_id.to_string(),
            dependencies,
            confidence_score: ai_response.confidence.unwrap_or(0.8),
            model_version: "claude-3-opus".to_string(),
            analyzed_at: Utc::now(),
        })
    }

    /// Store dependency in database
    async fn store_dependency(&self, dep: &FeatureDependency) -> Result<()> {
        let strength_str = match dep.strength {
            DependencyStrength::Required => "required",
            DependencyStrength::Recommended => "recommended",
            DependencyStrength::Optional => "optional",
        };

        sqlx::query(
            "INSERT INTO feature_dependencies
             (id, session_id, from_feature_id, to_feature_id, dependency_type, strength, reason, auto_detected, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (from_feature_id, to_feature_id) DO NOTHING",
        )
        .bind(&dep.id)
        .bind(&dep.session_id)
        .bind(&dep.from_feature_id)
        .bind(&dep.to_feature_id)
        .bind(&dep.dependency_type)
        .bind(strength_str)
        .bind(&dep.reason)
        .bind(dep.auto_detected)
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Compute hash of features for caching
    fn compute_features_hash(&self, features: &[IdeateFeature]) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        for feature in features {
            feature.id.hash(&mut hasher);
            feature.feature_name.hash(&mut hasher);
            feature.what_it_does.hash(&mut hasher);
        }
        format!("{:x}", hasher.finish())
    }

    /// Get cached analysis if available
    async fn get_cached_analysis(
        &self,
        session_id: &str,
        features_hash: &str,
    ) -> Result<Option<DependencyAnalysis>> {
        let row = sqlx::query(
            "SELECT analysis_result, confidence_score, model_version, analyzed_at
             FROM dependency_analysis_cache
             WHERE session_id = $1 AND features_hash = $2 AND analysis_type = 'dependencies'
             AND (expires_at IS NULL OR expires_at > datetime('now', 'utc'))
             ORDER BY analyzed_at DESC
             LIMIT 1",
        )
        .bind(session_id)
        .bind(features_hash)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = row {
            let result_json: String = row.get("analysis_result");
            let dependencies: Vec<FeatureDependency> = serde_json::from_str(&result_json)?;

            return Ok(Some(DependencyAnalysis {
                session_id: session_id.to_string(),
                dependencies,
                confidence_score: row.get("confidence_score"),
                model_version: row.get("model_version"),
                analyzed_at: row.get("analyzed_at"),
            }));
        }

        Ok(None)
    }

    /// Cache analysis results
    async fn cache_analysis(
        &self,
        session_id: &str,
        features_hash: &str,
        analysis: &DependencyAnalysis,
    ) -> Result<()> {
        let id = nanoid::nanoid!(8);
        let result_json = serde_json::to_string(&analysis.dependencies)?;

        // Cache for 1 hour
        let expires_at = Utc::now() + chrono::Duration::hours(1);

        sqlx::query(
            "INSERT INTO dependency_analysis_cache
             (id, session_id, features_hash, analysis_type, analysis_result, confidence_score, model_version, analyzed_at, expires_at, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(&id)
        .bind(session_id)
        .bind(features_hash)
        .bind("dependencies")
        .bind(&result_json)
        .bind(analysis.confidence_score)
        .bind(&analysis.model_version)
        .bind(analysis.analyzed_at)
        .bind(expires_at)
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get all dependencies for a session
    pub async fn get_dependencies(&self, session_id: &str) -> Result<Vec<FeatureDependency>> {
        let rows = sqlx::query(
            "SELECT id, session_id, from_feature_id, to_feature_id, dependency_type, strength, reason, auto_detected
             FROM feature_dependencies
             WHERE session_id = $1
             ORDER BY created_at ASC",
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await?;

        let dependencies = rows
            .into_iter()
            .map(|row| {
                let strength_str: String = row.get("strength");
                let strength = match strength_str.as_str() {
                    "required" => DependencyStrength::Required,
                    "recommended" => DependencyStrength::Recommended,
                    "optional" => DependencyStrength::Optional,
                    _ => DependencyStrength::Required,
                };

                FeatureDependency {
                    id: row.get("id"),
                    session_id: row.get("session_id"),
                    from_feature_id: row.get("from_feature_id"),
                    to_feature_id: row.get("to_feature_id"),
                    dependency_type: row.get("dependency_type"),
                    strength,
                    reason: row.get("reason"),
                    auto_detected: row.get::<i32, _>("auto_detected") != 0,
                }
            })
            .collect();

        Ok(dependencies)
    }

    /// Create a manual dependency
    pub async fn create_dependency(
        &self,
        session_id: &str,
        input: CreateDependencyInput,
    ) -> Result<FeatureDependency> {
        let id = nanoid::nanoid!(8);

        let dependency = FeatureDependency {
            id: id.clone(),
            session_id: session_id.to_string(),
            from_feature_id: input.from_feature_id,
            to_feature_id: input.to_feature_id,
            dependency_type: input.dependency_type,
            strength: input.strength,
            reason: input.reason,
            auto_detected: false,
        };

        self.store_dependency(&dependency).await?;

        Ok(dependency)
    }

    /// Delete a dependency
    pub async fn delete_dependency(&self, dependency_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM feature_dependencies WHERE id = $1")
            .bind(dependency_id)
            .execute(&self.db)
            .await?;

        Ok(())
    }
}

const DEPENDENCY_ANALYSIS_SYSTEM_PROMPT: &str = r#"You are an expert software architect analyzing feature dependencies for a software project.

Your task is to identify dependencies between features. A dependency exists when one feature requires another feature to be built first.

Types of dependencies:
- Technical: One feature technically requires another (e.g., API before UI, authentication before protected features)
- Logical: One feature logically builds upon another (e.g., data model before CRUD operations, user management before user profiles)
- Business: Business logic dictates the order (e.g., MVP features before enhancements, core features before nice-to-haves)

Dependency strength:
- Required: Feature absolutely cannot be built without the dependency
- Recommended: Feature can technically be built but should wait for the dependency
- Optional: Feature would benefit from the dependency but can proceed independently

Be conservative - only identify clear dependencies. Avoid creating circular dependencies."#;
