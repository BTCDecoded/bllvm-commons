//! Nostr Integration Module
//!
//! This module provides real-time transparency for governance operations
//! by publishing status updates to the Nostr protocol.

pub mod client;
pub mod publisher;
pub mod events;
pub mod governance_publisher;
pub mod helpers;

pub use client::NostrClient;
pub use publisher::StatusPublisher;
pub use governance_publisher::GovernanceActionPublisher;
pub use helpers::{publish_merge_action, publish_review_period_notification, create_keyholder_announcement_event};
pub use events::{
    GovernanceStatus, ServerHealth, Hashes,
    GovernanceActionEvent, KeyholderAnnouncement, NodeStatusReport,
    LayerRequirement, TierRequirement, CombinedRequirement,
    KeyholderSignature, EconomicVetoStatus,
};
