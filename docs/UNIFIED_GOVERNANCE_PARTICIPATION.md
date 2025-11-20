# Unified Governance Participation Model

## Overview

A single, fair, and equitable monetization and voting system that unifies:
- **Zap-to-Vote**: Users zap governance events to vote on proposals
- **Fee Forwarding**: Node operators forward transaction fees
- **Merge Mining**: Miners contribute merge mining revenue

All three contribute to both **qualification** (ongoing participation) and **voting weight** (decision-making power) using a unified quadratic weighting system.

## Core Design Principles

1. **Unified Contribution Model**: All three types (zaps, fees, merge mining) are treated as contributions
2. **Zap-to-Vote**: Zapping a governance event = voting on that proposal
3. **Quadratic Weighting**: Weight = √(total_contribution_btc) for fairness
4. **Dual Purpose**: Contributions count for both qualification and voting
5. **Fair Distribution**: Small contributors get meaningful weight, large contributors get more but not linearly

---

## Unified Contribution System

### Contribution Types

All contributions are tracked in a single unified system:

1. **Zap Contributions**
   - General zaps (to bot pubkeys) → Ongoing qualification
   - Proposal zaps (to governance events) → Voting on specific proposals
   - Both count toward total contribution

2. **Fee Forwarding Contributions**
   - Transaction fees forwarded to Commons address
   - Ongoing contribution (qualification + voting weight)

3. **Merge Mining Contributions**
   - 1% fee from merge mining revenue
   - Ongoing contribution (qualification + voting weight)

### Unified Weight Formula

**All contributions use the same quadratic formula:**

```
total_contribution_btc = zaps_btc + fees_forwarded_btc + merge_mining_btc
voting_weight = sqrt(total_contribution_btc / normalization_factor)
```

Where:
- `normalization_factor = 1.0 BTC` (standardizes across all types)
- All contribution types are summed, then quadratic is applied

**Example:**
- 0.5 BTC zapped + 0.3 BTC fees + 0.2 BTC merge mining = 1.0 BTC total
- Weight = sqrt(1.0 / 1.0) = 1.0

---

## Zap-to-Vote Mechanism

### How It Works

1. **Governance Event Published**: PR/Proposal published to Nostr with unique event ID
2. **User Zaps Event**: User zaps the governance event (NIP-57)
3. **Zap = Vote**: Zap amount determines vote weight (quadratic)
4. **Vote Direction**: Zap amount = support, negative zaps = veto (if supported)

### Governance Event Format

```json
{
  "id": "governance_event_id",
  "pubkey": "commons_bot_pubkey",
  "kind": 30078,  // Governance status event
  "tags": [
    ["d", "governance-proposal"],
    ["pr", "123"],  // PR number
    ["tier", "3"],  // Governance tier
    ["repository", "bllvm-consensus"],
    ["zap", "donations@btcdecoded.org"],  // Zap address
    ["vote_type", "support"],  // support/veto/abstain
    ["voting_window", "2024-01-15T00:00:00Z", "2024-02-15T00:00:00Z"]
  ],
  "content": "{\"title\":\"Proposal Title\",\"description\":\"...\",\"pr_id\":123}",
  "sig": "signature"
}
```

### Zap-to-Vote Processing

