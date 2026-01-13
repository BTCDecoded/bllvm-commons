//! Block webhook handler
//!
//! Receives block notifications from blvm-node (fee forwarding removed)

use axum::{extract::State, response::Json};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::{error, info};

use crate::config::AppConfig;
use crate::database::Database;

/// Block notification payload
/// Block should be provided as JSON object that can be deserialized to blvm_protocol::Block
#[derive(Debug, Deserialize)]
pub struct BlockNotification {
    pub block_hash: String,
    pub block_height: i32,
    pub block: Value, // Block data as JSON - will be converted to blvm_protocol::Block
    pub contributor_id: Option<String>, // Optional: node/miner identifier
}

/// Block notification response
#[derive(Debug, Serialize)]
pub struct BlockNotificationResponse {
    pub success: bool,
    pub message: String,
    pub contributions_found: usize,
}

/// Handle block notification webhook
/// Fee forwarding removed - this is now a placeholder
pub async fn handle_block_notification(
    State((_config, _database)): State<(AppConfig, Database)>,
    Json(_payload): Json<BlockNotification>,
) -> Json<BlockNotificationResponse> {
    // Fee forwarding removed - no longer processing blocks
    Json(BlockNotificationResponse {
        success: true,
        message: "Fee forwarding removed - block notification received but not processed".to_string(),
        contributions_found: 0,
    })
}
