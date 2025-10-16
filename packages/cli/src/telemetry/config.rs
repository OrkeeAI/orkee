// ABOUTME: Telemetry configuration and settings management
// ABOUTME: Handles user preferences for opt-in telemetry with privacy controls

use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use sqlx::SqlitePool;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetrySettings {
    pub first_run: bool,
    pub onboarding_completed: bool,
    pub error_reporting: bool,
    pub usage_metrics: bool,
    pub non_anonymous_metrics: bool,
    pub machine_id: Option<String>,
    pub user_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Default for TelemetrySettings {
    fn default() -> Self {
        Self {
            first_run: true,
            onboarding_completed: false,
            error_reporting: false,
            usage_metrics: false,
            non_anonymous_metrics: false,
            machine_id: None,
            user_id: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    pub enabled: bool,
    pub endpoint: String,
    pub debug_mode: bool,
    pub batch_size: usize,
    pub flush_interval_secs: u64,
}

impl TelemetryConfig {
    pub fn from_env() -> Self {
        // Check if PostHog API key is available
        // If no key, disable telemetry gracefully
        let has_api_key = super::posthog::get_posthog_api_key().is_some();

        // Check if telemetry is globally disabled via environment variable
        let env_enabled = env::var("ORKEE_TELEMETRY_ENABLED")
            .unwrap_or_else(|_| "true".to_string())
            .parse::<bool>()
            .unwrap_or(true);

        // Telemetry is only enabled if BOTH the env var is true AND we have an API key
        let enabled = env_enabled && has_api_key;

        // Hardcoded endpoint - users shouldn't be able to change this
        // Using PostHog for privacy-focused analytics
        #[cfg(debug_assertions)]
        let endpoint = env::var("ORKEE_TELEMETRY_ENDPOINT")
            .unwrap_or_else(|_| "https://app.posthog.com/capture".to_string());

        #[cfg(not(debug_assertions))]
        let endpoint = "https://app.posthog.com/capture".to_string();

        let debug_mode = env::var("ORKEE_TELEMETRY_DEBUG")
            .unwrap_or_else(|_| "false".to_string())
            .parse::<bool>()
            .unwrap_or(false);

        Self {
            enabled,
            endpoint,
            debug_mode,
            batch_size: 50,
            flush_interval_secs: 300, // 5 minutes
        }
    }
}

pub struct TelemetryManager {
    settings: Arc<RwLock<TelemetrySettings>>,
    config: TelemetryConfig,
    pool: SqlitePool,
}

impl TelemetryManager {
    pub async fn new(pool: SqlitePool) -> Result<Self, Box<dyn std::error::Error>> {
        let config = TelemetryConfig::from_env();
        let settings = Self::load_settings(&pool).await?;

        Ok(Self {
            settings: Arc::new(RwLock::new(settings)),
            config,
            pool,
        })
    }

    async fn load_settings(pool: &SqlitePool) -> Result<TelemetrySettings, Box<dyn std::error::Error>> {
        let row = sqlx::query!(
            r#"
            SELECT
                first_run,
                onboarding_completed,
                error_reporting,
                usage_metrics,
                non_anonymous_metrics,
                machine_id,
                user_id,
                created_at,
                updated_at
            FROM telemetry_settings
            WHERE id = 1
            "#
        )
        .fetch_optional(pool)
        .await?;

        if let Some(row) = row {
            Ok(TelemetrySettings {
                first_run: row.first_run.unwrap_or(true),
                onboarding_completed: row.onboarding_completed.unwrap_or(false),
                error_reporting: row.error_reporting.unwrap_or(false),
                usage_metrics: row.usage_metrics.unwrap_or(false),
                non_anonymous_metrics: row.non_anonymous_metrics.unwrap_or(false),
                machine_id: row.machine_id,
                user_id: row.user_id,
                created_at: DateTime::parse_from_rfc3339(&row.created_at)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
                updated_at: DateTime::parse_from_rfc3339(&row.updated_at)
                    .unwrap_or_else(|_| Utc::now().into())
                    .with_timezone(&Utc),
            })
        } else {
            // First run - create default settings
            let settings = TelemetrySettings::default();
            Self::save_settings(pool, &settings).await?;
            Ok(settings)
        }
    }

    async fn save_settings(pool: &SqlitePool, settings: &TelemetrySettings) -> Result<(), Box<dyn std::error::Error>> {
        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO telemetry_settings (
                id,
                first_run,
                onboarding_completed,
                error_reporting,
                usage_metrics,
                non_anonymous_metrics,
                machine_id,
                user_id,
                updated_at
            ) VALUES (1, ?, ?, ?, ?, ?, ?, ?, datetime('now'))
            "#,
            settings.first_run,
            settings.onboarding_completed,
            settings.error_reporting,
            settings.usage_metrics,
            settings.non_anonymous_metrics,
            settings.machine_id,
            settings.user_id,
        )
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn get_settings(&self) -> TelemetrySettings {
        self.settings.read().await.clone()
    }

    pub async fn update_settings(&self, new_settings: TelemetrySettings) -> Result<(), Box<dyn std::error::Error>> {
        // Save to database
        Self::save_settings(&self.pool, &new_settings).await?;

        // Update in-memory cache
        let mut settings = self.settings.write().await;
        *settings = new_settings;

        Ok(())
    }

    pub async fn complete_onboarding(
        &self,
        error_reporting: bool,
        usage_metrics: bool,
        non_anonymous_metrics: bool,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut settings = self.settings.write().await;

        // Generate machine ID if enabling any telemetry
        if (error_reporting || usage_metrics) && settings.machine_id.is_none() {
            settings.machine_id = Some(Self::generate_machine_id());
        }

        settings.first_run = false;
        settings.onboarding_completed = true;
        settings.error_reporting = error_reporting;
        settings.usage_metrics = usage_metrics;
        settings.non_anonymous_metrics = non_anonymous_metrics;
        settings.updated_at = Utc::now();

        Self::save_settings(&self.pool, &settings).await?;

        Ok(())
    }

    fn generate_machine_id() -> String {
        Uuid::new_v4().to_string()
    }

    pub fn is_telemetry_enabled(&self) -> bool {
        self.config.enabled
    }

    pub async fn is_any_telemetry_enabled(&self) -> bool {
        if !self.config.enabled {
            return false;
        }

        let settings = self.settings.read().await;
        settings.error_reporting || settings.usage_metrics || settings.non_anonymous_metrics
    }

    pub async fn should_show_onboarding(&self) -> bool {
        let settings = self.settings.read().await;
        settings.first_run && !settings.onboarding_completed
    }
}