```rust
// bllvm-commons/src/nostr/zap_voting.rs

#[derive(Debug, Clone)]
pub struct ZapVote {
    pub governance_event_id: String,  // Event being zapped
    pub sender_pubkey: String,  // Voter's pubkey
    pub amount_msat: u64,  // Zap amount
    pub vote_weight: f64,  // Calculated voting weight (quadratic)
    pub timestamp: DateTime<Utc>,
    pub invoice: Option<String>,  // Bolt11 invoice
}

impl ZapVote {
    /// Calculate vote weight from zap amount (quadratic)
    pub fn calculate_weight(amount_msat: u64) -> f64 {
        let amount_btc = amount_msat as f64 / 100_000_000_000.0;
        let normalization_factor = 1.0;
        (amount_btc / normalization_factor).sqrt().max(0.01)
    }
    
    /// Process zap as vote on governance proposal
    pub async fn process_vote(
        zap_event: &ZapEvent,  // From Nostr zap receipt
        governance_event_id: &str,
    ) -> Result<Self> {
        // Verify zap is for this governance event
        if zap_event.zapped_event_id.as_ref() != Some(&governance_event_id.to_string()) {
            return Err(anyhow!("Zap not for this governance event"));
        }
        
        // Calculate vote weight
        let vote_weight = Self::calculate_weight(zap_event.amount_msat);
        
        Ok(Self {
            governance_event_id: governance_event_id.to_string(),
            sender_pubkey: zap_event.sender_pubkey.clone().unwrap_or_default(),
            amount_msat: zap_event.amount_msat,
            vote_weight,
            timestamp: DateTime::from_timestamp(zap_event.timestamp, 0)?,
            invoice: zap_event.invoice.clone(),
        })
    }
}
```

### Vote Aggregation

```rust
// bllvm-commons/src/governance/vote_aggregator.rs

pub struct VoteAggregator {
    pool: SqlitePool,
}

impl VoteAggregator {
    /// Aggregate all votes for a proposal
    pub async fn aggregate_proposal_votes(
        &self,
        pr_id: i32,
    ) -> Result<ProposalVoteResult> {
        // Get all zap votes for this proposal
        let zap_votes = self.get_zap_votes_for_proposal(pr_id).await?;
        
        // Get all economic node votes (traditional + contributors)
        let node_votes = self.get_economic_node_votes(pr_id).await?;
        
        // Combine votes with unified weighting
        let mut total_support_weight = 0.0;
        let mut total_veto_weight = 0.0;
        let mut total_abstain_weight = 0.0;
        
        // Aggregate zap votes
        for vote in &zap_votes {
            match vote.vote_type {
                VoteType::Support => total_support_weight += vote.vote_weight,
                VoteType::Veto => total_veto_weight += vote.vote_weight,
                VoteType::Abstain => total_abstain_weight += vote.vote_weight,
            }
        }
        
        // Aggregate economic node votes (weighted by contribution)
        for node_vote in &node_votes {
            // Node weight = sqrt(total_contribution_btc)
            let node_weight = self.calculate_node_weight(node_vote.node_id).await?;
            
            match node_vote.signal_type {
                SignalType::Support => total_support_weight += node_weight,
                SignalType::Veto => total_veto_weight += node_weight,
                SignalType::Abstain => total_abstain_weight += node_weight,
            }
        }
        
        // Calculate percentages
        let total_weight = total_support_weight + total_veto_weight + total_abstain_weight;
        let support_percent = if total_weight > 0.0 {
            (total_support_weight / total_weight) * 100.0
        } else {
            0.0
        };
        
        let veto_percent = if total_weight > 0.0 {
            (total_veto_weight / total_weight) * 100.0
        } else {
            0.0
        };
        
        Ok(ProposalVoteResult {
            pr_id,
            total_weight,
            support_weight: total_support_weight,
            veto_weight: total_veto_weight,
            abstain_weight: total_abstain_weight,
            support_percent,
            veto_percent,
            zap_vote_count: zap_votes.len(),
            node_vote_count: node_votes.len(),
        })
    }
    
    /// Calculate unified weight for a node (includes all contribution types)
    pub async fn calculate_node_weight(&self, node_id: i32) -> Result<f64> {
        // Get all contributions for this node
        let contributions = sqlx::query!(
            r#"
            SELECT 
                COALESCE(SUM(z.amount_btc), 0) as zaps_btc,
                COALESCE(SUM(f.amount_btc), 0) as fees_btc,
                COALESCE(SUM(m.amount_btc), 0) as merge_mining_btc
            FROM economic_nodes en
            LEFT JOIN zap_contributions z ON z.sender_pubkey = en.public_key
                AND z.timestamp >= datetime('now', '-90 days')
            LEFT JOIN fee_forwarding_contributions f ON f.node_id = en.id
                AND f.timestamp >= datetime('now', '-90 days')
            LEFT JOIN merge_mining_contributions m ON m.node_id = en.id
                AND m.timestamp >= datetime('now', '-90 days')
            WHERE en.id = ?
            "#,
            node_id
        )
        .fetch_one(&self.pool)
        .await?;
        
        let total_btc = (contributions.zaps_btc.unwrap_or(0.0) +
                        contributions.fees_btc.unwrap_or(0.0) +
                        contributions.merge_mining_btc.unwrap_or(0.0));
        
        // Apply quadratic formula
        let normalization_factor = 1.0;
        Ok((total_btc / normalization_factor).sqrt().max(0.01))
    }
}
```

