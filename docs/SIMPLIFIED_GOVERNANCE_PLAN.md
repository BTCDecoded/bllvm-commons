# Simplified Governance Implementation Plan - 30 Days

## Overview

**Goal**: Ship basic zap voting + merge mining tracking in 30 days

**Skip for now**:
- Time decay (#1)
- Multi-round voting (#6)
- Dollar conversion (use BTC directly)
- Diversity requirements (#4)

**Include**:
- Weight caps (#2) - Simple, prevents whale dominance
- Cooling-off periods (#3) - Prevents vote buying, simple to implement

**Focus**: Core functionality only - track contributions and enable voting

---

## What We're Building

### 1. Basic Zap Voting
- Track zaps to governance events (NIP-57)
- Calculate vote weight using quadratic formula: `√(zap_amount_btc)`
- Simple voting: zap amount = vote weight

### 2. Merge Mining Tracking
- Track merge mining contributions (1% of secondary chain rewards)
- Calculate vote weight: `√(merge_mining_contribution_btc)`
- Monthly aggregation (30-day rolling)

### 3. Fee Forwarding Tracking
- Track fees forwarded to Commons address
- Calculate vote weight: `√(fee_forwarding_btc)`
- Monthly aggregation (30-day rolling)
- On-chain verification (track transactions to Commons address)

---

## Simplified Weight Formula

**No dollar conversion - use BTC directly:**

```rust
// For ongoing participation (qualification)
total_contribution_btc = merge_mining_btc + fee_forwarding_btc + cumulative_zaps_btc
participation_weight = sqrt(total_contribution_btc)

// For per-proposal voting (zap-to-vote)
proposal_zap_weight = sqrt(zap_amount_btc)
vote_weight = max(proposal_zap_weight, participation_weight * 0.1)
```

**Minimal balancing**: Quadratic weighting + weight caps (5% max) + cooling-off (30 days for ≥0.1 BTC)

---

## Implementation Tasks

### Phase 1: Core Infrastructure (Week 1-2)

#### 1.1 Database Schema ✅
- [x] Unified contributions table (BTC-based)
- [x] Zap contributions table
- [x] Participation weights table (cached)
- [x] Proposal zap votes table
- [x] Fee forwarding tracking table

#### 1.2 Zap Tracking (Week 1)
- [ ] Implement zap subscription in Nostr client (NIP-57 kind 9735)
- [ ] Create zap tracker service
- [ ] Track zaps to bot pubkeys
- [ ] Track proposal zaps (zaps to governance events)

#### 1.3 Merge Mining Tracking (Week 1-2)
- [x] Merge mining infrastructure exists
- [ ] Add contribution tracking to merge mining coordinator
- [ ] Record 1% fee contributions
- [ ] Monthly aggregation (30-day rolling)

#### 1.4 Fee Forwarding Tracking (Week 1-2)
- [x] Coinbase fix complete (fees now included)
- [ ] Add fee forwarding configuration (Commons address)
- [ ] Track transactions to Commons address
- [ ] Calculate forwarded amounts
- [ ] Monthly aggregation (30-day rolling)

### Phase 2: Weight Calculation (Week 2-3)

#### 2.1 Weight Calculator (Week 2)
- [ ] Implement quadratic weight calculation
- [ ] Calculate participation weight (all contributions)
- [ ] Calculate proposal zap weight
- [ ] Implement weight cap (5% of total system weight)
- [ ] Implement cooling-off period (30 days for ≥0.1 BTC)
- [ ] Apply cap to all weights
- [ ] Check cooling-off before allowing votes
- [ ] Use max(proposal_zap, participation * 0.1)

#### 2.2 Contribution Aggregation (Week 2-3)
- [ ] Monthly aggregation for merge mining
- [ ] Monthly aggregation for fee forwarding
- [ ] Cumulative tracking for zaps
- [ ] Calculate total system weight
- [ ] Update participation weights (with caps)

### Phase 3: Voting Integration (Week 3-4)

#### 3.1 Governance Event Publishing (Week 3)
- [ ] Publish governance proposals to Nostr
- [ ] Include zap address in event
- [ ] Track voting window

#### 3.2 Vote Aggregation (Week 3-4)
- [ ] Aggregate zap votes for proposals
- [ ] Aggregate participation-based votes
- [ ] Calculate total vote weight
- [ ] Check thresholds (fixed vote counts)

#### 3.3 Integration (Week 4)
- [ ] Connect to governance app
- [ ] Display vote weights
- [ ] Show contribution totals

---

## Database Schema (Simplified)

### Unified Contributions (BTC-based)

```sql
CREATE TABLE unified_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contributor_id TEXT NOT NULL,
    contributor_type TEXT NOT NULL,  -- 'merge_miner', 'fee_forwarder', 'zap_user'
    contribution_type TEXT NOT NULL,  -- 'merge_mining', 'fee_forwarding', 'zap'
    amount_btc REAL NOT NULL,  -- No USD conversion
    timestamp DATETIME NOT NULL,
    contribution_age_days INTEGER DEFAULT 0,  -- For cooling-off check
    period_type TEXT NOT NULL,  -- 'monthly', 'cumulative'
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Zap Contributions

```sql
CREATE TABLE zap_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient_pubkey TEXT NOT NULL,
    sender_pubkey TEXT,
    amount_msat INTEGER NOT NULL,
    amount_btc REAL NOT NULL,
    timestamp DATETIME NOT NULL,
    invoice_hash TEXT,
    message TEXT,
    zapped_event_id TEXT,  -- For proposal zaps
    is_proposal_zap BOOLEAN DEFAULT FALSE,
    governance_event_id TEXT,  -- If zapping a governance proposal
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Fee Forwarding Contributions

```sql
CREATE TABLE fee_forwarding_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contributor_id TEXT NOT NULL,  -- Miner/node identifier
    tx_hash TEXT NOT NULL,  -- Transaction hash
    block_height INTEGER NOT NULL,
    amount_btc REAL NOT NULL,  -- Amount forwarded to Commons address
    commons_address TEXT NOT NULL,  -- Commons address that received funds
    timestamp DATETIME NOT NULL,
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(tx_hash)  -- Prevent duplicate tracking
);

CREATE INDEX idx_fee_forwarding_contributor ON fee_forwarding_contributions(contributor_id);
CREATE INDEX idx_fee_forwarding_timestamp ON fee_forwarding_contributions(timestamp);
CREATE INDEX idx_fee_forwarding_contributor_time ON fee_forwarding_contributions(contributor_id, timestamp);
```

### Participation Weights (Simplified)

```sql
CREATE TABLE participation_weights (
    contributor_id TEXT PRIMARY KEY,
    contributor_type TEXT NOT NULL,
    merge_mining_btc REAL DEFAULT 0.0,
    fee_forwarding_btc REAL DEFAULT 0.0,
    cumulative_zaps_btc REAL DEFAULT 0.0,
    total_contribution_btc REAL NOT NULL,
    base_weight REAL NOT NULL,  -- sqrt(total_contribution_btc)
    capped_weight REAL NOT NULL,  -- After 5% cap applied
    total_system_weight REAL NOT NULL,  -- For cap calculation
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### Proposal Zap Votes

```sql
CREATE TABLE proposal_zap_votes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pr_id INTEGER NOT NULL,
    governance_event_id TEXT NOT NULL,
    sender_pubkey TEXT NOT NULL,
    amount_msat INTEGER NOT NULL,
    amount_btc REAL NOT NULL,
    vote_weight REAL NOT NULL,  -- sqrt(amount_btc)
    vote_type TEXT NOT NULL,  -- 'support', 'veto', 'abstain'
    timestamp DATETIME NOT NULL,
    verified BOOLEAN DEFAULT FALSE
);
```

---

## Weight Calculation (Simplified)

```rust
// bllvm-commons/src/governance/weight_calculator.rs

pub struct WeightCalculator {
    cap_percentage: f64,  // 0.05 = 5% cap
}

impl WeightCalculator {
    pub fn new() -> Self {
        Self {
            cap_percentage: 0.05,  // 5% cap
        }
    }
    
    /// Calculate ongoing participation weight (quadratic, BTC-based)
    pub fn calculate_participation_weight(
        &self,
        merge_mining_btc: f64,
        fee_forwarding_btc: f64,
        cumulative_zaps_btc: f64,
    ) -> f64 {
        let total_btc = merge_mining_btc + fee_forwarding_btc + cumulative_zaps_btc;
        total_btc.sqrt()
    }
    
    /// Apply weight cap to prevent whale dominance
    pub fn apply_weight_cap(
        &self,
        calculated_weight: f64,
        total_system_weight: f64,
    ) -> f64 {
        let max_weight = total_system_weight * self.cap_percentage;
        calculated_weight.min(max_weight)
    }
    
    /// Calculate per-proposal zap vote weight (quadratic, BTC-based)
    pub fn calculate_zap_vote_weight(&self, zap_amount_btc: f64) -> f64 {
        zap_amount_btc.sqrt()
    }
    
    /// Check if contribution is eligible for voting (cooling-off period)
    pub fn check_cooling_off(
        &self,
        contribution_amount_btc: f64,
        contribution_age_days: u32,
        threshold_btc: f64,  // 0.1 BTC
        cooling_period_days: u32,  // 30 days
    ) -> bool {
        if contribution_amount_btc >= threshold_btc {
            contribution_age_days >= cooling_period_days
        } else {
            true  // No cooling period for small contributions
        }
    }
    
    /// Get vote weight for proposal (uses higher of zap or participation)
    pub fn get_proposal_vote_weight(
        &self,
        participation_weight: f64,
        proposal_zap_amount_btc: Option<f64>,
        total_system_weight: f64,
        contribution_age_days: Option<u32>,  // For cooling-off check
    ) -> f64 {
        let base_weight = if let Some(zap_btc) = proposal_zap_amount_btc {
            // Check cooling-off for proposal zap
            if let Some(age) = contribution_age_days {
                if !self.check_cooling_off(zap_btc, age, 0.1, 30) {
                    // Contribution too new, use participation weight only
                    return self.apply_weight_cap(participation_weight, total_system_weight);
                }
            }
            let zap_weight = self.calculate_zap_vote_weight(zap_btc);
            // Use 10% of participation weight as minimum, or zap weight if higher
            zap_weight.max(participation_weight * 0.1)
        } else {
            participation_weight
        };
        
        // Apply weight cap
        self.apply_weight_cap(base_weight, total_system_weight)
    }
}
```

**Minimal balancing**: Quadratic weighting + 5% weight cap per entity

---

## What We're NOT Doing (For Now)

### Skipped Features

1. **Time Decay** - Contributions don't expire
2. **Weight Caps** - ✅ **INCLUDED** (5% cap per entity)
3. **Cooling-Off Periods** - ✅ **INCLUDED** (30 days for ≥0.1 BTC)
4. **Multi-Round Voting** - Single round only
5. **Dollar Conversion** - Use BTC directly
6. **Reputation Multipliers** - No behavior adjustments
7. **Diversity Requirements** - No tier requirements
8. **Delegation** - No delegation system
9. **Anti-Coordination** - No coordination detection
10. **Exit Mechanisms** - No exit signals

**Rationale**: Keep it simple, ship fast, iterate later

---

## 30-Day Timeline

### Week 1: Zap Tracking + Merge Mining + Fee Forwarding
- **Days 1-3**: Zap subscription and tracking
- **Days 4-5**: Merge mining contribution tracking
- **Days 6-7**: Fee forwarding tracking + database schema

### Week 2: Weight Calculation
- **Days 8-10**: Weight calculator implementation
- **Days 11-12**: Contribution aggregation
- **Days 13-14**: Participation weight updates

### Week 3: Voting System
- **Days 15-17**: Governance event publishing
- **Days 18-19**: Vote aggregation
- **Days 20-21**: Integration with governance app

### Week 4: Testing & Polish
- **Days 22-24**: Testing and bug fixes
- **Days 25-26**: Documentation
- **Days 27-28**: Final integration
- **Days 29-30**: Deployment preparation

---

## Success Criteria

### Must Have (MVP)
- ✅ Zap tracking works (NIP-57)
- ✅ Merge mining contributions tracked
- ✅ Fee forwarding contributions tracked
- ✅ Quadratic weight calculation
- ✅ Basic voting on proposals
- ✅ Vote aggregation works

### Nice to Have (If Time)
- Dashboard for viewing contributions
- Vote history

### Future Iterations
- Add balancing mechanisms if needed
- Dollar conversion if volatility becomes issue
- Multi-round voting for major changes
- Time decay if power concentration becomes problem

---

## Implementation Files

### New Files to Create

1. `bllvm-commons/src/nostr/zap_tracker.rs` - Zap tracking service
2. `bllvm-commons/src/nostr/zap_voting.rs` - Zap-to-vote logic
3. `bllvm-commons/src/governance/weight_calculator.rs` - Weight calculation
4. `bllvm-commons/src/governance/contributions.rs` - Contribution tracking
5. `bllvm-commons/src/governance/aggregator.rs` - Monthly aggregation
6. `bllvm-commons/src/governance/fee_forwarding.rs` - Fee forwarding tracker
7. `bllvm-commons/migrations/XXXX_governance_contributions.sql` - Database schema

### Files to Modify

1. `bllvm-commons/src/nostr/client.rs` - Add zap subscription
2. `bllvm-node/src/network/stratum_v2/merge_mining.rs` - Add contribution tracking
3. `bllvm-node/src/node/miner.rs` - Add fee forwarding configuration
4. `bllvm-commons/src/economic_nodes/registry.rs` - Use new weight calculator
5. `bllvm-commons/src/config.rs` - Add governance contribution config (Commons address)

---

## Summary

**Simplified Approach**:
- ✅ BTC-based (no dollar conversion)
- ✅ Quadratic weighting + 5% weight cap
- ✅ Cooling-off periods (30 days for ≥0.1 BTC)
- ✅ Minimal balancing (caps + cooling-off)
- ✅ Single-round voting
- ✅ Basic tracking and voting

**Timeline**: 30 days to MVP

**Next Steps**: Start with zap tracking, then merge mining integration

