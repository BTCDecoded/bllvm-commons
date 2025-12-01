//! Governance-Controlled Configuration Registry
//!
//! Tracks all configurable parameters that require governance approval (Tier 5) to change.
//! This includes feature flags, thresholds, time windows, limits, and other "dials" in the system.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info, warn};
use crate::error::GovernanceError;
use crate::governance::yaml_loader::YamlConfigLoader;

/// Configuration category for organizing configurable parameters
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigCategory {
    FeatureFlags,
    Thresholds,
    TimeWindows,
    Limits,
    Network,
    Security,
    Other,
}

impl ConfigCategory {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigCategory::FeatureFlags => "feature_flags",
            ConfigCategory::Thresholds => "thresholds",
            ConfigCategory::TimeWindows => "time_windows",
            ConfigCategory::Limits => "limits",
            ConfigCategory::Network => "network",
            ConfigCategory::Security => "security",
            ConfigCategory::Other => "other",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "feature_flags" => Some(ConfigCategory::FeatureFlags),
            "thresholds" => Some(ConfigCategory::Thresholds),
            "time_windows" => Some(ConfigCategory::TimeWindows),
            "limits" => Some(ConfigCategory::Limits),
            "network" => Some(ConfigCategory::Network),
            "security" => Some(ConfigCategory::Security),
            "other" => Some(ConfigCategory::Other),
            _ => None,
        }
    }
}

/// Status of a configuration change proposal
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigChangeStatus {
    Pending,
    Approved,
    Rejected,
    Activated,
    Cancelled,
}

impl ConfigChangeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ConfigChangeStatus::Pending => "pending",
            ConfigChangeStatus::Approved => "approved",
            ConfigChangeStatus::Rejected => "rejected",
            ConfigChangeStatus::Activated => "activated",
            ConfigChangeStatus::Cancelled => "cancelled",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(ConfigChangeStatus::Pending),
            "approved" => Some(ConfigChangeStatus::Approved),
            "rejected" => Some(ConfigChangeStatus::Rejected),
            "activated" => Some(ConfigChangeStatus::Activated),
            "cancelled" => Some(ConfigChangeStatus::Cancelled),
            _ => None,
        }
    }
}

/// Configuration registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigEntry {
    pub id: i32,
    pub config_key: String,
    pub config_category: ConfigCategory,
    pub current_value: serde_json::Value,
    pub default_value: serde_json::Value,
    pub description: Option<String>,
    pub tier_requirement: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub notes: Option<String>,
}

/// Configuration change proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChange {
    pub id: i32,
    pub config_key: String,
    pub proposed_value: serde_json::Value,
    pub current_value: serde_json::Value,
    pub change_reason: Option<String>,
    pub proposed_by: String,
    pub proposed_at: DateTime<Utc>,
    pub tier_requirement: i32,
    pub status: ConfigChangeStatus,
    pub approval_pr_id: Option<i32>,
    pub approved_at: Option<DateTime<Utc>>,
    pub approved_by: Option<String>,
    pub activated_at: Option<DateTime<Utc>>,
    pub activation_pr_id: Option<i32>,
    pub notes: Option<String>,
}

/// Governance-controlled configuration registry manager
#[derive(Clone)]
pub struct ConfigRegistry {
    pool: SqlitePool,
    /// Optional ConfigReader for automatic cache invalidation
    config_reader: Option<Arc<crate::governance::config_reader::ConfigReader>>,
    /// Optional path to governance config directory (for YAML sync)
    config_path: Option<PathBuf>,
}