---

## Database Schema

### Unified Contribution Tracking

```sql
-- Unified contribution tracking
CREATE TABLE unified_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contributor_id TEXT NOT NULL,  -- Pubkey or node_id
    contributor_type TEXT NOT NULL,  -- 'zap_user', 'fee_forwarder', 'merge_miner', 'economic_node'
    contribution_type TEXT NOT NULL,  -- 'zap', 'fee_forwarding', 'merge_mining'
    amount_btc REAL NOT NULL,
    timestamp DATETIME NOT NULL,
    proof_data TEXT,  -- JSON with verification data
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Zap votes on proposals
CREATE TABLE zap_votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pr_id INTEGER NOT NULL,
    governance_event_id TEXT NOT NULL,  -- Nostr event ID
    sender_pubkey TEXT NOT NULL,  -- Voter's pubkey
    amount_msat INTEGER NOT NULL,
    amount_btc REAL NOT NULL,
    vote_weight REAL NOT NULL,  -- Quadratic weight
    vote_type TEXT NOT NULL,  -- 'support', 'veto', 'abstain'
    timestamp DATETIME NOT NULL,
    invoice_hash TEXT,  -- For verification
    message TEXT,  -- Optional zap message
    verified BOOLEAN DEFAULT FALSE
);

-- Unified voting results
CREATE TABLE proposal_votes (
    pr_id INTEGER PRIMARY KEY,
    total_weight REAL NOT NULL,
    support_weight REAL NOT NULL,
    veto_weight REAL NOT NULL,
    abstain_weight REAL NOT NULL,
    support_percent REAL NOT NULL,
    veto_percent REAL NOT NULL,
    zap_vote_count INTEGER NOT NULL,
    node_vote_count INTEGER NOT NULL,
    calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes
CREATE INDEX idx_unified_contributions_contributor ON unified_contributions(contributor_id, contributor_type);
CREATE INDEX idx_unified_contributions_type ON unified_contributions(contribution_type);
CREATE INDEX idx_unified_contributions_time ON unified_contributions(timestamp);
CREATE INDEX idx_zap_votes_pr ON zap_votes(pr_id);
CREATE INDEX idx_zap_votes_event ON zap_votes(governance_event_id);
CREATE INDEX idx_zap_votes_sender ON zap_votes(sender_pubkey);
```

---

## Voting Mechanism

### Vote Types

1. **Zap Votes** (User votes via zaps)
   - Zap governance event = vote
   - Weight = sqrt(zap_amount_btc / 1.0)
   - Direction: Support (default) or Veto (if negative zap supported)

2. **Economic Node Votes** (Traditional + Contributors)
   - Submit veto/support signal
   - Weight = sqrt(total_contribution_btc / 1.0)
   - Includes: zaps + fees + merge mining

### Unified Vote Calculation

