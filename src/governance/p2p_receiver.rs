//! P2P Governance Message Receiver
//!
//! Receives governance messages from bllvm-node (via internal API) that were
//! originally broadcast over the Bitcoin P2P network by economic actors.

use crate::crypto::signatures::SignatureManager;
use crate::economic_nodes::registry::EconomicNodeRegistry;
use crate::economic_nodes::types::SignalType;
use crate::economic_nodes::VetoManager;
use crate::error::GovernanceError;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::str::FromStr;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// P2P governance message receiver
pub struct P2PReceiver {
    pool: SqlitePool,
    registry: Arc<EconomicNodeRegistry>,
    dedup: Arc<MessageDeduplicator>,
    signature_manager: SignatureManager,
    veto_manager: Option<Arc<VetoManager>>,
}

impl P2PReceiver {
    pub fn new(
        pool: SqlitePool,
        registry: Arc<EconomicNodeRegistry>,
        dedup: Arc<MessageDeduplicator>,
    ) -> Self {
        Self {
            pool: pool.clone(),
            registry,
            dedup,
            signature_manager: SignatureManager::new(),
            veto_manager: Some(Arc::new(VetoManager::new(pool))),
        }
    }

    pub fn with_veto_manager(mut self, veto_manager: Option<Arc<VetoManager>>) -> Self {
        self.veto_manager = veto_manager;
        self
    }

    /// Process EconomicNodeRegistration message from P2P
    pub async fn process_registration(
        &self,
        msg: EconomicNodeRegistrationMessage,
    ) -> Result<RegistrationResponse, GovernanceError> {
        // Check for duplicate
        if self.dedup.is_duplicate(&msg.message_id).await? {
            warn!("Duplicate registration message: {}", msg.message_id);
            return Err(GovernanceError::CryptoError(
                "Duplicate message".to_string(),
            ));
        }

        // Verify signature
        if !self.verify_registration_signature(&msg)? {
            return Err(GovernanceError::CryptoError(
                "Invalid signature".to_string(),
            ));
        }

        // Mark as processed
        self.dedup.mark_processed(&msg.message_id).await?;

        // Parse qualification data
        let qualification_proof: crate::economic_nodes::types::QualificationProof =
            serde_json::from_value(msg.qualification_data.clone())
                .map_err(|e| {
                    GovernanceError::CryptoError(format!("Invalid qualification data: {}", e))
                })?;

        // Register node
        let node_type = NodeType::from_str(&msg.node_type).ok_or_else(|| {
            GovernanceError::CryptoError(format!("Invalid node type: {}", msg.node_type))
        })?;

        let node_id = self
            .registry
            .register_economic_node(
                node_type,
                &msg.entity_name,
                &msg.public_key,
                &qualification_proof,
                Some("p2p_registration"),
            )
            .await?;

        info!(
            "Registered economic node via P2P: node_id={}, entity={}, message_id={}",
            node_id, msg.entity_name, msg.message_id
        );

        Ok(RegistrationResponse {
            node_id,
            status: "active".to_string(),
            message: "Registration successful".to_string(),
        })
    }

    /// Process EconomicNodeVeto message from P2P
    pub async fn process_veto(
        &self,
        msg: EconomicNodeVetoMessage,
    ) -> Result<VetoResponse, GovernanceError> {
        // Check for duplicate
        if self.dedup.is_duplicate(&msg.message_id).await? {
            warn!("Duplicate veto message: {}", msg.message_id);
            return Err(GovernanceError::CryptoError(
                "Duplicate message".to_string(),
            ));
        }

        // Verify signature
        if !self.verify_veto_signature(&msg)? {
            return Err(GovernanceError::CryptoError(
                "Invalid signature".to_string(),
            ));
        }

        // Mark as processed
        self.dedup.mark_processed(&msg.message_id).await?;

        // Process veto signal via VetoManager
        if let Some(veto_manager) = &self.veto_manager {
            // Get node_id from public_key if not provided
            let node_id = if let Some(id) = msg.node_id {
                id
            } else {
                // Look up node by public key
                let active_nodes = self.registry.get_active_nodes().await?;
                active_nodes
                    .into_iter()
                    .find(|node| node.public_key == msg.public_key)
                    .ok_or_else(|| {
                        GovernanceError::CryptoError(
                            format!("Node not found for public key: {}", msg.public_key),
                        )
                    })?
                    .id
            };

            // Parse signal type
            let signal_type = SignalType::from_str(&msg.signal_type).ok_or_else(|| {
                GovernanceError::CryptoError(format!("Invalid signal type: {}", msg.signal_type))
            })?;

            // Collect veto signal (using registration key for now, voting key support can be added later)
            let _signal_id = veto_manager
                .collect_veto_signal(
                    msg.pr_id,
                    node_id,
                    signal_type,
                    &msg.signature,
                    "", // Rationale not in P2P message, could be added
                )
                .await?;

            info!(
                "Processed veto signal via P2P: pr_id={}, signal={}, node_id={}, message_id={}",
                msg.pr_id, msg.signal_type, node_id, msg.message_id
            );

            Ok(VetoResponse {
                pr_id: msg.pr_id,
                status: "processed".to_string(),
                message: "Veto signal processed and recorded".to_string(),
            })
        } else {
            // VetoManager not available, just log
            warn!(
                "VetoManager not available, cannot process veto signal: pr_id={}, message_id={}",
                msg.pr_id, msg.message_id
            );
            Ok(VetoResponse {
                pr_id: msg.pr_id,
                status: "received".to_string(),
                message: "Veto signal received but not processed (VetoManager unavailable)".to_string(),
            })
        }
    }

