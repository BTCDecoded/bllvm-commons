# Governance Power Balancing Mechanisms

## Overview

This document explores additional mechanisms to balance power in the Bitcoin Commons governance system, complementing existing safeguards (quadratic weighting, Sybil resistance, rationale requirements, public accountability).

**Goal**: Prevent capture, ensure fair representation, and maintain system integrity while preserving efficiency and participation incentives.

---

## Existing Balancing Mechanisms

### Current Safeguards

1. **Quadratic Weighting**: `√(contribution)` prevents linear whale dominance
2. **Sybil Resistance**: Cryptographic proofs, minimum thresholds, entity verification
3. **Rationale Requirements**: Vetoes must include substantive technical analysis
4. **Public Accountability**: All votes and vetoes are public and signed
5. **Time Limits**: Vetoes only during review windows
6. **Reputation System**: Good behavior increases influence, bad behavior decreases
7. **Fixed Thresholds**: Clear, predictable vote requirements per tier

---

## Proposed Additional Balancing Mechanisms

### 1. Time-Based Contribution Decay

**Problem**: Long-term contributors accumulate unlimited power, creating permanent power structures.

**Solution**: Contributions decay over time, requiring ongoing participation.

```rust
// Contribution decay formula
decay_factor = 1.0 - (age_in_days / decay_period_days)
effective_contribution = original_contribution * max(decay_factor, 0.1)  // Minimum 10% retained

// Example: 6-month decay period
// - 0 days old: 100% weight
// - 90 days old: 50% weight
// - 180 days old: 0% weight (but minimum 10% retained)
```

**Parameters**:
- **Decay Period**: 180 days (6 months) for merge mining/fee forwarding
- **Decay Period**: 365 days (1 year) for cumulative zaps (longer-term commitment)
- **Minimum Retention**: 10% of original weight (prevents complete loss)
- **Refresh Mechanism**: New contributions reset decay timer

**Benefits**:
- Prevents permanent power accumulation
- Encourages ongoing participation
- Allows new participants to gain influence
- Maintains some recognition for historical contributions

**Trade-offs**:
- May discourage long-term planning
- Requires more frequent participation
- Could favor short-term actors

**Recommendation**: **Implement with 180-day decay for mining contributions, 365-day for zaps**

---

### 2. Per-Entity Weight Caps

**Problem**: Even with quadratic weighting, extremely large contributors could still dominate.

**Solution**: Cap maximum weight per entity at a fixed percentage of total system weight.

```rust
// Weight cap calculation
max_weight_per_entity = total_system_weight * cap_percentage
capped_weight = min(calculated_weight, max_weight_per_entity)

// Example: 5% cap
// If total system weight = 10,000 votes
// Max per entity = 500 votes (5%)
// Even if quadratic weight = 1,000, cap at 500
```

**Parameters**:
- **Cap Percentage**: 5% of total system weight
- **Recalculation**: Cap recalculated monthly based on total system weight
- **Grandfathering**: Existing entities above cap gradually reduced over 6 months

**Benefits**:
- Prevents absolute dominance by single entity
- Ensures minimum diversity requirement
- Maintains quadratic benefits for smaller contributors
- Prevents coordinated single-entity capture

**Trade-offs**:
- May discourage very large contributions
- Requires tracking total system weight
- Could be gamed by splitting into multiple entities (mitigated by Sybil resistance)

**Recommendation**: **Implement with 5% cap, recalculated monthly**

---

### 3. Minimum Diversity Requirements

**Problem**: All votes could come from a single tier (e.g., only miners), creating imbalance.

**Solution**: Require participation from multiple tiers for major decisions.

```rust
// Diversity check
fn check_diversity_requirement(
    votes: &[Vote],
    tier: u8,
) -> bool {
    if tier >= 4 {  // Major changes only
        let has_mining = votes.iter().any(|v| v.source == "merge_mining" || v.source == "fee_forwarding");
        let has_zaps = votes.iter().any(|v| v.source == "zap");
        let has_economic = votes.iter().any(|v| v.source == "economic_node");
        
        // Require at least 2 of 3 tiers
        let tier_count = [has_mining, has_zaps, has_economic].iter().filter(|&&x| x).count();
        tier_count >= 2
    } else {
        true  // No requirement for minor changes
    }
}
```