```rust
pub struct UnifiedVote {
    pub pr_id: i32,
    pub total_weight: f64,
    pub support_weight: f64,
    pub veto_weight: f64,
    pub abstain_weight: f64,
    pub zap_votes: Vec<ZapVote>,
    pub node_votes: Vec<NodeVote>,
}

impl UnifiedVote {
    /// Calculate unified vote result
    pub async fn calculate(
        &self,
        aggregator: &VoteAggregator,
    ) -> Result<ProposalVoteResult> {
        // All votes use the same quadratic weighting
        // Zap votes: weight = sqrt(zap_amount_btc)
        // Node votes: weight = sqrt(total_contribution_btc)
        
        // Sum all weights by type
        let mut support = 0.0;
        let mut veto = 0.0;
        let mut abstain = 0.0;
        
        for zap_vote in &self.zap_votes {
            match zap_vote.vote_type {
                VoteType::Support => support += zap_vote.vote_weight,
                VoteType::Veto => veto += zap_vote.vote_weight,
                VoteType::Abstain => abstain += zap_vote.vote_weight,
            }
        }
        
        for node_vote in &self.node_votes {
            let node_weight = aggregator.calculate_node_weight(node_vote.node_id).await?;
            match node_vote.signal_type {
                SignalType::Support => support += node_weight,
                SignalType::Veto => veto += node_weight,
                SignalType::Abstain => abstain += node_weight,
            }
        }
        
        let total = support + veto + abstain;
        
        Ok(ProposalVoteResult {
            pr_id: self.pr_id,
            total_weight: total,
            support_weight: support,
            veto_weight: veto,
            abstain_weight: abstain,
            support_percent: if total > 0.0 { (support / total) * 100.0 } else { 0.0 },
            veto_percent: if total > 0.0 { (veto / total) * 100.0 } else { 0.0 },
            zap_vote_count: self.zap_votes.len(),
            node_vote_count: self.node_votes.len(),
        })
    }
}
```

---

## Qualification System

### Unified Qualification

All contributors qualify based on **total contribution** across all types:

```rust
pub struct ContributorQualification {
    pub contributor_id: String,  // Pubkey or node_id
    pub contributor_type: ContributorType,
    pub total_contribution_btc: f64,  // Sum of all types
    pub zaps_btc: f64,
    pub fees_forwarded_btc: f64,
    pub merge_mining_btc: f64,
    pub voting_weight: f64,  // sqrt(total_contribution_btc)
    pub qualified: bool,
}

impl ContributorQualification {
    /// Check if contributor qualifies
    pub fn qualifies(&self) -> bool {
        // Minimum: 0.05 BTC total contribution over 90 days
        // OR: 3+ contributions of any type
        self.total_contribution_btc >= 0.05 || self.contribution_count() >= 3
    }
    
    /// Calculate unified voting weight
    pub fn calculate_weight(&self) -> f64 {
        let normalization_factor = 1.0;
        (self.total_contribution_btc / normalization_factor).sqrt().max(0.01)
    }
}
```

### Qualification Thresholds

**Unified Minimums:**
- **Total Contribution**: 0.05 BTC across all types (90 days)
- **OR Minimum Contributions**: 3+ contributions of any type
- **Measurement Period**: 90-day rolling window

**Per-Type Minimums (for qualification):**
- Zaps: 0.01 BTC (but counts toward total)
- Fee Forwarding: 0.1 BTC (but counts toward total)
- Merge Mining: 0.01 BTC (but counts toward total)

**Key Insight**: You can qualify with any combination - 0.05 BTC in zaps, or 0.03 BTC zaps + 0.02 BTC fees, etc.

---

## Voting Process

### For Tier 3+ Proposals

1. **Proposal Published**: Governance event published to Nostr
2. **Voting Window Opens**: 30-90 days depending on tier
3. **Users Zap to Vote**: 
   - Zap governance event = vote
   - Weight = sqrt(zap_amount_btc)
   - Can zap multiple times (weights sum quadratically)
4. **Nodes Submit Signals**:
   - Traditional economic nodes submit veto/support
   - Contributor nodes submit veto/support (weighted by contribution)
