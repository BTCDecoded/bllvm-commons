//! Configuration Reader
//!
//! Provides a unified, type-safe interface for reading governance-controlled
//! configuration values with caching and fallback support.
//!
//! Fallback chain:
//! 1. Cache (in-memory, 5-minute TTL)
//! 2. Config Registry (database, governance-controlled, can be changed via Tier 5)
//! 3. YAML Config (file-based, source of truth for initial defaults)
//! 4. Hardcoded defaults (safety fallback)

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::error::GovernanceError;
use crate::governance::config_registry::ConfigRegistry;
use crate::governance::yaml_loader::YamlConfigLoader;
use crate::config::loader::CommonsContributorThresholdsConfig;
use std::path::PathBuf;

/// Configuration reader with caching
pub struct ConfigReader {
    registry: Arc<ConfigRegistry>,
    /// Optional YAML config for Commons contributor thresholds
    yaml_config: Option<CommonsContributorThresholdsConfig>,
    /// Optional YAML loader for direct YAML file access (fallback)
    yaml_loader: Option<YamlConfigLoader>,
    /// Cache for frequently accessed config values
    cache: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Cache TTL (seconds) - refresh cache periodically
    cache_ttl: u64,
}

impl ConfigReader {
    /// Create a new config reader
    pub fn new(registry: Arc<ConfigRegistry>) -> Self {
        Self {
            registry,
            yaml_config: None,
            yaml_loader: None,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: 300, // 5 minutes default
        }
    }

    /// Create with YAML config for Commons contributor thresholds
    pub fn with_yaml_config(
        registry: Arc<ConfigRegistry>,
        yaml_config: Option<CommonsContributorThresholdsConfig>,
    ) -> Self {
        Self {
            registry,
            yaml_config,
            yaml_loader: None,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: 300,
        }
    }

    /// Create with YAML loader for direct YAML file access (fallback)
    pub fn with_yaml_loader(
        registry: Arc<ConfigRegistry>,
        yaml_loader: Option<YamlConfigLoader>,
    ) -> Self {
        Self {
            registry,
            yaml_config: None,
            yaml_loader,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: 300,
        }
    }