**Parameters**:
- **Tier Threshold**: Apply only to Tier 4+ (major changes)
- **Required Tiers**: At least 2 of 3 tiers must participate
- **Tier Definitions**:
  - Tier A: Mining (merge mining + fee forwarding)
  - Tier B: Community (zaps)
  - Tier C: Economic nodes (exchanges, custodians, etc.)

**Benefits**:
- Ensures broad representation
- Prevents single-tier capture
- Encourages cross-tier participation
- Maintains system legitimacy

**Trade-offs**:
- Could block legitimate proposals if one tier abstains
- Requires careful tier definition
- May slow down decision-making

**Recommendation**: **Implement for Tier 4+ only, require 2 of 3 tiers**

---

### 4. Cooling-Off Periods

**Problem**: Large contributions could be timed to influence specific votes.

**Solution**: Require time delay between large contributions and voting eligibility.

```rust
// Cooling-off period
fn can_vote_with_contribution(
    contribution_amount_usd: f64,
    contribution_age_days: u32,
) -> bool {
    let threshold = 10_000.0;  // $10,000 threshold
    let cooling_period_days = 30;  // 30-day cooling period
    
    if contribution_amount_usd >= threshold {
        contribution_age_days >= cooling_period_days
    } else {
        true  // No cooling period for small contributions
    }
}
```

**Parameters**:
- **Threshold**: $10,000 USD contribution triggers cooling period
- **Cooling Period**: 30 days before contribution counts toward voting
- **Scope**: Applies to all contribution types
- **Exception**: Ongoing participation weight (cumulative) not subject to cooling period

**Benefits**:
- Prevents vote buying
- Prevents timing attacks
- Encourages long-term commitment
- Maintains integrity of voting process

**Trade-offs**:
- Delays legitimate participation
- May discourage large contributions
- Requires tracking contribution age

**Recommendation**: **Implement with $10,000 threshold, 30-day cooling period**

---

### 5. Reputation Multipliers

**Problem**: No distinction between good-faith and bad-faith participants.

**Solution**: Adjust weight based on historical behavior and contribution quality.

```rust
// Reputation multiplier
fn calculate_reputation_multiplier(
    entity_id: &str,
    behavior_score: f64,  // 0.0 to 1.0
) -> f64 {
    // Base multiplier: 0.5 to 1.5
    // Good behavior (0.8-1.0): 1.0 to 1.5 multiplier
    // Neutral (0.5-0.8): 1.0 multiplier
    // Bad behavior (0.0-0.5): 0.5 to 1.0 multiplier
    
    if behavior_score >= 0.8 {
        1.0 + (behavior_score - 0.8) * 2.5  // 1.0 to 1.5
    } else if behavior_score >= 0.5 {
        1.0
    } else {
        0.5 + behavior_score * 1.0  // 0.5 to 1.0
    }
}

// Behavior score factors
// +0.1: Valid technical rationale for vetoes
// +0.1: Consistent participation over 6+ months
// +0.1: Public transparency and disclosure
// -0.2: Frivolous vetoes (no rationale)
// -0.3: Pattern of bad-faith behavior
// -0.5: Proven Sybil attack or fraud
```

**Parameters**:
- **Score Range**: 0.0 to 1.0
- **Multiplier Range**: 0.5x to 1.5x
- **Update Frequency**: Quarterly recalculation
- **Transparency**: All scores and multipliers public

**Benefits**:
- Rewards good behavior
- Penalizes bad behavior
- Encourages quality participation
- Maintains system integrity

**Trade-offs**:
- Requires subjective judgment
- Could be gamed
- May create power imbalances
- Requires careful implementation

**Recommendation**: **Implement with conservative scoring, quarterly updates, full transparency**

---

### 6. Delegation Mechanisms

**Problem**: Small contributors may feel their votes don't matter.

**Solution**: Allow smaller contributors to delegate their voting power to trusted entities.