5. **Vote Aggregation**: All votes combined with unified weighting
6. **Threshold Check**: 
   - Support: 60%+ weight
   - Veto: 40%+ weight blocks

### Vote Weight Examples

| Contribution | Zap Vote | Node Vote | Total Weight |
|-------------|----------|-----------|--------------|
| User zaps 0.01 BTC | 0.10 | - | 0.10 |
| User zaps 0.25 BTC | 0.50 | - | 0.50 |
| Node: 0.5 BTC zaps + 0.3 BTC fees + 0.2 BTC merge | - | 1.0 | 1.0 |
| User zaps 1.0 BTC | 1.0 | - | 1.0 |
| Node: 4.0 BTC total | - | 2.0 | 2.0 |

**Key**: All use the same quadratic formula, so 1 BTC zap = 1 BTC node contribution = same weight.

---

## Fairness Mechanisms

### 1. Quadratic Weighting

**Prevents Whale Dominance:**
- 100 BTC contributor: 10.0 weight (not 100.0)
- 1 BTC contributor: 1.0 weight
- 0.01 BTC contributor: 0.1 weight (10x relative power vs linear)

### 2. Unified Contribution Model

**No Type Discrimination:**
- 1 BTC zap = 1 BTC fee forwarding = 1 BTC merge mining
- All count equally toward qualification and voting
- Users can contribute in any way they prefer

### 3. Dual Purpose Contributions

**Contributions Count Twice:**
- Ongoing qualification (90-day rolling)
- Voting weight (current total)
- Encourages sustained participation

### 4. Zap-to-Vote Direct Democracy

**Users Can Vote Directly:**
- No need to register as economic node
- Just zap the governance event
- Weight based on zap amount (quadratic)
- Real-time voting on proposals

---

## Implementation

### Zap-to-Vote Service

```rust
// bllvm-commons/src/governance/zap_voting_service.rs

pub struct ZapVotingService {
    pool: SqlitePool,
    nostr_client: NostrClient,
    vote_aggregator: VoteAggregator,
}

impl ZapVotingService {
    /// Process zap as vote on governance proposal
    pub async fn process_zap_vote(
        &self,
        zap_event: &ZapEvent,
        governance_event_id: &str,
    ) -> Result<ZapVote> {
        // 1. Verify governance event exists
        let proposal = self.get_proposal_by_event_id(governance_event_id).await?;
        
        // 2. Verify voting window is open
        if !proposal.is_voting_window_open() {
            return Err(anyhow!("Voting window closed"));
        }
        
        // 3. Create zap vote
        let zap_vote = ZapVote::process_vote(zap_event, governance_event_id).await?;
        
        // 4. Record vote
        self.record_zap_vote(&zap_vote).await?;
        
        // 5. Update proposal vote totals
        self.update_proposal_votes(proposal.pr_id).await?;
        
        // 6. Check if thresholds met
        let result = self.vote_aggregator.aggregate_proposal_votes(proposal.pr_id).await?;
        self.check_voting_thresholds(proposal.pr_id, &result).await?;
        
        Ok(zap_vote)
    }
    
    /// Record zap as both vote and contribution
    pub async fn record_zap_vote(&self, vote: &ZapVote) -> Result<()> {
        // Record as vote
        sqlx::query!(
            r#"
            INSERT INTO zap_votes
            (pr_id, governance_event_id, sender_pubkey, amount_msat, amount_btc, vote_weight, vote_type, timestamp, invoice_hash, message, verified)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, TRUE)
            "#,
            vote.pr_id,
            vote.governance_event_id,
            vote.sender_pubkey,
            vote.amount_msat as i64,
            vote.amount_btc,
            vote.vote_weight,
            "support",  // Default, can be veto if negative zap
            vote.timestamp,
            vote.invoice.as_ref().map(|i| extract_payment_hash(i)),
            None::<String>
        )
        .execute(&self.pool)
        .await?;
        
        // Also record as contribution (for qualification)
        sqlx::query!(
            r#"
            INSERT INTO unified_contributions
            (contributor_id, contributor_type, contribution_type, amount_btc, timestamp, verified)
            VALUES (?, 'zap_user', 'zap', ?, ?, TRUE)
            "#,
            vote.sender_pubkey,
            vote.amount_btc,
            vote.timestamp
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
}
```

