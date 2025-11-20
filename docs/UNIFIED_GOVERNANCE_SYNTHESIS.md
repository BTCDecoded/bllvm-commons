# Unified Governance Model - Synthesis

## Comparison Analysis

### Your Model vs My Model

| Aspect | Your Model | My Model | Best Approach |
|--------|-----------|----------|---------------|
| **Weighting Base** | Dollars | BTC | **Dollars** (more stable) |
| **Tier Structure** | 3 clear tiers | Unified model | **3 tiers** (clearer) |
| **Thresholds** | Fixed votes (100, 500, etc.) | Percentages (60%, 40%) | **Fixed votes** (easier to understand) |
| **Zap Mechanism** | Cumulative zaps | Zap-to-vote | **Both** (cumulative + per-proposal) |
| **Time Windows** | Monthly (mining), Cumulative (zaps) | 90-day rolling (all) | **Your approach** (more flexible) |
| **Voting** | Ongoing participation | Per-proposal voting | **Both** (ongoing + per-proposal) |

---

## Synthesized Model

### Core Principles

1. **Three Participation Tiers**: Merge Mining, Fee Forwarding, Community Zaps
2. **Dollar-Based Weighting**: More stable than BTC (avoids volatility)
3. **Quadratic Formula**: `√(contribution_in_dollars)` for all tiers
4. **Dual Voting System**: 
   - Ongoing participation weight (qualification)
   - Per-proposal voting (zap-to-vote)
5. **Fixed Thresholds**: Clear, understandable vote requirements

---

## Three Tiers of Participation

### Tier 1: Merge Mining

**Who**: Miners running BTCDecoded with merge mining enabled

**Contribution**: Monthly merge mining fees (1% of merged chain rewards)

**Verification**: On-chain proof via merge mining commitments

**Weight Calculation**: `√(monthly_fees_in_dollars)`

**Time Window**: 30-day rolling average

### Tier 2: Fee Forwarding

**Who**: Miners voluntarily forwarding percentage of block rewards

**Contribution**: Monthly Bitcoin forwarded to development address

**Verification**: On-chain transactions to designated address

**Weight Calculation**: `√(monthly_forwards_in_dollars)`

**Time Window**: 30-day rolling average

### Tier 3: Community Zaps

**Who**: Anyone with Lightning wallet and Nostr

**Contribution**: 
- **Cumulative zaps** (ongoing participation weight)
- **Proposal zaps** (per-proposal voting)

**Verification**: 
- Nostr event signatures
- Lightning payment proofs

**Weight Calculation**: 
- Ongoing: `√(cumulative_zaps_in_dollars)`
- Per-proposal: `√(zap_amount_in_dollars)` for that proposal

**Time Window**: 
- Cumulative: All-time (or 1-year rolling)
- Per-proposal: During voting window

---

## Unified Weight Formula

### For Ongoing Participation (Qualification)

```rust
// Calculate total contribution across all tiers
let total_contribution_usd = 
    merge_mining_monthly_usd +
    fee_forwarding_monthly_usd +
    cumulative_zaps_usd;

// Apply quadratic formula
let participation_weight = sqrt(total_contribution_usd);
```

### For Per-Proposal Voting

```rust
// Proposal-specific zap vote
let proposal_zap_weight = sqrt(zap_amount_usd);

// OR use ongoing participation weight
let participation_weight = sqrt(total_contribution_usd);

// Use whichever is higher (encourages both)
let vote_weight = max(proposal_zap_weight, participation_weight * 0.1);
```

**Key Innovation**: Users can vote either by:
1. Zapping the specific proposal (direct democracy)
2. Using their ongoing participation weight (representative)

---

## Vote Weight Calculation

### Formula

```
vote_weight = √(contribution_in_dollars)
```

All three tiers use the same quadratic formula.

### Conversion to Dollars

- **Merge mining**: Dollar value of monthly fees (using BTC price at time)
- **Fee forwarding**: Dollar value of monthly forwards (using BTC price at time)
- **Zaps**: Dollar value of cumulative sats (100 sats ≈ $0.09 at current prices)

### Examples

**Small Miner (Merge Mining Only)**
- Monthly contribution: $1,000
- Vote weight: √1,000 = 31.6 votes

**Large Miner (Both Mining Methods)**
- Merge mining: $10,000/month = √10,000 = 100 votes
- Fee forwarding: $2,000/month = √2,000 = 44.7 votes
- **Total**: 144.7 votes