    /// Get a configuration value (with caching and fallback chain)
    pub async fn get_value(&self, key: &str) -> Result<Option<serde_json::Value>, GovernanceError> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(key) {
                debug!("Config cache hit: {}", key);
                return Ok(Some(cached.clone()));
            }
        }

        // Try registry (database) - this should have YAML values synced
        let value = self.registry.get_current_value(key).await?;

        // If found in registry, cache and return
        if let Some(ref val) = value {
            let mut cache = self.cache.write().await;
            cache.insert(key.to_string(), val.clone());
            return Ok(Some(val.clone()));
        }

        // Fallback to YAML files if loader is available
        if let Some(ref yaml_loader) = self.yaml_loader {
            if let Ok(yaml_values) = yaml_loader.extract_all_config_values() {
                if let Some(yaml_value) = yaml_values.get(key) {
                    debug!("Config value found in YAML: {}", key);
                    // Cache the YAML value
                    let mut cache = self.cache.write().await;
                    cache.insert(key.to_string(), yaml_value.clone());
                    return Ok(Some(yaml_value.clone()));
                }
            }
        }

        // Not found in registry or YAML
        Ok(None)
    }

    /// Get a configuration value with fallback
    pub async fn get_value_with_fallback(
        &self,
        key: &str,
        fallback: serde_json::Value,
    ) -> Result<serde_json::Value, GovernanceError> {
        match self.get_value(key).await? {
            Some(value) => Ok(value),
            None => {
                warn!("Config key '{}' not found in registry, using fallback", key);
                Ok(fallback)
            }
        }
    }

    /// Get an i32 configuration value
    pub async fn get_i32(&self, key: &str, fallback: i32) -> Result<i32, GovernanceError> {
        match self.get_value(key).await? {
            Some(value) => {
                value.as_i64()
                    .map(|v| v as i32)
                    .or_else(|| value.as_u64().map(|v| v as i32))
                    .ok_or_else(|| {
                        GovernanceError::ConfigError(format!(
                            "Config '{}' is not a valid integer",
                            key
                        ))
                    })
            }
            None => {
                debug!("Config '{}' not found, using fallback: {}", key, fallback);
                Ok(fallback)
            }
        }
    }

    /// Get a u32 configuration value
    pub async fn get_u32(&self, key: &str, fallback: u32) -> Result<u32, GovernanceError> {
        match self.get_value(key).await? {
            Some(value) => {
                value.as_u64()
                    .map(|v| v as u32)
                    .or_else(|| value.as_i64().map(|v| v as u32))
                    .ok_or_else(|| {
                        GovernanceError::ConfigError(format!(
                            "Config '{}' is not a valid unsigned integer",
                            key
                        ))
                    })
            }
            None => {
                debug!("Config '{}' not found, using fallback: {}", key, fallback);
                Ok(fallback)
            }
        }
    }

    /// Get an f64 configuration value
    pub async fn get_f64(&self, key: &str, fallback: f64) -> Result<f64, GovernanceError> {
        match self.get_value(key).await? {
            Some(value) => {
                value.as_f64()
                    .or_else(|| value.as_i64().map(|v| v as f64))
                    .or_else(|| value.as_u64().map(|v| v as f64))
                    .ok_or_else(|| {
                        GovernanceError::ConfigError(format!(
                            "Config '{}' is not a valid float",
                            key
                        ))
                    })
            }
            None => {
                debug!("Config '{}' not found, using fallback: {}", key, fallback);
                Ok(fallback)
            }
        }
    }

    /// Get a bool configuration value
    pub async fn get_bool(&self, key: &str, fallback: bool) -> Result<bool, GovernanceError> {
        match self.get_value(key).await? {
            Some(value) => {
                value.as_bool().ok_or_else(|| {
                    GovernanceError::ConfigError(format!(
                        "Config '{}' is not a valid boolean",
                        key
                    ))
                })
            }
            None => {
                debug!("Config '{}' not found, using fallback: {}", key, fallback);
                Ok(fallback)
            }
        }
    }

    /// Get a string configuration value
    pub async fn get_string(&self, key: &str, fallback: &str) -> Result<String, GovernanceError> {
        match self.get_value(key).await? {
            Some(value) => {
                value.as_str()
                    .map(|s| s.to_string())
                    .ok_or_else(|| {
                        GovernanceError::ConfigError(format!(
                            "Config '{}' is not a valid string",
                            key
                        ))
                    })
            }
            None => {
                debug!("Config '{}' not found, using fallback: {}", key, fallback);
                Ok(fallback.to_string())
            }
        }
    }

    /// Get a threshold pair (N-of-M) from a string like "5-of-7"
    pub async fn get_threshold_pair(
        &self,
        key: &str,
        fallback: (u32, u32),
    ) -> Result<(u32, u32), GovernanceError> {
        match self.get_string(key, "").await? {
            s if s.is_empty() => Ok(fallback),
            s => {
                // Parse "N-of-M" format
                if let Some((n_str, m_str)) = s.split_once("-of-") {
                    let n = n_str.parse::<u32>().map_err(|_| {
                        GovernanceError::ConfigError(format!(
                            "Invalid threshold format in '{}': cannot parse N",
                            key
                        ))
                    })?;
                    let m = m_str.parse::<u32>().map_err(|_| {
                        GovernanceError::ConfigError(format!(
                            "Invalid threshold format in '{}': cannot parse M",
                            key
                        ))
                    })?;
                    Ok((n, m))
                } else {
                    warn!("Invalid threshold format '{}' for key '{}', using fallback", s, key);
                    Ok(fallback)
                }
            }
        }
    }

    /// Clear the cache (useful after config changes)
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        debug!("Config cache cleared");
    }

    /// Invalidate a specific cache key
    pub async fn invalidate_key(&self, key: &str) {
        let mut cache = self.cache.write().await;
        cache.remove(key);
        debug!("Config cache invalidated for key: {}", key);
    }

    // ============================================================================
    // Convenience methods for common configuration patterns
    // ============================================================================

    /// Get tier signature requirements
    pub async fn get_tier_signatures(&self, tier: u32) -> Result<(usize, usize), GovernanceError> {
        let required_key = format!("tier_{}_signatures_required", tier);
        let total_key = format!("tier_{}_signatures_total", tier);

        let required = self.get_u32(&required_key, self.get_tier_default_required(tier)).await? as usize;
        let total = self.get_u32(&total_key, self.get_tier_default_total(tier)).await? as usize;

        Ok((required, total))
    }

    /// Get tier review period
    pub async fn get_tier_review_period(&self, tier: u32) -> Result<i64, GovernanceError> {
        let key = format!("tier_{}_review_period_days", tier);
        self.get_i32(&key, self.get_tier_default_review_period(tier)).await.map(|v| v as i64)
    }

    /// Get layer signature requirements
    pub async fn get_layer_signatures(&self, layer: i32) -> Result<(usize, usize), GovernanceError> {
        let (req_key, total_key, default_req, default_total) = match layer {
            1 | 2 => ("layer_1_2_signatures_required", "layer_1_2_signatures_total", 6, 7),
            3 => ("layer_3_signatures_required", "layer_3_signatures_total", 4, 5),
            4 => ("layer_4_signatures_required", "layer_4_signatures_total", 3, 5),
            5 => ("layer_5_signatures_required", "layer_5_signatures_total", 2, 3),
            _ => return Ok((1, 1)),
        };

        let required = self.get_u32(req_key, default_req).await? as usize;
        let total = self.get_u32(total_key, default_total).await? as usize;

        Ok((required, total))
    }

    /// Get layer review period
    pub async fn get_layer_review_period(&self, layer: i32) -> Result<i64, GovernanceError> {
        let (key, default) = match layer {
            1 | 2 => ("layer_1_2_review_period_days", 180),
            3 => ("layer_3_review_period_days", 90),
            4 => ("layer_4_review_period_days", 60),
            5 => ("layer_5_review_period_days", 14),
            _ => return Ok(30),
        };

        self.get_i32(key, default).await.map(|v| v as i64)
    }

    /// Get veto thresholds for a tier
    pub async fn get_veto_thresholds(&self, tier: u32) -> Result<(f64, f64), GovernanceError> {
        let (mining_key, economic_key, default_mining, default_economic) = match tier {
            3 => ("veto_tier_3_mining_percent", "veto_tier_3_economic_percent", 30.0, 40.0),
            4 => ("veto_tier_4_mining_percent", "veto_tier_4_economic_percent", 25.0, 35.0),
            5 => ("signaling_tier_5_mining_percent", "signaling_tier_5_economic_percent", 50.0, 60.0),
            _ => return Ok((30.0, 40.0)), // Default to Tier 3
        };

        let mining = self.get_f64(mining_key, default_mining).await?;
        let economic = self.get_f64(economic_key, default_economic).await?;

        Ok((mining, economic))
    }

    /// Get Commons contributor threshold (with YAML fallback)
    pub async fn get_commons_contributor_threshold(
        &self,
        threshold_type: &str,
    ) -> Result<f64, GovernanceError> {
        let key = format!("commons_contributor_min_{}_btc", threshold_type);

        // Try config registry first
        if let Ok(value) = self.get_f64(&key, 0.0).await {
            if value > 0.0 {
                return Ok(value);
            }
        }

        // Fallback to YAML config
        if let Some(ref yaml) = self.yaml_config {
            let thresholds = &yaml.commons_contributor_thresholds;
            let threshold = match threshold_type {
                "merge_mining" => thresholds.merge_mining.minimum_contribution_btc,
                "fee_forwarding" => thresholds.fee_forwarding.minimum_contribution_btc,
                "zaps" => thresholds.zaps.minimum_contribution_btc,
                "marketplace" => thresholds.marketplace.minimum_contribution_btc,
                _ => return Err(GovernanceError::ConfigError(format!(
                    "Unknown threshold type: {}",
                    threshold_type
                ))),
            };
            return Ok(threshold);
        }

        // Final fallback to hardcoded defaults
        let default = match threshold_type {
            "merge_mining" => 0.01,
            "fee_forwarding" => 0.1,
            "zaps" => 0.01,
            "marketplace" => 0.01,
            _ => return Err(GovernanceError::ConfigError(format!(
                "Unknown threshold type: {}",
                threshold_type
            ))),
        };

        Ok(default)
    }

    /// Get Commons contributor measurement period
    pub async fn get_commons_contributor_measurement_period(&self) -> Result<u32, GovernanceError> {
        // Try config registry first
        if let Ok(value) = self.get_u32("commons_contributor_measurement_period_days", 0).await {
            if value > 0 {
                return Ok(value);
            }
        }

        // Fallback to YAML
        if let Some(ref yaml) = self.yaml_config {
            return Ok(yaml.commons_contributor_thresholds.measurement_period_days);
        }

        // Final fallback
        Ok(90)
    }

    /// Get phase boundary thresholds
    pub async fn get_phase_boundaries(
        &self,
        phase: &str,
        metric: &str,
    ) -> Result<u64, GovernanceError> {
        let key = format!("phase_{}_{}", phase, metric);
        let default = self.get_phase_default(phase, metric);
        self.get_u32(&key, default as u32).await.map(|v| v as u64)
    }

    /// Get emergency tier configuration
    pub async fn get_emergency_tier_config(
        &self,
        tier: u32,
        config_type: &str,
    ) -> Result<u32, GovernanceError> {
        let key = format!("emergency_tier_{}_{}", tier, config_type);
        let default = self.get_emergency_default(tier, config_type);
        self.get_u32(&key, default).await
    }

    // ============================================================================
    // Default value helpers (fallbacks)
    // ============================================================================

    fn get_tier_default_required(&self, tier: u32) -> u32 {
        match tier {
            1 => 3,
            2 => 4,
            3 => 5,
            4 => 4,
            5 => 5,
            _ => 1,
        }
    }

    fn get_tier_default_total(&self, tier: u32) -> u32 {
        match tier {
            1..=5 => 5,
            _ => 1,
        }
    }

    fn get_tier_default_review_period(&self, tier: u32) -> i32 {
        match tier {
            1 => 7,
            2 => 30,
            3 => 90,
            4 => 0,
            5 => 180,
            _ => 30,
        }
    }

    fn get_phase_default(&self, phase: &str, metric: &str) -> u64 {
        match (phase, metric) {
            ("early", "max_blocks") => 50_000,
            ("early", "max_nodes") => 10,
            ("early", "max_contributors") => 10,
            ("growth", "min_blocks") => 50_000,
            ("growth", "max_blocks") => 200_000,
            ("growth", "min_nodes") => 10,
            ("growth", "max_nodes") => 30,
            ("growth", "min_contributors") => 10,
            ("growth", "max_contributors") => 100,
            ("mature", "min_blocks") => 200_000,
            ("mature", "min_nodes") => 30,
            ("mature", "min_contributors") => 100,
            _ => 0,
        }
    }

    fn get_emergency_default(&self, tier: u32, config_type: &str) -> u32 {
        match (tier, config_type) {
            (1, "review_period_days") => 0,
            (1, "max_duration_days") => 7,
            (1, "signature_threshold") => 4, // N in N-of-7
            (2, "review_period_days") => 7,
            (2, "max_duration_days") => 30,
            (2, "max_extensions") => 1,
            (2, "signature_threshold") => 5,
            (3, "review_period_days") => 30,
            (3, "max_duration_days") => 90,
            (3, "max_extensions") => 2,
            (3, "signature_threshold") => 6,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::governance::config_registry::{ConfigRegistry, ConfigCategory};
    use sqlx::SqlitePool;

    async fn create_test_registry() -> Arc<ConfigRegistry> {
        // Use in-memory SQLite for testing
        let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
        let registry = ConfigRegistry::new(pool);
        Arc::new(registry)
    }

    #[tokio::test]
    async fn test_get_i32_with_fallback() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        // Test fallback when key doesn't exist
        let value = reader.get_i32("nonexistent_key", 42).await.unwrap();
        assert_eq!(value, 42);
    }

    #[tokio::test]
    async fn test_get_i32_from_registry() {
        let registry = create_test_registry().await;
        
        // Register a test value
        registry.register_config(
            "test_int",
            ConfigCategory::Thresholds,
            serde_json::json!(100),
            Some("Test integer"),
            5,
            Some("test"),
        ).await.unwrap();
        
        let reader = ConfigReader::new(registry.clone());
        let value = reader.get_i32("test_int", 0).await.unwrap();
        assert_eq!(value, 100);
    }

    #[tokio::test]
    async fn test_get_u32_with_fallback() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        let value = reader.get_u32("nonexistent_key", 99).await.unwrap();
        assert_eq!(value, 99);
    }

    #[tokio::test]
    async fn test_get_f64_with_fallback() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        let value = reader.get_f64("nonexistent_key", 3.14).await.unwrap();
        assert_eq!(value, 3.14);
    }

    #[tokio::test]
    async fn test_get_bool_with_fallback() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        let value = reader.get_bool("nonexistent_key", true).await.unwrap();
        assert!(value);
    }

    #[tokio::test]
    async fn test_get_string_with_fallback() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        let value = reader.get_string("nonexistent_key", "default").await.unwrap();
        assert_eq!(value, "default");
    }

    #[tokio::test]
    async fn test_get_threshold_pair() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        // Test fallback
        let (n, m) = reader.get_threshold_pair("nonexistent", (5, 7)).await.unwrap();
        assert_eq!((n, m), (5, 7));
        
        // Test with valid format
        registry.register_config(
            "test_threshold",
            ConfigCategory::Thresholds,
            serde_json::json!("6-of-7"),
            Some("Test threshold"),
            5,
            Some("test"),
        ).await.unwrap();
        
        let (n, m) = reader.get_threshold_pair("test_threshold", (0, 0)).await.unwrap();
        assert_eq!((n, m), (6, 7));
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        // Register and read value
        registry.register_config(
            "cache_test",
            ConfigCategory::Thresholds,
            serde_json::json!(100),
            Some("Cache test"),
            5,
            Some("test"),
        ).await.unwrap();
        
        let value1 = reader.get_i32("cache_test", 0).await.unwrap();
        assert_eq!(value1, 100);
        
        // Clear cache and verify it still works
        reader.clear_cache().await;
        let value2 = reader.get_i32("cache_test", 0).await.unwrap();
        assert_eq!(value2, 100);
        
        // Invalidate specific key
        reader.invalidate_key("cache_test").await;
        let value3 = reader.get_i32("cache_test", 0).await.unwrap();
        assert_eq!(value3, 100);
    }

    #[tokio::test]
    async fn test_tier_signatures() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        // Test default values
        let (req, total) = reader.get_tier_signatures(1).await.unwrap();
        assert_eq!((req, total), (3, 5));
        
        let (req, total) = reader.get_tier_signatures(3).await.unwrap();
        assert_eq!((req, total), (5, 5));
    }

    #[tokio::test]
    async fn test_tier_review_period() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        // Test default values
        let period = reader.get_tier_review_period(1).await.unwrap();
        assert_eq!(period, 7);
        
        let period = reader.get_tier_review_period(3).await.unwrap();
        assert_eq!(period, 90);
    }

    #[tokio::test]
    async fn test_layer_signatures() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        // Test default values
        let (req, total) = reader.get_layer_signatures(1).await.unwrap();
        assert_eq!((req, total), (6, 7));
        
        let (req, total) = reader.get_layer_signatures(4).await.unwrap();
        assert_eq!((req, total), (3, 5));
    }

    #[tokio::test]
    async fn test_veto_thresholds() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        // Test default values
        let (mining, economic) = reader.get_veto_thresholds(3).await.unwrap();
        assert_eq!((mining, economic), (30.0, 40.0));
        
        let (mining, economic) = reader.get_veto_thresholds(4).await.unwrap();
        assert_eq!((mining, economic), (25.0, 35.0));
    }

    #[tokio::test]
    async fn test_emergency_tier_config() {
        let registry = create_test_registry().await;
        let reader = ConfigReader::new(registry.clone());
        
        // Test default values
        let review_period = reader.get_emergency_tier_config(1, "review_period_days").await.unwrap();
        assert_eq!(review_period, 0);
        
        let max_duration = reader.get_emergency_tier_config(1, "max_duration_days").await.unwrap();
        assert_eq!(max_duration, 7);
        
        let sig_threshold = reader.get_emergency_tier_config(2, "signature_threshold").await.unwrap();
        assert_eq!(sig_threshold, 5);
    }
}

