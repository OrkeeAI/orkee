use serde::{Deserialize, Serialize};

/// Cloud subscription tiers
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CloudTier {
    Free,
    Starter,
    Pro,
    Enterprise,
}

impl CloudTier {
    /// Get display name for the tier
    pub fn display_name(&self) -> &str {
        match self {
            CloudTier::Free => "Free",
            CloudTier::Starter => "Starter",
            CloudTier::Pro => "Pro",
            CloudTier::Enterprise => "Enterprise",
        }
    }

    /// Get the price for the tier
    pub fn price(&self) -> &str {
        match self {
            CloudTier::Free => "$0",
            CloudTier::Starter => "$9",
            CloudTier::Pro => "$29",
            CloudTier::Enterprise => "Custom",
        }
    }

    /// Check if this is a paid tier
    pub fn is_paid(&self) -> bool {
        !matches!(self, CloudTier::Free)
    }
}

impl std::fmt::Display for CloudTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Cloud subscription information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSubscription {
    pub tier: CloudTier,
    pub project_limit: i32,
    pub storage_limit_mb: i32,
    pub auto_sync_enabled: bool,
    pub realtime_enabled: bool,
    pub collaboration_enabled: bool,
}

impl CloudSubscription {
    /// Create a free tier subscription
    pub fn free() -> Self {
        Self {
            tier: CloudTier::Free,
            project_limit: 2,
            storage_limit_mb: 100,
            auto_sync_enabled: false,
            realtime_enabled: false,
            collaboration_enabled: false,
        }
    }

    /// Create a starter tier subscription
    pub fn starter() -> Self {
        Self {
            tier: CloudTier::Starter,
            project_limit: 10,
            storage_limit_mb: 5120, // 5GB
            auto_sync_enabled: true,
            realtime_enabled: false,
            collaboration_enabled: false,
        }
    }

    /// Create a pro tier subscription
    pub fn pro() -> Self {
        Self {
            tier: CloudTier::Pro,
            project_limit: -1, // Unlimited
            storage_limit_mb: 51200, // 50GB
            auto_sync_enabled: true,
            realtime_enabled: true,
            collaboration_enabled: true,
        }
    }

    /// Create an enterprise tier subscription
    pub fn enterprise() -> Self {
        Self {
            tier: CloudTier::Enterprise,
            project_limit: -1, // Unlimited
            storage_limit_mb: -1, // Unlimited
            auto_sync_enabled: true,
            realtime_enabled: true,
            collaboration_enabled: true,
        }
    }

    /// Check if a feature is available for this subscription
    pub fn has_feature(&self, feature: CloudFeature) -> bool {
        match feature {
            CloudFeature::AutoSync => self.auto_sync_enabled,
            CloudFeature::Realtime => self.realtime_enabled,
            CloudFeature::Collaboration => self.collaboration_enabled,
            CloudFeature::UnlimitedProjects => self.project_limit < 0,
            CloudFeature::UnlimitedStorage => self.storage_limit_mb < 0,
        }
    }

    /// Get a human-readable description of limits
    pub fn describe_limits(&self) -> String {
        let projects = if self.project_limit < 0 {
            "Unlimited projects".to_string()
        } else {
            format!("{} projects", self.project_limit)
        };

        let storage = if self.storage_limit_mb < 0 {
            "Unlimited storage".to_string()
        } else if self.storage_limit_mb >= 1024 {
            format!("{}GB storage", self.storage_limit_mb / 1024)
        } else {
            format!("{}MB storage", self.storage_limit_mb)
        };

        format!("{}, {}", projects, storage)
    }
}

/// Cloud features that can be checked
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudFeature {
    AutoSync,
    Realtime,
    Collaboration,
    UnlimitedProjects,
    UnlimitedStorage,
}

impl CloudFeature {
    /// Get the minimum tier required for this feature
    pub fn required_tier(&self) -> CloudTier {
        match self {
            CloudFeature::AutoSync => CloudTier::Starter,
            CloudFeature::Realtime => CloudTier::Pro,
            CloudFeature::Collaboration => CloudTier::Pro,
            CloudFeature::UnlimitedProjects => CloudTier::Pro,
            CloudFeature::UnlimitedStorage => CloudTier::Enterprise,
        }
    }

    /// Get a user-friendly description of the feature
    pub fn description(&self) -> &str {
        match self {
            CloudFeature::AutoSync => "Automatic cloud synchronization",
            CloudFeature::Realtime => "Real-time sync across devices",
            CloudFeature::Collaboration => "Team collaboration features",
            CloudFeature::UnlimitedProjects => "Unlimited project storage",
            CloudFeature::UnlimitedStorage => "Unlimited cloud storage",
        }
    }
}

/// Subscription status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionStatus {
    pub tier: CloudTier,
    pub current_period_start: Option<chrono::DateTime<chrono::Utc>>,
    pub current_period_end: Option<chrono::DateTime<chrono::Utc>>,
    pub cancel_at_period_end: bool,
    pub trial_ends_at: Option<chrono::DateTime<chrono::Utc>>,
    pub projects_used: usize,
    pub storage_used_mb: i32,
}

impl SubscriptionStatus {
    /// Check if currently in trial period
    pub fn is_trial(&self) -> bool {
        if let Some(trial_ends) = self.trial_ends_at {
            chrono::Utc::now() < trial_ends
        } else {
            false
        }
    }

    /// Check if subscription is active
    pub fn is_active(&self) -> bool {
        if let Some(period_end) = self.current_period_end {
            chrono::Utc::now() < period_end
        } else {
            self.tier == CloudTier::Free // Free tier is always active
        }
    }

    /// Days remaining in current period
    pub fn days_remaining(&self) -> Option<i64> {
        self.current_period_end.map(|end| {
            (end - chrono::Utc::now()).num_days().max(0)
        })
    }
}