```rust
// Delegation system
struct Delegation {
    delegator_id: String,
    delegate_id: String,
    contribution_types: Vec<String>,  // Which contribution types to delegate
    expiration_date: DateTime,
    revocable: bool,
}

// Delegated weight calculation
fn calculate_delegated_weight(
    delegate_id: &str,
    proposal_id: i32,
) -> f64 {
    let delegations = get_active_delegations(delegate_id);
    let total_delegated_weight: f64 = delegations.iter()
        .map(|d| calculate_contributor_weight(&d.delegator_id))
        .sum();
    
    // Delegated weight also uses quadratic formula
    total_delegated_weight.sqrt()
}
```

**Parameters**:
- **Delegation Scope**: Can delegate specific contribution types or all
- **Expiration**: Default 90 days, renewable
- **Revocability**: Always revocable by delegator
- **Transparency**: All delegations public
- **Quadratic Application**: Delegated weight also subject to quadratic formula

**Benefits**:
- Empowers small contributors
- Enables expert representation
- Maintains quadratic benefits
- Increases participation

**Trade-offs**:
- Could concentrate power
- Requires trust in delegates
- May reduce direct participation
- Needs careful design to prevent abuse

**Recommendation**: **Implement with strict transparency, revocability, and quadratic weighting**

---

### 7. Anti-Coordination Detection

**Problem**: Coordinated voting by related entities could bypass quadratic weighting.

**Solution**: Detect and penalize coordinated voting patterns.

```rust
// Coordination detection
fn detect_coordination(
    votes: &[Vote],
) -> Vec<CoordinationGroup> {
    // Detection heuristics:
    // 1. Same voting pattern across multiple entities
    // 2. Timing correlation (votes within short window)
    // 3. Geographic/IP correlation
    // 4. Known business relationships
    
    let mut groups = Vec::new();
    
    // Check for identical voting patterns
    for vote_group in group_by_pattern(votes) {
        if vote_group.len() >= 3 {  // 3+ entities with identical pattern
            groups.push(CoordinationGroup {
                entities: vote_group,
                confidence: 0.7,
                penalty: 0.5,  // 50% weight reduction
            });
        }
    }
    
    groups
}

// Penalty application
fn apply_coordination_penalty(
    weight: f64,
    penalty: f64,
) -> f64 {
    weight * (1.0 - penalty)
}
```

**Parameters**:
- **Detection Threshold**: 3+ entities with identical patterns
- **Penalty**: 50% weight reduction for detected coordination
- **Appeal Process**: Entities can challenge detection
- **Transparency**: All detections and penalties public

**Benefits**:
- Prevents coordinated capture
- Maintains quadratic benefits
- Deters gaming
- Preserves system integrity

**Trade-offs**:
- Requires sophisticated detection
- Could have false positives
- May be gamed with better coordination
- Needs careful calibration

**Recommendation**: **Implement conservatively, with strong appeal process and transparency**

---

### 8. Exit Mechanisms

**Problem**: Participants may feel locked in if governance goes wrong.

**Solution**: Provide clear, easy exit mechanisms with proportional representation.

```rust
// Exit mechanism
struct ExitSignal {
    entity_id: String,
    exit_reason: String,
    exit_timestamp: DateTime,
    weight_at_exit: f64,
}

// Exit impact
fn calculate_exit_impact(
    exits: &[ExitSignal],
    total_system_weight: f64,
) -> f64 {
    let total_exit_weight: f64 = exits.iter()
        .map(|e| e.weight_at_exit)
        .sum();
    
    total_exit_weight / total_system_weight  // Percentage of system exiting
}

// Emergency override threshold
const EXIT_OVERRIDE_THRESHOLD: f64 = 0.25;  // 25% of system weight exits

// If 25%+ exits, governance change is automatically blocked
```

**Parameters**:
- **Exit Signal**: Public declaration of exit with reason
- **Override Threshold**: 25% of system weight exiting blocks change
- **Cooling Period**: 30 days between exit signal and override activation
- **Transparency**: All exits public and logged

**Benefits**:
- Provides safety valve
- Prevents forced participation
- Maintains user sovereignty
- Deters bad governance