### Governance Event Publisher

```rust
// bllvm-commons/src/nostr/governance_publisher.rs

impl GovernanceActionPublisher {
    /// Publish governance proposal with zap-to-vote support
    pub async fn publish_proposal(
        &self,
        pr_id: i32,
        tier: u8,
        repository: &str,
    ) -> Result<String> {
        // Create governance event
        let event_id = format!("proposal-{}", pr_id);
        
        // Create event tags
        let mut tags = vec![
            Tag::Generic(TagKind::Custom("d".into()), vec!["governance-proposal".to_string()]),
            Tag::Generic(TagKind::Custom("pr".into()), vec![pr_id.to_string()]),
            Tag::Generic(TagKind::Custom("tier".into()), vec![tier.to_string()]),
            Tag::Generic(TagKind::Custom("repository".into()), vec![repository.to_string()]),
            Tag::Generic(TagKind::Custom("zap".into()), vec![self.zap_address.clone().unwrap_or_default()]),
            Tag::Generic(TagKind::Custom("vote_type".into()), vec!["support".to_string()]),
        ];
        
        // Add voting window
        let voting_start = Utc::now();
        let voting_end = voting_start + chrono::Duration::days(30);  // Tier 3 = 30 days
        tags.push(Tag::Generic(
            TagKind::Custom("voting_window".into()),
            vec![voting_start.to_rfc3339(), voting_end.to_rfc3339()],
        ));
        
        // Create event
        let event = EventBuilder::new(
            Kind::Custom(30078),  // Governance status
            "Proposal content...",
            &tags,
        )
        .to_event(&self.client.keys)?;
        
        // Publish to Nostr
        self.client.publish_event(event.clone()).await?;
        
        info!("Published governance proposal {} with zap-to-vote", event_id);
        
        Ok(event.id.to_string())
    }
}
```

---

## Voting Thresholds

### Unified Thresholds

**Tier 3 (Consensus-Adjacent):**
- **Support Required**: 60%+ of total voting weight
- **Veto Blocks**: 40%+ of total voting weight
- **Voting Window**: 30 days
- **Combined**: Zap votes + node votes counted together

**Tier 4 (Emergency):**
- **Veto Blocks**: 2+ economic nodes OR 20%+ total weight
- **Voting Window**: 24 hours
- **Zap Votes**: Count toward veto threshold

**Tier 5 (Governance Changes):**
- **Support Required**: 60%+ total weight
- **Voting Window**: 90 days
- **Both Required**: Support from both zap voters and nodes

### Example Calculation

