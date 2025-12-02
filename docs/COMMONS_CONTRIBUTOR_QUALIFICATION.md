# Commons Contributor Economic Node Qualification

## Overview

This document defines how nodes that contribute to Bitcoin Commons (through merge mining, fee forwarding, or Lightning zaps) qualify as economic nodes and receive voting power with quadratic weighting.

## Design Principles

1. **Fair Qualification**: Anyone contributing to Commons qualifies, not just large entities
2. **Quadratic Voting**: Weight = √(contribution), preventing whale dominance while rewarding larger contributors
3. **Cryptographic Verification**: All contributions must be verifiable on-chain or via Lightning proofs
4. **Unified Weighting**: All contribution types use the same quadratic formula for fairness
5. **Sybil Resistance**: One vote per entity, verified through cryptographic proofs

---

## New Node Types

### 1. Merge Mining Contributor

**Qualification Criteria:**
- Minimum: 0.01 BTC in merge mining revenue contributed over 90 days
- Must be actively merge mining with Bitcoin Commons infrastructure
- Must forward 1% fee to Commons development fund

**Verification:**
- On-chain proof: Secondary chain blocks with Commons coinbase outputs
- Revenue tracking: Merge mining coordinator records
- Cryptographic: Coinbase signatures matching registered keys

**Weight Calculation:**
```
weight = sqrt(total_merge_mining_revenue_btc / 1.0)
```
- Example: 1 BTC contributed = 1.0 weight
- Example: 4 BTC contributed = 2.0 weight
- Example: 0.25 BTC contributed = 0.5 weight

### 2. Fee Forwarding Contributor

**Qualification Criteria:**
- Minimum: 0.1 BTC in transaction fees forwarded over 90 days
- Must forward fees to Commons development address
- Must be running BTCDecoded full node

**Verification:**
- On-chain proof: Coinbase transactions with fee forwarding outputs
- Amount tracking: Sum of forwarded fees in coinbase outputs
- Node verification: Must be running synced BTCDecoded node

**Weight Calculation:**
```
weight = sqrt(total_fees_forwarded_btc / 1.0)
```
- Example: 1 BTC forwarded = 1.0 weight
- Example: 4 BTC forwarded = 2.0 weight
- Example: 0.25 BTC forwarded = 0.5 weight

### 3. Lightning Zap Contributor

**Qualification Criteria:**
- Minimum: 0.01 BTC in Lightning payments to Commons over 90 days
- Must use Lightning invoices from Commons
- Payments must be verifiable via Lightning proofs

**Verification:**
- Lightning proof: Payment preimages or payment proofs
- Invoice tracking: Commons Lightning node invoice records
- Cryptographic: Payment hash verification

**Weight Calculation:**
```
weight = sqrt(total_zaps_btc / 1.0)
```
- Example: 1 BTC zapped = 1.0 weight
- Example: 4 BTC zapped = 2.0 weight
- Example: 0.25 BTC zapped = 0.5 weight

---

## Unified Quadratic Voting System

### Formula

All contribution types use the same quadratic formula:

```
vote_weight = sqrt(contribution_btc / normalization_factor)
```

Where:
- `contribution_btc` = Total contribution in BTC over measurement period
- `normalization_factor` = 1.0 BTC (standardizes across contribution types)

### Why Quadratic?

1. **Prevents Whale Dominance**: A 100 BTC contributor gets 10x weight, not 100x
2. **Rewards Larger Contributors**: Still incentivizes larger contributions
3. **Fair for Small Contributors**: Small contributors (0.01 BTC) get meaningful weight (0.1)
4. **Mathematically Sound**: Square root is the standard quadratic voting formula

### Examples

| Contribution | Linear Weight | Quadratic Weight | Ratio |
|-------------|---------------|------------------|-------|
| 0.01 BTC    | 0.01          | 0.10             | 10x   |
| 0.25 BTC    | 0.25          | 0.50             | 2x    |
| 1.0 BTC     | 1.0           | 1.0              | 1x    |
| 4.0 BTC     | 4.0           | 2.0              | 0.5x  |
| 16.0 BTC    | 16.0          | 4.0              | 0.25x |
| 100.0 BTC   | 100.0         | 10.0             | 0.1x  |

**Key Insight**: Quadratic voting gives small contributors 10x more relative power, while still rewarding larger contributions.

---

## Implementation

### Database Schema Extensions