**Trade-offs**:
- Could be used strategically
- May create instability
- Requires careful threshold setting
- Needs clear communication

**Recommendation**: **Implement with 25% threshold, 30-day cooling period, full transparency**

---

### 9. Multiple Voting Rounds

**Problem**: Single votes may not reflect true consensus over time.

**Solution**: Require multiple votes over time for major changes.

```rust
// Multi-round voting
struct VotingRound {
    round_number: u8,
    start_date: DateTime,
    end_date: DateTime,
    votes: Vec<Vote>,
    threshold_met: bool,
}

// Multi-round requirement
fn requires_multi_round(
    tier: u8,
) -> bool {
    tier >= 4  // Tier 4+ requires multiple rounds
}

// Round requirements
fn get_round_requirements(
    tier: u8,
) -> (u8, u32) {  // (rounds, days_between)
    match tier {
        4 => (2, 30),  // 2 rounds, 30 days apart
        5 => (3, 60),  // 3 rounds, 60 days apart
        _ => (1, 0),   // Single round
    }
}

// All rounds must pass
fn check_multi_round_approval(
    rounds: &[VotingRound],
) -> bool {
    rounds.iter().all(|r| r.threshold_met)
}
```

**Parameters**:
- **Tier 4**: 2 rounds, 30 days apart
- **Tier 5**: 3 rounds, 60 days apart
- **Requirement**: All rounds must pass
- **Transparency**: All rounds public and tracked

**Benefits**:
- Ensures sustained consensus
- Prevents rushed decisions
- Allows for reflection and reconsideration
- Maintains system stability

**Trade-offs**:
- Slows down decision-making
- May reduce participation in later rounds
- Requires sustained engagement
- Could block legitimate urgent changes

**Recommendation**: **Implement for Tier 4+ only, with clear round requirements**

---

### 10. Veto Override Mechanisms

**Problem**: Bad-faith vetoes could block legitimate proposals.

**Solution**: Allow override of vetoes if they lack merit or are clearly bad-faith.

```rust
// Veto override
struct VetoOverride {
    proposal_id: i32,
    veto_id: i32,
    override_reason: String,
    override_votes: Vec<Vote>,
    override_threshold: f64,  // 60% of non-veto votes
}

// Override check
fn can_override_veto(
    proposal: &Proposal,
    veto: &Veto,
) -> bool {
    // Check if veto has valid rationale
    if veto.rationale_quality_score < 0.3 {
        return true;  // Low-quality rationale can be overridden
    }
    
    // Check if override threshold met
    let non_veto_weight = proposal.total_weight - veto.weight;
    let override_weight: f64 = proposal.override_votes.iter()
        .map(|v| v.weight)
        .sum();
    
    (override_weight / non_veto_weight) >= 0.6  // 60% of non-veto votes
}
```

**Parameters**:
- **Override Threshold**: 60% of non-veto votes
- **Rationale Quality**: Vetoes with low-quality rationale (<0.3) can be overridden
- **Transparency**: All overrides public and logged
- **Appeal Process**: Veto issuer can appeal override

**Benefits**:
- Prevents bad-faith blocking
- Maintains system functionality
- Encourages quality vetoes
- Preserves legitimate veto power

**Trade-offs**:
- Could undermine veto system
- Requires careful threshold setting
- May create conflict
- Needs clear criteria

**Recommendation**: **Implement conservatively, with high override threshold (60%) and quality checks**

---

## Implementation Priority

### High Priority (Implement First)

1. **Time-Based Contribution Decay** ⭐⭐⭐⭐⭐
   - High impact, medium effort
   - Prevents permanent power structures
   - Encourages ongoing participation

2. **Per-Entity Weight Caps** ⭐⭐⭐⭐⭐
   - High impact, low effort
   - Prevents absolute dominance
   - Easy to implement and understand

3. **Cooling-Off Periods** ⭐⭐⭐⭐
   - Medium-high impact, low effort
   - Prevents vote buying
   - Simple to implement

### Medium Priority (Implement Second)

4. **Minimum Diversity Requirements** ⭐⭐⭐⭐
   - Medium impact, medium effort
   - Ensures broad representation
   - Requires careful tier definition

