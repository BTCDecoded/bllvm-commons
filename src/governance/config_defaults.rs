//! Default Governance Configuration Values
//!
//! Registers all forkable governance variables with sensible defaults for the
//! default Commons distribution, aligned with the growth plan (Phase 1 → Phase 2 → Phase 3).

use crate::governance::config_registry::{ConfigCategory, ConfigRegistry};
use crate::error::GovernanceError;
use sqlx::SqlitePool;
use tracing::info;

/// Initialize all governance-controlled configuration parameters with sensible defaults
///
/// This should be called during system initialization to ensure all forkable
/// governance variables are registered in the configuration registry.
pub async fn initialize_governance_defaults(
    registry: &ConfigRegistry,
) -> Result<(), GovernanceError> {
    info!("Initializing governance-controlled configuration defaults");

    // ============================================================================
    // ACTION TIER THRESHOLDS
    // ============================================================================
    
    // Tier 1: Routine Maintenance
    registry.register_config(
        "tier_1_signatures_required",
        ConfigCategory::Thresholds,
        serde_json::json!(3),
        Some("Tier 1: Required maintainer signatures (out of total)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "tier_1_signatures_total",
        ConfigCategory::Thresholds,
        serde_json::json!(5),
        Some("Tier 1: Total maintainer signatures available"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "tier_1_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(7),
        Some("Tier 1: Minimum review period in days"),
        5,
        Some("system"),
    ).await?;

    // Tier 2: Feature Changes
    registry.register_config(
        "tier_2_signatures_required",
        ConfigCategory::Thresholds,
        serde_json::json!(4),
        Some("Tier 2: Required maintainer signatures (out of total)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "tier_2_signatures_total",
        ConfigCategory::Thresholds,
        serde_json::json!(5),
        Some("Tier 2: Total maintainer signatures available"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "tier_2_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(30),
        Some("Tier 2: Minimum review period in days"),
        5,
        Some("system"),
    ).await?;

    // Tier 3: Consensus-Adjacent
    registry.register_config(
        "tier_3_signatures_required",
        ConfigCategory::Thresholds,
        serde_json::json!(5),
        Some("Tier 3: Required maintainer signatures (out of total)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "tier_3_signatures_total",
        ConfigCategory::Thresholds,
        serde_json::json!(5),
        Some("Tier 3: Total maintainer signatures available"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "tier_3_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(90),
        Some("Tier 3: Minimum review period in days"),
        5,
        Some("system"),
    ).await?;

    // Tier 4: Emergency Actions
    registry.register_config(
        "tier_4_signatures_required",
        ConfigCategory::Thresholds,
        serde_json::json!(4),
        Some("Tier 4: Required maintainer signatures (out of total)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "tier_4_signatures_total",
        ConfigCategory::Thresholds,
        serde_json::json!(5),
        Some("Tier 4: Total maintainer signatures available"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "tier_4_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(0),
        Some("Tier 4: Minimum review period in days (emergency)"),
        5,
        Some("system"),
    ).await?;

    // Tier 5: Governance Changes
    registry.register_config(
        "tier_5_signatures_required",
        ConfigCategory::Thresholds,
        serde_json::json!(5),
        Some("Tier 5: Required maintainer signatures (out of total)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "tier_5_signatures_total",
        ConfigCategory::Thresholds,
        serde_json::json!(5),
        Some("Tier 5: Total maintainer signatures available"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "tier_5_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(180),
        Some("Tier 5: Minimum review period in days (constitutional changes)"),
        5,
        Some("system"),
    ).await?;

    // ============================================================================
    // ECONOMIC NODE VETO THRESHOLDS
    // ============================================================================

    // Tier 3 Veto Thresholds
    registry.register_config(
        "veto_tier_3_mining_percent",
        ConfigCategory::Thresholds,
        serde_json::json!(30.0),
        Some("Tier 3: Minimum mining hashpower percentage to trigger veto"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "veto_tier_3_economic_percent",
        ConfigCategory::Thresholds,
        serde_json::json!(40.0),
        Some("Tier 3: Minimum economic activity percentage to trigger veto"),
        5,
        Some("system"),
    ).await?;

    // Tier 4 Veto Thresholds (Emergency)
    registry.register_config(
        "veto_tier_4_mining_percent",
        ConfigCategory::Thresholds,
        serde_json::json!(25.0),
        Some("Tier 4: Minimum mining hashpower percentage to trigger veto (emergency)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "veto_tier_4_economic_percent",
        ConfigCategory::Thresholds,
        serde_json::json!(35.0),
        Some("Tier 4: Minimum economic activity percentage to trigger veto (emergency)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "veto_tier_4_window_hours",
        ConfigCategory::TimeWindows,
        serde_json::json!(24),
        Some("Tier 4: Veto window in hours (emergency actions)"),
        5,
        Some("system"),
    ).await?;

    // Tier 5 Signaling Thresholds (Support required, not just lack of opposition)
    registry.register_config(
        "signaling_tier_5_mining_percent",
        ConfigCategory::Thresholds,
        serde_json::json!(50.0),
        Some("Tier 5: Minimum mining hashpower percentage required for support"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "signaling_tier_5_economic_percent",
        ConfigCategory::Thresholds,
        serde_json::json!(60.0),
        Some("Tier 5: Minimum economic activity percentage required for support"),
        5,
        Some("system"),
    ).await?;

    // ============================================================================
    // COMMONS CONTRIBUTOR THRESHOLDS
    // ============================================================================

    registry.register_config(
        "commons_contributor_measurement_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(90),
        Some("Commons Contributor: Rolling measurement period in days"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "commons_contributor_qualification_logic",
        ConfigCategory::FeatureFlags,
        serde_json::json!("OR"),
        Some("Commons Contributor: Qualification logic (OR = any threshold, AND = all thresholds)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "commons_contributor_min_merge_mining_btc",
        ConfigCategory::Thresholds,
        serde_json::json!(0.01),
        Some("Commons Contributor: Minimum merge mining contribution (BTC over measurement period)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "commons_contributor_min_fee_forwarding_btc",
        ConfigCategory::Thresholds,
        serde_json::json!(0.1),
        Some("Commons Contributor: Minimum fee forwarding contribution (BTC over measurement period)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "commons_contributor_min_zaps_btc",
        ConfigCategory::Thresholds,
        serde_json::json!(0.01),
        Some("Commons Contributor: Minimum zap contributions (BTC over measurement period)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "commons_contributor_min_marketplace_btc",
        ConfigCategory::Thresholds,
        serde_json::json!(0.01),
        Some("Commons Contributor: Minimum marketplace sales (BTC over measurement period)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "commons_contributor_weight_normalization_factor",
        ConfigCategory::Thresholds,
        serde_json::json!(1.0),
        Some("Commons Contributor: Weight normalization factor (1 BTC = 1.0 weight)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "commons_contributor_min_weight",
        ConfigCategory::Thresholds,
        serde_json::json!(0.01),
        Some("Commons Contributor: Minimum weight to qualify (prevents dust)"),
        5,
        Some("system"),
    ).await?;

    // ============================================================================
    // GOVERNANCE PHASE CALCULATION THRESHOLDS
    // ============================================================================
    // These determine when the system transitions between Early → Growth → Mature phases
    // Defaults are conservative for Phase 1, can be adjusted as system matures

    registry.register_config(
        "phase_early_max_blocks",
        ConfigCategory::Thresholds,
        serde_json::json!(50000),
        Some("Governance Phase: Maximum block height for Early phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_early_max_nodes",
        ConfigCategory::Thresholds,
        serde_json::json!(10),
        Some("Governance Phase: Maximum economic nodes for Early phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_early_max_contributors",
        ConfigCategory::Thresholds,
        serde_json::json!(10),
        Some("Governance Phase: Maximum Commons contributors for Early phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_growth_min_blocks",
        ConfigCategory::Thresholds,
        serde_json::json!(50000),
        Some("Governance Phase: Minimum block height for Growth phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_growth_max_blocks",
        ConfigCategory::Thresholds,
        serde_json::json!(200000),
        Some("Governance Phase: Maximum block height for Growth phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_growth_min_nodes",
        ConfigCategory::Thresholds,
        serde_json::json!(10),
        Some("Governance Phase: Minimum economic nodes for Growth phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_growth_max_nodes",
        ConfigCategory::Thresholds,
        serde_json::json!(30),
        Some("Governance Phase: Maximum economic nodes for Growth phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_growth_min_contributors",
        ConfigCategory::Thresholds,
        serde_json::json!(10),
        Some("Governance Phase: Minimum Commons contributors for Growth phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_growth_max_contributors",
        ConfigCategory::Thresholds,
        serde_json::json!(100),
        Some("Governance Phase: Maximum Commons contributors for Growth phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_mature_min_blocks",
        ConfigCategory::Thresholds,
        serde_json::json!(200000),
        Some("Governance Phase: Minimum block height for Mature phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_mature_min_nodes",
        ConfigCategory::Thresholds,
        serde_json::json!(30),
        Some("Governance Phase: Minimum economic nodes for Mature phase"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "phase_mature_min_contributors",
        ConfigCategory::Thresholds,
        serde_json::json!(100),
        Some("Governance Phase: Minimum Commons contributors for Mature phase"),
        5,
        Some("system"),
    ).await?;

    // ============================================================================
    // REPOSITORY LAYER THRESHOLDS
    // ============================================================================

    // Layer 1-2: Constitutional (Orange Paper, Consensus Proof)
    registry.register_config(
        "layer_1_2_signatures_required",
        ConfigCategory::Thresholds,
        serde_json::json!(6),
        Some("Layer 1-2: Required maintainer signatures (constitutional layer)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "layer_1_2_signatures_total",
        ConfigCategory::Thresholds,
        serde_json::json!(7),
        Some("Layer 1-2: Total maintainer signatures available"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "layer_1_2_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(180),
        Some("Layer 1-2: Minimum review period in days"),
        5,
        Some("system"),
    ).await?;

    // Layer 3: Protocol Engine
    registry.register_config(
        "layer_3_signatures_required",
        ConfigCategory::Thresholds,
        serde_json::json!(4),
        Some("Layer 3: Required maintainer signatures (protocol engine)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "layer_3_signatures_total",
        ConfigCategory::Thresholds,
        serde_json::json!(5),
        Some("Layer 3: Total maintainer signatures available"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "layer_3_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(90),
        Some("Layer 3: Minimum review period in days"),
        5,
        Some("system"),
    ).await?;

    // Layer 4: Reference Node
    registry.register_config(
        "layer_4_signatures_required",
        ConfigCategory::Thresholds,
        serde_json::json!(3),
        Some("Layer 4: Required maintainer signatures (reference node)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "layer_4_signatures_total",
        ConfigCategory::Thresholds,
        serde_json::json!(5),
        Some("Layer 4: Total maintainer signatures available"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "layer_4_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(60),
        Some("Layer 4: Minimum review period in days"),
        5,
        Some("system"),
    ).await?;

    // Layer 5: Developer SDK
    registry.register_config(
        "layer_5_signatures_required",
        ConfigCategory::Thresholds,
        serde_json::json!(2),
        Some("Layer 5: Required maintainer signatures (developer SDK)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "layer_5_signatures_total",
        ConfigCategory::Thresholds,
        serde_json::json!(3),
        Some("Layer 5: Total maintainer signatures available"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "layer_5_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(14),
        Some("Layer 5: Minimum review period in days"),
        5,
        Some("system"),
    ).await?;

    // ============================================================================
    // EMERGENCY TIER THRESHOLDS
    // ============================================================================

    // Emergency Tier 1: Critical
    registry.register_config(
        "emergency_tier_1_activation_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("5-of-7"),
        Some("Emergency Tier 1: Activation threshold (critical emergencies)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "emergency_tier_1_signature_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("4-of-7"),
        Some("Emergency Tier 1: Signature threshold for PRs"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "emergency_tier_1_max_duration_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(7),
        Some("Emergency Tier 1: Maximum duration in days"),
        5,
        Some("system"),
    ).await?;

    // Emergency Tier 2: Urgent
    registry.register_config(
        "emergency_tier_2_activation_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("5-of-7"),
        Some("Emergency Tier 2: Activation threshold (urgent security)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "emergency_tier_2_signature_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("5-of-7"),
        Some("Emergency Tier 2: Signature threshold for PRs"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "emergency_tier_2_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(7),
        Some("Emergency Tier 2: Review period in days"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "emergency_tier_2_max_duration_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(30),
        Some("Emergency Tier 2: Maximum duration in days"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "emergency_tier_2_max_extensions",
        ConfigCategory::Limits,
        serde_json::json!(1),
        Some("Emergency Tier 2: Maximum allowed extensions"),
        5,
        Some("system"),
    ).await?;

    // Emergency Tier 3: Elevated
    registry.register_config(
        "emergency_tier_3_activation_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("5-of-7"),
        Some("Emergency Tier 3: Activation threshold (elevated priority)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "emergency_tier_3_signature_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("6-of-7"),
        Some("Emergency Tier 3: Signature threshold for PRs"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "emergency_tier_3_review_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(30),
        Some("Emergency Tier 3: Review period in days"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "emergency_tier_3_max_duration_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(90),
        Some("Emergency Tier 3: Maximum duration in days"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "emergency_tier_3_max_extensions",
        ConfigCategory::Limits,
        serde_json::json!(2),
        Some("Emergency Tier 3: Maximum allowed extensions"),
        5,
        Some("system"),
    ).await?;

    // ============================================================================
    // GOVERNANCE REVIEW POLICY THRESHOLDS
    // ============================================================================

    registry.register_config(
        "governance_review_private_warning_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("4-of-7"),
        Some("Governance Review: Private warning threshold"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "governance_review_public_warning_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("5-of-7"),
        Some("Governance Review: Public warning threshold"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "governance_review_removal_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("6-of-7"),
        Some("Governance Review: Removal threshold"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "governance_review_inter_team_removal_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("4-of-7"),
        Some("Governance Review: Inter-team removal threshold (teams)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "governance_review_improvement_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(90),
        Some("Governance Review: Improvement period in days (public warnings)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "governance_review_response_deadline_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(30),
        Some("Governance Review: Response deadline in days"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "governance_review_resolution_deadline_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(180),
        Some("Governance Review: Resolution deadline in days"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "governance_review_appeal_deadline_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(60),
        Some("Governance Review: Appeal deadline in days"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "governance_review_mediation_period_days",
        ConfigCategory::TimeWindows,
        serde_json::json!(30),
        Some("Governance Review: Mediation period in days"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "governance_review_emergency_removal_threshold",
        ConfigCategory::Thresholds,
        serde_json::json!("4-of-5"),
        Some("Governance Review: Emergency removal threshold (emergency keyholders)"),
        5,
        Some("system"),
    ).await?;

    // ============================================================================
    // FEATURE FLAGS
    // ============================================================================

    registry.register_config(
        "feature_governance_enforcement",
        ConfigCategory::FeatureFlags,
        serde_json::json!(false),
        Some("Feature Flag: Enable governance enforcement (Phase 2 activation)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "feature_p2p_governance",
        ConfigCategory::FeatureFlags,
        serde_json::json!(true),
        Some("Feature Flag: Enable P2P governance message relay"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "feature_economic_node_veto",
        ConfigCategory::FeatureFlags,
        serde_json::json!(true),
        Some("Feature Flag: Enable economic node veto system"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "feature_commons_contributor_auto_registration",
        ConfigCategory::FeatureFlags,
        serde_json::json!(true),
        Some("Feature Flag: Enable automatic Commons contributor registration"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "feature_governance_fork",
        ConfigCategory::FeatureFlags,
        serde_json::json!(true),
        Some("Feature Flag: Enable governance fork mechanism"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "feature_merkle_veto_proofs",
        ConfigCategory::FeatureFlags,
        serde_json::json!(true),
        Some("Feature Flag: Enable Merkle tree proofs for veto signals"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "feature_privacy_preserving_votes",
        ConfigCategory::FeatureFlags,
        serde_json::json!(true),
        Some("Feature Flag: Enable privacy-preserving voting keys (BIP32)"),
        5,
        Some("system"),
    ).await?;

    // ============================================================================
    // NETWORK & SECURITY CONFIGURATION
    // ============================================================================

    registry.register_config(
        "network_default",
        ConfigCategory::Network,
        serde_json::json!("mainnet"),
        Some("Network: Default network (mainnet, testnet, regtest)"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "security_audit_required_tier_3",
        ConfigCategory::Security,
        serde_json::json!(true),
        Some("Security: Require audit for Tier 3 changes"),
        5,
        Some("system"),
    ).await?;

    registry.register_config(
        "security_formal_verification_required_layer_2",
        ConfigCategory::Security,
        serde_json::json!(true),
        Some("Security: Require formal verification for Layer 2 (consensus proof) changes"),
        5,
        Some("system"),
    ).await?;

    info!("Governance configuration defaults initialized successfully");
    Ok(())
}

