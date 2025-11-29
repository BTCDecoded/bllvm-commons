//! Governance Module
//!
//! Handles governance contribution tracking, weight calculation, and voting.

pub mod aggregator;
pub mod config_defaults;
pub mod config_reader;
pub mod config_registry;
pub mod contributions;
pub mod fee_forwarding;
pub mod internal_api;
pub mod message_dedup;
pub mod merkle_veto;
pub mod p2p_receiver;
pub mod phase_calculator;
pub mod time_lock;
pub mod vote_aggregator;
pub mod weight_calculator;

pub use aggregator::{ContributionAggregator, ContributorAggregates};
pub use config_defaults::initialize_governance_defaults;
pub use config_reader::ConfigReader;
pub use config_registry::{ConfigCategory, ConfigChange, ConfigChangeStatus, ConfigEntry, ConfigRegistry};
pub use contributions::{ContributionTracker, ContributorTotal};
pub use fee_forwarding::{FeeForwardingContribution, FeeForwardingTracker};
pub use phase_calculator::{AdaptiveParameters, GovernancePhase, GovernancePhaseCalculator};
pub use vote_aggregator::{ProposalVoteResult, VoteAggregator};
pub use weight_calculator::WeightCalculator;
