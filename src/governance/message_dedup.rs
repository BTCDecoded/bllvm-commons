//! Message Deduplication
//!
//! Prevents processing duplicate P2P governance messages using in-memory
//! tracking with TTL and optional database persistence.

use crate::error::GovernanceError;
use chrono::{DateTime, Duration, Utc};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Message deduplicator
pub struct MessageDeduplicator {
    pool: SqlitePool,
    /// In-memory cache of processed message IDs with timestamps
    cache: Arc<RwLock<HashMap<String, DateTime<Utc>>>>,
    /// TTL for message IDs (default: 24 hours)
    ttl: Duration,
}

impl MessageDeduplicator {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            pool,
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::hours(24),
        }
    }

    /// Check if message ID is a duplicate
    pub async fn is_duplicate(&self, message_id: &str) -> Result<bool, GovernanceError> {
        // Check in-memory cache first
        let cache = self.cache.read().await;
        if let Some(timestamp) = cache.get(message_id) {
            if Utc::now() - *timestamp < self.ttl {
                debug!("Duplicate message found in cache: {}", message_id);
                return Ok(true);
            }
        }
        drop(cache);

        // Check database
        let exists: Option<i64> = sqlx::query_scalar(
            r#"
            SELECT 1 FROM received_p2p_messages
            WHERE message_id = ? AND timestamp > datetime('now', '-' || ? || ' hours')
            "#,
        )
        .bind(message_id)
        .bind(self.ttl.num_hours())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to check duplicate: {}", e))
        })?;

        Ok(exists.is_some())
    }

    /// Mark message as processed
    pub async fn mark_processed(&self, message_id: &str) -> Result<(), GovernanceError> {
        let now = Utc::now();

        // Add to in-memory cache
        let mut cache = self.cache.write().await;
        cache.insert(message_id.to_string(), now);
        drop(cache);

        // Store in database
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO received_p2p_messages (message_id, timestamp)
            VALUES (?, ?)
            "#,
        )
        .bind(message_id)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to mark message processed: {}", e))
        })?;

        debug!("Marked message as processed: {}", message_id);
        Ok(())
    }

    /// Clean up old entries from cache and database
    pub async fn cleanup(&self) -> Result<(), GovernanceError> {
        let cutoff = Utc::now() - self.ttl;

        // Clean in-memory cache
        let mut cache = self.cache.write().await;
        cache.retain(|_, timestamp| *timestamp > cutoff);
        let cache_size = cache.len();
        drop(cache);

        // Clean database
        let deleted = sqlx::query(
            r#"
            DELETE FROM received_p2p_messages
            WHERE timestamp < ?
            "#,
        )
        .bind(cutoff)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            GovernanceError::DatabaseError(format!("Failed to cleanup messages: {}", e))
        })?;

        debug!(
            "Cleaned up {} cache entries and {} database entries",
            cache_size, deleted.rows_affected()
        );

        Ok(())
    }
}

