# Nostr Integration Plan Validation

**Date:** November 17, 2025  
**Status:** Gap Analysis Complete

---

## Current Implementation vs New Plan

### ✅ What We Have (Single Bot Approach)

1. **Single Nostr Client** - One bot identity in governance-app
2. **Webhook Integration** - Automatic publishing on PR merges
3. **Event Schemas** - GovernanceActionEvent, KeyholderAnnouncement, etc.
4. **Logo Support** - Bitcoin Commons logo in keyholder announcements
5. **Config** - Basic NostrConfig with zap_address, logo_url

### 🎯 What New Plan Requires (Multi-Bot Approach)

1. **Multiple Bot Identities**:
   - @BTCCommons_Gov - Governance announcements
   - @BTCCommons_Dev - Development updates
   - @BTCCommons_Research - Educational content (optional)
   - @BTCCommons_Network - Network metrics (optional)

2. **GitHub Actions Integration**:
   - Jobs triggered for announcements
   - Secrets for NOSTR_PRIVATE_KEY (nsec) per bot
   - Automated publishing from workflows

3. **Enhanced Features**:
   - NIP-57 (Zaps) - Already have zap_address
   - NIP-65 (Relay List Metadata) - Need to add
   - Zap goals - Need to add
   - Profile optimization - Need to add

4. **Content Strategy**:
   - Value-first posting templates
   - Posting schedules
   - Bot-specific content types

---

## Gap Analysis

### Critical Gaps

1. **Multi-Bot Architecture** ❌
   - Current: Single client
   - Needed: Multiple bot identities with separate keys

2. **GitHub Actions Jobs** ❌
   - Current: Webhook-triggered only
   - Needed: Workflow jobs for announcements

3. **Bot Configuration** ❌
   - Current: Single config
   - Needed: Per-bot config (npub, profile, lightning address)

4. **NIP-65 Support** ❌
   - Current: Not implemented
   - Needed: Relay list metadata publishing

5. **Zap Goals (NIP-57)** ❌
   - Current: Basic zap_address
   - Needed: Zap goal metadata in events

### Compatible Features

1. **Event Schemas** ✅
   - Current schemas work for multi-bot approach
   - Just need to route to correct bot

2. **Logo Support** ✅
   - Already implemented
   - Can be used for all bots

3. **Webhook Integration** ✅
   - Can be extended to select bot based on event type

4. **Layer + Tier Logic** ✅
   - Already implemented correctly

---

## Implementation Plan

### Phase 1: Multi-Bot Architecture

**Changes Required:**

1. **Extend NostrConfig**:
   ```rust
   pub struct NostrConfig {
       pub enabled: bool,
       pub bots: HashMap<String, BotConfig>,  // "gov", "dev", "research", "network"
       pub relays: Vec<String>,
       pub publish_interval_secs: u64,
       pub governance_config: String,
   }

   pub struct BotConfig {
       pub nsec_path: String,  // Path to nsec file (or GitHub secret)
       pub npub: String,      // Public key
       pub lightning_address: String,
       pub profile: BotProfile,
   }

   pub struct BotProfile {
       pub name: String,
       pub about: String,
       pub picture: String,  // Logo variant
   }
   ```

2. **Multi-Bot Client Manager**:
   ```rust
   pub struct NostrBotManager {
       bots: HashMap<String, NostrClient>,
       config: NostrConfig,
   }

   impl NostrBotManager {
       pub async fn publish_governance_action(&self, event: GovernanceActionEvent) -> Result<()> {
           let bot = self.bots.get("gov")?;
           // Publish via gov bot
       }

       pub async fn publish_dev_update(&self, update: DevUpdate) -> Result<()> {
           let bot = self.bots.get("dev")?;
           // Publish via dev bot
       }
   }
   ```

3. **Bot Selection Logic**:
   - Governance actions → @BTCCommons_Gov
   - PR merges → @BTCCommons_Dev (or Gov if governance-related)
   - Releases → @BTCCommons_Dev
   - Network metrics → @BTCCommons_Network

### Phase 2: GitHub Actions Integration

**New Workflow:** `.github/workflows/nostr-announce.yml`

```yaml
name: Nostr Announcements

on:
  workflow_dispatch:
    inputs:
      bot:
        description: 'Bot identity (gov, dev, research, network)'
        required: true
        type: choice
        options:
          - gov
          - dev
          - research
          - network
      event_type:
        description: 'Event type (merge, release, milestone, etc.)'
        required: true
        type: string
      content:
        description: 'Event content (JSON or text)'
        required: true
        type: string

jobs:
  publish:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Publish to Nostr
        env:
          NOSTR_NSEC: ${{ secrets.NOSTR_NSEC_GOV }}  # Or DEV, RESEARCH, NETWORK
        run: |
          cargo run --bin nostr-publisher -- \
            --bot ${{ inputs.bot }} \
            --event-type ${{ inputs.event_type }} \
            --content "${{ inputs.content }}"
```

