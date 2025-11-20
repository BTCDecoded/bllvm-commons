# Nostr Integration Plan - Changes from Original

**Date:** November 17, 2025  
**Purpose:** Summary of changes made to align the plan with actual Bitcoin Commons codebase

---

## Key Changes Summary

### 1. ✅ Repository Naming Corrections

**Original:** Referenced "governance-app"  
**Updated:** Uses "bllvm-commons" (GitHub repo name)

**Rationale:** The GitHub repository is `bllvm-commons`, while the local directory is `governance-app`. All GitHub references now use the correct name.

---

### 2. ✅ Architecture Model Clarification

**Original:** Described a "3-Layer Model" (Consensus, Governance, Coordination)  
**Updated:** Documents actual 5-layer repository architecture + 5-tier action classification

**Added:**
- Complete repository-to-layer mapping table
- Layer + tier combination rules ("most restrictive wins")
- Examples of combined requirements

**Rationale:** The actual system uses 5 layers (Orange Paper, Consensus Proof, Protocol, Node, SDK) plus governance-app, with 5 tiers of action classification. The "3-layer model" is now clarified as a conceptual separation for Nostr integration purposes.

---

### 3. ✅ Enhanced Event Schemas

**Original:** Events included only tier information  
**Updated:** Events include both layer and tier information, plus combined requirements

**Added to Governance Action Events:**
```json
{
  "governance_tier": "3",
  "governance_layer": "2",
  "repository": "bllvm-consensus",
  "final_signatures": "6-of-7",
  "final_review_days": "180",
  "layer_requirement": { ... },
  "tier_requirement": { ... },
  "combined_requirement": { ... }
}
```

**Rationale:** The actual governance system combines layer and tier requirements. Events must reflect this to provide full transparency.

---

### 4. ✅ Keyholder Type Clarification

**Original:** Referenced "keyholders" generically  
**Updated:** Distinguishes between:
- **Maintainers**: Regular governance (different sets per layer, 3-7 maintainers)
- **Emergency Keyholders**: Emergency activation (5-of-7 to activate)
- **Economic Nodes**: Veto power for Tier 3+ changes

**Added:** `keyholder_type` field in keyholder announcements

**Rationale:** The system has distinct types of keyholders with different roles and requirements.

---

### 5. ✅ Governance Fork Support

**Original:** Mentioned governance forks  
**Updated:** Added `governance_config` tag to all events

**Example:**
```
["governance_config", "commons_mainnet"]
```

**Rationale:** Governance forks are explicitly supported in the system. All events must include this tag to enable fork separation.

---

### 6. ✅ Economic Node Veto Clarification

**Original:** Mentioned economic node veto for Tier 3+  
**Updated:** Clarified that economic node veto is tier-based, not layer-based

**Added:**
- `economic_veto_status` field in governance action events
- Clarification that Tier 3, 4, and 5 require economic node involvement
- Tier 4 has "real-time oversight" rather than formal veto

**Rationale:** Economic node veto applies to tiers, not layers. This distinction is important for transparency.

---

### 7. ✅ Repository-to-Layer Mapping

**Original:** No explicit mapping  
**Updated:** Complete mapping table added

| Repository | Layer | Signatures | Review Days |
|------------|-------|------------|-------------|
| `bllvm-spec` | 1 | 6-of-7 | 180 (365 consensus) |
| `bllvm-consensus` | 2 | 6-of-7 | 180 (365 consensus) |
| `bllvm-protocol` | 3 | 4-of-5 | 90 |
| `bllvm-node` | 4 | 3-of-5 | 60 |
| `bllvm-sdk` | 5 | 2-of-3 | 14 |
| `bllvm-commons` | 6 | Varies by tier | Varies by tier |

**Rationale:** Events must include repository information to determine layer requirements.

---

### 8. ✅ Layer + Tier Combination Examples

**Original:** No examples of combination logic  
**Updated:** Added comprehensive examples table

**Example:**
- Bug fix in `bllvm-protocol` (Layer 3, Tier 1):
  - Layer 3: 4-of-5 signatures, 90 days
  - Tier 1: 3-of-5 signatures, 7 days
  - **Result**: 4-of-5 signatures, 90 days (Layer 3 wins)

**Rationale:** The "most restrictive wins" rule is complex. Examples help implementers understand it.

---

### 9. ✅ Enhanced Tag Conventions

**Original:** Basic tag conventions  
**Updated:** Comprehensive tag reference including:
- `governance_config` - For fork support
- `governance_layer` - Repository layer
- `governance_tier` - Action tier
- `repository` - Specific repository
- `final_signatures` - Combined requirement
- `final_review_days` - Combined requirement

**Rationale:** Standardized tags ensure consistent event parsing across clients.

---

### 10. ✅ Appendix Enhancements

**Original:** Basic event type reference  
**Updated:** Comprehensive appendix including:
- Repository-to-layer quick reference
- Layer + tier combination examples
- Complete tag conventions
- Event kind selection rationale

**Rationale:** Implementers need quick reference materials.

---

## Validation Against Codebase

### ✅ Confirmed Correct

1. **5-Tier System**: All tiers correctly identified with proper thresholds
2. **Repository Structure**: All repositories correctly mapped to layers
3. **Signature Thresholds**: All thresholds match actual configuration
4. **Review Periods**: All review periods match actual configuration
5. **Emergency System**: Emergency tier system correctly documented
6. **Governance Fork Support**: Fork support correctly identified

### ✅ Architecture Alignment

1. **Layer System**: 5-layer architecture correctly documented
2. **Tier System**: 5-tier classification correctly documented
3. **Combination Rules**: "Most restrictive wins" correctly explained
4. **Repository Mapping**: All repositories correctly mapped

---

## Implementation Readiness

### ✅ Ready for Phase 1

The updated plan is ready for Phase 1 implementation:

1. **Event Schemas**: Complete and validated
2. **Integration Points**: Clearly identified
3. **Security Model**: Comprehensive
4. **Privacy Protections**: Well-defined
5. **Fork Support**: Explicitly included

### ✅ Codebase Alignment

All references align with actual codebase:
- Repository names match GitHub
- Governance structure matches actual system
- Layer + tier combination matches actual logic
- Signature thresholds match actual configuration

---

## Next Steps

1. **Review Updated Plan**: Community review of updated document
2. **Begin Phase 1**: Start `bllvm-commons` Nostr integration
3. **Implement Event Schemas**: Create Rust structs for Nostr events
4. **Add Integration Points**: Wire up `bllvm-commons` to publish events
5. **Create Status Page**: Build Nostr-consuming status page

---

**Status:** ✅ Plan Updated and Validated  
**Ready for Implementation:** ✅ Yes  
**Codebase Alignment:** ✅ Complete