**Proposal has:**
- Zap votes: 0.5 weight support, 0.2 weight veto
- Node votes: 2.0 weight support, 1.0 weight veto
- **Total**: 3.7 weight
- **Support**: 2.5 / 3.7 = 67.6% ✅ (meets 60% threshold)
- **Veto**: 1.2 / 3.7 = 32.4% ❌ (doesn't meet 40% threshold)
- **Result**: Proposal passes

---

## Benefits

### 1. Unified Monetization

- **Single System**: All contributions (zaps, fees, merge mining) in one model
- **Fair Weighting**: Same quadratic formula for all
- **No Discrimination**: 1 BTC zap = 1 BTC fee = 1 BTC merge mining

### 2. Direct Democracy

- **Zap-to-Vote**: Users vote directly by zapping
- **No Registration Required**: Just zap the governance event
- **Real-time**: Votes counted immediately
- **Transparent**: All votes public on Nostr

### 3. Fair Participation

- **Small Contributors**: 0.01 BTC zap gets 0.1 weight (meaningful)
- **Large Contributors**: 100 BTC gets 10.0 weight (not 100.0)
- **Quadratic**: Prevents whale dominance while rewarding larger contributions

### 4. Economic Alignment

- **Users**: Can participate via zaps
- **Miners**: Can participate via merge mining
- **Node Operators**: Can participate via fee forwarding
- **All Aligned**: Everyone benefits from Commons success

---

## Configuration

```yaml
# governance/config/unified-governance.yml

unified_participation:
  contribution_types:
    - zap
    - fee_forwarding
    - merge_mining
  
  qualification:
    minimum_total_btc: 0.05
    minimum_contributions: 3
    measurement_period_days: 90
  
  voting:
    weight_formula: "sqrt(total_contribution_btc / 1.0)"
    normalization_factor: 1.0
    minimum_weight: 0.01
  
  zap_to_vote:
    enabled: true
    default_vote_type: "support"
    negative_zaps_supported: false  # Future: allow veto zaps
  
  thresholds:
    tier_3:
      support_required_percent: 60.0
      veto_blocks_percent: 40.0
      voting_window_days: 30
    tier_4:
      veto_blocks_percent: 20.0
      voting_window_hours: 24
    tier_5:
      support_required_percent: 60.0
      voting_window_days: 90
```

---

## Example Scenarios

### Scenario 1: User Zap Vote

1. **Proposal Published**: PR #123 published to Nostr
2. **User Zaps**: User zaps 0.25 BTC to governance event
3. **Vote Recorded**: 
   - Vote weight = sqrt(0.25) = 0.5
   - Counts as "support" vote
   - Also counts toward user's total contribution
4. **Result**: User has 0.5 weight on this proposal

### Scenario 2: Node Multi-Contribution

1. **Node Contributions**:
   - 0.3 BTC in zaps (general)
   - 0.4 BTC in fee forwarding
   - 0.3 BTC in merge mining
   - **Total**: 1.0 BTC
2. **Node Weight**: sqrt(1.0) = 1.0
3. **Node Votes**: Submits "support" signal
4. **Result**: Node has 1.0 weight on proposal

### Scenario 3: Combined Voting

**Proposal has:**
- 10 users zap 0.01 BTC each = 10 × 0.1 = 1.0 weight support
- 5 users zap 0.25 BTC each = 5 × 0.5 = 2.5 weight support
- 2 nodes with 1.0 BTC each = 2 × 1.0 = 2.0 weight support
- 1 node with 4.0 BTC = 1 × 2.0 = 2.0 weight veto

**Total**: 8.0 weight
- **Support**: 6.0 / 8.0 = 75% ✅
- **Veto**: 2.0 / 8.0 = 25% ❌
- **Result**: Proposal passes

---

## Security Considerations

### 1. Zap Verification

- **Invoice Verification**: Verify payment hash from invoice
- **Event Verification**: Verify zap receipt event signature
- **Relay Verification**: Query multiple relays for redundancy

### 2. Sybil Resistance

- **Quadratic Weighting**: Prevents trivial Sybil attacks
- **Minimum Thresholds**: Requires real contributions
- **Time Windows**: 90-day rolling prevents gaming

### 3. Vote Manipulation

- **Public Votes**: All votes visible on Nostr
- **Cryptographic Proof**: All votes cryptographically verifiable
- **Transparency**: Anyone can verify vote totals

---

## Summary

**Unified Governance Participation Model:**

1. **Three Contribution Types**: Zaps, fee forwarding, merge mining
2. **Unified Weighting**: sqrt(total_contribution_btc) for all
3. **Zap-to-Vote**: Users zap governance events to vote
4. **Dual Purpose**: Contributions count for qualification + voting
5. **Fair Distribution**: Quadratic prevents whale dominance
6. **Direct Democracy**: No registration required to vote via zaps
7. **Economic Alignment**: All participants benefit from Commons success

**Key Innovation**: Zaps serve dual purpose - they're both ongoing contributions (qualification) AND direct votes on proposals (governance participation).

