# Final Governance Implementation Plan

## Overview

This document validates the final governance system design combining:
- **Three Contribution Types**: Merge Mining, Fee Forwarding, Zaps as Votes
- **Four Balancing Mechanisms**: Time Decay (#1), Weight Caps (#2), Cooling-Off (#3), Multiple Rounds (#6)

---

## Final System Design

### Contribution Types

1. **Merge Mining**
   - Monthly merge mining fees (1% of merged chain rewards)
   - 30-day rolling average
   - On-chain verification

2. **Fee Forwarding**
   - Monthly Bitcoin forwarded to development address
   - 30-day rolling average
   - On-chain verification

3. **Zaps as Votes**
   - Cumulative zaps → Ongoing participation weight
   - Proposal zaps → Direct voting on specific proposals
   - All-time cumulative (or 1-year rolling)
   - Nostr event verification (NIP-57)

### Weight Calculation

**Base Formula (Quadratic)**:
```rust
// For ongoing participation
total_contribution_usd = merge_mining_monthly_usd + 
                         fee_forwarding_monthly_usd + 
                         cumulative_zaps_usd
base_weight = sqrt(total_contribution_usd)
```

**With Balancing Mechanisms**:
```rust
// 1. Apply time decay
decayed_weight = apply_time_decay(base_weight, contribution_age)

// 2. Apply weight cap
capped_weight = min(decayed_weight, max_weight_per_entity)

// 3. Check cooling-off period
eligible_weight = if cooling_off_required { 0.0 } else { capped_weight }

// Final weight
final_participation_weight = eligible_weight
```

**For Per-Proposal Voting**:
```rust
// Proposal-specific zap vote
proposal_zap_weight = sqrt(zap_amount_usd)

// OR use ongoing participation weight
participation_weight = final_participation_weight

// Use whichever is higher (encourages both)
vote_weight = max(proposal_zap_weight, participation_weight * 0.1)
```

---

## Balancing Mechanisms

### 1. Time-Based Contribution Decay

**Parameters**:
- **Merge Mining**: 180-day decay period, 10% minimum retention
- **Fee Forwarding**: 180-day decay period, 10% minimum retention
- **Zaps**: 365-day decay period, 10% minimum retention
- **Refresh**: New contributions reset decay timer

**Implementation**:
```rust
fn apply_time_decay(
    contribution_amount_usd: f64,
    contribution_age_days: u32,
    decay_period_days: u32,
) -> f64 {
    let decay_factor = 1.0 - (contribution_age_days as f64 / decay_period_days as f64);
    let effective_factor = decay_factor.max(0.1);  // Minimum 10%
    contribution_amount_usd * effective_factor
}
```

### 2. Per-Entity Weight Caps

**Parameters**:
- **Cap Percentage**: 5% of total system weight
- **Recalculation**: Monthly based on total system weight
- **Grandfathering**: Existing entities above cap gradually reduced over 6 months

**Implementation**:
```rust
fn apply_weight_cap(
    calculated_weight: f64,
    total_system_weight: f64,
    cap_percentage: f64,
) -> f64 {
    let max_weight = total_system_weight * cap_percentage;
    calculated_weight.min(max_weight)
}
```

### 3. Cooling-Off Periods

**Parameters**:
- **Threshold**: $10,000 USD contribution triggers cooling period
- **Cooling Period**: 30 days before contribution counts toward voting
- **Scope**: Applies to all contribution types
- **Exception**: Ongoing participation weight (cumulative) not subject to cooling period

**Implementation**:
```rust
fn check_cooling_off(
    contribution_amount_usd: f64,
    contribution_age_days: u32,
    threshold: f64,
    cooling_period_days: u32,
) -> bool {
    if contribution_amount_usd >= threshold {
        contribution_age_days >= cooling_period_days
    } else {
        true  // No cooling period for small contributions
    }
}
```

### 6. Multiple Voting Rounds

**Parameters**:
- **Tier 4**: 2 rounds, 30 days apart, both must pass
- **Tier 5**: 3 rounds, 60 days apart, all must pass
- **Lower Tiers**: Single round

**Implementation**:
```rust
struct VotingRound {
    round_number: u8,
    start_date: DateTime<Utc>,
    end_date: DateTime<Utc>,
    votes: Vec<Vote>,
    threshold_met: bool,
}

fn requires_multi_round(tier: u8) -> bool {
    tier >= 4
}

fn get_round_requirements(tier: u8) -> (u8, u32) {
    match tier {
        4 => (2, 30),  // 2 rounds, 30 days apart
        5 => (3, 60),  // 3 rounds, 60 days apart
        _ => (1, 0),   // Single round
    }
}
```

---

## Implementation Status

### ✅ Already Implemented

1. **Merge Mining Infrastructure**
   - ✅ `bllvm-node/src/network/stratum_v2/merge_mining.rs` - Core coordinator
   - ✅ Secondary chain configuration
   - ✅ Revenue tracking and distribution calculation
   - ✅ Status: Core infrastructure complete

2. **Fee Calculation**
   - ✅ `bllvm-node/src/node/mempool.rs` - Fee calculation working
   - ✅ `bllvm-node/src/rpc/mining.rs` - Fee calculation in RPC
   - ⚠️ **Missing**: Fees not included in coinbase (needs fix)

3. **Nostr Integration**
   - ✅ `bllvm-commons/src/nostr/` - Nostr client infrastructure
   - ✅ Zap tracking design documented
   - ⚠️ **Missing**: Zap subscription and tracking implementation

4. **Economic Node Registry**
   - ✅ `bllvm-commons/src/economic_nodes/registry.rs` - Registry structure
   - ✅ Qualification verification framework
   - ⚠️ **Missing**: Contributor types (merge mining, fee forwarding, zaps)

### ❌ Needs Implementation

1. **Fee Forwarding**
   - ❌ Fix coinbase to include fees
   - ❌ Add fee forwarding configuration
   - ❌ Track forwarded fees to Commons address
   - ❌ Calculate monthly forwarded amounts

2. **Zap Tracking**
   - ❌ Implement zap subscription in Nostr client
   - ❌ Create zap tracker service
   - ❌ Database schema for zap contributions
   - ❌ Integration with contributor qualification

3. **Contributor Qualification System**
   - ❌ Extend `NodeType` enum with contributor types
   - ❌ Add qualification proofs for merge mining, fee forwarding, zaps
   - ❌ Implement weight calculation with balancing mechanisms
   - ❌ Database schema for unified contributions

4. **Weight Calculation with Balancing**
   - ❌ Time decay calculation
   - ❌ Weight cap calculation
   - ❌ Cooling-off period checks
   - ❌ Monthly weight recalculation

5. **Multiple Voting Rounds**
   - ❌ Voting round tracking
   - ❌ Round scheduling logic
   - ❌ Multi-round approval checks
   - ❌ Integration with governance app

6. **Dollar-Based Conversion**
   - ❌ BTC price service
   - ❌ USD conversion at contribution time
   - ❌ Price history tracking

---

## Database Schema

### Unified Contributions Table

```sql
CREATE TABLE unified_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contributor_id TEXT NOT NULL,
    contributor_type TEXT NOT NULL,  -- 'merge_miner', 'fee_forwarder', 'zap_user'
    contribution_type TEXT NOT NULL,  -- 'merge_mining', 'fee_forwarding', 'zap'
    amount_btc REAL NOT NULL,
    amount_usd REAL NOT NULL,  -- Calculated at time of contribution
    btc_price_usd REAL NOT NULL,  -- Price at time of contribution
    timestamp DATETIME NOT NULL,
    period_type TEXT NOT NULL,  -- 'monthly', 'cumulative'
    contribution_age_days INTEGER DEFAULT 0,  -- For decay calculation
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_contributor ON unified_contributions(contributor_id);
CREATE INDEX idx_contributor_type ON unified_contributions(contributor_type);
CREATE INDEX idx_timestamp ON unified_contributions(timestamp);
CREATE INDEX idx_contributor_time ON unified_contributions(contributor_id, timestamp);
```

### Zap Contributions Table

```sql
CREATE TABLE zap_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient_pubkey TEXT NOT NULL,
    sender_pubkey TEXT,
    amount_msat INTEGER NOT NULL,
    amount_btc REAL NOT NULL,
    amount_usd REAL NOT NULL,
    btc_price_usd REAL NOT NULL,
    timestamp DATETIME NOT NULL,
    invoice_hash TEXT,
    message TEXT,
    zapped_event_id TEXT,  -- For proposal zaps
    is_proposal_zap BOOLEAN DEFAULT FALSE,
    governance_event_id TEXT,  -- If zapping a governance proposal
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_zap_recipient ON zap_contributions(recipient_pubkey);
CREATE INDEX idx_zap_sender ON zap_contributions(sender_pubkey);
CREATE INDEX idx_zap_timestamp ON zap_contributions(timestamp);
CREATE INDEX idx_zap_governance ON zap_contributions(governance_event_id);
```

### Participation Weights Table

```sql
CREATE TABLE participation_weights (
    contributor_id TEXT PRIMARY KEY,
    contributor_type TEXT NOT NULL,
    merge_mining_usd REAL DEFAULT 0.0,
    fee_forwarding_usd REAL DEFAULT 0.0,
    cumulative_zaps_usd REAL DEFAULT 0.0,
    total_contribution_usd REAL NOT NULL,
    base_weight REAL NOT NULL,  -- sqrt(total_contribution_usd)
    decayed_weight REAL NOT NULL,  -- After time decay
    capped_weight REAL NOT NULL,  -- After weight cap
    final_weight REAL NOT NULL,  -- Final eligible weight
    total_system_weight REAL NOT NULL,  -- For cap calculation
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Voting Rounds Table

```sql
CREATE TABLE voting_rounds (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    proposal_id INTEGER NOT NULL,
    round_number INTEGER NOT NULL,
    tier INTEGER NOT NULL,
    start_date DATETIME NOT NULL,
    end_date DATETIME NOT NULL,
    threshold REAL NOT NULL,
    total_votes REAL DEFAULT 0.0,
    support_votes REAL DEFAULT 0.0,
    veto_votes REAL DEFAULT 0.0,
    threshold_met BOOLEAN DEFAULT FALSE,
    status TEXT NOT NULL,  -- 'pending', 'active', 'completed', 'failed'
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(proposal_id, round_number)
);

CREATE INDEX idx_proposal_rounds ON voting_rounds(proposal_id);
CREATE INDEX idx_status ON voting_rounds(status);
```

---

## Implementation Roadmap

### Phase 1: Core Infrastructure (2-3 months)

#### 1.1 Fix Fee Forwarding (1 month)
- [ ] Fix coinbase transaction to include fees
- [ ] Add fee forwarding configuration
- [ ] Implement fee tracking to Commons address
- [ ] Add monthly fee aggregation

**Files to Modify**:
- `bllvm-node/src/node/miner.rs` - Fix coinbase creation
- `bllvm-node/src/config.rs` - Add fee forwarding config
- `bllvm-node/src/node/mempool.rs` - Track forwarded fees

#### 1.2 Zap Tracking (1 month)
- [ ] Implement zap subscription in Nostr client
- [ ] Create zap tracker service
- [ ] Database schema for zap contributions
- [ ] Integration with contributor qualification

**Files to Create**:
- `bllvm-commons/src/nostr/zap_tracker.rs`
- `bllvm-commons/src/nostr/zap_voting.rs`
- Database migrations for zap tables

**Files to Modify**:
- `bllvm-commons/src/nostr/client.rs` - Add zap subscription
- `bllvm-commons/src/nostr/mod.rs` - Export new modules

#### 1.3 Dollar-Based Conversion (2 weeks)
- [ ] BTC price service
- [ ] USD conversion at contribution time
- [ ] Price history tracking

**Files to Create**:
- `bllvm-commons/src/services/btc_price.rs`

**Files to Modify**:
- `bllvm-commons/src/config.rs` - Add price service config

### Phase 2: Contributor Qualification (2-3 months)

#### 2.1 Extend Economic Node Types (1 month)
- [ ] Add contributor types to `NodeType` enum
- [ ] Add qualification proofs for merge mining, fee forwarding, zaps
- [ ] Update qualification verification

**Files to Modify**:
- `bllvm-commons/src/economic_nodes/types.rs` - Add contributor types
- `bllvm-commons/src/economic_nodes/registry.rs` - Update qualification

#### 2.2 Unified Contribution Tracking (1 month)
- [ ] Database schema for unified contributions
- [ ] Contribution tracking service
- [ ] Monthly aggregation for merge mining and fee forwarding
- [ ] Cumulative tracking for zaps

**Files to Create**:
- `bllvm-commons/src/governance/contributions.rs`
- `bllvm-commons/src/governance/aggregator.rs`

**Files to Modify**:
- Database migrations

#### 2.3 Weight Calculation with Balancing (1 month)
- [ ] Time decay calculation
- [ ] Weight cap calculation
- [ ] Cooling-off period checks
- [ ] Monthly weight recalculation

**Files to Create**:
- `bllvm-commons/src/governance/weight_calculator.rs`

**Files to Modify**:
- `bllvm-commons/src/economic_nodes/registry.rs` - Use new weight calculator

### Phase 3: Voting System (2-3 months)

#### 3.1 Multiple Voting Rounds (1 month)
- [ ] Voting round tracking
- [ ] Round scheduling logic
- [ ] Multi-round approval checks

**Files to Create**:
- `bllvm-commons/src/governance/voting_rounds.rs`

**Files to Modify**:
- `bllvm-commons/src/governance/vote_aggregator.rs` - Add round support

#### 3.2 Zap-to-Vote Integration (1 month)
- [ ] Governance event publishing
- [ ] Proposal zap tracking
- [ ] Per-proposal vote aggregation

**Files to Modify**:
- `bllvm-commons/src/nostr/zap_voting.rs` - Add proposal zap support
- `bllvm-commons/src/governance/vote_aggregator.rs` - Integrate zap votes

#### 3.3 Governance App Integration (1 month)
- [ ] Update governance app to use new voting system
- [ ] Multi-round scheduling
- [ ] Weight calculation integration
- [ ] Dashboard and reporting

**Files to Modify**:
- Governance app (external repository)

---

## Validation Checklist

### Design Validation

- ✅ **Three Contribution Types**: Merge mining, fee forwarding, zaps
- ✅ **Quadratic Weighting**: Prevents whale dominance
- ✅ **Dollar-Based**: Stable, avoids BTC volatility
- ✅ **Time Decay**: Prevents permanent power structures
- ✅ **Weight Caps**: Prevents absolute dominance
- ✅ **Cooling-Off**: Prevents vote buying
- ✅ **Multiple Rounds**: Ensures sustained consensus

### Implementation Readiness

- ✅ **Merge Mining**: Core infrastructure ready
- ⚠️ **Fee Forwarding**: Needs coinbase fix + tracking
- ⚠️ **Zap Tracking**: Design ready, needs implementation
- ⚠️ **Contributor Qualification**: Framework ready, needs extension
- ⚠️ **Weight Calculation**: Needs balancing mechanisms
- ⚠️ **Voting Rounds**: Needs implementation
- ⚠️ **Dollar Conversion**: Needs price service

### Gaps Identified

1. **Fee Forwarding**: Coinbase bug needs fixing first
2. **Zap Tracking**: Nostr client needs zap subscription
3. **Contributor Types**: Need to extend existing economic node system
4. **Weight Calculation**: Need to implement balancing mechanisms
5. **Voting Rounds**: New feature, needs design and implementation
6. **Price Service**: Need external price API integration

---

## Next Steps

### Immediate (Week 1-2)

1. **Review and Validate**
   - [ ] Review this plan with stakeholders
   - [ ] Validate design decisions
   - [ ] Confirm implementation priorities

2. **Fix Critical Bug**
   - [ ] Fix coinbase transaction to include fees
   - [ ] Add tests for fee inclusion
   - [ ] Verify fee calculation accuracy

### Short-Term (Month 1-3)

1. **Phase 1 Implementation**
   - [ ] Fee forwarding tracking
   - [ ] Zap tracking infrastructure
   - [ ] Dollar-based conversion

### Medium-Term (Month 4-6)

1. **Phase 2 Implementation**
   - [ ] Contributor qualification system
   - [ ] Unified contribution tracking
   - [ ] Weight calculation with balancing

### Long-Term (Month 7-9)

1. **Phase 3 Implementation**
   - [ ] Multiple voting rounds
   - [ ] Zap-to-vote integration
   - [ ] Governance app integration

---

## Summary

**Status**: ✅ **Ready to Implement** (with identified gaps)

**What's Ready**:
- Design validated and complete
- Core infrastructure exists (merge mining, economic nodes)
- Clear implementation roadmap
- Database schema designed

**What Needs Work**:
- Fee forwarding (coinbase fix + tracking)
- Zap tracking (Nostr subscription)
- Contributor qualification extension
- Weight calculation with balancing
- Multiple voting rounds
- Dollar conversion service

**Estimated Timeline**: 6-9 months for full implementation

**Recommendation**: Start with Phase 1 (fee forwarding fix + zap tracking), then proceed to Phase 2 and Phase 3 sequentially.

