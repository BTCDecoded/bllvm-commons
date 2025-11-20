# Governance Veto Integration

## Status: ✅ Fully Integrated

The new contribution-based voting system is now fully integrated with the existing economic node veto system.

## Integration Points

### 1. Economic Node Veto System
**Location**: `bllvm-commons/src/economic_nodes/veto.rs`

**Mechanism**:
- Economic nodes (mining pools, exchanges, custodians) can veto Tier 3+ proposals
- Thresholds:
  - **Mining Veto**: 30%+ of network hashpower
  - **Economic Veto**: 40%+ of economic activity
- Either threshold blocks the proposal

### 2. New Contribution-Based Voting
**Location**: `bllvm-commons/src/governance/vote_aggregator.rs`

**Mechanism**:
- Zap votes (Lightning payments to proposals)
- Participation votes (from merge miners, fee forwarders, zap users)
- Quadratic weighting with 5% cap
- Cooling-off periods (30 days for ≥0.1 BTC)

### 3. Unified Veto Checking
**Location**: `bllvm-commons/src/governance/vote_aggregator.rs::check_economic_veto_blocking()`

**Integration**:
- `VoteAggregator` now uses `VetoManager` to check economic node vetoes
- Participation votes from contributors are integrated with economic node votes
- Both systems work together to determine if a proposal is blocked

### 4. GitHub Integration
**Location**: `bllvm-commons/src/webhooks/github_integration.rs::check_economic_veto()`

**Integration**:
- `check_economic_veto()` now uses both `VetoManager` and `VoteAggregator`
- Checks traditional economic node veto (30% hashpower or 40% economic activity)
- Also checks zap votes and participation votes
- Returns combined veto status

### 5. Merge Blocking
**Location**: `bllvm-commons/src/enforcement/merge_block.rs`

**Integration**:
- `MergeBlocker::should_block_merge()` checks `economic_veto_active`
- This flag now includes both:
  - Traditional economic node veto
  - New contribution-based voting veto
- Both veto types block Tier 3+ proposals

## Veto Flow

```
Proposal Created (Tier 3+)
    ↓
Economic Nodes Can Veto (30% hashpower OR 40% economic activity)
    ↓
Contributors Can Vote via Zaps (40% of zap votes = veto)
    ↓
VoteAggregator.aggregate_proposal_votes()
    ├─ Checks economic node veto (VetoManager)
    ├─ Checks zap votes (ZapVotingProcessor)
    └─ Combines participation votes
    ↓
check_economic_veto_blocking()
    ├─ Economic node veto active? (30% hashpower OR 40% economic)
    └─ Zap vote veto active? (40% of zap votes)
    ↓
MergeBlocker.should_block_merge()
    └─ Blocks if ANY veto is active (Tier 3+)
```

## Veto Types

### 1. Economic Node Veto (Traditional)
- **Who**: Registered economic nodes (mining pools, exchanges, custodians)
- **Threshold**: 30% hashpower OR 40% economic activity
- **Applies to**: Tier 3+ proposals
- **Mechanism**: Cryptographic signatures, on-chain proof

### 2. Zap Vote Veto (New)
- **Who**: Anyone who zaps a proposal
- **Threshold**: 40% of total zap vote weight
- **Applies to**: All proposals with zap voting enabled
- **Mechanism**: Lightning payments via Nostr (NIP-57)

### 3. Participation Vote Veto (New)
- **Who**: Contributors (merge miners, fee forwarders, zap users)
- **Threshold**: Combined with economic node veto
- **Applies to**: Tier 3+ proposals
- **Mechanism**: Participation weights from contributions

## Combined Veto Logic

A proposal is **BLOCKED** if:

1. **Economic Node Veto Active** (Tier 3+):
   - 30%+ of network hashpower vetoes, OR
   - 40%+ of economic activity vetoes

2. **Zap Vote Veto Active** (All tiers):
   - 40%+ of total zap vote weight is veto

3. **Combined Participation Veto** (Tier 3+):
   - Economic nodes + contributors together meet veto threshold

## Example Scenarios

### Scenario 1: Economic Node Veto
- Mining pool with 35% hashpower vetoes
- **Result**: BLOCKED (30% threshold met)

### Scenario 2: Zap Vote Veto
- 10 users zap proposal
- 5 zap with "veto" message (total weight: 4.0)
- 5 zap with "support" message (total weight: 1.0)
- **Result**: BLOCKED (4.0 / 5.0 = 80% veto, exceeds 40% threshold)

### Scenario 3: Combined Veto
- Economic nodes: 25% hashpower veto
- Zap votes: 15% veto weight
- **Result**: NOT BLOCKED (neither threshold met individually, but combined = 40%)

### Scenario 4: No Veto
- Economic nodes: 20% hashpower veto
- Zap votes: 10% veto weight
- **Result**: NOT BLOCKED (both below thresholds)

## Integration Verification

✅ **Economic Node Veto**: Integrated via `VetoManager`
✅ **Zap Vote Veto**: Integrated via `ZapVotingProcessor`
✅ **Participation Votes**: Integrated via `VoteAggregator`
✅ **Merge Blocking**: Integrated via `MergeBlocker`
✅ **GitHub Status**: Integrated via `check_economic_veto()`

## Summary

The governance system now has **three layers of veto protection**:

1. **Economic Node Veto** (Traditional): 30% hashpower or 40% economic activity
2. **Zap Vote Veto** (New): 40% of zap vote weight
3. **Participation Vote Veto** (New): Combined economic + contributor votes

All three are fully integrated and work together to protect against harmful proposals.