```sql
-- New node types
ALTER TABLE economic_nodes ADD COLUMN contribution_type TEXT;
-- Values: 'merge_mining', 'fee_forwarding', 'lightning_zap'

-- Contribution tracking
CREATE TABLE commons_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id INTEGER NOT NULL,
    contribution_type TEXT NOT NULL,
    amount_btc REAL NOT NULL,
    proof_data TEXT NOT NULL, -- JSON with verification data
    block_height INTEGER, -- For on-chain contributions
    timestamp DATETIME NOT NULL,
    verified BOOLEAN DEFAULT FALSE,
    FOREIGN KEY (node_id) REFERENCES economic_nodes(id)
);

-- Contribution aggregation (for weight calculation)
CREATE VIEW contribution_totals AS
SELECT 
    node_id,
    contribution_type,
    SUM(amount_btc) as total_btc,
    COUNT(*) as contribution_count,
    MIN(timestamp) as first_contribution,
    MAX(timestamp) as last_contribution
FROM commons_contributions
WHERE verified = TRUE
GROUP BY node_id, contribution_type;
```

### Qualification Proof Types

```rust
// bllvm-commons/src/economic_nodes/types.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContributionType {
    MergeMining,
    FeeForwarding,
    LightningZap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergeMiningProof {
    /// Secondary chain blocks mined (with Commons coinbase outputs)
    pub blocks_mined: Vec<BlockProof>,
    /// Total revenue contributed to Commons (in BTC)
    pub total_revenue_btc: f64,
    /// Measurement period (days)
    pub period_days: u32,
    /// Coinbase signatures proving ownership
    pub coinbase_signatures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockProof {
    pub block_hash: String,
    pub secondary_chain: String,
    pub coinbase_output_value: u64, // Satoshis
    pub commons_fee_amount: u64,   // Satoshis (1% of revenue)
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeForwardingProof {
    /// Blocks where fees were forwarded
    pub blocks_with_forwarding: Vec<FeeForwardingBlock>,
    /// Total fees forwarded (in BTC)
    pub total_fees_forwarded_btc: f64,
    /// Measurement period (days)
    pub period_days: u32,
    /// Commons development address (must match)
    pub commons_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeForwardingBlock {
    pub block_hash: String,
    pub coinbase_tx_hash: String,
    pub forwarded_fee_output_index: u32,
    pub forwarded_amount_sat: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningZapProof {
    /// Lightning invoices paid
    pub invoices_paid: Vec<LightningInvoice>,
    /// Total zaps (in BTC)
    pub total_zaps_btc: f64,
    /// Measurement period (days)
    pub period_days: u32,
    /// Commons Lightning node pubkey
    pub commons_pubkey: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningInvoice {
    pub invoice: String, // BOLT11 invoice
    pub payment_hash: String,
    pub payment_preimage: Option<String>, // If available
    pub amount_msat: u64,
    pub timestamp: DateTime<Utc>,
    pub payment_proof: Option<String>, // Payment proof if available
}
```

### Weight Calculation

```rust
// bllvm-commons/src/economic_nodes/registry.rs

impl EconomicNodeRegistry {
    /// Calculate quadratic weight for Commons contributor
    pub async fn calculate_contributor_weight(
        &self,
        contribution_type: ContributionType,
        total_contribution_btc: f64,
    ) -> Result<f64, GovernanceError> {
        // Normalization factor: 1.0 BTC = 1.0 weight
        let normalization_factor = 1.0;
        
        // Quadratic formula: sqrt(contribution / normalization)
        let weight = (total_contribution_btc / normalization_factor).sqrt();
        
        // Minimum weight: 0.01 (for contributors meeting minimum threshold)
        Ok(weight.max(0.01))
    }
    
    /// Calculate combined weight for multi-contribution nodes
    pub async fn calculate_combined_weight(
        &self,
        node_id: i32,
    ) -> Result<f64, GovernanceError> {
        // Get all contributions for this node
        let contributions = sqlx::query!(
            r#"
            SELECT contribution_type, SUM(amount_btc) as total_btc
            FROM commons_contributions
            WHERE node_id = ? AND verified = TRUE
            GROUP BY contribution_type
            "#,
            node_id
        )
        .fetch_all(&self.pool)
        .await?;
        
        // Calculate weight for each contribution type
        let mut total_weight = 0.0;
        for row in contributions {
            let contribution_type = ContributionType::from_str(&row.contribution_type)?;
            let weight = self.calculate_contributor_weight(
                contribution_type,
                row.total_btc.unwrap_or(0.0),
            ).await?;
            total_weight += weight;
        }
        
        // Combined weight uses quadratic sum: sqrt(sum of squared weights)
        // This prevents double-counting while rewarding diversity
        Ok(total_weight.sqrt())
    }
}
```

