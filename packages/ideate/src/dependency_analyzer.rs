// ABOUTME: Dependency management for feature relationships
// ABOUTME: Handles CRUD operations for technical, logical, and business dependencies

use crate::error::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use tracing::info;

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

/// Dependency analyzer for CRUD operations
pub struct DependencyAnalyzer {
    db: SqlitePool,
}

impl DependencyAnalyzer {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
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
            from_feature_id: input.from_feature_id.clone(),
            to_feature_id: input.to_feature_id.clone(),
            dependency_type: input.dependency_type,
            strength: input.strength,
            reason: input.reason.clone(),
            auto_detected: false,
        };

        let strength_str = match dependency.strength {
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
        .bind(&dependency.id)
        .bind(&dependency.session_id)
        .bind(&dependency.from_feature_id)
        .bind(&dependency.to_feature_id)
        .bind(dependency.dependency_type)
        .bind(strength_str)
        .bind(&dependency.reason)
        .bind(dependency.auto_detected)
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

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

