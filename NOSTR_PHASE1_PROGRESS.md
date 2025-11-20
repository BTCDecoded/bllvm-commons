# Nostr Integration Phase 1 - Implementation Progress

**Date:** November 17, 2025  
**Status:** In Progress  
**Phase:** Phase 1 - Foundation

---

## Completed Work

### ✅ 1. Event Schemas Created

**File:** `governance-app/src/nostr/events.rs`

**Added Event Types:**
- `GovernanceActionEvent` - For governance actions (merges, releases, etc.)
- `KeyholderAnnouncement` - For keyholder metadata (Kind 0)
- `NodeStatusReport` - For node telemetry (opt-in)

**Supporting Types:**
- `LayerRequirement` - Layer-specific requirements
- `TierRequirement` - Tier-specific requirements
- `CombinedRequirement` - Final combined requirements (most restrictive wins)
- `KeyholderSignature` - Signature information
- `EconomicVetoStatus` - Economic node veto status

**Status:** ✅ Complete - All event schemas match the plan

---

### ✅ 2. Nostr Client Library

**File:** `governance-app/Cargo.toml`

**Dependency:**
- `nostr-sdk = "0.27"` - Already present

**Status:** ✅ Complete - No changes needed

---

### ✅ 3. Governance Action Publisher

**File:** `governance-app/src/nostr/governance_publisher.rs`

**Features:**
- Publishes governance action events (Kind 30078)
- Includes layer + tier information
- Includes combined requirements
- Supports zap addresses
- Includes governance_config tag for fork support

**Methods:**
- `publish_action()` - Main publishing method
- `create_nostr_event()` - Event creation with proper tags

**Status:** ✅ Complete - Ready for integration

---

## In Progress

### 🔄 4. Integration with GitHub Webhooks

**Next Steps:**
1. Add Nostr publishing to PR merge handler
2. Add Nostr publishing to review period start
3. Add configuration for governance_config and zap addresses
4. Wire up layer + tier combination calculation

**Files to Modify:**
- `governance-app/src/webhooks/pull_request.rs` - Add merge event publishing
- `governance-app/src/enforcement/merge_block.rs` - Add merge completion publishing
- `governance-app/src/config.rs` - Add Nostr configuration

**Status:** 🔄 In Progress

---

## Pending Work

### ⏳ 5. Zap Forwarding Service

**Requirements:**
- Zap forwarding to donations wallet
- Zap logging for transparency
- Configuration for zap addresses

**Status:** ⏳ Pending

---

### ⏳ 6. Configuration

**Requirements:**
- Nostr relay URLs
- Governance config tag (e.g., "commons_mainnet")
- Zap addresses (Lightning addresses)
- Server keys (nsec)

**Status:** ⏳ Pending

---

### ⏳ 7. Node Telemetry Service

**Requirements:**
- Opt-in node status reporting
- Ephemeral key rotation
- Privacy-preserving (no IPs)

**Status:** ⏳ Pending (Lower priority - can be Phase 1.5)

---

### ⏳ 8. Documentation

**Requirements:**
- Node operator guide
- Configuration examples
- Event schema reference

**Status:** ⏳ Pending

---

## Implementation Notes

### Event Tag Structure

All governance action events include:
- `d: btc-commons-governance-action` - Event identifier
- `action: merge|release|budget|keyholder_change` - Action type
- `governance_tier: 1-5` - Tier classification
- `governance_layer: 1-6` - Layer classification
- `repository: bllvm-*` - Repository name
- `governance_config: commons_mainnet` - Governance fork identifier
- `final_signatures: N-of-M` - Combined signature requirement
- `final_review_days: N` - Combined review period
- `zap: <lightning_address>` - Optional zap address

### Layer + Tier Combination

The system uses "most restrictive wins":
- Signatures: Higher requirement wins
- Review Period: Longer period wins
- Economic Veto: Tier-based (Tier 3+)

---

## Next Steps

1. **Integrate into Webhook Handlers** (High Priority)
   - Add Nostr publishing when PR merges
   - Add Nostr publishing when review period starts
   - Calculate layer + tier combination

2. **Add Configuration** (High Priority)
   - Add Nostr config section
   - Add governance_config setting
   - Add zap address setting

3. **Zap Forwarding** (Medium Priority)
   - Basic zap forwarding service
   - Zap logging

4. **Testing** (Medium Priority)
   - Unit tests for event creation
   - Integration tests for publishing
   - Mock relay for testing

5. **Documentation** (Lower Priority)
   - Update NOSTR_INTEGRATION.md
   - Add configuration examples
   - Add event schema reference

---

## Files Created/Modified

### Created
- `governance-app/src/nostr/governance_publisher.rs` - New governance action publisher

### Modified
- `governance-app/src/nostr/events.rs` - Added new event types
- `governance-app/src/nostr/mod.rs` - Exported new types and publisher

### To Modify
- `governance-app/src/webhooks/pull_request.rs` - Add merge publishing
- `governance-app/src/enforcement/merge_block.rs` - Add merge completion publishing
- `governance-app/src/config.rs` - Add Nostr configuration
- `governance-app/config/app.toml` - Add Nostr config section

---

**Status:** ✅ Foundation Complete, Integration In Progress  
**Next:** Integrate into webhook handlers