**Secrets Required:**
- `NOSTR_NSEC_GOV` - Governance bot private key
- `NOSTR_NSEC_DEV` - Development bot private key
- `NOSTR_NSEC_RESEARCH` - Research bot private key (optional)
- `NOSTR_NSEC_NETWORK` - Network bot private key (optional)

### Phase 3: Enhanced Features

1. **NIP-65 (Relay List Metadata)**:
   ```rust
   pub async fn publish_relay_list(&self, bot: &str, relays: &[String]) -> Result<()> {
       let tags = relays.iter().map(|r| Tag::Relay(r.clone())).collect();
       let event = EventBuilder::new(Kind::RelayListMetadata, "", tags)
           .to_event(&keys)?;
       self.publish_event(event).await
   }
   ```

2. **NIP-57 (Zap Goals)**:
   ```rust
   pub fn create_zap_goal_tags(goal_sats: u64, current_sats: u64) -> Vec<Tag> {
       vec![
           Tag::Generic(TagKind::Custom("zap".into()), vec![lightning_address]),
           Tag::Generic(TagKind::Custom("goal".into()), vec![goal_sats.to_string()]),
           Tag::Generic(TagKind::Custom("current".into()), vec![current_sats.to_string()]),
       ]
   }
   ```

3. **Profile Publishing (Kind 0)**:
   ```rust
   pub async fn publish_bot_profile(&self, bot: &str, profile: &BotProfile) -> Result<()> {
       let content = serde_json::json!({
           "name": profile.name,
           "about": profile.about,
           "picture": profile.picture,
           "lud16": profile.lightning_address,
       });
       // Publish Kind 0 event
   }
   ```

---

## Migration Strategy

### Step 1: Extend Current Implementation

1. Add multi-bot config structure
2. Create BotManager to handle multiple clients
3. Keep existing webhook integration, route to appropriate bot

### Step 2: Add GitHub Actions

1. Create workflow file
2. Add secrets to GitHub repo
3. Create CLI tool for publishing from workflows

### Step 3: Enhance Features

1. Add NIP-65 support
2. Add zap goals
3. Add profile publishing

### Step 4: Testing & Documentation

1. Unit tests for multi-bot logic
2. Integration tests for GitHub Actions
3. Update documentation

---

## Configuration Example

```toml
[nostr]
enabled = true
governance_config = "commons_mainnet"
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]
publish_interval_secs = 3600

[nostr.bots.gov]
nsec_path = "env:NOSTR_NSEC_GOV"  # GitHub secret
npub = "npub1..."  # Public key
lightning_address = "donations@btcdecoded.org"
profile_name = "@BTCCommons_Gov"
profile_about = "Official governance announcements from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo-gov.png"

[nostr.bots.dev]
nsec_path = "env:NOSTR_NSEC_DEV"
npub = "npub1..."
lightning_address = "dev@btcdecoded.org"
profile_name = "@BTCCommons_Dev"
profile_about = "Development updates from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo-dev.png"

[nostr.bots.research]
nsec_path = "env:NOSTR_NSEC_RESEARCH"
npub = "npub1..."
lightning_address = "research@btcdecoded.org"
profile_name = "@BTCCommons_Research"
profile_about = "Educational content and research from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo-research.png"

[nostr.bots.network]
nsec_path = "env:NOSTR_NSEC_NETWORK"
npub = "npub1..."
lightning_address = "network@btcdecoded.org"
profile_name = "@BTCCommons_Network"
profile_about = "Network metrics and statistics from Bitcoin Commons..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo-network.png"
```

---

## Decision Points

### 1. Bot Selection Logic

**Question:** How do we determine which bot to use?

**Answer:**
- **Governance actions** (merges, releases, budget votes) → @BTCCommons_Gov
- **Development updates** (code releases, benchmarks, bug fixes) → @BTCCommons_Dev
- **Research content** (educational threads, analysis) → @BTCCommons_Research (manual)
- **Network metrics** (node stats, adoption) → @BTCCommons_Network

### 2. GitHub Actions vs Webhook

**Question:** Should we use GitHub Actions for all announcements or keep webhooks?

**Answer:**
- **Webhooks** for automated events (PR merges, releases)
- **GitHub Actions** for manual announcements (research threads, strategic posts)
- Both can coexist

### 3. Zap Forwarding

**Question:** Do we need a forwarding service?

**Answer:**
- **No** - Just accept zaps to lightning addresses
- Lightning node/wallet handles receipt
- No forwarding service needed

### 4. Node Telemetry

**Question:** How to implement node telemetry?

**Answer:**
- Opt-in service in bllvm-node
- Ephemeral keys for privacy
- Publish to @BTCCommons_Network
- No IP addresses, only metrics

---

## Next Steps

1. ✅ **Validate Plan** - This document
2. ⏳ **Implement Multi-Bot Architecture** - Extend current code
3. ⏳ **Create GitHub Actions Workflow** - For manual announcements
4. ⏳ **Add Enhanced Features** - NIP-65, zap goals
5. ⏳ **Testing** - Unit and integration tests
6. ⏳ **Documentation** - Update with multi-bot approach
7. ⏳ **Node Telemetry** - Implement in bllvm-node

---

**Status:** Plan validated, ready for implementation