**Dedicated Community Member**
- Cumulative zaps: 1M sats ($900)
- Ongoing weight: √900 = 30 votes
- Can also zap proposals directly

**Whale Zapper**
- Cumulative zaps: 10M sats ($9,000)
- Ongoing weight: √9,000 = 94.9 votes
- Can also zap proposals with additional weight

---

## Governance Thresholds

### Decision Types and Required Votes

**Tier 1: Routine Maintenance**
- Examples: Bug fixes, dependency updates, documentation
- Threshold: **100 total votes** (3-of-5 maintainer signatures)
- Review period: 7 days

**Tier 2: Minor Changes**
- Examples: Performance optimizations, minor feature additions
- Threshold: **500 total votes** (4-of-5 signatures)
- Review period: 30 days

**Tier 3: Significant Changes**
- Examples: New modules, architectural changes
- Threshold: **1,000 total votes** (5-of-6 signatures)
- Review period: 90 days

**Tier 4: Major Changes**
- Examples: Consensus-adjacent features, governance modifications
- Threshold: **2,500 total votes** (5-of-7 signatures)
- Review period: 180 days

**Tier 5: Constitutional Changes**
- Examples: Fundamental governance rules, economic model changes
- Threshold: **5,000 total votes** (6-of-7 signatures)
- Review period: 365 days

---

## Dual Voting System

### 1. Ongoing Participation Weight

**Purpose**: Qualification and general voting power

**Calculation**:
- Merge mining: 30-day rolling average
- Fee forwarding: 30-day rolling average
- Zaps: Cumulative (all-time or 1-year rolling)

**Use**: 
- Economic node qualification
- General governance participation
- Veto/support signals on proposals

### 2. Zap-to-Vote (Per-Proposal)

**Purpose**: Direct democracy on specific proposals

**How It Works**:
1. Governance proposal published to Nostr
2. User zaps the governance event
3. Zap amount = vote weight for that proposal
4. Weight = √(zap_amount_usd)

**Advantages**:
- No registration required
- Direct participation
- Real-time voting
- Can vote on multiple proposals independently

**Example**:
- User has 30 votes from cumulative zaps (ongoing)
- User zaps 0.1 BTC ($9,000) to specific proposal
- Vote weight on that proposal: √9,000 = 94.9 votes
- User's ongoing weight still 30 for other purposes

---

## Implementation

### Database Schema

```sql
-- Unified contributions (for ongoing participation)
CREATE TABLE unified_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contributor_id TEXT NOT NULL,
    contributor_type TEXT NOT NULL,  -- 'merge_miner', 'fee_forwarder', 'zap_user'
    contribution_type TEXT NOT NULL,  -- 'merge_mining', 'fee_forwarding', 'zap'
    amount_btc REAL NOT NULL,
    amount_usd REAL NOT NULL,  -- Calculated at time of contribution
    timestamp DATETIME NOT NULL,
    period_type TEXT NOT NULL,  -- 'monthly', 'cumulative'
    verified BOOLEAN DEFAULT FALSE
);

-- Per-proposal zap votes
CREATE TABLE proposal_zap_votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pr_id INTEGER NOT NULL,
    governance_event_id TEXT NOT NULL,
    sender_pubkey TEXT NOT NULL,
    amount_msat INTEGER NOT NULL,
    amount_usd REAL NOT NULL,
    vote_weight REAL NOT NULL,  -- sqrt(amount_usd)
    vote_type TEXT NOT NULL,  -- 'support', 'veto', 'abstain'
    timestamp DATETIME NOT NULL,
    verified BOOLEAN DEFAULT FALSE
);

-- Ongoing participation weights (cached)
CREATE TABLE participation_weights (
    contributor_id TEXT PRIMARY KEY,
    contributor_type TEXT NOT NULL,
    merge_mining_usd REAL DEFAULT 0.0,
    fee_forwarding_usd REAL DEFAULT 0.0,
    cumulative_zaps_usd REAL DEFAULT 0.0,
    total_contribution_usd REAL NOT NULL,
    participation_weight REAL NOT NULL,  -- sqrt(total_contribution_usd)
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Proposal vote aggregation
CREATE TABLE proposal_votes (
    pr_id INTEGER PRIMARY KEY,
    total_votes REAL NOT NULL,
    support_votes REAL NOT NULL,
    veto_votes REAL NOT NULL,
    abstain_votes REAL NOT NULL,
    zap_vote_count INTEGER NOT NULL,
    participation_vote_count INTEGER NOT NULL,
    threshold_met BOOLEAN DEFAULT FALSE,
    calculated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Weight Calculation

```rust
// bllvm-commons/src/governance/weight_calculator.rs

