# Nostr Integration Plan Validation

**Date:** 2025-11-17  
**Status:** Plan Review Complete  
**Purpose:** Validate the Nostr integration design document against actual Bitcoin Commons architecture

---

## Executive Summary

The Nostr integration plan is **fundamentally sound** but requires **architectural clarifications** to align with the actual Bitcoin Commons governance model. The plan correctly identifies core principles (separation of concerns, opt-in, graceful degradation) but needs adjustments to match the actual 5-layer + 5-tier governance system.

**Overall Assessment:** ✅ **APPROVED WITH MODIFICATIONS**

---

## Validation Results

### ✅ **CORRECT** - Core Principles

1. **Separation of Concerns**: ✅ Correctly identifies that consensus never depends on Nostr
2. **Opt-in by Default**: ✅ All Nostr features are optional
3. **Graceful Degradation**: ✅ System works without Nostr
4. **Censorship Resistance**: ✅ Multiple relay redundancy
5. **Privacy Preservation**: ✅ Minimal identifying information

### ✅ **CORRECT** - Governance System Understanding

1. **5-Tier System**: ✅ Plan correctly identifies 5 tiers:
   - Tier 1: Routine (3-of-5, 7 days)
   - Tier 2: Features (4-of-5, 30 days)
   - Tier 3: Consensus-Adjacent (5-of-5, 90 days)
   - Tier 4: Emergency (4-of-5, 0 days)
   - Tier 5: Governance (5-of-5, 180 days)

2. **Multisig Thresholds**: ✅ Correctly identifies signature requirements
3. **governance-app (bllvm-commons)**: ✅ Correctly identifies GitHub App
4. **Keyholder Coordination**: ✅ Correctly identifies maintainer + emergency keyholder system

### ⚠️ **NEEDS CLARIFICATION** - Architecture Model

**Issue:** Plan describes a "3-Layer Model" (Consensus, Governance, Coordination) which is a **conceptual separation**, not the actual repository architecture.

**Actual Architecture:**
- **5-Layer Repository System** (Orange Paper, Consensus Proof, Protocol Engine, Reference Node, Developer SDK)
- **5-Tier Action Classification** (Routine, Features, Consensus-Adjacent, Emergency, Governance)
- **Layer + Tier Combination** (most restrictive wins)

**Recommendation:** Update the plan to clarify:
1. The "3-Layer Model" is a **conceptual separation** for Nostr integration purposes
2. The actual governance uses **5 layers + 5 tiers** with combination rules
3. Nostr events should include **both layer and tier** information

---

## Required Modifications

### 1. Governance Action Events - Add Layer Information

**Current Plan:**
```json
{
  "governance_tier": "1" | "2" | "3" | "4" | "5",
  ...
}
```

**Should Include:**
```json
{
  "governance_tier": "1" | "2" | "3" | "4" | "5",
  "governance_layer": "1" | "2" | "3" | "4" | "5",
  "final_signatures": "6-of-7",  // Combined requirement
  "final_review_days": 180,       // Combined requirement
  ...
}
```

**Rationale:** The actual system combines layer and tier requirements. Nostr events should reflect the final combined requirements, not just tier.

### 2. Repository Naming

**Current Plan:** References "governance-app"  
**Should Use:** "bllvm-commons" (GitHub repo name)

**Note:** Local directory is still `governance-app`, but GitHub repo is `bllvm-commons`. The plan should consistently use `bllvm-commons` for GitHub references.

### 3. Signature Thresholds - More Accurate

**Current Plan Examples:**
- "3-of-5" for Tier 1 ✅ Correct
- "4-of-5" for Tier 2 ✅ Correct
- "5-of-5" for Tier 3 ✅ Correct

**Missing:** Layer requirements (6-of-7 for Layers 1-2, 4-of-5 for Layer 3, etc.)

**Recommendation:** Include both layer and tier in event structure, and document that final requirements use "most restrictive wins" rule.

### 4. Economic Node Veto

**Current Plan:** Mentions economic node veto for Tier 3+  
**Actual System:** Economic node veto applies to:
- Tier 3 (Consensus-Adjacent)
- Tier 4 (Emergency) - with real-time oversight
- Tier 5 (Governance Changes)

**Recommendation:** Clarify that economic node veto is tier-based, not layer-based.

### 5. Emergency Keyholders

**Current Plan:** Mentions "emergency keyholders"  
**Actual System:** 
- **Maintainers**: Regular governance (3-of-5 to 6-of-7 depending on layer/tier)
- **Emergency Keyholders**: Separate set for emergency activation (5-of-7 to activate emergency mode)

**Recommendation:** Clarify the distinction between maintainers and emergency keyholders in Nostr events.

---

## Architecture Alignment

### Conceptual Model (Plan) vs Actual Architecture

| Plan's 3-Layer Model | Actual 5-Layer + 5-Tier System | Alignment |
|---------------------|--------------------------------|-----------|
| **Layer 1: Consensus** | Layers 1-2 (Orange Paper, Consensus Proof) | ✅ Aligned - Constitutional layer |
| **Layer 2: Governance** | Layer 6 (governance-app/bllvm-commons) | ✅ Aligned - Governance enforcement |
| **Layer 3: Coordination** | Community layer (Nostr) | ✅ Aligned - New coordination layer |

**Conclusion:** The plan's conceptual model is valid, but Nostr events should include actual layer/tier information for transparency.

---

## Event Schema Recommendations

### Enhanced Governance Action Event

