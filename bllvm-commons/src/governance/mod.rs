//! Governance Module
//!
//! Handles governance contribution tracking, weight calculation, and voting.

pub mod time_lock;
pub mod contributions;
pub mod weight_calculator;
pub mod aggregator;
pub mod fee_forwarding;
pub mod vote_aggregator;

pub use contributions::{ContributionTracker, ContributorTotal};
pub use weight_calculator::WeightCalculator;
pub use aggregator::{ContributionAggregator, ContributorAggregates};
pub use fee_forwarding::{FeeForwardingTracker, FeeForwardingContribution};
pub use vote_aggregator::{VoteAggregator, ProposalVoteResult};