pub struct WeightCalculator {
    btc_price_service: BtcPriceService,  // Get current BTC price
}

impl WeightCalculator {
    /// Calculate ongoing participation weight
    pub async fn calculate_participation_weight(
        &self,
        contributor_id: &str,
    ) -> Result<f64> {
        // Get contributions
        let contributions = self.get_contributions(contributor_id).await?;
        
        // Convert to USD (using price at time of contribution)
        let merge_mining_usd = contributions.merge_mining
            .iter()
            .map(|c| c.amount_usd)
            .sum::<f64>();
        
        let fee_forwarding_usd = contributions.fee_forwarding
            .iter()
            .map(|c| c.amount_usd)
            .sum::<f64>();
        
        let cumulative_zaps_usd = contributions.zaps
            .iter()
            .map(|c| c.amount_usd)
            .sum::<f64>();
        
        // Total contribution
        let total_usd = merge_mining_usd + fee_forwarding_usd + cumulative_zaps_usd;
        
        // Quadratic weight
        Ok(total_usd.sqrt())
    }
    
    /// Calculate per-proposal zap vote weight
    pub async fn calculate_zap_vote_weight(
        &self,
        zap_amount_msat: u64,
    ) -> Result<f64> {
        // Convert to USD
        let btc_price = self.btc_price_service.get_current_price().await?;
        let amount_btc = zap_amount_msat as f64 / 100_000_000_000.0;
        let amount_usd = amount_btc * btc_price;
        
        // Quadratic weight
        Ok(amount_usd.sqrt())
    }
    
    /// Get vote weight for proposal (uses higher of zap or participation)
    pub async fn get_proposal_vote_weight(
        &self,
        contributor_id: &str,
        proposal_zap_amount_msat: Option<u64>,
    ) -> Result<f64> {
        let participation_weight = self.calculate_participation_weight(contributor_id).await?;
        
        if let Some(zap_amount) = proposal_zap_amount_msat {
            let zap_weight = self.calculate_zap_vote_weight(zap_amount).await?;
            // Use 10% of participation weight as minimum, or zap weight if higher
            Ok(zap_weight.max(participation_weight * 0.1))
        } else {
            // No zap, use participation weight
            Ok(participation_weight)
        }
    }
}
```

### Vote Aggregation

```rust
// bllvm-commons/src/governance/vote_aggregator.rs

impl VoteAggregator {
    /// Aggregate all votes for a proposal
    pub async fn aggregate_proposal_votes(
        &self,
        pr_id: i32,
        tier: u8,
    ) -> Result<ProposalVoteResult> {
        // Get fixed threshold for this tier
        let threshold = self.get_threshold_for_tier(tier)?;
        
        // Get all zap votes for this proposal
        let zap_votes = self.get_proposal_zap_votes(pr_id).await?;
        let zap_support: f64 = zap_votes.iter()
            .filter(|v| v.vote_type == "support")
            .map(|v| v.vote_weight)
            .sum();
        let zap_veto: f64 = zap_votes.iter()
            .filter(|v| v.vote_type == "veto")
            .map(|v| v.vote_weight)
            .sum();
        
        // Get all participation-based votes (economic nodes + contributors)
        let participation_votes = self.get_participation_votes(pr_id).await?;
        let participation_support: f64 = participation_votes.iter()
            .filter(|v| v.signal_type == SignalType::Support)
            .map(|v| v.weight)
            .sum();
        let participation_veto: f64 = participation_votes.iter()
            .filter(|v| v.signal_type == SignalType::Veto)
            .map(|v| v.weight)
            .sum();
        
        // Combine all votes
        let total_support = zap_support + participation_support;
        let total_veto = zap_veto + participation_veto;
        let total_votes = total_support + total_veto;
        
        // Check if threshold met
        let threshold_met = total_votes >= threshold as f64;
        
        // Check if veto blocks (40% of total)
        let veto_blocks = if total_votes > 0.0 {
            (total_veto / total_votes) >= 0.4
        } else {
            false
        };
        
        Ok(ProposalVoteResult {
            pr_id,
            tier,
            threshold,
            total_votes,
            support_votes: total_support,
            veto_votes: total_veto,
            zap_vote_count: zap_votes.len(),
            participation_vote_count: participation_votes.len(),
            threshold_met,
            veto_blocks,
        })
    }
    