    /// Process EconomicNodeStatus query from P2P
    pub async fn process_status_query(
        &self,
        msg: EconomicNodeStatusMessage,
    ) -> Result<StatusResponse, GovernanceError> {
        // Look up node by identifier
        let node = if msg.query_type == "by_id" {
            let node_id: i32 = msg.node_identifier.parse().map_err(|_| {
                GovernanceError::CryptoError("Invalid node ID".to_string())
            })?;
            self.registry.get_node_by_id(node_id).await?
        } else if msg.query_type == "by_public_key" {
            // Query by public key
            let nodes = self.registry.get_active_nodes().await?;
            nodes
                .into_iter()
                .find(|n| n.public_key == msg.node_identifier)
        } else {
            return Err(GovernanceError::CryptoError(
                "Invalid query type".to_string(),
            ));
        };

        Ok(StatusResponse {
            request_id: msg.request_id,
            status: Some(NodeStatusData {
                node_id: node.id,
                node_type: node.node_type.as_str().to_string(),
                entity_name: node.entity_name,
                status: node.status,
                weight: node.weight,
                registered_at: node.created_at.timestamp(),
                last_verified_at: node.last_verified_at.map(|dt| dt.timestamp()),
            }),
        })
    }

    /// Verify registration message signature
    ///
    /// The message to sign is: "economic_node_registration:{node_type}:{entity_name}:{public_key}:{qualification_data_hash}:{timestamp}:{message_id}"
    /// where qualification_data_hash is SHA256(qualification_data JSON)
    fn verify_registration_signature(
        &self,
        msg: &EconomicNodeRegistrationMessage,
    ) -> Result<bool, GovernanceError> {
        use sha2::{Digest, Sha256};

        // Hash qualification data for deterministic message
        let qual_data_hash = {
            let qual_json = serde_json::to_string(&msg.qualification_data)
                .map_err(|e| GovernanceError::CryptoError(format!("Failed to serialize qualification data: {}", e)))?;
            let hash = Sha256::digest(qual_json.as_bytes());
            hex::encode(hash)
        };

        // Construct message to verify (matches what the node signed)
        let message = format!(
            "economic_node_registration:{}:{}:{}:{}:{}:{}",
            msg.node_type,
            msg.entity_name,
            msg.public_key,
            qual_data_hash,
            msg.timestamp,
            msg.message_id
        );

        debug!("Verifying registration signature for message_id={}, message={}", msg.message_id, message);

        // Verify signature
        self.signature_manager.verify_governance_signature(
            &message,
            &msg.signature,
            &msg.public_key,
        )
    }

    /// Verify veto message signature
    ///
    /// The message to sign is: "economic_node_veto:{pr_id}:{signal_type}:{public_key}:{timestamp}:{message_id}"
    fn verify_veto_signature(
        &self,
        msg: &EconomicNodeVetoMessage,
    ) -> Result<bool, GovernanceError> {
        // Construct message to verify (matches what the node signed)
        let message = format!(
            "economic_node_veto:{}:{}:{}:{}:{}",
            msg.pr_id,
            msg.signal_type,
            msg.public_key,
            msg.timestamp,
            msg.message_id
        );

        debug!("Verifying veto signature for message_id={}, message={}", msg.message_id, message);

        // Verify signature
        self.signature_manager.verify_governance_signature(
            &message,
            &msg.signature,
            &msg.public_key,
        )
    }

    /// Verify fork decision message signature
    ///
    /// The message to sign is: "economic_node_fork_decision:{chosen_ruleset}:{public_key}:{timestamp}:{message_id}"
    fn verify_fork_decision_signature(
        &self,
        msg: &EconomicNodeForkDecisionMessage,
    ) -> Result<bool, GovernanceError> {
        // Construct message to verify (matches what the node signed)
        let message = format!(
            "economic_node_fork_decision:{}:{}:{}:{}",
            msg.chosen_ruleset,
            msg.public_key,
            msg.timestamp,
            msg.message_id
        );

        debug!("Verifying fork decision signature for message_id={}, message={}", msg.message_id, message);

        // Verify signature
        self.signature_manager.verify_governance_signature(
            &message,
            &msg.signature,
            &msg.public_key,
        )
    }
}

// Request/Response types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicNodeRegistrationMessage {
    pub node_type: String,
    pub entity_name: String,
    pub public_key: String,
    pub qualification_data: serde_json::Value,
    pub timestamp: i64,
    pub signature: String,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicNodeVetoMessage {
    pub node_id: Option<i32>,
    pub public_key: String,
    pub pr_id: i32,
    pub signal_type: String,
    pub timestamp: i64,
    pub signature: String,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicNodeStatusMessage {
    pub request_id: u64,
    pub node_identifier: String,
    pub query_type: String,
    pub status: Option<NodeStatusData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicNodeForkDecisionMessage {
    pub node_id: Option<i32>,
    pub public_key: String,
    pub chosen_ruleset: String,
    pub decision_reason: String,
    pub timestamp: i64,
    pub signature: String,
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatusData {
    pub node_id: i32,
    pub node_type: String,
    pub entity_name: String,
    pub status: String,
    pub weight: f64,
    pub registered_at: i64,
    pub last_verified_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrationResponse {
    pub node_id: i32,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VetoResponse {
    pub pr_id: i32,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusResponse {
    pub request_id: u64,
    pub status: Option<NodeStatusData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForkDecisionResponse {
    pub ruleset: String,
    pub status: String,
    pub message: String,
}

// Message deduplication

use crate::governance::message_dedup::MessageDeduplicator;
use crate::economic_nodes::types::NodeType;