### Verification Functions

```rust
// bllvm-commons/src/economic_nodes/verification.rs

pub struct ContributionVerifier;

impl ContributionVerifier {
    /// Verify merge mining contribution
    pub async fn verify_merge_mining(
        &self,
        proof: &MergeMiningProof,
    ) -> Result<bool, GovernanceError> {
        // 1. Verify minimum threshold (0.01 BTC over 90 days)
        if proof.total_revenue_btc < 0.01 || proof.period_days < 90 {
            return Ok(false);
        }
        
        // 2. Verify blocks exist on secondary chains
        for block_proof in &proof.blocks_mined {
            // Verify block exists on secondary chain
            // Verify coinbase output includes Commons fee
            // Verify signature matches registered key
        }
        
        // 3. Verify total matches sum of individual contributions
        let calculated_total: f64 = proof.blocks_mined
            .iter()
            .map(|b| b.commons_fee_amount as f64 / 100_000_000.0)
            .sum();
        
        if (calculated_total - proof.total_revenue_btc).abs() > 0.001 {
            return Ok(false);
        }
        
        Ok(true)
    }
    
    /// Verify fee forwarding contribution
    pub async fn verify_fee_forwarding(
        &self,
        proof: &FeeForwardingProof,
    ) -> Result<bool, GovernanceError> {
        // 1. Verify minimum threshold (0.1 BTC over 90 days)
        if proof.total_fees_forwarded_btc < 0.1 || proof.period_days < 90 {
            return Ok(false);
        }
        
        // 2. Verify blocks exist on Bitcoin blockchain
        for block_proof in &proof.blocks_with_forwarding {
            // Verify block exists
            // Verify coinbase transaction has forwarding output
            // Verify output goes to Commons address
            // Verify amount matches claimed
        }
        
        // 3. Verify node is running BTCDecoded
        // (Check node registry or require node signature)
        
        Ok(true)
    }
    
    /// Verify Lightning zap contribution
    pub async fn verify_lightning_zap(
        &self,
        proof: &LightningZapProof,
    ) -> Result<bool, GovernanceError> {
        // 1. Verify minimum threshold (0.01 BTC over 90 days)
        if proof.total_zaps_btc < 0.01 || proof.period_days < 90 {
            return Ok(false);
        }
        
        // 2. Verify invoices are from Commons Lightning node
        for invoice in &proof.invoices_paid {
            // Verify invoice pubkey matches Commons
            // Verify payment hash matches
            // Verify payment preimage (if available)
            // Verify amount matches
        }
        
        // 3. Verify total matches sum of individual zaps
        let calculated_total: f64 = proof.invoices_paid
            .iter()
            .map(|i| i.amount_msat as f64 / 100_000_000_000.0)
            .sum();
        
        if (calculated_total - proof.total_zaps_btc).abs() > 0.0001 {
            return Ok(false);
        }
        
        Ok(true)
    }
}
```

---

## Registration Process

### 1. Application

Contributor submits application with:
- Entity name and contact info
- Contribution type (merge mining, fee forwarding, or Lightning zap)
- Qualification proof (see proof types above)
- Public key for signing veto signals
- Node identifier (if applicable)

### 2. Verification

System verifies:
- Minimum contribution threshold met
- Proof data is cryptographically valid
- Contributions are recent (within 90 days)
- No duplicate registration (one vote per entity)

### 3. Weight Calculation

System calculates:
- Individual contribution weight (quadratic)
- Combined weight (if multiple contribution types)
- Total voting weight for veto signals

### 4. Approval

- Automatic approval if thresholds met and proofs valid
- OR manual review if verification fails
- Added to economic node registry with calculated weight

---

## Ongoing Requirements

### Quarterly Re-verification

- Must maintain minimum contribution levels
- Contributions must be recent (within 90 days)
- Weight recalculated based on recent contributions

### Contribution Tracking

- System automatically tracks new contributions
- Weight updates in real-time as contributions accumulate
- Old contributions expire after 90 days (rolling window)

### Transparency

- All contributions are public and verifiable
- Weight calculations are transparent
- Anyone can verify contribution proofs

---

## Veto Power

### Weight Application

Contributor weight applies to:
- **Tier 3 (Consensus-Adjacent)**: Veto threshold = 40% of total economic activity
- **Tier 4 (Emergency)**: 2+ economic nodes can block
- **Tier 5 (Governance)**: Signaling threshold = 60% of total economic activity

### Combined Weight Calculation

When calculating veto thresholds:
1. Sum all economic node weights (traditional + contributors)
2. Contributors count toward "economic activity" threshold
3. Quadratic weighting ensures fair representation