5. **Reputation Multipliers** ⭐⭐⭐
   - Medium impact, high effort
   - Rewards good behavior
   - Requires subjective scoring system

6. **Multiple Voting Rounds** ⭐⭐⭐
   - Medium impact, medium effort
   - Ensures sustained consensus
   - Slows down decision-making

### Lower Priority (Consider Later)

7. **Delegation Mechanisms** ⭐⭐
   - Low-medium impact, high effort
   - Empowers small contributors
   - Could concentrate power

8. **Anti-Coordination Detection** ⭐⭐
   - Low-medium impact, very high effort
   - Prevents coordinated capture
   - Requires sophisticated detection

9. **Exit Mechanisms** ⭐⭐
   - Low-medium impact, medium effort
   - Provides safety valve
   - Could create instability

10. **Veto Override Mechanisms** ⭐
    - Low impact, medium effort
    - Prevents bad-faith blocking
    - Could undermine veto system

---

## Recommended Implementation Plan

### Phase 1: Core Balancing (3-4 months)

1. **Time-Based Contribution Decay**
   - 180-day decay for mining, 365-day for zaps
   - 10% minimum retention
   - Refresh on new contributions

2. **Per-Entity Weight Caps**
   - 5% of total system weight
   - Monthly recalculation
   - 6-month grandfathering period

3. **Cooling-Off Periods**
   - $10,000 threshold
   - 30-day cooling period
   - Applies to all contribution types

**Result**: Prevents permanent power structures, absolute dominance, and vote buying.

### Phase 2: Representation & Quality (4-6 months)

4. **Minimum Diversity Requirements**
   - Tier 4+ only
   - Require 2 of 3 tiers
   - Clear tier definitions

5. **Reputation Multipliers**
   - 0.5x to 1.5x range
   - Quarterly updates
   - Full transparency

6. **Multiple Voting Rounds**
   - Tier 4: 2 rounds, 30 days apart
   - Tier 5: 3 rounds, 60 days apart
   - All rounds must pass

**Result**: Ensures broad representation, rewards quality, and ensures sustained consensus.

### Phase 3: Advanced Mechanisms (6+ months)

7-10. **Delegation, Anti-Coordination, Exit, Veto Override**
   - Evaluate based on Phase 1-2 results
   - Implement if needed
   - Careful design and testing

---

## Combined Impact

### Before Additional Balancing

- Quadratic weighting: Prevents linear dominance
- Sybil resistance: Prevents fake entities
- Public accountability: Deters bad behavior
- **Estimated Capture Resistance: 85-90%**

### After Phase 1 (Core Balancing)

- Time decay: Prevents permanent power
- Weight caps: Prevents absolute dominance
- Cooling-off: Prevents vote buying
- **Estimated Capture Resistance: 92-95%**

### After Phase 2 (Representation & Quality)

- Diversity requirements: Ensures broad representation
- Reputation multipliers: Rewards quality
- Multiple rounds: Ensures sustained consensus
- **Estimated Capture Resistance: 95-97%**

### After Phase 3 (Advanced Mechanisms)

- Delegation: Empowers small contributors
- Anti-coordination: Prevents coordinated capture
- Exit mechanisms: Provides safety valve
- Veto override: Prevents bad-faith blocking
- **Estimated Capture Resistance: 97-99%**

---

## Conclusion

These additional balancing mechanisms complement existing safeguards to create a more robust, fair, and capture-resistant governance system. The recommended phased implementation prioritizes high-impact, low-effort mechanisms first, building toward a comprehensive system that balances power effectively while maintaining participation incentives.

**Key Principles**:
- **Defense in Depth**: Multiple independent mechanisms
- **Transparency**: All mechanisms public and auditable
- **Fairness**: Quadratic weighting + caps + diversity
- **Flexibility**: Mechanisms can be adjusted based on experience
- **User Sovereignty**: Exit mechanisms and override options

**Next Steps**:
1. Review and refine proposed mechanisms
2. Implement Phase 1 (Core Balancing)
3. Monitor and adjust based on results
4. Proceed to Phase 2 and Phase 3 as appropriate

