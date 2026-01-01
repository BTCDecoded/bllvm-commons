//! Internal API Handlers for P2P Governance Messages
//!
//! Handles HTTP endpoints for blvm-node to forward P2P governance messages
//! to blvm-commons.

use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::json;
use std::sync::Arc;
use tracing::{error, info};

use crate::economic_nodes::registry::EconomicNodeRegistry;
use crate::governance::message_dedup::MessageDeduplicator;
use crate::governance::p2p_receiver::{
    EconomicNodeForkDecisionMessage, EconomicNodeRegistrationMessage, EconomicNodeStatusMessage, EconomicNodeVetoMessage,
    ForkDecisionResponse, P2PReceiver, RegistrationResponse, StatusResponse, VetoResponse,
};
use crate::Database;

/// Handle health check endpoint
async fn handle_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "ok",
        "service": "blvm-commons",
        "governance_enabled": true,
        "timestamp": chrono::Utc::now()
    }))
}

/// Create internal API router for P2P governance messages
pub fn create_internal_router(
    pool: sqlx::SqlitePool,
    registry: Arc<EconomicNodeRegistry>,
    dedup: Arc<MessageDeduplicator>,
) -> Router {
    let receiver = Arc::new(P2PReceiver::new(pool, registry, dedup));

    Router::new()
        .route("/health", get(handle_health))
        .route(
            "/governance/registration",
            post(handle_registration),
        )
        .route("/governance/veto", post(handle_veto))
        .route("/governance/status", post(handle_status))
        .route("/governance/fork-decision", post(handle_fork_decision))
        .with_state(receiver)
}

/// Handle EconomicNodeRegistration message
async fn handle_registration(
    State(receiver): State<Arc<P2PReceiver>>,
    Json(msg): Json<EconomicNodeRegistrationMessage>,
) -> Result<Json<RegistrationResponse>, StatusCode> {
    info!("Received P2P registration: entity={}, message_id={}", msg.entity_name, msg.message_id);

    match receiver.process_registration(msg).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Failed to process registration: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Handle EconomicNodeVeto message
async fn handle_veto(
    State(receiver): State<Arc<P2PReceiver>>,
    Json(msg): Json<EconomicNodeVetoMessage>,
) -> Result<Json<VetoResponse>, StatusCode> {
    info!("Received P2P veto: pr_id={}, message_id={}", msg.pr_id, msg.message_id);

    match receiver.process_veto(msg).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Failed to process veto: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Handle EconomicNodeStatus query
async fn handle_status(
    State(receiver): State<Arc<P2PReceiver>>,
    Json(msg): Json<EconomicNodeStatusMessage>,
) -> Result<Json<StatusResponse>, StatusCode> {
    info!("Received P2P status query: request_id={}", msg.request_id);

    match receiver.process_status_query(msg).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Failed to process status query: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Handle EconomicNodeForkDecision message
async fn handle_fork_decision(
    State(receiver): State<Arc<P2PReceiver>>,
    Json(msg): Json<EconomicNodeForkDecisionMessage>,
) -> Result<Json<ForkDecisionResponse>, StatusCode> {
    info!("Received P2P fork decision: ruleset={}, message_id={}", msg.chosen_ruleset, msg.message_id);

    match receiver.process_fork_decision(msg).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            error!("Failed to process fork decision: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