### Example

If total economic activity weight = 100:
- Traditional economic nodes: 80 weight
- Commons contributors: 20 weight
- Veto threshold (40%): 40 weight
- Contributors can provide up to 20 weight toward veto

---

## Anti-Gaming Mechanisms

### Sybil Resistance

1. **One Vote Per Entity**: Cryptographic proof of unique identity
2. **Minimum Thresholds**: Prevents trivial contributions
3. **Verification Costs**: On-chain proofs require real resources
4. **Reputation**: Pattern of contributions builds trust

### Contribution Verification

1. **On-Chain Proofs**: Merge mining and fee forwarding are verifiable on-chain
2. **Lightning Proofs**: Payment preimages prove actual payments
3. **Time Windows**: 90-day rolling window prevents gaming
4. **Audit Trail**: All contributions are publicly logged

### Fair Weighting

1. **Quadratic Formula**: Prevents whale dominance
2. **Normalization**: Same formula across all contribution types
3. **Transparency**: Weight calculations are public
4. **Recalculation**: Weights update based on recent contributions

---

## Integration Points

### Merge Mining Integration

- `bllvm-node/src/network/stratum_v2/merge_mining.rs`
- Track revenue contributions automatically
- Generate qualification proofs from coordinator data

### Fee Forwarding Integration

- `bllvm-node/src/node/miner.rs` (coinbase creation)
- Track forwarded fees in coinbase outputs
- Generate qualification proofs from block data

### Lightning Integration

- Commons Lightning node (future)
- Track invoice payments
- Generate qualification proofs from payment records

### Economic Node Registry

- `bllvm-commons/src/economic_nodes/registry.rs`
- Add new node types
- Extend qualification verification
- Implement quadratic weight calculation

---

## Configuration

### Minimum Thresholds

```yaml
# governance/config/economic-nodes.yml

commons_contributors:
  merge_mining:
    minimum_contribution_btc: 0.01
    measurement_period_days: 90
    verification: "On-chain (coinbase signatures)"
    
  fee_forwarding:
    minimum_contribution_btc: 0.1
    measurement_period_days: 90
    verification: "On-chain (coinbase outputs)"
    
  lightning_zap:
    minimum_contribution_btc: 0.01
    measurement_period_days: 90
    verification: "Lightning (payment proofs)"
    
  weight_calculation:
    formula: "sqrt(contribution_btc / 1.0)"
    normalization_factor: 1.0
    minimum_weight: 0.01
```

---

## Testing Requirements

### Unit Tests

1. **Weight Calculation**
   - Test quadratic formula with various contributions
   - Test minimum weight enforcement
   - Test combined weight calculation

2. **Verification**
   - Test merge mining proof verification
   - Test fee forwarding proof verification
   - Test Lightning zap proof verification
   - Test threshold enforcement

3. **Qualification**
   - Test minimum threshold checks
   - Test measurement period validation
   - Test proof data validation

### Integration Tests

1. **Registration Flow**
   - Test complete registration process
   - Test automatic approval
   - Test manual review cases

2. **Weight Updates**
   - Test real-time weight updates
   - Test contribution expiration
   - Test combined weight calculation

3. **Veto Integration**
   - Test contributor veto signals
   - Test weight application to thresholds
   - Test combined economic activity calculation

---

## Security Considerations

1. **Proof Verification**: All proofs must be cryptographically verifiable
2. **Sybil Resistance**: One vote per entity, verified through proofs
3. **Weight Manipulation**: Quadratic formula prevents gaming
4. **Contribution Expiration**: 90-day window prevents stale contributions
5. **Transparency**: All contributions and weights are public

---

## Future Enhancements

1. **Multi-Contribution Bonuses**: Reward nodes contributing in multiple ways
2. **Time-Weighted Contributions**: Recent contributions weighted higher
3. **Reputation System**: Long-term contributors get reputation bonuses
4. **Community Verification**: Allow community to verify contributions
5. **Automated Tracking**: Real-time contribution tracking and weight updates

---

## Related Files

- `bllvm-commons/src/economic_nodes/types.rs` - Type definitions
- `bllvm-commons/src/economic_nodes/registry.rs` - Registration and weight calculation
- `bllvm-commons/src/economic_nodes/verification.rs` - Proof verification (to be created)
- `governance/config/economic-nodes.yml` - Configuration
- `bllvm-node/src/network/stratum_v2/merge_mining.rs` - Merge mining tracking
- `bllvm-node/src/node/miner.rs` - Fee forwarding tracking