impl ConfigRegistry {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            config_reader: None,
            config_path: None,
        }
    }

    /// Set the governance config path for YAML sync
    pub fn set_config_path(&mut self, config_path: PathBuf) {
        self.config_path = Some(config_path);
    }

    /// Create with ConfigReader for automatic cache invalidation
    pub fn with_config_reader(
        pool: SqlitePool,
        config_reader: std::sync::Arc<crate::governance::config_reader::ConfigReader>,
    ) -> Self {
        Self {
            pool,
            config_reader: Some(config_reader),
            config_path: None,
        }
    }

    /// Set ConfigReader for cache invalidation (can be called after creation)
    pub fn set_config_reader(
        &mut self,
        config_reader: std::sync::Arc<crate::governance::config_reader::ConfigReader>,
    ) {
        self.config_reader = Some(config_reader);
    }

    /// Register a new configurable parameter
    pub async fn register_config(
        &self,
        config_key: &str,
        category: ConfigCategory,
        default_value: serde_json::Value,
        description: Option<&str>,
        tier_requirement: i32,
        created_by: Option<&str>,
    ) -> Result<i32, GovernanceError> {
        let now = Utc::now().timestamp();

        let id = sqlx::query(
            r#"
            INSERT INTO governance_config_registry 
            (config_key, config_category, current_value, default_value, description, tier_requirement, created_at, updated_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(config_key)
        .bind(category.as_str())
        .bind(serde_json::to_string(&default_value).map_err(|e| {
            GovernanceError::ConfigError(format!("Failed to serialize default value: {}", e))
        })?)
        .bind(serde_json::to_string(&default_value).map_err(|e| {
            GovernanceError::ConfigError(format!("Failed to serialize default value: {}", e))
        })?)
        .bind(description)
        .bind(tier_requirement)
        .bind(now)
        .bind(now)
        .bind(created_by)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to register config: {}", e))
        })?;

        let id: i32 = id.get(0);
        info!("Registered governance-controlled config: key={}, category={}, tier={}", 
            config_key, category.as_str(), tier_requirement);
        Ok(id)
    }

    /// Get a configuration entry by key
    pub async fn get_config(&self, config_key: &str) -> Result<Option<ConfigEntry>, GovernanceError> {
        let row = sqlx::query(
            r#"
            SELECT id, config_key, config_category, current_value, default_value, 
                   description, tier_requirement, created_at, updated_at, created_by, notes
            FROM governance_config_registry
            WHERE config_key = ?
            "#,
        )
        .bind(config_key)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get config: {}", e))
        })?;

        if let Some(row) = row {
            Ok(Some(ConfigEntry {
                id: row.get("id"),
                config_key: row.get("config_key"),
                config_category: ConfigCategory::from_str(row.get::<String, _>("config_category").as_str())
                    .ok_or_else(|| GovernanceError::ConfigError("Invalid category".to_string()))?,
                current_value: serde_json::from_str(row.get::<String, _>("current_value").as_str())
                    .map_err(|e| GovernanceError::ConfigError(format!("Invalid JSON: {}", e)))?,
                default_value: serde_json::from_str(row.get::<String, _>("default_value").as_str())
                    .map_err(|e| GovernanceError::ConfigError(format!("Invalid JSON: {}", e)))?,
                description: row.get("description"),
                tier_requirement: row.get("tier_requirement"),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .ok_or_else(|| GovernanceError::ConfigError("Invalid timestamp".to_string()))?,
                updated_at: DateTime::from_timestamp(row.get::<i64, _>("updated_at"), 0)
                    .ok_or_else(|| GovernanceError::ConfigError("Invalid timestamp".to_string()))?,
                created_by: row.get("created_by"),
                notes: row.get("notes"),
            }))
        } else {
            Ok(None)
        }
    }

    /// Get current value of a configuration parameter
    pub async fn get_current_value(&self, config_key: &str) -> Result<Option<serde_json::Value>, GovernanceError> {
        if let Some(entry) = self.get_config(config_key).await? {
            Ok(Some(entry.current_value))
        } else {
            Ok(None)
        }
    }

    /// Propose a configuration change (requires Tier 5 governance approval)
    pub async fn propose_change(
        &self,
        config_key: &str,
        proposed_value: serde_json::Value,
        change_reason: Option<&str>,
        proposed_by: &str,
    ) -> Result<i32, GovernanceError> {
        // Get current config entry
        let entry = self.get_config(config_key).await?
            .ok_or_else(|| GovernanceError::ConfigError(format!("Config key not found: {}", config_key)))?;

        let current_value_str = serde_json::to_string(&entry.current_value)
            .map_err(|e| GovernanceError::ConfigError(format!("Failed to serialize current value: {}", e)))?;
        let proposed_value_str = serde_json::to_string(&proposed_value)
            .map_err(|e| GovernanceError::ConfigError(format!("Failed to serialize proposed value: {}", e)))?;

        let now = Utc::now().timestamp();

        let id = sqlx::query(
            r#"
            INSERT INTO governance_config_changes
            (config_key, proposed_value, current_value, change_reason, proposed_by, proposed_at, tier_requirement, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, 'pending')
            RETURNING id
            "#,
        )
        .bind(config_key)
        .bind(&proposed_value_str)
        .bind(&current_value_str)
        .bind(change_reason)
        .bind(proposed_by)
        .bind(now)
        .bind(entry.tier_requirement)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to propose change: {}", e))
        })?;

        let id: i32 = id.get(0);
        info!("Proposed configuration change: id={}, key={}, tier={}", id, config_key, entry.tier_requirement);
        Ok(id)
    }

    /// Approve a configuration change (after Tier 5 governance approval)
    pub async fn approve_change(
        &self,
        change_id: i32,
        approval_pr_id: i32,
        approved_by: &str,
    ) -> Result<(), GovernanceError> {
        let now = Utc::now().timestamp();

        sqlx::query(
            r#"
            UPDATE governance_config_changes
            SET status = 'approved', approval_pr_id = ?, approved_at = ?, approved_by = ?
            WHERE id = ? AND status = 'pending'
            "#,
        )
        .bind(approval_pr_id)
        .bind(now)
        .bind(approved_by)
        .bind(change_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to approve change: {}", e))
        })?;

        info!("Approved configuration change: id={}, pr_id={}", change_id, approval_pr_id);
        Ok(())
    }

    /// Activate an approved configuration change
    /// Optionally invalidates ConfigReader cache for the changed key
    pub async fn activate_change(
        &self,
        change_id: i32,
        activation_pr_id: Option<i32>,
    ) -> Result<String, GovernanceError> {
        // Get the change
        let change = self.get_change(change_id).await?
            .ok_or_else(|| GovernanceError::ConfigError("Change not found".to_string()))?;

        if change.status != ConfigChangeStatus::Approved {
            return Err(GovernanceError::ConfigError(
                format!("Change is not approved (status: {:?})", change.status)
            ));
        }

        let now = Utc::now().timestamp();
        let config_key = change.config_key.clone();

        // Update the registry with new value
        sqlx::query(
            r#"
            UPDATE governance_config_registry
            SET current_value = ?, updated_at = ?
            WHERE config_key = ?
            "#,
        )
        .bind(serde_json::to_string(&change.proposed_value).map_err(|e| {
            GovernanceError::ConfigError(format!("Failed to serialize value: {}", e))
        })?)
        .bind(now)
        .bind(&config_key)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to update config: {}", e))
        })?;

        // Mark change as activated
        sqlx::query(
            r#"
            UPDATE governance_config_changes
            SET status = 'activated', activated_at = ?, activation_pr_id = ?
            WHERE id = ?
            "#,
        )
        .bind(now)
        .bind(activation_pr_id)
        .bind(change_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to activate change: {}", e))
        })?;

        // Record in history
        sqlx::query(
            r#"
            INSERT INTO governance_config_history
            (config_key, old_value, new_value, changed_at, changed_by, change_id, activation_method)
            VALUES (?, ?, ?, ?, ?, ?, 'governance_approved')
            "#,
        )
        .bind(&change.config_key)
        .bind(serde_json::to_string(&change.current_value).map_err(|e| {
            GovernanceError::ConfigError(format!("Failed to serialize old value: {}", e))
        })?)
        .bind(serde_json::to_string(&change.proposed_value).map_err(|e| {
            GovernanceError::ConfigError(format!("Failed to serialize new value: {}", e))
        })?)
        .bind(now)
        .bind(&change.proposed_by)
        .bind(change_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to record history: {}", e))
        })?;

        let config_key = change.config_key.clone();
        info!("Activated configuration change: id={}, key={}", change_id, config_key);

        // Invalidate ConfigReader cache if available
        if let Some(ref reader) = self.config_reader {
            reader.invalidate_key(&config_key).await;
            debug!("Invalidated ConfigReader cache for key: {}", config_key);
        }

        // Sync to YAML if config path is set
        if let Some(ref config_path) = self.config_path {
            if let Err(e) = self.sync_to_yaml(&config_key, &change.proposed_value, config_path.clone()).await {
                warn!("Failed to sync config change to YAML: {}", e);
                // Don't fail activation if YAML sync fails
            }
        }

        Ok(config_key)
    }

    /// Get a configuration change by ID
    pub async fn get_change(&self, change_id: i32) -> Result<Option<ConfigChange>, GovernanceError> {
        let row = sqlx::query(
            r#"
            SELECT id, config_key, proposed_value, current_value, change_reason, proposed_by,
                   proposed_at, tier_requirement, status, approval_pr_id, approved_at, approved_by,
                   activated_at, activation_pr_id, notes
            FROM governance_config_changes
            WHERE id = ?
            "#,
        )
        .bind(change_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get change: {}", e))
        })?;

        if let Some(row) = row {
            Ok(Some(self.row_to_change(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get pending configuration changes
    pub async fn get_pending_changes(&self) -> Result<Vec<ConfigChange>, GovernanceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, config_key, proposed_value, current_value, change_reason, proposed_by,
                   proposed_at, tier_requirement, status, approval_pr_id, approved_at, approved_by,
                   activated_at, activation_pr_id, notes
            FROM governance_config_changes
            WHERE status = 'pending'
            ORDER BY proposed_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get pending changes: {}", e))
        })?;

        let mut changes = Vec::new();
        for row in rows {
            changes.push(self.row_to_change(row)?);
        }
        Ok(changes)
    }

    /// Get approved but not yet activated changes
    pub async fn get_approved_changes(&self) -> Result<Vec<ConfigChange>, GovernanceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, config_key, proposed_value, current_value, change_reason, proposed_by,
                   proposed_at, tier_requirement, status, approval_pr_id, approved_at, approved_by,
                   activated_at, activation_pr_id, notes
            FROM governance_config_changes
            WHERE status = 'approved'
            ORDER BY approved_at ASC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get approved changes: {}", e))
        })?;

        let mut changes = Vec::new();
        for row in rows {
            changes.push(self.row_to_change(row)?);
        }
        Ok(changes)
    }

    /// Helper to convert database row to ConfigChange
    fn row_to_change(&self, row: sqlx::sqlite::SqliteRow) -> Result<ConfigChange, GovernanceError> {
        Ok(ConfigChange {
            id: row.get("id"),
            config_key: row.get("config_key"),
            proposed_value: serde_json::from_str(row.get::<String, _>("proposed_value").as_str())
                .map_err(|e| GovernanceError::ConfigError(format!("Invalid JSON: {}", e)))?,
            current_value: serde_json::from_str(row.get::<String, _>("current_value").as_str())
                .map_err(|e| GovernanceError::ConfigError(format!("Invalid JSON: {}", e)))?,
            change_reason: row.get("change_reason"),
            proposed_by: row.get("proposed_by"),
            proposed_at: DateTime::from_timestamp(row.get::<i64, _>("proposed_at"), 0)
                .ok_or_else(|| GovernanceError::ConfigError("Invalid timestamp".to_string()))?,
            tier_requirement: row.get("tier_requirement"),
            status: ConfigChangeStatus::from_str(row.get::<String, _>("status").as_str())
                .ok_or_else(|| GovernanceError::ConfigError("Invalid status".to_string()))?,
            approval_pr_id: row.get("approval_pr_id"),
            approved_at: row.get::<Option<i64>, _>("approved_at")
                .and_then(|ts| DateTime::from_timestamp(ts, 0)),
            approved_by: row.get("approved_by"),
            activated_at: row.get::<Option<i64>, _>("activated_at")
                .and_then(|ts| DateTime::from_timestamp(ts, 0)),
            activation_pr_id: row.get("activation_pr_id"),
            notes: row.get("notes"),
        })
    }

    /// Get all registered configuration keys
    pub async fn list_configs(&self) -> Result<Vec<ConfigEntry>, GovernanceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, config_key, config_category, current_value, default_value,
                   description, tier_requirement, created_at, updated_at, created_by, notes
            FROM governance_config_registry
            ORDER BY config_category, config_key
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to list configs: {}", e))
        })?;

        let mut configs = Vec::new();
        for row in rows {
            configs.push(ConfigEntry {
                id: row.get("id"),
                config_key: row.get("config_key"),
                config_category: ConfigCategory::from_str(row.get::<String, _>("config_category").as_str())
                    .ok_or_else(|| GovernanceError::ConfigError("Invalid category".to_string()))?,
                current_value: serde_json::from_str(row.get::<String, _>("current_value").as_str())
                    .map_err(|e| GovernanceError::ConfigError(format!("Invalid JSON: {}", e)))?,
                default_value: serde_json::from_str(row.get::<String, _>("default_value").as_str())
                    .map_err(|e| GovernanceError::ConfigError(format!("Invalid JSON: {}", e)))?,
                description: row.get("description"),
                tier_requirement: row.get("tier_requirement"),
                created_at: DateTime::from_timestamp(row.get::<i64, _>("created_at"), 0)
                    .ok_or_else(|| GovernanceError::ConfigError("Invalid timestamp".to_string()))?,
                updated_at: DateTime::from_timestamp(row.get::<i64, _>("updated_at"), 0)
                    .ok_or_else(|| GovernanceError::ConfigError("Invalid timestamp".to_string()))?,
                created_by: row.get("created_by"),
                notes: row.get("notes"),
            });
        }
        Ok(configs)
    }

    /// Detect configuration changes from a merged PR and activate them
    /// This is called when a Tier 5 PR is merged that contains configuration changes
    /// Returns list of activated config keys (for cache invalidation)
    pub async fn process_pr_config_changes(
        &self,
        pr_id: i32,
        pr_body: &str,
    ) -> Result<Vec<String>, GovernanceError> {
        // Parse PR body for config change references
        // Format: "Config: key=value" or "Config Change: key -> value"
        // For now, we'll look for approved changes linked to this PR
        let changes = sqlx::query(
            r#"
            SELECT id FROM governance_config_changes
            WHERE approval_pr_id = ? AND status = 'approved'
            "#,
        )
        .bind(pr_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to get changes for PR: {}", e))
        })?;

        let mut activated_keys = Vec::new();
        for row in changes {
            let change_id: i32 = row.get(0);
            match self.activate_change(change_id, Some(pr_id)).await {
                Ok(config_key) => {
                    activated_keys.push(config_key);
                }
                Err(e) => {
                    warn!("Failed to activate change {}: {}", change_id, e);
                }
            }
        }

        Ok(activated_keys)
    }

    /// Link a configuration change to a PR (when PR is created/updated)
    pub async fn link_change_to_pr(
        &self,
        change_id: i32,
        pr_id: i32,
    ) -> Result<(), GovernanceError> {
        sqlx::query(
            r#"
            UPDATE governance_config_changes
            SET approval_pr_id = ?
            WHERE id = ? AND status = 'pending'
            "#,
        )
        .bind(pr_id)
        .bind(change_id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to link change to PR: {}", e))
        })?;

        info!("Linked configuration change {} to PR {}", change_id, pr_id);
        Ok(())
    }

    /// Sync configuration values from YAML files to database
    /// This is called on startup to ensure database is in sync with YAML source of truth
    /// Only updates values if they differ (preserves governance-controlled changes)
    pub async fn sync_from_yaml(
        &self,
        config_path: PathBuf,
    ) -> Result<usize, GovernanceError> {
        info!("Syncing configuration from YAML files: {:?}", config_path);

        let yaml_loader = YamlConfigLoader::new(config_path);
        let yaml_values = yaml_loader.extract_all_config_values()?;

        let mut updated_count = 0;

        for (config_key, yaml_value) in yaml_values {
            // Check if config exists in database
            if let Some(mut entry) = self.get_config(&config_key).await? {
                // Compare YAML value with database value
                let db_value_str = serde_json::to_string(&entry.current_value)
                    .map_err(|e| GovernanceError::ConfigError(format!("Failed to serialize DB value: {}", e)))?;
                let yaml_value_str = serde_json::to_string(&yaml_value)
                    .map_err(|e| GovernanceError::ConfigError(format!("Failed to serialize YAML value: {}", e)))?;

                // Only update if values differ
                if db_value_str != yaml_value_str {
                    // Check if this value was changed via governance (has history)
                    let has_governance_history = sqlx::query(
                        r#"
                        SELECT COUNT(*) FROM governance_config_history
                        WHERE config_key = ? AND activation_method = 'governance_approved'
                        "#,
                    )
                    .bind(&config_key)
                    .fetch_one(&self.pool)
                    .await
                    .map_err(|e| {
                        GovernanceError::DatabaseError(format!("Failed to check history: {}", e))
                    })?;

                    let history_count: i64 = has_governance_history.get(0);

                    // Only sync if no governance history (YAML is source of truth for initial values)
                    if history_count == 0 {
                        let now = Utc::now().timestamp();
                        sqlx::query(
                            r#"
                            UPDATE governance_config_registry
                            SET current_value = ?, updated_at = ?
                            WHERE config_key = ?
                            "#,
                        )
                        .bind(&yaml_value_str)
                        .bind(now)
                        .bind(&config_key)
                        .execute(&self.pool)
                        .await
                        .map_err(|e| {
                            GovernanceError::DatabaseError(format!("Failed to sync config: {}", e))
                        })?;

                        // Record sync in history
                        sqlx::query(
                            r#"
                            INSERT INTO governance_config_history
                            (config_key, old_value, new_value, changed_at, changed_by, activation_method)
                            VALUES (?, ?, ?, ?, ?, 'yaml_sync')
                            "#,
                        )
                        .bind(&config_key)
                        .bind(&db_value_str)
                        .bind(&yaml_value_str)
                        .bind(now)
                        .bind("yaml_sync")
                        .execute(&self.pool)
                        .await
                        .map_err(|e| {
                            GovernanceError::DatabaseError(format!("Failed to record sync history: {}", e))
                        })?;

                        updated_count += 1;
                        info!("Synced config from YAML: {} = {}", config_key, yaml_value_str);
                    } else {
                        debug!("Skipping sync for {} (has governance history)", config_key);
                    }
                }
            } else {
                // Config doesn't exist in database, register it
                // Determine category from key
                let category = if config_key.contains("tier_") || config_key.contains("layer_") || config_key.contains("veto_") {
                    ConfigCategory::Thresholds
                } else if config_key.contains("review_period") || config_key.contains("duration") || config_key.contains("period") {
                    ConfigCategory::TimeWindows
                } else if config_key.contains("max_") || config_key.contains("min_") {
                    ConfigCategory::Limits
                } else {
                    ConfigCategory::Other
                };

                self.register_config(
                    &config_key,
                    category,
                    yaml_value,
                    Some(&format!("Synced from YAML: {}", config_key)),
                    5,
                    Some("yaml_sync"),
                ).await?;
                updated_count += 1;
            }
        }

        info!("YAML sync completed: {} configs updated", updated_count);
        Ok(updated_count)
    }

    /// Sync a configuration change back to YAML file
    /// This is called when a governance change is activated
    /// Note: This is a placeholder - actual YAML file updates would require git operations
    pub async fn sync_to_yaml(
        &self,
        config_key: &str,
        new_value: &serde_json::Value,
        config_path: PathBuf,
    ) -> Result<(), GovernanceError> {
        info!("Syncing config change to YAML: {} = {:?}", config_key, new_value);

        // This is a placeholder implementation
        // In a full implementation, this would:
        // 1. Load the appropriate YAML file
        // 2. Update the value in the YAML structure
        // 3. Write the YAML file back
        // 4. Optionally create a git commit or PR

        // For now, we just log the change
        // A full implementation would require:
        // - YAML file structure mapping (reverse of extract_all_config_values)
        // - Git operations for committing changes
        // - Or creating a PR with the changes

        warn!("YAML sync to file not yet fully implemented. Config change: {} = {:?}", config_key, new_value);
        warn!("YAML file update would be: {:?}", config_path);

        // TODO: Implement full YAML file update
        // This would involve:
        // 1. Mapping config_key back to YAML structure (e.g., "tier_1_signatures_required" -> "tiers.tier_1_routine.signatures.required")
        // 2. Loading the YAML file
        // 3. Updating the value
        // 4. Writing back to file
        // 5. Optionally committing to git or creating a PR

        Ok(())
    }
}