```json
{
  "kind": 30078,
  "tags": [
    ["d", "btc-commons-governance-action"],
    ["action", "merge"],
    ["governance_tier", "3"],
    ["governance_layer", "2"],
    ["final_signatures", "6-of-7"],
    ["final_review_days", "180"],
    ["commit_hash", "abc123..."],
    ["repository", "bllvm-consensus"],
    ["pr_number", "123"]
  ],
  "content": {
    "description": "Merge PR #123: Consensus rule update",
    "pr_url": "https://github.com/BTCDecoded/bllvm-consensus/pull/123",
    "layer_requirement": {
      "layer": 2,
      "signatures": "6-of-7",
      "review_days": 180
    },
    "tier_requirement": {
      "tier": 3,
      "signatures": "5-of-5",
      "review_days": 90,
      "economic_veto": true
    },
    "combined_requirement": {
      "signatures": "6-of-7",
      "review_days": 180,
      "economic_veto": true
    },
    "signatures": [
      {
        "keyholder": "maintainer_pubkey_1",
        "signature": "sig1",
        "timestamp": 1234567890
      }
    ],
    "economic_veto_status": "pending" | "passed" | "vetoed"
  }
}
```

---

## Implementation Priorities - Validated

### Phase 1: Foundation ✅ APPROVED

**Essential Integrations:**
1. **Governance Transparency** ✅
   - Governance action events
   - Keyholder announcements
   - Integration with bllvm-commons GitHub App

2. **Network Telemetry** ✅
   - Node status reporting (opt-in)
   - Status page Nostr subscription

**Modifications Needed:**
- Include layer + tier information in events
- Use "bllvm-commons" for GitHub repo references
- Include combined requirements (most restrictive wins)

### Phase 2: Community ✅ APPROVED

**Priority Integrations:**
1. **Development Announcements** ✅
2. **Module Proposals** ✅

**Note:** Module marketplace aligns with the modular architecture (bllvm-sdk, plugin system).

### Phase 3: Ecosystem ✅ APPROVED

**Advanced Integrations:**
1. **Module Marketplace** ✅
2. **Dispute Resolution** ✅

**Note:** Governance fork support is critical - the system explicitly supports governance forks.

---

## Security Considerations - Validated

### ✅ Cryptographic Requirements

- Event signing ✅ Correct
- Signature verification ✅ Correct
- Governance signatures must meet threshold ✅ Correct

### ✅ Privacy Protections

- No IP addresses ✅ Correct
- Ephemeral keys for telemetry ✅ Correct
- Key rotation ✅ Correct

### ✅ Relay Selection

- Multiple relays ✅ Correct
- Graceful degradation ✅ Correct

---

## Specific Corrections Needed

### 1. Repository References

**Change:**
- `governance-app` → `bllvm-commons` (for GitHub references)
- Keep `governance-app` for local directory references only

### 2. Governance Tier References

**Add:** Layer information alongside tier information

**Example:**
```json
{
  "governance_tier": "3",
  "governance_layer": "2",
  "repository": "bllvm-consensus"
}
```

### 3. Signature Threshold Examples

**Current:** "3-of-5" for Tier 1  
**Clarify:** This is tier-only. Layer 1-2 would require 6-of-7 regardless of tier.

**Add:** Combined requirements calculation (most restrictive wins)

### 4. Economic Node Veto

**Clarify:** Economic node veto applies to Tier 3+, not layer-based.

**Add:** Economic node veto status in governance action events.

### 5. Emergency Keyholders

**Clarify:** Distinction between:
- **Maintainers**: Regular governance (N-of-M signatures)
- **Emergency Keyholders**: Emergency activation (5-of-7 to activate)

---

## Additional Recommendations

### 1. Governance Fork Support

**Current Plan:** Mentions governance forks ✅  
**Enhancement:** Add `governance_config` tag to all events:
```
["governance_config", "commons_mainnet"]
```

This allows governance forks to use separate Nostr tags/relays.

### 2. Layer + Tier Combination

**Add:** Documentation explaining how layer and tier combine:
- Most restrictive wins
- Examples of combined requirements
- How to interpret Nostr events with both layer and tier

### 3. Repository Layer Mapping

**Add:** Mapping from repository to layer:
- `bllvm-spec` (Orange Paper) → Layer 1
- `bllvm-consensus` → Layer 2
- `bllvm-protocol` → Layer 3
- `bllvm-node` → Layer 4
- `bllvm-sdk` → Layer 5
- `bllvm-commons` (governance-app) → Layer 6 (governance)

### 4. Event Kind Selection

**Current Plan:** Uses Kind 30078 (Parameterized Replaceable) ✅  
**Validation:** Appropriate for governance actions, node status, module registry

**Recommendation:** Document why each kind is chosen.

---

## Conclusion

The Nostr integration plan is **fundamentally sound** and aligns well with Bitcoin Commons architecture. The core principles, use cases, and implementation priorities are all valid.

**Required Changes:**
1. Add layer information to governance events
2. Use "bllvm-commons" for GitHub references
3. Include combined requirements (layer + tier)
4. Clarify maintainer vs emergency keyholder distinction
5. Add governance_config tag for fork support

**Recommended Enhancements:**
1. Document layer + tier combination rules
2. Add repository-to-layer mapping
3. Include economic node veto status in events
4. Document event kind selection rationale

**Overall:** ✅ **APPROVED** - Proceed with implementation after incorporating these modifications.

---

## Next Steps

1. **Update Plan Document** with architectural clarifications
2. **Define Event Schemas** with layer + tier information
3. **Begin Phase 1 Implementation** in bllvm-commons
4. **Create Integration Tests** for Nostr event publishing
5. **Document Layer + Tier Combination** for event consumers

---

**Validation Status:** ✅ Complete  
**Plan Status:** ✅ Approved with Modifications  
**Implementation Ready:** ✅ Yes (after modifications)

