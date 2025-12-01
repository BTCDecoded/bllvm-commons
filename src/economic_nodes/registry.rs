//! Economic Node Registry
//!
//! Handles registration, qualification verification, and weight calculation

use sqlx::{Row, SqlitePool};
use tracing::{info, warn};

use super::types::*;
use crate::error::GovernanceError;
use crate::config::loader::CommonsContributorThresholdsConfig;
use crate::governance::phase_calculator::GovernancePhaseCalculator;
use crate::governance::config_reader::ConfigReader;
use std::sync::Arc;

pub struct EconomicNodeRegistry {
    pool: SqlitePool,
    /// Maintainer-configurable thresholds for Commons contributors
    /// Loaded from governance/config/commons-contributor-thresholds.yml (legacy)
    commons_contributor_thresholds: Option<CommonsContributorThresholdsConfig>,
    /// ConfigReader for governance-controlled configuration (preferred)
    config_reader: Option<Arc<ConfigReader>>,
    /// Phase calculator for adaptive parameters (mining pool weight caps)
    phase_calculator: Option<GovernancePhaseCalculator>,
}

impl EconomicNodeRegistry {
    pub fn new(pool: SqlitePool) -> Self {
        Self { 
            pool: pool.clone(),
            commons_contributor_thresholds: None,
            config_reader: None,
            // Automatically create phase calculator for mining pool weight caps
            phase_calculator: Some(GovernancePhaseCalculator::new(pool)),
        }
    }

    /// Create registry with Commons contributor thresholds loaded from config
    pub fn with_thresholds(
        pool: SqlitePool,
        thresholds: Option<CommonsContributorThresholdsConfig>,
    ) -> Self {
        Self {
            pool: pool.clone(),
            commons_contributor_thresholds: thresholds,
            config_reader: None,
            // Automatically create phase calculator for mining pool weight caps
            phase_calculator: Some(GovernancePhaseCalculator::new(pool)),
        }
    }

    /// Create registry with phase calculator for adaptive parameters
    pub fn with_phase_calculator(
        pool: SqlitePool,
        phase_calculator: Option<GovernancePhaseCalculator>,
    ) -> Self {
        Self {
            pool,
            commons_contributor_thresholds: None,
            config_reader: None,
            phase_calculator,
        }
    }

    /// Create registry with ConfigReader for governance-controlled configuration
    pub fn with_config_reader(
        pool: SqlitePool,
        config_reader: Option<Arc<ConfigReader>>,
        phase_calculator: Option<GovernancePhaseCalculator>,
    ) -> Self {
        Self {
            pool,
            commons_contributor_thresholds: None,
            config_reader,
            phase_calculator,
        }
    }

    /// Set phase calculator (for runtime configuration updates)
    pub fn set_phase_calculator(&mut self, phase_calculator: Option<GovernancePhaseCalculator>) {
        self.phase_calculator = phase_calculator;
    }

    /// Set Commons contributor thresholds (for runtime configuration updates)
    pub fn set_commons_contributor_thresholds(
        &mut self,
        thresholds: Option<CommonsContributorThresholdsConfig>,
    ) {
        self.commons_contributor_thresholds = thresholds;
    }

    /// Get Commons contributor thresholds
    pub fn get_commons_contributor_thresholds(
        &self,
    ) -> Result<Option<&CommonsContributorThresholdsConfig>, GovernanceError> {
        Ok(self.commons_contributor_thresholds.as_ref())
    }

