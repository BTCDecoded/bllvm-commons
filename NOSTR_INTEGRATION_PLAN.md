# Bitcoin Commons: Nostr Integration Design Document

**Project:** Bitcoin Commons / BTCDecoded  
**Document Version:** 2.0 (Validated Against Codebase)  
**Date:** November 17, 2025  
**Status:** Ready for Implementation  
**Purpose:** Comprehensive design for Nostr integrations across governance, coordination, and transparency layers

---

## Executive Summary

This document specifies how Nostr integrations enable decentralized coordination, transparency, and communication across the Bitcoin Commons ecosystem. Nostr provides the censorship-resistant, decentralized communication layer that complements Bitcoin's consensus layer and Bitcoin Commons' governance layer.

**Core Principle:** Nostr integrations must maintain separation of concerns - governance coordination uses Nostr, but consensus validation does not depend on it.

**Architecture Alignment:** This plan is validated against the actual Bitcoin Commons 5-layer repository architecture and 5-tier governance system, with proper layer + tier combination rules.

---

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [Repository and Governance Model](#repository-and-governance-model)
3. [Governance Transparency](#governance-transparency)
4. [Network Telemetry](#network-telemetry)
5. [Community Coordination](#community-coordination)
6. [Module Marketplace](#module-marketplace)
7. [Dispute Resolution](#dispute-resolution)
8. [Security Considerations](#security-considerations)
9. [Implementation Priorities](#implementation-priorities)
10. [Future Extensions](#future-extensions)

---

## Architecture Overview

### Bitcoin Commons Architecture

Bitcoin Commons uses a **5-layer repository architecture** combined with a **5-tier action classification system**:

#### Repository Layers (Architectural Hierarchy)

| Layer | Repository | Purpose | Signatures | Review Period |
|-------|------------|---------|------------|---------------|
| **Layer 1** | `bllvm-spec` | Orange Paper (Constitutional) | 6-of-7 | 180 days (365 for consensus) |
| **Layer 2** | `bllvm-consensus` | Consensus Proof (Constitutional) | 6-of-7 | 180 days (365 for consensus) |
| **Layer 3** | `bllvm-protocol` | Protocol Engine (Implementation) | 4-of-5 | 90 days |
| **Layer 4** | `bllvm-node` + `bllvm` | Reference Node + Binary (Application) | 3-of-5 | 60 days |
| **Layer 5** | `bllvm-sdk` | Developer SDK (Extension) | 2-of-3 | 14 days |
| **Layer 6** | `bllvm-commons` | Governance App (Governance) | Varies by tier | Varies by tier |

**Note:** `bllvm` is the final user-facing binary that wraps `bllvm-node`, providing CLI, configuration management, and server orchestration. It is considered part of Layer 4 (Application Layer) alongside `bllvm-node`.

#### Action Tiers (Change Classification)

| Tier | Type | Signatures | Review Period | Economic Veto |
|------|------|------------|---------------|---------------|
| **Tier 1** | Routine Maintenance | 3-of-5 | 7 days | No |
| **Tier 2** | Feature Changes | 4-of-5 | 30 days | No |
| **Tier 3** | Consensus-Adjacent | 5-of-5 | 90 days | Yes |
| **Tier 4** | Emergency Actions | 4-of-5 | 0 days | No (real-time oversight) |
| **Tier 5** | Governance Changes | 5-of-5 | 180 days | Yes |

#### Layer + Tier Combination Rule

When both Layer and Tier requirements apply, the system uses **"most restrictive wins"**:

- **Signatures**: Take the higher requirement (e.g., Layer 1's 6-of-7 beats Tier 1's 3-of-5)
- **Review Period**: Take the longer period (e.g., Layer 1's 180 days beats Tier 1's 7 days)
- **Economic Veto**: Tier-based (Tier 3+ requires veto)

**Example:** Bug fix in `bllvm-protocol` (Layer 3, Tier 1):
- Layer 3: 4-of-5 signatures, 90 days
- Tier 1: 3-of-5 signatures, 7 days
- **Result**: 4-of-5 signatures, 90 days (Layer 3 wins)

### Three-Layer Conceptual Model (Nostr Integration)

For Nostr integration purposes, we conceptually separate:

**Layer 1: Consensus (Bitcoin)**
- Transaction validation
- Block validation
- Chain selection
- **NO Nostr dependency** - must work without any external communication

**Layer 2: Governance (Bitcoin Commons)**
- Cryptographic multisig enforcement
- Keyholder coordination
- Governance state tracking
- **Optional Nostr integration** - enhanced transparency and coordination

**Layer 3: Coordination (Community)**
- Developer communication
- Module discovery
- Status reporting
- **Primary Nostr use** - decentralized communication infrastructure

### Design Principles

1. **Separation of Concerns:** Consensus never depends on Nostr
2. **Opt-in by Default:** All Nostr features are optional
3. **Graceful Degradation:** System works without Nostr, just less visible
4. **Censorship Resistance:** Use Nostr's relay architecture for resilience
5. **Privacy Preservation:** Minimal identifying information in events
6. **Forkable:** Governance forks can use separate Nostr tags/relays
7. **Layer + Tier Transparency:** All events include both layer and tier information

---

## Repository and Governance Model

### Repository-to-Layer Mapping

All Nostr events must include repository information to determine layer:

| Repository | GitHub Name | Layer | Maintainer Set |
|------------|-------------|-------|---------------|
| Orange Paper | `bllvm-spec` | 1 | 7 maintainers |
| Consensus Proof | `bllvm-consensus` | 2 | 7 maintainers |
| Protocol Engine | `bllvm-protocol` | 3 | 5 maintainers |
| Reference Node | `bllvm-node` | 4 | 5 maintainers |
| Binary Wrapper | `bllvm` | 4 | 5 maintainers (same as bllvm-node) |
| Developer SDK | `bllvm-sdk` | 5 | 3 maintainers |
| Governance App | `bllvm-commons` | 6 | Varies by tier |

**Note:** `bllvm` is the final binary that users run. It wraps `bllvm-node` and provides CLI, configuration, and server functionality. Changes to `bllvm` follow the same Layer 4 requirements as `bllvm-node`.

### Keyholder Types

**Maintainers:**
- Regular governance participants
- Sign PRs based on layer + tier requirements
- Different sets per layer (3-7 maintainers depending on layer)

**Emergency Keyholders:**
- Separate set for emergency activation
- 5-of-7 required to activate emergency mode
- Real-time oversight during emergencies

**Economic Nodes:**
- Veto power for Tier 3+ changes
- 30%+ hashpower or 40%+ economic activity threshold
- Signaling required for Tier 5 (governance changes)

### Governance Fork Support

All events include `governance_config` tag to support governance forks:

```
["governance_config", "commons_mainnet"]
```

Governance forks can use separate tags/relays:
- `commons_mainnet` - Original governance
- `commons_alt` - Forked governance
- `commons_testnet` - Test governance

---

## Governance Transparency

### Purpose

Make all governance actions visible, auditable, and verifiable through decentralized infrastructure.

### Event Types

#### 1. Governance Action Events

**Purpose:** Record all governance decisions (code merges, releases, budget allocations)

**Nostr Event Structure:**

```json
{
  "kind": 30078,
  "tags": [
    ["d", "btc-commons-governance-action"],
    ["action", "merge" | "release" | "budget" | "keyholder_change"],
    ["governance_tier", "1" | "2" | "3" | "4" | "5"],
    ["governance_layer", "1" | "2" | "3" | "4" | "5" | "6"],
    ["repository", "bllvm-spec" | "bllvm-consensus" | "bllvm-protocol" | "bllvm-node" | "bllvm" | "bllvm-sdk" | "bllvm-commons"],
    ["governance_config", "commons_mainnet"],
    ["final_signatures", "6-of-7"],
    ["final_review_days", "180"],
    ["commit_hash", "abc123..."],
    ["pr_number", "123"],
    ["timestamp", "1234567890"]
  ],
  "content": {
    "description": "Merge PR #123: Fix consensus bug",
    "pr_url": "https://github.com/BTCDecoded/bllvm-consensus/pull/123",
    "layer_requirement": {
      "layer": 2,
      "signatures": "6-of-7",
      "review_days": 180
    },
    "tier_requirement": {
      "tier": 1,
      "signatures": "3-of-5",
      "review_days": 7,
      "economic_veto": false
    },
    "combined_requirement": {
      "signatures": "6-of-7",
      "review_days": 180,
      "economic_veto": false,
      "source": "layer"
    },
    "signatures": [
      {
        "keyholder": "maintainer_pubkey_1",
        "keyholder_type": "maintainer",
        "signature": "sig1",
        "timestamp": 1234567890
      }
    ],
    "economic_veto_status": "not_required" | "pending" | "passed" | "vetoed",
    "review_period_ends": 1234567890
  }
}
```

**Published by:** `bllvm-commons` (governance-app) after successful multisig verification

**Subscribed by:** 
- Community monitoring tools
- Governance dashboards
- Archival services
- Audit tools

#### 2. Keyholder Announcements

**Purpose:** Keyholders announce their public keys, contact methods, and jurisdiction

**Nostr Event Structure:**

```json
{
  "kind": 0,
  "tags": [
    ["governance_config", "commons_mainnet"],
    ["keyholder_type", "maintainer" | "emergency_keyholder"],
    ["layer", "1" | "2" | "3" | "4" | "5" | "6"],
    ["governance_pubkey", "governance_key_hash"]
  ],
  "content": {
    "name": "Keyholder 1",
    "about": "Bitcoin Commons Governance Keyholder",
    "role": "maintainer",
    "governance_pubkey": "governance_key_hash",
    "jurisdiction": "jurisdiction_code",
    "backup_contact": "nostr_npub_or_email",
    "joined": 1234567890,
    "layer": 2,
    "keyholder_type": "maintainer"
  }
}
```

**Published by:** Each keyholder upon onboarding

**Privacy considerations:** 
- Keyholder identity can be pseudonymous
- Jurisdiction disclosed for transparency (multi-jurisdictional requirement)
- No personal identifying information required

#### 3. Review Period Notifications

**Purpose:** Alert community to active review periods for significant changes

**Nostr Event Structure:**

```json
{
  "kind": 30023,
  "tags": [
    ["d", "btc-commons-review-{pr_number}"],
    ["t", "governance-review"],
    ["governance_tier", "3"],
    ["governance_layer", "2"],
    ["repository", "bllvm-consensus"],
    ["governance_config", "commons_mainnet"],
    ["review_ends", "1234567890"],
    ["pr", "https://github.com/BTCDecoded/bllvm-consensus/pull/123"]
  ],
  "content": {
    "title": "Consensus-Adjacent Change Under Review",
    "description": "Markdown description of proposed change",
    "layer": 2,
    "tier": 3,
    "final_signatures": "6-of-7",
    "final_review_days": 180,
    "economic_veto_required": true,
    "review_period_ends": 1234567890
  }
}
```

**Published by:** `bllvm-commons` when PR enters review period

**Subscribed by:**
- Community members wanting governance notifications
- Developers tracking changes
- Institutional stakeholders

### Integration Points

**GitHub App (bllvm-commons):**

1. After successful multisig merge → publish governance action event
2. When PR enters review period → publish review notification
3. On keyholder rotation → publish keyholder change event
4. Include layer + tier information in all events
5. Calculate and include combined requirements (most restrictive wins)

**Benefits:**

- Immutable audit trail (Nostr events are timestamped and signed)
- Censorship-resistant transparency (multiple relay redundancy)
- Community can verify governance actions independently
- Historical record even if GitHub goes down
- Full transparency of layer + tier combination logic

---

## Network Telemetry

### Purpose

Enable decentralized status reporting without central servers or IP exposure.

### Event Types

#### 1. Node Status Reports

**Purpose:** Aggregate network health without centralized infrastructure

**Nostr Event Structure:**

```json
{
  "kind": 30078,
  "tags": [
    ["d", "btc-commons-node-status"],
    ["version", "0.2.0"],
    ["features", "base,lightning,merge_mining"],
    ["governance_config", "commons_mainnet"],
    ["network", "mainnet" | "testnet" | "signet"],
    ["last_block", "850000"]
  ],
  "content": {
    "node_type": "full" | "archival" | "pruned",
    "uptime_hours": 720,
    "sync_status": "synced" | "syncing",
    "modules_enabled": ["lightning", "merge_mining"],
    "reported_at": 1234567890
  }
}
```

**Published by:** Node's `bllvm-commons` telemetry service (opt-in)

**Privacy considerations:**

- Events signed by ephemeral node keys (not operator identity)
- No IP addresses
- No geographic location beyond optional continent/region
- Keys rotate periodically to prevent tracking

#### 2. Network Health Aggregates

**Purpose:** Anyone can aggregate and publish network statistics

**Nostr Event Structure:**

```json
{
  "kind": 30078,
  "tags": [
    ["d", "btc-commons-network-stats"],
    ["aggregator", "aggregator_pubkey"],
    ["governance_config", "commons_mainnet"],
    ["period", "daily" | "weekly" | "monthly"],
    ["computed_at", "1234567890"]
  ],
  "content": {
    "total_nodes": 1247,
    "by_version": {
      "0.2.0": 892,
      "0.1.0": 355
    },
    "by_features": {
      "base": 1247,
      "lightning": 423,
      "merge_mining": 89
    },
    "governance_nodes": 7,
    "current_phase": 2,
    "avg_uptime_hours": 648
  }
}
```

**Published by:** Anyone running aggregation service (decentralized)

**Subscribed by:**

- Status page displays
- Community dashboards
- Analytics tools
- Research projects

### Integration Points

**Node (bllvm-commons telemetry module):**

1. Periodically (e.g., every 24 hours) publish node status event
2. Rotate ephemeral signing key monthly
3. Allow configuration of which relays to publish to

**Status Page (static HTML):**

1. Subscribe to node status events from multiple relays
2. Filter by `governance_config` tag (handle governance forks)
3. Count unique event IDs (not pubkeys - allows key rotation)
4. Display aggregates in real-time

**Benefits:**

- No central telemetry server needed
- Status page can be hosted anywhere (even GitHub Pages)
- Privacy-preserving (no IP logging)
- Forkable (governance forks use different tags)
- Resilient (multiple relay redundancy)

---

## Community Coordination

### Purpose

Facilitate developer communication, issue tracking, and community discussion without platform dependencies.

### Event Types

#### 1. Development Announcements

**Purpose:** Broadcast significant updates to the community

**Nostr Event Structure:**

```json
{
  "kind": 30023,
  "tags": [
    ["t", "btc-commons-announcement"],
    ["governance_config", "commons_mainnet"],
    ["announcement_type", "release" | "security" | "governance" | "roadmap"],
    ["version", "0.2.0"],
    ["severity", "low" | "medium" | "high" | "critical"]
  ],
  "content": {
    "title": "Release v0.2.0",
    "body": "Markdown content of announcement",
    "repositories": ["bllvm-consensus", "bllvm-node"],
    "changes": [
      {
        "repository": "bllvm-consensus",
        "layer": 2,
        "summary": "Consensus rule update"
      }
    ]
  }
}
```

**Published by:** Project maintainers

**Examples:**

- New release announcements
- Security advisories
- Roadmap updates
- Governance phase transitions

#### 2. Module Proposals

**Purpose:** Developers propose new modules for community review

**Nostr Event Structure:**

```json
{
  "kind": 30023,
  "tags": [
    ["t", "btc-commons-module-proposal"],
    ["governance_config", "commons_mainnet"],
    ["module_name", "lightning-v2"],
    ["author", "author_pubkey"],
    ["category", "network" | "economic" | "application" | "experimental"],
    ["consensus_impact", "true" | "false"]
  ],
  "content": {
    "title": "Enhanced Lightning Integration",
    "description": "...",
    "repository": "https://github.com/...",
    "dependencies": ["base"],
    "review_requested": true
  }
}
```

**Published by:** Module developers

**Subscribed by:**

- Community reviewers
- Potential users
- Module marketplace aggregators

#### 3. Issue Discussions

**Purpose:** Decentralized issue tracking and discussion (GitHub backup)

**Nostr Event Structure:**

```json
{
  "kind": 30023,
  "tags": [
    ["t", "btc-commons-issue"],
    ["governance_config", "commons_mainnet"],
    ["issue_type", "bug" | "feature" | "documentation" | "question"],
    ["repository", "bllvm-node"],
    ["status", "open" | "in_progress" | "resolved" | "closed"],
    ["severity", "low" | "medium" | "high" | "critical"]
  ],
  "content": {
    "title": "...",
    "description": "...",
    "reproduction_steps": "...",
    "github_issue": "https://github.com/...",
    "repository": "bllvm-node",
    "layer": 4
  }
}
```

**Replies:** Use Kind 1 (Short Text Note) replies for discussion

**Benefits:**

- Issues remain accessible even if GitHub restricts access
- Decentralized discussion (not dependent on GitHub)
- Can reference GitHub issues without duplicating everything

### Integration Points

**Project Website:**

- Display recent announcements from Nostr
- Link to full announcement content
- Subscribe to announcement feed

**Module Marketplace:**

- Aggregate module proposals from Nostr
- Display module metadata and reviews
- Link to repositories

**Community Forums/Chats:**

- Mirror important Nostr events to other platforms
- Allow cross-platform discussion
- Maintain Nostr as source of truth

---

## Module Marketplace

### Purpose

Decentralized discovery and distribution of Bitcoin Commons modules.

### Event Types

#### 1. Module Registry

**Purpose:** Canonical metadata for available modules

**Nostr Event Structure:**

```json
{
  "kind": 30078,
  "tags": [
    ["d", "btc-commons-module-{module_id}"],
    ["governance_config", "commons_mainnet"],
    ["module_name", "lightning-integration"],
    ["version", "1.0.0"],
    ["author", "author_pubkey"],
    ["category", "network" | "economic" | "application" | "experimental"],
    ["license", "MIT" | "Apache-2.0"],
    ["verified", "true" | "false"]
  ],
  "content": {
    "description": "...",
    "repository": "https://github.com/...",
    "documentation": "https://docs.../",
    "dependencies": ["base", "bllvm-protocol"],
    "consensus_impact": false,
    "install_hash": "sha256_of_module",
    "security_audit": "url_or_null",
    "governance_signatures": ["pubkey1", "pubkey2"]
  }
}
```

**Published by:** Module authors (self-published) or governance (verified modules)

**Verified vs Unverified:**

- **Verified:** Reviewed by governance, signed by keyholders, listed in official marketplace
- **Unverified:** Community modules, use at own risk, still discoverable

#### 2. Module Reviews

**Purpose:** Community feedback and ratings for modules

**Nostr Event Structure:**

```json
{
  "kind": 30078,
  "tags": [
    ["d", "btc-commons-module-review-{module_id}-{reviewer_pubkey}"],
    ["governance_config", "commons_mainnet"],
    ["module_id", "lightning-integration"],
    ["rating", "1" | "2" | "3" | "4" | "5"],
    ["version_tested", "1.0.0"],
    ["reviewer_reputation", "score_0_to_100"]
  ],
  "content": {
    "review": "Detailed review text...",
    "pros": ["..."],
    "cons": ["..."],
    "tested_on": "hardware_specs",
    "use_case": "description"
  }
}
```

**Published by:** Community members who have tested modules

**Benefits:**

- Decentralized reputation system
- Module quality signals emerge organically
- Users can filter by ratings

#### 3. Module Security Reports

**Purpose:** Vulnerability disclosure and tracking

**Nostr Event Structure:**

```json
{
  "kind": 30078,
  "tags": [
    ["d", "btc-commons-security-{module_id}-{vulnerability_id}"],
    ["governance_config", "commons_mainnet"],
    ["module_id", "lightning-integration"],
    ["severity", "low" | "medium" | "high" | "critical"],
    ["status", "disclosed" | "patched" | "wontfix"],
    ["affected_versions", "1.0.0-1.2.3"],
    ["fixed_version", "1.2.4"]
  ],
  "content": {
    "vulnerability": "Description...",
    "impact": "What could happen...",
    "mitigation": "How to protect...",
    "patch": "link_to_fix",
    "disclosure_timeline": "..."
  }
}
```

**Published by:** Security researchers, module authors, governance

**Benefits:**

- Transparent vulnerability tracking
- Users can verify module security status
- Encourages responsible disclosure

### Integration Points

**Module Marketplace UI:**

1. Subscribe to module registry events
2. Display verified vs unverified modules
3. Show ratings and reviews
4. Alert users to security issues

**Node Configuration:**

1. Fetch module metadata from Nostr before install
2. Verify governance signatures for verified modules
3. Warn if installing unverified modules
4. Check for security reports before enabling

**Benefits:**

- No central module registry to capture or censor
- Module authors publish directly
- Users discover modules without intermediaries
- Governance provides verification layer without gatekeeping

---

## Dispute Resolution

### Purpose

Transparent, documented conflict resolution process.

### Event Types

#### 1. Dispute Filings

**Purpose:** Formal record of governance disputes

**Nostr Event Structure:**

```json
{
  "kind": 30023,
  "tags": [
    ["t", "btc-commons-dispute"],
    ["governance_config", "commons_mainnet"],
    ["dispute_type", "governance" | "technical" | "economic" | "community"],
    ["parties", "pubkey1", "pubkey2"],
    ["status", "filed" | "under_review" | "mediation" | "resolved" | "escalated"],
    ["related_pr", "https://github.com/..."]
  ],
  "content": {
    "dispute_summary": "...",
    "party_a_position": "...",
    "party_b_position": "...",
    "requested_resolution": "...",
    "evidence": ["link1", "link2"],
    "related_repository": "bllvm-consensus",
    "related_layer": 2,
    "related_tier": 3
  }
}
```

**Published by:** Dispute participants

**Benefits:**

- Transparent record of conflicts
- Community can observe resolution process
- Historical precedent for future disputes

#### 2. Mediation Records

**Purpose:** Document mediation attempts and outcomes

**Nostr Event Structure:**

```json
{
  "kind": 30023,
  "tags": [
    ["t", "btc-commons-mediation"],
    ["governance_config", "commons_mainnet"],
    ["dispute_id", "dispute_event_id"],
    ["mediator", "mediator_pubkey"],
    ["outcome", "resolved" | "unresolved" | "escalated"]
  ],
  "content": {
    "mediation_summary": "...",
    "agreed_resolution": "..." (if resolved),
    "next_steps": "..." (if escalated),
    "timeline": "..."
  }
}
```

**Published by:** Mediators (could be keyholders, respected community members)

#### 3. Governance Fork Announcements

**Purpose:** If dispute leads to governance fork, document the split

**Nostr Event Structure:**

```json
{
  "kind": 30023,
  "tags": [
    ["t", "btc-commons-governance-fork"],
    ["governance_config", "commons_mainnet"],
    ["original_governance", "commons_mainnet"],
    ["forked_governance", "commons_alt"],
    ["reason", "brief_description"],
    ["fork_date", "1234567890"]
  ],
  "content": {
    "fork_rationale": "Why this governance fork occurred...",
    "key_differences": "What changed...",
    "keyholders": ["pubkey1", "pubkey2", "pubkey3"],
    "migration_guide": "How to switch governance configs...",
    "nostr_relays": ["relay1", "relay2"]
  }
}
```

**Published by:** Forking party

**Benefits:**

- Users can choose which governance to follow
- Transparent record of why forks occur
- Enables governance competition

### Integration Points

**bllvm-commons:**

1. Detect when dispute escalation occurs
2. Publish dispute records automatically
3. Track resolution status

**Community Dashboard:**

1. Display active disputes
2. Show mediation progress
3. Alert community to governance forks

**Benefits:**

- Conflicts are visible, not hidden
- Resolution process is transparent
- Fork option remains available (Ostrom principle: protection from external interference)

---

## Security Considerations

### Cryptographic Requirements

**Event Signing:**

- All governance events MUST be signed by authorized keys
- Keyholder events signed by governance keys
- Node telemetry signed by ephemeral node keys
- Module registry events signed by author keys (verified modules also signed by governance)

**Signature Verification:**

- Clients MUST verify signatures before trusting events
- Governance signatures MUST meet threshold requirements
- Invalid signatures MUST be rejected
- Layer + tier combination MUST be verified

### Privacy Protections

**Personal Information:**

- NO IP addresses in any event
- NO geographic location more specific than continent/region (optional)
- NO personally identifying information unless voluntarily disclosed

**Key Management:**

- Node telemetry uses ephemeral keys that rotate periodically
- Keyholder identity can be pseudonymous
- Module authors can use pseudonymous keys

**Data Minimization:**

- Only publish information necessary for coordination
- Avoid metadata that could enable tracking
- Use content-addressed references rather than personal identifiers

### Nostr Zaps (Lightning Payments)

**Purpose:** Enable community donations via Nostr zaps (Lightning payments)

**Implementation:**

1. **Zap Handling:**
   - All Nostr events published by Bitcoin Commons include a `zap` tag pointing to a donations Lightning address
   - Community members can zap (tip) governance events, announcements, or keyholders
   - Zaps are automatically forwarded to a configured donations wallet

2. **Zap Configuration:**
   ```json
   {
     "kind": 0,
     "tags": [
       ["zap", "lnbc1..."],
       ["zap", "bitcoin:bc1q..."]
     ],
     "content": {
       "lud16": "donations@btcdecoded.org",
       "lud06": "lnurl1..."
     }
   }
   ```

3. **Donation Routing:**
   - Zaps received on governance event pubkeys → Forward to donations wallet
   - Zaps received on keyholder pubkeys → Forward to donations wallet (or allow keyholder to keep)
   - Zaps received on node telemetry pubkeys → Forward to donations wallet
   - All zaps logged for transparency (amount, timestamp, optional message)

4. **Privacy Considerations:**
   - Zap amounts are public on Lightning network
   - Zap messages are optional and can be filtered
   - Donations wallet address is public (transparency)
   - Individual zap sources are not tracked (privacy-preserving)

5. **Integration Points:**
   - `bllvm-commons` includes zap forwarding service
   - Keyholder metadata includes optional zap address
   - Governance events include donations wallet in metadata
   - Status page displays donation information

**Benefits:**
- Enables community financial support
- Transparent donation tracking
- Privacy-preserving (no source tracking)
- Aligns with Bitcoin Commons values (Lightning integration)

### Relay Selection

**Recommended Relay Strategy:**

- Use multiple public relays for redundancy
- Prefer established, censorship-resistant relays
- Allow users to configure custom relays
- Governance can publish recommended relay list (not mandatory)

**Relay Independence:**

- System MUST work if some relays are unavailable
- Clients SHOULD connect to 3+ relays
- No single relay is critical (graceful degradation)

### Spam and Abuse Prevention

**Rate Limiting:**

- Nodes publish telemetry once per 24 hours maximum
- Governance events are infrequent (legitimate actions only)
- Module proposals require reputation or governance approval

**Reputation Systems:**

- Module reviews weighted by reviewer reputation
- Dispute resolution considers party reputation
- Keyholders have inherent reputation (governance role)

**Content Moderation:**

- Relays can filter spam independently
- Clients can implement local filters
- No central moderation authority

### Attack Scenarios

**Scenario 1: Relay Censorship**

- **Mitigation:** Multiple relay redundancy, users can add custom relays

**Scenario 2: Fake Governance Events**

- **Mitigation:** Signature verification, only trust events signed by known keyholders

**Scenario 3: Sybil Attack on Telemetry**

- **Mitigation:** Events signed by ephemeral keys, reputation-weighted aggregation

**Scenario 4: Dispute Forum Spam**

- **Mitigation:** Require reputation to file disputes, governance can deprecate spam

**Scenario 5: Malicious Module Listings**

- **Mitigation:** Verified vs unverified distinction, security reports, community reviews

---

## Implementation Priorities

### Phase 1: Foundation (Immediate - Q4 2025)

**Essential Integrations:**

1. **Governance Transparency**

   - Governance action events (with layer + tier information)
   - Keyholder announcements
   - Integration with `bllvm-commons` GitHub App
   - Layer + tier combination calculation

2. **Network Telemetry**

   - Node status reporting (opt-in)
   - Status page Nostr subscription
   - Privacy-preserving aggregation

3. **Zap Support (Basic)**

   - Zap forwarding to donations wallet
   - Zap metadata in keyholder announcements
   - Basic zap logging

**Deliverables:**

- Nostr event schemas defined (with layer + tier)
- `bllvm-commons` publishes to Nostr
- Status page consumes from Nostr
- Documentation for node operators
- Basic zap forwarding service

**Success Criteria:**

- All governance actions visible on Nostr
- Status page shows real-time node count
- Privacy protections verified
- Layer + tier information correctly included
- Zaps can be received and forwarded

**Phase 1 Scope Assessment:**

✅ **NOT Scope Creep** - Phase 1 aligns with existing `bllvm-commons` architecture:

1. **Governance Transparency:**
   - ✅ `bllvm-commons` already has GitHub integration (status checks, merge blocking)
   - ✅ Governance events are already tracked in database
   - ✅ Adding Nostr publishing is a natural extension (already mentioned in README)
   - ✅ Layer + tier information already calculated for PR classification

2. **Network Telemetry:**
   - ✅ Node status reporting is opt-in (no breaking changes)
   - ✅ Status page is separate component (can be built independently)
   - ✅ Privacy-preserving by design (ephemeral keys, no IPs)

3. **Zap Support:**
   - ✅ Basic zap forwarding is lightweight (Lightning address + forwarding service)
   - ✅ No consensus impact (purely coordination layer)
   - ✅ Aligns with Bitcoin Commons values (Lightning integration)

**Existing Infrastructure:**
- ✅ `bllvm-commons` has database layer (can store zap logs)
- ✅ `bllvm-commons` has GitHub webhook handling (can trigger Nostr publishing)
- ✅ `bllvm-commons` has configuration system (can configure Nostr relays, zap addresses)
- ✅ `bllvm-commons` has audit logging (can log zap events)

**Risk Assessment:**
- **Low Risk:** All Phase 1 features are additive (opt-in, no breaking changes)
- **Low Complexity:** Nostr publishing is straightforward (event creation + relay publishing)
- **High Value:** Transparency and coordination are core Bitcoin Commons values

**Conclusion:** Phase 1 is **feasible and aligned** with existing system architecture. No scope creep detected.

### Phase 2: Community (Q1-Q2 2026)

**Priority Integrations:**

1. **Development Announcements**

   - Release notifications
   - Security advisories
   - Roadmap updates

2. **Module Proposals**

   - Module metadata publishing
   - Community review process
   - Governance approval workflow

**Deliverables:**

- Announcement publishing system
- Module proposal templates
- Community review dashboard

**Success Criteria:**

- Announcements reach community via Nostr
- Module proposals have standard format
- Review process is transparent

### Phase 3: Ecosystem (Q3-Q4 2026)

**Advanced Integrations:**

1. **Module Marketplace**

   - Full module registry
   - Reviews and ratings
   - Security reporting

2. **Dispute Resolution**

   - Formal dispute filing
   - Mediation tracking
   - Fork documentation

**Deliverables:**

- Module marketplace UI
- Dispute resolution interface
- Governance fork tooling

**Success Criteria:**

- Modules discoverable via Nostr
- Disputes handled transparently
- Governance forks are smooth

### Phase 4: Maturity (2027+)

**Future Enhancements:**

1. Advanced reputation systems
2. Cross-implementation coordination (if other implementations adopt)
3. Economic coordination (treasury management, grants)
4. Research and analytics tools

---

## Future Extensions

### Potential Additional Use Cases

**1. Economic Coordination**

- Merge mining revenue reporting
- Grant allocation proposals
- Budget transparency

**2. Research and Analytics**

- Network health metrics
- Performance benchmarks
- Adoption tracking

**3. Educational Content**

- Tutorial publications
- Documentation updates
- Community guides

**4. Cross-Implementation Coordination**

- Standardized event schemas
- Interoperability testing
- Shared governance frameworks

### Integration with Other Protocols

**Bitcoin:**

- OpenTimestamps anchoring of governance events
- On-chain references for critical decisions

**Lightning:**

- Module marketplace payments via Lightning
- Grant distributions via Lightning

**Fedimint/Cashu:**

- Ecash tokens for module licensing
- Anonymous payments for services

### Governance Evolution

**As governance matures:**

- More sophisticated dispute resolution
- Economic voting mechanisms
- Liquid democracy experiments
- Quadratic funding for modules

---

## Appendix: Event Type Reference

### Quick Reference Table

| Event Kind | Purpose | Publisher | Subscribers |
|------------|---------|-----------|-------------|
| 0 (Metadata) | Keyholder info | Keyholders | Community, dashboards |
| 1 (Short Text) | Comments, discussions | Anyone | Threaded discussions |
| 30023 (Long-form) | Announcements, proposals, disputes | Maintainers, community | Everyone |
| 30078 (Parameterized) | Governance actions, node status, modules | `bllvm-commons`, nodes, devs | Status pages, marketplaces |

### Tag Conventions

**Governance Tags:**

- `t: btc-commons-*` - All Bitcoin Commons related events
- `governance_config: commons_mainnet` - Which governance this event belongs to
- `governance_tier: 1-5` - Action tier classification
- `governance_layer: 1-6` - Repository layer
- `repository: bllvm-*` - Specific repository
- `final_signatures: N-of-M` - Combined signature requirement
- `final_review_days: N` - Combined review period
- `verified: true/false` - Governance verification status

**Module Tags:**

- `module_name: *` - Module identifier
- `category: network|economic|application|experimental` - Module type
- `consensus_impact: true/false` - Whether module affects consensus

**Status Tags:**

- `version: *` - Software version
- `features: *` - Enabled features (comma-separated)
- `network: mainnet|testnet|signet` - Which Bitcoin network

### Repository-to-Layer Quick Reference

| Repository | Layer | Signatures | Review Days |
|------------|-------|------------|-------------|
| `bllvm-spec` | 1 | 6-of-7 | 180 (365 consensus) |
| `bllvm-consensus` | 2 | 6-of-7 | 180 (365 consensus) |
| `bllvm-protocol` | 3 | 4-of-5 | 90 |
| `bllvm-node` | 4 | 3-of-5 | 60 |
| `bllvm` | 4 | 3-of-5 | 60 (same as bllvm-node) |
| `bllvm-sdk` | 5 | 2-of-3 | 14 |
| `bllvm-commons` | 6 | Varies by tier | Varies by tier |

### Layer + Tier Combination Examples

| Repository | Layer | Tier | Final Signatures | Final Review | Source |
|------------|-------|------|------------------|--------------|---------|
| `bllvm-protocol` | 3 | 1 | 4-of-5 | 90 days | Layer 3 |
| `bllvm-sdk` | 5 | 2 | 4-of-5 | 30 days | Tier 2 |
| `bllvm-spec` | 1 | 3 | 6-of-7 | 180 days | Layer 1 |
| `bllvm-node` | 4 | 4 | 4-of-5 | 0 days | Tier 4 |

---

## Conclusion

Nostr integration provides Bitcoin Commons with:

1. **Decentralized Transparency** - All governance actions visible without central servers
2. **Censorship Resistance** - Multiple relay redundancy prevents silencing
3. **Privacy Preservation** - Coordination without exposing identities or IPs
4. **Forkable Coordination** - Governance forks can use separate Nostr tags/relays
5. **Community Empowerment** - Anyone can build tools on top of events
6. **Economic Sustainability** - No infrastructure costs for coordination layer
7. **Full Transparency** - Layer + tier combination logic visible in all events

The integration maintains architectural purity (consensus never depends on Nostr) while enabling the transparency and coordination that Bitcoin's governance has always needed.

By leveraging Nostr, Bitcoin Commons creates the institutional layer Bitcoin always needed - transparent, decentralized, and censorship-resistant - while preserving the technical excellence and user sovereignty that make Bitcoin valuable.

---

**Document Status:** Design Complete (Validated Against Codebase)  
**Implementation Status:** Phase 1 Ready  
**Next Steps:** Begin `bllvm-commons` Nostr integration  
**Feedback:** Community review requested before Phase 1 implementation

---

*This design document is living documentation. As Bitcoin Commons evolves and community needs emerge, this document will be updated to reflect new use cases and integration patterns.*