    fn get_threshold_for_tier(&self, tier: u8) -> Result<u32> {
        match tier {
            1 => Ok(100),
            2 => Ok(500),
            3 => Ok(1_000),
            4 => Ok(2_500),
            5 => Ok(5_000),
            _ => Err(anyhow!("Invalid tier")),
        }
    }
}
```

---

## Key Improvements from Synthesis

### 1. Dollar-Based Weighting ✅

**Your Idea**: Use dollars instead of BTC

**Why Better**: 
- More stable (avoids BTC volatility)
- Easier to understand
- Better for economic projections

**Implementation**: Convert all contributions to USD at time of contribution

### 2. Three Clear Tiers ✅

**Your Idea**: Merge Mining, Fee Forwarding, Community Zaps as distinct tiers

**Why Better**:
- Clearer participation paths
- Easier to explain
- Better for economic modeling

**Implementation**: Track separately but use unified formula

### 3. Fixed Vote Thresholds ✅

**Your Idea**: 100, 500, 1000, 2500, 5000 votes

**Why Better**:
- Easier to understand than percentages
- Clear goals for participation
- Predictable governance

**Implementation**: Use fixed thresholds per tier

### 4. Flexible Time Windows ✅

**Your Idea**: Monthly rolling for mining, cumulative for zaps

**Why Better**:
- Mining contributions are ongoing (monthly makes sense)
- Zaps are one-time (cumulative rewards long-term support)
- More flexible than single 90-day window

**Implementation**: Different time windows per contribution type

### 5. Zap-to-Vote Mechanism ✅

**My Idea**: Zap governance events to vote on proposals

**Why Better**:
- Direct democracy
- No registration required
- Real-time voting

**Implementation**: Track per-proposal zaps separately from cumulative

### 6. Dual Purpose Contributions ✅

**My Idea**: Contributions count for both qualification and voting

**Why Better**:
- Encourages sustained participation
- Rewards long-term contributors
- More efficient use of contributions

**Implementation**: Track both ongoing weight and per-proposal votes

---

## Final Synthesized Model

### Participation Tiers

1. **Merge Mining**: Monthly fees, 30-day rolling, √(monthly_usd)
2. **Fee Forwarding**: Monthly forwards, 30-day rolling, √(monthly_usd)
3. **Community Zaps**: 
   - Cumulative: All-time, √(cumulative_usd) → ongoing participation weight
   - Per-proposal: During voting window, √(zap_usd) → direct vote

### Voting System

**Two Ways to Vote:**

1. **Ongoing Participation**:
   - Weight = √(total_contribution_usd)
   - Submit veto/support signal
   - Counts toward fixed thresholds

2. **Zap-to-Vote**:
   - Zap governance event = vote
   - Weight = √(zap_amount_usd)
   - Can exceed ongoing weight
   - Direct democracy

**Vote Aggregation:**
- All votes combined (zap + participation)
- Fixed thresholds per tier
- Veto blocks if 40%+ of total votes

### Weight Formula

```
ongoing_weight = √(merge_mining_usd + fee_forwarding_usd + cumulative_zaps_usd)
proposal_zap_weight = √(zap_amount_usd)
vote_weight = max(proposal_zap_weight, ongoing_weight * 0.1)
```

---

## Benefits of Synthesis

1. **Stability**: Dollar-based avoids BTC volatility
2. **Clarity**: Three tiers, fixed thresholds
3. **Flexibility**: Multiple time windows, dual voting
4. **Accessibility**: Zap-to-vote for direct democracy
5. **Fairness**: Quadratic weighting prevents plutocracy
6. **Balance**: Miners + community can both participate meaningfully

---

## Implementation Priority

### Phase 1: Core System
1. Dollar-based contribution tracking
2. Three-tier participation system
3. Quadratic weight calculation
4. Fixed vote thresholds

### Phase 2: Zap-to-Vote
1. Governance event publishing
2. Zap vote tracking
3. Per-proposal vote aggregation

### Phase 3: Integration
1. Unified vote aggregation
2. Dashboard and reporting
3. Economic projections

---

## Summary

**Best of Both Models:**

✅ **Your Model**: Dollar-based, three tiers, fixed thresholds, flexible time windows  
✅ **My Model**: Zap-to-vote, dual purpose, unified aggregation

**Result**: A governance system that is:
- Stable (dollar-based)
- Clear (three tiers, fixed thresholds)
- Flexible (multiple participation paths)
- Democratic (zap-to-vote)
- Fair (quadratic weighting)
- Balanced (miners + community)

This synthesis creates the best possible governance model by combining the economic stability and clarity of your approach with the direct democracy and flexibility of mine.