    /// Register a new economic node with qualification proof
    pub async fn register_economic_node(
        &self,
        node_type: NodeType,
        entity_name: &str,
        public_key: &str,
        qualification_data: &QualificationProof,
        created_by: Option<&str>,
    ) -> Result<i32, GovernanceError> {
        // Check for duplicate public key
        let existing = sqlx::query("SELECT id FROM economic_nodes WHERE public_key = ?")
            .bind(public_key)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                GovernanceError::DatabaseError(format!("Failed to check for duplicate: {}", e))
            })?;

        if existing.is_some() {
            return Err(GovernanceError::CryptoError(format!(
                "Node with public key {} already registered",
                public_key
            )));
        }

        // Verify qualification meets thresholds (cryptographic verification)
        let verified = self
            .verify_qualification(node_type.clone(), qualification_data)
            .await?;
        if !verified {
            return Err(GovernanceError::CryptoError(
                "Node does not meet qualification thresholds".to_string(),
            ));
        }

        // Verify cryptographic proofs (trust-minimized activation)
        let crypto_verified = self
            .verify_cryptographic_proofs(node_type.clone(), qualification_data, public_key)
            .await?;
        if !crypto_verified {
            return Err(GovernanceError::CryptoError(
                "Cryptographic proof verification failed".to_string(),
            ));
        }

        // Calculate initial weight
        let weight = self
            .calculate_weight(node_type.clone(), qualification_data)
            .await?;

        // Auto-activate if cryptographic proofs verify (trust-minimized, no maintainer approval)
        let status = if crypto_verified { "active" } else { "pending" };
        let now = chrono::Utc::now();

        // Insert into database with auto-activation
        let result = sqlx::query(
            r#"
            INSERT INTO economic_nodes 
            (node_type, entity_name, public_key, qualification_data, weight, status, registered_at, verified_at, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(node_type.as_str())
        .bind(entity_name)
        .bind(public_key)
        .bind(serde_json::to_string(qualification_data)?)
        .bind(weight)
        .bind(status)
        .bind(now)
        .bind(if crypto_verified { Some(now) } else { None })
        .bind(created_by)
        .execute(&self.pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to register node: {}", e)))?;

        let node_id = result.last_insert_rowid() as i32;
        if crypto_verified {
            info!("Auto-activated economic node {} (ID: {}) - cryptographic proofs verified", entity_name, node_id);
        } else {
            warn!("Registered economic node {} (ID: {}) with pending status - cryptographic proof verification failed", entity_name, node_id);
        }
        Ok(node_id)
    }

    /// Verify that a node meets qualification thresholds
    pub async fn verify_qualification(
        &self,
        node_type: NodeType,
        qualification_data: &QualificationProof,
    ) -> Result<bool, GovernanceError> {
        let thresholds = node_type.qualification_thresholds();

        // Check hashpower threshold (mining pools)
        if let Some(min_hashpower) = thresholds.minimum_hashpower_percent {
            if let Some(hashpower_proof) = &qualification_data.hashpower_proof {
                if hashpower_proof.percentage < min_hashpower {
                    warn!(
                        "Hashpower {}% below threshold {}%",
                        hashpower_proof.percentage, min_hashpower
                    );
                    return Ok(false);
                }
            } else {
                warn!("Hashpower proof required for mining pools");
                return Ok(false);
            }
        }

        // Check holdings threshold
        if let Some(min_holdings) = thresholds.minimum_holdings_btc {
            if let Some(holdings_proof) = &qualification_data.holdings_proof {
                if holdings_proof.total_btc < min_holdings as f64 {
                    warn!(
                        "Holdings {} BTC below threshold {} BTC",
                        holdings_proof.total_btc, min_holdings
                    );
                    return Ok(false);
                }
            } else {
                warn!("Holdings proof required for this node type");
                return Ok(false);
            }
        }

        // Check volume threshold (BTC, on-chain verifiable)
        if let Some(min_volume) = thresholds.minimum_volume_btc {
            if let Some(volume_proof) = &qualification_data.volume_proof {
                let volume = if node_type == NodeType::Exchange {
                    volume_proof.daily_volume_btc
                } else {
                    volume_proof.monthly_volume_btc
                };

                if volume < min_volume {
                    warn!("Volume {} BTC below threshold {} BTC", volume, min_volume);
                    return Ok(false);
                }
            } else {
                warn!("Volume proof required for this node type");
                return Ok(false);
            }
        }

        // Check Commons contributor thresholds (maintainer-configurable)
        if node_type == NodeType::CommonsContributor {
            return self.verify_commons_contributor_qualification(qualification_data).await;
        }

        Ok(true)
    }

    /// Verify Commons contributor qualification using maintainer-configurable thresholds
    async fn verify_commons_contributor_qualification(
        &self,
        qualification_data: &QualificationProof,
    ) -> Result<bool, GovernanceError> {
        // Prefer ConfigReader (governance-controlled) over YAML config
        if let Some(ref config_reader) = self.config_reader {
            return self.verify_commons_contributor_with_config_reader(
                qualification_data,
                config_reader,
            ).await;
        }

        // Fallback to YAML config (legacy)
        let thresholds_config = match &self.commons_contributor_thresholds {
            Some(config) => &config.commons_contributor_thresholds,
            None => {
                warn!("Commons contributor thresholds not configured, using defaults");
                // Fall back to hardcoded defaults if config not loaded
                return self.verify_commons_contributor_defaults(qualification_data).await;
            }
        };

        let proof = match &qualification_data.commons_contributor_proof {
            Some(p) => p,
            None => {
                warn!("Commons contributor proof required");
                return Ok(false);
            }
        };

        let mut qualifications_met = Vec::new();

        // Check merge mining threshold
        if thresholds_config.merge_mining.enabled {
            if let Some(merge_proof) = &proof.merge_mining_proof {
                if merge_proof.total_revenue_btc >= thresholds_config.merge_mining.minimum_contribution_btc
                    && merge_proof.period_days >= thresholds_config.measurement_period_days
                {
                    qualifications_met.push("merge_mining");
                }
            }
        }

        // Check fee forwarding threshold
        if thresholds_config.fee_forwarding.enabled {
            if let Some(fee_proof) = &proof.fee_forwarding_proof {
                if fee_proof.total_fees_forwarded_btc >= thresholds_config.fee_forwarding.minimum_contribution_btc
                    && fee_proof.period_days >= thresholds_config.measurement_period_days
                {
                    qualifications_met.push("fee_forwarding");
                }
            }
        }

        // Check zap threshold
        if thresholds_config.zaps.enabled {
            if let Some(zap_proof) = &proof.zap_proof {
                if zap_proof.total_zaps_btc >= thresholds_config.zaps.minimum_contribution_btc
                    && zap_proof.period_days >= thresholds_config.measurement_period_days
                {
                    qualifications_met.push("zaps");
                }
            }
        }

        // Check marketplace sales threshold (BTC via BIP70 payments)
        if thresholds_config.marketplace.enabled {
            if let Some(marketplace_proof) = &proof.marketplace_sales_proof {
                let min_btc = thresholds_config.marketplace.minimum_sales_btc
                    .unwrap_or(0.0);
                if marketplace_proof.total_sales_btc >= min_btc
                    && marketplace_proof.period_days >= thresholds_config.measurement_period_days
                {
                    qualifications_met.push("marketplace");
                }
            }
        }

        // Apply qualification logic (OR or AND)
        let qualified = match thresholds_config.qualification_logic.as_str() {
            "OR" => !qualifications_met.is_empty(),
            "AND" => {
                // Count enabled thresholds
                let enabled_count = [
                    thresholds_config.merge_mining.enabled,
                    thresholds_config.fee_forwarding.enabled,
                    thresholds_config.zaps.enabled,
                    thresholds_config.marketplace.enabled,
                ]
                .iter()
                .filter(|&&enabled| enabled)
                .count();
                
                qualifications_met.len() == enabled_count
            }
            _ => {
                warn!("Invalid qualification_logic: {}, defaulting to OR", thresholds_config.qualification_logic);
                !qualifications_met.is_empty()
            }
        };

        if !qualified {
            warn!(
                "Commons contributor qualification not met. Met: {:?}, Logic: {}",
                qualifications_met, thresholds_config.qualification_logic
            );
        }

        Ok(qualified)
    }

    /// Verify Commons contributor qualification using ConfigReader (governance-controlled)
    async fn verify_commons_contributor_with_config_reader(
        &self,
        qualification_data: &QualificationProof,
        config_reader: &ConfigReader,
    ) -> Result<bool, GovernanceError> {
        let proof = match &qualification_data.commons_contributor_proof {
            Some(p) => p,
            None => {
                warn!("Commons contributor proof required");
                return Ok(false);
            }
        };

        let mut qualifications_met = Vec::new();
        let measurement_period = config_reader.get_commons_contributor_measurement_period().await?;

        // Check merge mining threshold
        let merge_mining_threshold = config_reader.get_commons_contributor_threshold("merge_mining").await?;
        if merge_mining_threshold > 0.0 {
            if let Some(merge_proof) = &proof.merge_mining_proof {
                if merge_proof.total_revenue_btc >= merge_mining_threshold
                    && merge_proof.period_days >= measurement_period
                {
                    qualifications_met.push("merge_mining");
                }
            }
        }

        // Check fee forwarding threshold
        let fee_forwarding_threshold = config_reader.get_commons_contributor_threshold("fee_forwarding").await?;
        if fee_forwarding_threshold > 0.0 {
            if let Some(fee_proof) = &proof.fee_forwarding_proof {
                if fee_proof.total_fees_forwarded_btc >= fee_forwarding_threshold
                    && fee_proof.period_days >= measurement_period
                {
                    qualifications_met.push("fee_forwarding");
                }
            }
        }

        // Check zap threshold
        let zap_threshold = config_reader.get_commons_contributor_threshold("zaps").await?;
        if zap_threshold > 0.0 {
            if let Some(zap_proof) = &proof.zap_proof {
                if zap_proof.total_zaps_btc >= zap_threshold
                    && zap_proof.period_days >= measurement_period
                {
                    qualifications_met.push("zaps");
                }
            }
        }

        // Check marketplace sales threshold
        let marketplace_threshold = config_reader.get_commons_contributor_threshold("marketplace").await?;
        if marketplace_threshold > 0.0 {
            if let Some(marketplace_proof) = &proof.marketplace_sales_proof {
                if marketplace_proof.total_sales_btc >= marketplace_threshold
                    && marketplace_proof.period_days >= measurement_period
                {
                    qualifications_met.push("marketplace");
                }
            }
        }

        // Get qualification logic from config (default to OR)
        let qualification_logic = config_reader.get_string("commons_contributor_qualification_logic", "OR").await?;
        
        // Apply qualification logic
        let qualified = match qualification_logic.as_str() {
            "OR" => !qualifications_met.is_empty(),
            "AND" => {
                // Count enabled thresholds (non-zero thresholds are enabled)
                let enabled_count = [
                    merge_mining_threshold > 0.0,
                    fee_forwarding_threshold > 0.0,
                    zap_threshold > 0.0,
                    marketplace_threshold > 0.0,
                ]
                .iter()
                .filter(|&&enabled| enabled)
                .count();
                
                qualifications_met.len() == enabled_count && enabled_count > 0
            }
            _ => {
                warn!("Invalid qualification_logic: {}, defaulting to OR", qualification_logic);
                !qualifications_met.is_empty()
            }
        };

        if !qualified {
            warn!(
                "Commons contributor qualification not met. Met: {:?}, Logic: {}",
                qualifications_met, qualification_logic
            );
        }

        Ok(qualified)
    }

    /// Verify cryptographic proofs for trust-minimized auto-activation
    /// Returns true if all required cryptographic proofs verify
    async fn verify_cryptographic_proofs(
        &self,
        node_type: NodeType,
        qualification_data: &QualificationProof,
        public_key: &str,
    ) -> Result<bool, GovernanceError> {
        use crate::validation::signatures::SignatureValidator;
        let validator = SignatureValidator::new();

        match node_type {
            NodeType::MiningPool => {
                // Mining pools: Verify coinbase signatures (on-chain proof)
                if let Some(hashpower_proof) = &qualification_data.hashpower_proof {
                    // Verify blocks are actually mined (on-chain verification)
                    // In production, this would query the blockchain to verify block hashes
                    // For now, we verify the proof structure is valid
                    if hashpower_proof.blocks_mined.is_empty() {
                        warn!("Mining pool proof missing block hashes");
                        return Ok(false);
                    }
                    // TODO: Add actual blockchain verification of block hashes
                    // For now, structural validation passes
                    Ok(true)
                } else {
                    warn!("Mining pool missing hashpower proof");
                    Ok(false)
                }
            }
            NodeType::Exchange | NodeType::Custodian | NodeType::MajorHolder => {
                // Holdings: Verify signature challenge (cryptographic proof of control)
                if let Some(holdings_proof) = &qualification_data.holdings_proof {
                    if holdings_proof.addresses.is_empty() {
                        warn!("Holdings proof missing addresses");
                        return Ok(false);
                    }
                    // Verify signature challenge proves control of addresses
                    // Signature signs: public_key || addresses || timestamp (from proof)
                    // Note: In production, timestamp should be from proof and validated (not expired)
                    let challenge_message = format!("{}||{}", 
                        public_key, 
                        holdings_proof.addresses.join(",")
                    );
                    let verified = validator.verify_signature(
                        &challenge_message,
                        &holdings_proof.signature_challenge,
                        public_key,
                    )?;
                    if !verified {
                        warn!("Holdings signature challenge verification failed");
                        return Ok(false);
                    }
                    Ok(true)
                } else {
                    warn!("Holdings proof required for this node type");
                    Ok(false)
                }
            }
            NodeType::CommonsContributor => {
                // Commons contributors: Verify on-chain proofs
                if let Some(commons_proof) = &qualification_data.commons_contributor_proof {
                    let mut proofs_verified = 0;
                    let mut proofs_required = 0;

                    // Verify merge mining proof (coinbase signatures)
                    if let Some(merge_proof) = &commons_proof.merge_mining_proof {
                        proofs_required += 1;
                        // Verify block hashes and coinbase signatures
                        if !merge_proof.blocks_mined.is_empty() {
                            // TODO: Add actual blockchain verification
                            proofs_verified += 1;
                        }
                    }

                    // Verify fee forwarding proof (on-chain transactions)
                    if let Some(fee_proof) = &commons_proof.fee_forwarding_proof {
                        proofs_required += 1;
                        // Verify transactions exist on-chain
                        if !fee_proof.blocks_with_forwarding.is_empty() {
                            // TODO: Add actual blockchain verification
                            proofs_verified += 1;
                        }
                    }

                    // Verify zap proof (Lightning payment proofs)
                    if let Some(zap_proof) = &commons_proof.zap_proof {
                        proofs_required += 1;
                        // Verify zap events exist (Nostr + Lightning)
                        if !zap_proof.zap_events.is_empty() {
                            // TODO: Add actual Nostr/Lightning verification
                            proofs_verified += 1;
                        }
                    }

                    // Verify marketplace sales proof (BIP70 payments)
                    if let Some(marketplace_proof) = &commons_proof.marketplace_sales_proof {
                        proofs_required += 1;
                        // Verify BIP70 payment records
                        if !marketplace_proof.module_payments.is_empty() {
                            // TODO: Add actual BIP70 verification
                            proofs_verified += 1;
                        }
                    }

                    // At least one proof must verify
                    Ok(proofs_verified > 0 && proofs_verified == proofs_required)
                } else {
                    warn!("Commons contributor proof required");
                    Ok(false)
                }
            }
        }
    }

    /// Fallback verification using hardcoded defaults if config not loaded
    async fn verify_commons_contributor_defaults(
        &self,
        qualification_data: &QualificationProof,
    ) -> Result<bool, GovernanceError> {
        let proof = match &qualification_data.commons_contributor_proof {
            Some(p) => p,
            None => return Ok(false),
        };

        // Default thresholds (from documentation)
        let mut qualifications_met = Vec::new();

        if let Some(merge_proof) = &proof.merge_mining_proof {
            if merge_proof.total_revenue_btc >= 0.01 && merge_proof.period_days >= 90 {
                qualifications_met.push("merge_mining");
            }
        }

        if let Some(fee_proof) = &proof.fee_forwarding_proof {
            if fee_proof.total_fees_forwarded_btc >= 0.1 && fee_proof.period_days >= 90 {
                qualifications_met.push("fee_forwarding");
            }
        }

        if let Some(zap_proof) = &proof.zap_proof {
            if zap_proof.total_zaps_btc >= 0.01 && zap_proof.period_days >= 90 {
                qualifications_met.push("zaps");
            }
        }

        // Default to OR logic
        Ok(!qualifications_met.is_empty())
    }

    /// Calculate weight for an economic node
    pub async fn calculate_weight(
        &self,
        node_type: NodeType,
        qualification_data: &QualificationProof,
    ) -> Result<f64, GovernanceError> {
        match node_type {
            NodeType::MiningPool => {
                // Weight = hashpower percentage, capped by phase-based limit
                if let Some(hashpower_proof) = &qualification_data.hashpower_proof {
                    let base_weight = hashpower_proof.percentage / 100.0;
                    
                    // Apply phase-based weight cap if phase calculator is available
                    let capped_weight = if let Some(ref phase_calc) = self.phase_calculator {
                        match phase_calc.get_adaptive_parameters().await {
                            Ok(adaptive_params) => {
                                // Cap individual pool weight to prevent dominance
                                // Cap is percentage (0.10 = 10%, 0.20 = 20%)
                                base_weight.min(adaptive_params.mining_pool_weight_cap)
                            }
                            Err(e) => {
                                warn!(
                                    "Failed to get adaptive parameters for mining pool cap: {}. Using uncapped weight.",
                                    e
                                );
                                base_weight
                            }
                        }
                    } else {
                        // No phase calculator available, use uncapped weight
                        base_weight
                    };
                    
                    Ok(capped_weight)
                } else {
                    Err(GovernanceError::CryptoError(
                        "Hashpower proof required for mining pools".to_string(),
                    ))
                }
            }
            NodeType::Exchange => {
                // Weight = 70% holdings + 30% volume (both on-chain verifiable)
                let holdings_weight =
                    if let Some(holdings_proof) = &qualification_data.holdings_proof {
                        // Normalize to 0-1 scale (10K BTC = 1.0)
                        (holdings_proof.total_btc / 10_000.0).min(1.0) * 0.7
                    } else {
                        0.0
                    };

                let volume_weight = if let Some(volume_proof) = &qualification_data.volume_proof {
                    // Normalize to 0-1 scale (100 BTC daily = 1.0)
                    (volume_proof.daily_volume_btc / 100.0).min(1.0) * 0.3
                } else {
                    0.0
                };

                Ok(holdings_weight + volume_weight)
            }
            NodeType::Custodian => {
                // Weight = holdings percentage
                if let Some(holdings_proof) = &qualification_data.holdings_proof {
                    // Normalize to 0-1 scale (10K BTC = 1.0)
                    Ok((holdings_proof.total_btc / 10_000.0).min(1.0))
                } else {
                    Err(GovernanceError::CryptoError(
                        "Holdings proof required for custodians".to_string(),
                    ))
                }
            }
            NodeType::MajorHolder => {
                // Weight = holdings percentage
                if let Some(holdings_proof) = &qualification_data.holdings_proof {
                    // Normalize to 0-1 scale (5K BTC = 1.0)
                    Ok((holdings_proof.total_btc / 5_000.0).min(1.0))
                } else {
                    Err(GovernanceError::CryptoError(
                        "Holdings proof required for major holders".to_string(),
                    ))
                }
            }
            NodeType::CommonsContributor => {
                // Weight = sqrt(total_contribution_btc / normalization_factor)
                // Uses quadratic weighting for fairness
                self.calculate_commons_contributor_weight(qualification_data).await
            }
        }
    }

    /// Calculate weight for Commons contributor using quadratic formula
    async fn calculate_commons_contributor_weight(
        &self,
        qualification_data: &QualificationProof,
    ) -> Result<f64, GovernanceError> {
        let proof = match &qualification_data.commons_contributor_proof {
            Some(p) => p,
            None => {
                return Err(GovernanceError::CryptoError(
                    "Commons contributor proof required".to_string(),
                ));
            }
        };

        // Get normalization factor from config, or use default
        let normalization_factor = self
            .commons_contributor_thresholds
            .as_ref()
            .and_then(|c| Some(c.weight_calculation.normalization_factor))
            .unwrap_or(1.0);

        let mut total_contribution_btc = 0.0;

        // Sum all contribution types
        if let Some(merge_proof) = &proof.merge_mining_proof {
            total_contribution_btc += merge_proof.total_revenue_btc;
        }

        if let Some(fee_proof) = &proof.fee_forwarding_proof {
            total_contribution_btc += fee_proof.total_fees_forwarded_btc;
        }

        if let Some(zap_proof) = &proof.zap_proof {
            total_contribution_btc += zap_proof.total_zaps_btc;
        }

        // Convert USD revenue to BTC using moving average price (smooths volatility)
        // This prevents contributors from being penalized by sudden price movements
        let btc_price_ma = match &self.btc_price_service {
            Some(service) => service.get_moving_average(),
            None => {
                warn!("BTC price service not available, using default price");
                50000.0 // Fallback default
            }
        };

        // Marketplace sales are already in BTC (BIP70 payments)
        if let Some(marketplace_proof) = &proof.marketplace_sales_proof {
            total_contribution_btc += marketplace_proof.total_sales_btc;
        }

        if let Some(treasury_proof) = &proof.treasury_sales_proof {
            // Convert USD to BTC using moving average (prevents volatility penalty)
            let treasury_btc = treasury_proof.total_sales_usd / btc_price_ma;
            total_contribution_btc += treasury_btc;
            info!(
                "Converted treasury sales: ${:.2} USD -> {:.8} BTC (using ${:.2} MA price)",
                treasury_proof.total_sales_usd, treasury_btc, btc_price_ma
            );
        }

        if let Some(service_proof) = &proof.service_sales_proof {
            // Convert USD to BTC using moving average (prevents volatility penalty)
            let service_btc = service_proof.total_sales_usd / btc_price_ma;
            total_contribution_btc += service_btc;
            info!(
                "Converted service sales: ${:.2} USD -> {:.8} BTC (using ${:.2} MA price)",
                service_proof.total_sales_usd, service_btc, btc_price_ma
            );
        }

        // Apply quadratic formula: sqrt(total_contribution_btc / normalization_factor)
        let weight = (total_contribution_btc / normalization_factor).sqrt();

        // Apply minimum weight if configured
        let min_weight = self
            .commons_contributor_thresholds
            .as_ref()
            .and_then(|c| Some(c.weight_calculation.minimum_weight))
            .unwrap_or(0.01);

        Ok(weight.max(min_weight))
    }

    /// Get all active economic nodes
    pub async fn get_active_nodes(&self) -> Result<Vec<EconomicNode>, GovernanceError> {
        let rows = sqlx::query(
            r#"
            SELECT id, node_type, entity_name, public_key, qualification_data, 
                   weight, status, registered_at, verified_at, last_verified_at, 
                   created_by, notes
            FROM economic_nodes 
            WHERE status = 'active'
            ORDER BY weight DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to fetch nodes: {}", e)))?;

        let mut nodes = Vec::new();
        for row in rows {
            let node_type =
                NodeType::from_str(&row.get::<String, _>("node_type")).ok_or_else(|| {
                    GovernanceError::CryptoError(format!(
                        "Invalid node type: {}",
                        row.get::<String, _>("node_type")
                    ))
                })?;

            let status =
                NodeStatus::from_str(&row.get::<String, _>("status")).ok_or_else(|| {
                    GovernanceError::CryptoError(format!(
                        "Invalid status: {}",
                        row.get::<String, _>("status")
                    ))
                })?;

            let qualification_data: String = row.get("qualification_data");
            let qualification_data: QualificationProof = serde_json::from_str(&qualification_data)
                .map_err(|e| {
                    GovernanceError::CryptoError(format!("Invalid qualification data: {}", e))
                })?;

            nodes.push(EconomicNode {
                id: row.get("id"),
                node_type,
                entity_name: row.get("entity_name"),
                public_key: row.get("public_key"),
                qualification_data: serde_json::to_value(&qualification_data)
                    .unwrap_or_else(|_| serde_json::json!({})),
                weight: row.get("weight"),
                status,
                registered_at: row.get("registered_at"),
                verified_at: row.get("verified_at"),
                last_verified_at: row.get("last_verified_at"),
                created_by: row.get("created_by"),
                notes: row.get("notes"),
            });
        }

        Ok(nodes)
    }

    /// Get node by ID
    pub async fn get_node_by_id(&self, node_id: i32) -> Result<EconomicNode, GovernanceError> {
        let row = sqlx::query(
            r#"
            SELECT id, node_type, entity_name, public_key, qualification_data, 
                   weight, status, registered_at, verified_at, last_verified_at, 
                   created_by, notes
            FROM economic_nodes 
            WHERE id = ?
            "#,
        )
        .bind(node_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| GovernanceError::DatabaseError(format!("Failed to fetch node: {}", e)))?;

        let row = row.ok_or_else(|| {
            GovernanceError::CryptoError(format!("Node with ID {} not found", node_id))
        })?;

        let node_type =
            NodeType::from_str(&row.get::<String, _>("node_type")).ok_or_else(|| {
                GovernanceError::CryptoError(format!(
                    "Invalid node type: {}",
                    row.get::<String, _>("node_type")
                ))
            })?;

        let status = NodeStatus::from_str(&row.get::<String, _>("status")).ok_or_else(|| {
            GovernanceError::CryptoError(format!(
                "Invalid status: {}",
                row.get::<String, _>("status")
            ))
        })?;

        let qualification_data: String = row.get("qualification_data");
        let qualification_data: QualificationProof = serde_json::from_str(&qualification_data)
            .map_err(|e| {
                GovernanceError::CryptoError(format!("Invalid qualification data: {}", e))
            })?;

        Ok(EconomicNode {
            id: row.get("id"),
            node_type,
            entity_name: row.get("entity_name"),
            public_key: row.get("public_key"),
            qualification_data: serde_json::to_value(&qualification_data)
                .unwrap_or_else(|_| serde_json::json!({})),
            weight: row.get("weight"),
            status,
            registered_at: row.get("registered_at"),
            verified_at: row.get("verified_at"),
            last_verified_at: row.get("last_verified_at"),
            created_by: row.get("created_by"),
            notes: row.get("notes"),
        })
    }

    /// Update node status
    pub async fn update_node_status(
        &self,
        node_id: i32,
        status: NodeStatus,
    ) -> Result<(), GovernanceError> {
        sqlx::query("UPDATE economic_nodes SET status = ? WHERE id = ?")
            .bind(status.as_str())
            .bind(node_id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                GovernanceError::DatabaseError(format!("Failed to update node status: {}", e))
            })?;

        info!("Updated node {} status to {}", node_id, status.as_str());
        Ok(())
    }

    /// Recalculate weights for all nodes based on current qualification data
    pub async fn recalculate_all_weights(&self) -> Result<(), GovernanceError> {
        let nodes = sqlx::query("SELECT id, node_type, qualification_data FROM economic_nodes")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| GovernanceError::DatabaseError(format!("Failed to fetch nodes: {}", e)))?;

        for row in nodes {
            let node_id: i32 = row.get("id");
            let node_type_str: String = row.get("node_type");
            let node_type = NodeType::from_str(&node_type_str).ok_or_else(|| {
                GovernanceError::CryptoError(format!("Invalid node type: {}", node_type_str))
            })?;

            let qualification_data_str: String = row.get("qualification_data");
            let qualification_data: QualificationProof =
                serde_json::from_str(&qualification_data_str).map_err(|e| {
                    GovernanceError::CryptoError(format!("Invalid qualification data: {}", e))
                })?;

            let new_weight = self
                .calculate_weight(node_type, &qualification_data)
                .await?;

            sqlx::query("UPDATE economic_nodes SET weight = ? WHERE id = ?")
                .bind(new_weight)
                .bind(node_id)
                .execute(&self.pool)
                .await
                .map_err(|e| {
                    GovernanceError::DatabaseError(format!("Failed to update weight: {}", e))
                })?;
        }

        info!("Recalculated weights for all nodes");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::Database;
    use crate::economic_nodes::types::{
        ContactInfo, HoldingsProof, QualificationProof, VolumeProof,
    };

    async fn setup_test_registry() -> EconomicNodeRegistry {
        let db = Database::new_in_memory().await.unwrap();
        EconomicNodeRegistry::new(db.pool().unwrap().clone())
    }

    fn create_mining_pool_qualification(hashpower: f64) -> serde_json::Value {
        serde_json::json!({
            "node_type": "mining_pool",
            "hashpower_proof": {
                "blocks_mined": ["block1", "block2"],
                "time_period_days": 30,
                "total_network_blocks": 100,
                "percentage": hashpower
            },
            "contact_info": {
                "entity_name": "Test Pool",
                "contact_email": "test@example.com"
            }
        })
    }

    fn create_exchange_qualification(
        holdings_btc: f64,
        daily_volume_btc: f64,
    ) -> serde_json::Value {
        serde_json::json!({
            "node_type": "exchange",
            "holdings_proof": {
                "addresses": ["addr1"],
                "total_btc": holdings_btc,
                "signature_challenge": "challenge"
            },
            "volume_proof": {
                "daily_volume_btc": daily_volume_btc,
                "monthly_volume_btc": daily_volume_btc * 30.0,
                "transaction_hashes": ["tx1", "tx2"]
            },
            "contact_info": {
                "entity_name": "Test Exchange",
                "contact_email": "test@example.com"
            }
        })
    }

    #[tokio::test]
    async fn test_calculate_weight_mining_pool() {
        let registry = setup_test_registry().await;
        let qual_json = create_mining_pool_qualification(5.0); // 5% hashpower
        let qual: QualificationProof = serde_json::from_value(qual_json.clone()).unwrap();

        let weight = registry
            .calculate_weight(NodeType::MiningPool, &qual)
            .await
            .unwrap();
        // 5% hashpower = 0.05 weight (below cap, so uncapped)
        assert!((weight - 0.05).abs() < 0.001, "Mining pool weight should be hashpower / 100 when below cap");
    }

    #[tokio::test]
    async fn test_calculate_weight_mining_pool_capped() {
        let registry = setup_test_registry().await;
        // Test with 35% hashpower - should be capped at phase-based limit (10% or 20%)
        let qual_json = create_mining_pool_qualification(35.0);
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let weight = registry
            .calculate_weight(NodeType::MiningPool, &qual)
            .await
            .unwrap();
        // Should be capped at phase-based limit (Early/Mature = 10%, Growth = 20%)
        // Since we're in early phase by default (no blocks), should be capped at 0.10
        assert!(weight <= 0.20, "Mining pool weight should be capped at phase-based limit (max 20%)");
        assert!(weight >= 0.10, "Mining pool weight should be at least 10% cap in early phase");
    }

    #[tokio::test]
    async fn test_calculate_weight_mining_pool_small_below_cap() {
        let registry = setup_test_registry().await;
        // Test with 8% hashpower - should be uncapped (below 10% cap)
        let qual_json = create_mining_pool_qualification(8.0);
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let weight = registry
            .calculate_weight(NodeType::MiningPool, &qual)
            .await
            .unwrap();
        // 8% hashpower = 0.08 weight (below 10% cap, so uncapped)
        assert!((weight - 0.08).abs() < 0.001, "Small mining pool should not be capped");
    }

    #[tokio::test]
    async fn test_calculate_weight_mining_pool_no_proof() {
        let registry = setup_test_registry().await;
        let qual = QualificationProof {
            node_type: NodeType::MiningPool,
            hashpower_proof: None,
            holdings_proof: None,
            volume_proof: None,
            commons_contributor_proof: None,
            contact_info: ContactInfo {
                entity_name: "Test".to_string(),
                contact_email: "test@example.com".to_string(),
                website: None,
                github_username: None,
            },
        };

        let result = registry.calculate_weight(NodeType::MiningPool, &qual).await;
        assert!(result.is_err(), "Should fail without hashpower proof");
    }

    #[tokio::test]
    async fn test_calculate_weight_exchange() {
        let registry = setup_test_registry().await;
        let qual_json = create_exchange_qualification(5000.0, 50.0); // 50 BTC daily volume
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let weight = registry
            .calculate_weight(NodeType::Exchange, &qual)
            .await
            .unwrap();
        // holdings: 5000/10000 * 0.7 = 0.35
        // volume: 50M/100M * 0.3 = 0.15
        // total: 0.5
        assert!((weight - 0.5).abs() < 0.01, "Exchange weight should be 0.5");
    }

    #[tokio::test]
    async fn test_calculate_weight_exchange_capped() {
        let registry = setup_test_registry().await;
        let qual_json = create_exchange_qualification(20000.0, 200.0); // 200 BTC daily volume
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let weight = registry
            .calculate_weight(NodeType::Exchange, &qual)
            .await
            .unwrap();
        // Should be capped at 1.0
        assert!(weight <= 1.0, "Weight should be capped at 1.0");
    }

    #[tokio::test]
    async fn test_calculate_weight_custodian() {
        let registry = setup_test_registry().await;
        let qual = QualificationProof {
            node_type: NodeType::Custodian,
            hashpower_proof: None,
            holdings_proof: Some(HoldingsProof {
                addresses: vec!["addr1".to_string()],
                total_btc: 5000.0,
                signature_challenge: "challenge".to_string(),
            }),
            volume_proof: None,
            commons_contributor_proof: None,
            contact_info: ContactInfo {
                entity_name: "Test".to_string(),
                contact_email: "test@example.com".to_string(),
                website: None,
                github_username: None,
            },
        };

        let weight = registry
            .calculate_weight(NodeType::Custodian, &qual)
            .await
            .unwrap();
        // 5000/10000 = 0.5
        assert!(
            (weight - 0.5).abs() < 0.01,
            "Custodian weight should be 0.5"
        );
    }


    #[tokio::test]
    async fn test_calculate_weight_major_holder() {
        let registry = setup_test_registry().await;
        let qual = QualificationProof {
            node_type: NodeType::MajorHolder,
            hashpower_proof: None,
            holdings_proof: Some(HoldingsProof {
                addresses: vec!["addr1".to_string()],
                total_btc: 2500.0,
                signature_challenge: "challenge".to_string(),
            }),
            volume_proof: None,
            commons_contributor_proof: None,
            contact_info: ContactInfo {
                entity_name: "Test".to_string(),
                contact_email: "test@example.com".to_string(),
                website: None,
                github_username: None,
            },
        };

        let weight = registry
            .calculate_weight(NodeType::MajorHolder, &qual)
            .await
            .unwrap();
        // 2500/5000 = 0.5
        assert!(
            (weight - 0.5).abs() < 0.01,
            "Major holder weight should be 0.5"
        );
    }

    #[tokio::test]
    async fn test_verify_qualification_mining_pool_meets_threshold() {
        let registry = setup_test_registry().await;
        let qual_json = create_mining_pool_qualification(35.0); // Above 1% threshold (from types.rs)
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let verified = registry
            .verify_qualification(NodeType::MiningPool, &qual)
            .await
            .unwrap();
        assert!(verified, "Should verify when hashpower meets threshold");
    }

    #[tokio::test]
    async fn test_verify_qualification_mining_pool_below_threshold() {
        let registry = setup_test_registry().await;
        let qual_json = create_mining_pool_qualification(0.5); // Below 1% threshold
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let verified = registry
            .verify_qualification(NodeType::MiningPool, &qual)
            .await
            .unwrap();
        assert!(
            !verified,
            "Should not verify when hashpower below threshold"
        );
    }

    #[tokio::test]
    async fn test_verify_qualification_mining_pool_no_proof() {
        let registry = setup_test_registry().await;
        let qual = QualificationProof {
            node_type: NodeType::MiningPool,
            hashpower_proof: None,
            holdings_proof: None,
            volume_proof: None,
            commons_contributor_proof: None,
            contact_info: ContactInfo {
                entity_name: "Test".to_string(),
                contact_email: "test@example.com".to_string(),
                website: None,
                github_username: None,
            },
        };

        let verified = registry
            .verify_qualification(NodeType::MiningPool, &qual)
            .await
            .unwrap();
        assert!(!verified, "Should not verify without hashpower proof");
    }

    #[tokio::test]
    async fn test_verify_qualification_exchange_meets_threshold() {
        let registry = setup_test_registry().await;
        let qual_json = create_exchange_qualification(10000.0, 100.0); // Meets thresholds (10K BTC, 100 BTC daily)
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let verified = registry
            .verify_qualification(NodeType::Exchange, &qual)
            .await
            .unwrap();
        assert!(verified, "Should verify when exchange meets thresholds");
    }

    #[tokio::test]
    async fn test_verify_qualification_exchange_below_holdings() {
        let registry = setup_test_registry().await;
        let qual_json = create_exchange_qualification(9000.0, 100.0); // Below 10K BTC threshold
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let verified = registry
            .verify_qualification(NodeType::Exchange, &qual)
            .await
            .unwrap();
        assert!(!verified, "Should not verify when holdings below threshold");
    }

    #[tokio::test]
    async fn test_verify_qualification_exchange_below_volume() {
        let registry = setup_test_registry().await;
        let qual_json = create_exchange_qualification(10000.0, 90.0); // Below 100 BTC daily threshold
        let qual: QualificationProof = serde_json::from_value(qual_json).unwrap();

        let verified = registry
            .verify_qualification(NodeType::Exchange, &qual)
            .await
            .unwrap();
        assert!(!verified, "Should not verify when volume below threshold");
    }

    #[tokio::test]
    async fn test_get_node_by_id_not_found() {
        let registry = setup_test_registry().await;

        let result = registry.get_node_by_id(999).await;
        assert!(result.is_err(), "Should fail for non-existent node");
    }

    #[tokio::test]
    async fn test_get_active_nodes_empty() {
        let registry = setup_test_registry().await;

        let nodes = registry.get_active_nodes().await.unwrap();
        assert_eq!(
            nodes.len(),
            0,
            "Should return empty list when no active nodes"
        );
    }
}
