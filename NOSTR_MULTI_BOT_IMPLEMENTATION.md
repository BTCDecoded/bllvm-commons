# Nostr Multi-Bot Implementation Summary

**Date:** November 17, 2025  
**Status:** ✅ Core Implementation Complete

---

## ✅ Completed Implementation

### 1. Multi-Bot Architecture

**Files Created:**
- `governance-app/src/nostr/bot_manager.rs` - Bot manager for multiple identities
- `governance-app/src/bin/nostr-publisher.rs` - CLI tool for GitHub Actions
- `governance-app/.github/workflows/nostr-announce.yml` - GitHub Actions workflow

**Files Modified:**
- `governance-app/src/config.rs` - Extended with `BotConfig` and `BotProfile`
- `governance-app/src/nostr/mod.rs` - Exported `NostrBotManager`
- `governance-app/Cargo.toml` - Added binary entry for `nostr-publisher`

### 2. Bot Configuration

**Structure:**
```rust
pub struct NostrConfig {
    pub bots: HashMap<String, BotConfig>,  // Multi-bot support
    // ... other fields
}

pub struct BotConfig {
    pub nsec_path: String,  // Supports "env:VAR_NAME" for GitHub secrets
    pub npub: String,
    pub lightning_address: String,
    pub profile: BotProfile,
}

pub struct BotProfile {
    pub name: String,      // e.g., "@BTCCommons_Gov"
    pub about: String,
    pub picture: String,   // Logo variant
}
```

**Bot Identities:**
- `gov` - @BTCCommons_Gov (Governance announcements)
- `dev` - @BTCCommons_Dev (Development updates)
- `research` - @BTCCommons_Research (Educational content, optional)
- `network` - @BTCCommons_Network (Network metrics, optional)

### 3. GitHub Actions Integration

**Workflow:** `.github/workflows/nostr-announce.yml`

**Features:**
- Manual dispatch with bot selection
- Event type specification
- Content input (JSON or markdown)
- Event kind selection (default: 1 for text notes)
- Uses GitHub secrets for nsec keys

**Required Secrets:**
- `NOSTR_NSEC_GOV` - Governance bot private key
- `NOSTR_NSEC_DEV` - Development bot private key
- `NOSTR_NSEC_RESEARCH` - Research bot private key (optional)
- `NOSTR_NSEC_NETWORK` - Network bot private key (optional)

**Usage:**
1. Go to Actions → "Nostr Announcements"
2. Click "Run workflow"
3. Select bot, event type, content
4. Workflow publishes to Nostr

### 4. Bot Manager

**Features:**
- Loads multiple bot identities from config
- Supports `env:VAR_NAME` format for nsec paths (GitHub secrets)
- Fallback logic (dev → gov, research → dev → gov, etc.)
- Per-bot lightning address retrieval
- Connection management

**Key Methods:**
- `get_bot(bot_id)` - Get specific bot
- `get_gov_bot()` - Get governance bot
- `get_dev_bot()` - Get dev bot (with fallback)
- `get_research_bot()` - Get research bot (with fallback)
- `get_network_bot()` - Get network bot (with fallback)

### 5. CLI Tool

**Binary:** `nostr-publisher`

**Usage:**
```bash
cargo run --bin nostr-publisher -- \
  --bot gov \
  --event-type merge \
  --content "PR #123 merged" \
  --kind 1 \
  --relays "wss://relay.damus.io,wss://nos.lol"
```

**Environment Variables:**
- `NOSTR_NSEC_{BOT}` - Bot private key (from GitHub secrets)
- `NOSTR_RELAYS` - Comma-separated relay URLs (optional)
- `NOSTR_LIGHTNING_{BOT}` - Lightning address (optional, for zap tags)

---

## 🔄 Migration from Single Bot

### Backward Compatibility

The implementation maintains backward compatibility:

1. **Legacy Config Support:**
   - If `bots` HashMap is empty, falls back to `server_nsec_path`
   - Creates default "gov" bot from legacy config

2. **Gradual Migration:**
   - Can run both single-bot and multi-bot simultaneously
   - Existing webhook integration still works
   - New GitHub Actions workflow for manual announcements

### Configuration Migration

**Old Config:**
```toml
[nostr]
server_nsec_path = "/etc/governance/server.nsec"
zap_address = "donations@btcdecoded.org"
```

**New Config:**
```toml
[nostr]
governance_config = "commons_mainnet"
relays = ["wss://relay.damus.io", "wss://nos.lol"]

[nostr.bots.gov]
nsec_path = "env:NOSTR_NSEC_GOV"
npub = "npub1..."
lightning_address = "donations@btcdecoded.org"
profile_name = "@BTCCommons_Gov"
profile_about = "Official governance announcements..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo-gov.png"

[nostr.bots.dev]
nsec_path = "env:NOSTR_NSEC_DEV"
npub = "npub1..."
lightning_address = "dev@btcdecoded.org"
profile_name = "@BTCCommons_Dev"
profile_about = "Development updates..."
profile_picture = "https://btcdecoded.org/assets/bitcoin-commons-logo-dev.png"
```

---

## 📋 Remaining Work

### ⏳ Testing

**Status:** Pending  
**Priority:** High

**Required:**
- Unit tests for `NostrBotManager`
- Integration tests for bot selection logic
- GitHub Actions workflow testing (manual)
- CLI tool testing

### ⏳ Documentation

**Status:** Pending  
**Priority:** Medium

**Required:**
- Update `NOSTR_INTEGRATION_PLAN.md` with multi-bot architecture
- Add GitHub Actions setup guide
- Document bot configuration
- Add examples for each bot type

### ⏳ Enhanced Features

**Status:** Pending  
**Priority:** Medium

**Required:**
- NIP-65 (Relay List Metadata) support
- NIP-57 (Zap Goals) support
- Profile publishing (Kind 0) for all bots
- Bot selection logic in webhook handlers

### ⏳ Node Telemetry

**Status:** Pending  
**Priority:** Lower (Phase 2)

**Required:**
- Implement in `bllvm-node`
- Opt-in service
- Ephemeral keys
- Publish to @BTCCommons_Network

---

## 🎯 Bot Selection Logic

### Current Implementation

**Webhook Integration:**
- PR merges → @BTCCommons_Gov (via `publish_merge_action`)
- Can be extended to route to different bots based on event type

**GitHub Actions:**
- Manual selection via workflow input
- User chooses bot identity

### Recommended Logic

**Governance Actions** → @BTCCommons_Gov
- PR merges (governance-related)
- Budget votes
- Keyholder changes
- Security advisories

**Development Updates** → @BTCCommons_Dev
- Code releases
- Performance benchmarks
- Bug fixes
- Technical achievements

**Research Content** → @BTCCommons_Research
- Educational threads
- Governance analysis
- Research findings
- (Manual via GitHub Actions)

**Network Metrics** → @BTCCommons_Network
- Node statistics
- Adoption metrics
- Network health
- (Automated from bllvm-node)

---

## 🔐 Security

### GitHub Secrets

**Setup:**
1. Go to repository Settings → Secrets and variables → Actions
2. Add secrets:
   - `NOSTR_NSEC_GOV`
   - `NOSTR_NSEC_DEV`
   - `NOSTR_NSEC_RESEARCH` (optional)
   - `NOSTR_NSEC_NETWORK` (optional)

**Key Generation:**
```bash
# Generate nsec for each bot
nostr-tool generate

# Save nsec as GitHub secret
# Save npub in config file (safe to commit)
```

### Key Rotation

**Procedure:**
1. Generate new keys
2. Update GitHub secrets
3. Update npub in config
4. Publish key rotation announcement via old key
5. Publish profile update via new key

---

## 📊 Usage Examples

### Example 1: Governance Announcement (GitHub Actions)

1. Go to Actions → "Nostr Announcements"
2. Click "Run workflow"
3. Input:
   - Bot: `gov`
   - Event type: `merge`
   - Content: `PR #123 merged: Fix consensus bug in bllvm-consensus`
   - Kind: `1`
4. Workflow publishes to @BTCCommons_Gov

### Example 2: Development Update (GitHub Actions)

1. Go to Actions → "Nostr Announcements"
2. Click "Run workflow"
3. Input:
   - Bot: `dev`
   - Event type: `release`
   - Content: `v0.2.0 released: Performance improvements and bug fixes`
   - Kind: `1`
4. Workflow publishes to @BTCCommons_Dev

### Example 3: Research Thread (GitHub Actions)

1. Go to Actions → "Nostr Announcements"
2. Click "Run workflow"
3. Input:
   - Bot: `research`
   - Event type: `thread`
   - Content: `🧵 Bitcoin Commons Governance Analysis...`
   - Kind: `30023` (long-form)
4. Workflow publishes to @BTCCommons_Research

---

## ✅ Validation

### Code Compilation
- ✅ All new code compiles
- ✅ CLI tool builds successfully
- ✅ Bot manager integrates correctly

### Architecture
- ✅ Multi-bot support implemented
- ✅ GitHub Actions workflow created
- ✅ Backward compatibility maintained

### Configuration
- ✅ Config structure extended
- ✅ Environment variable support
- ✅ GitHub secrets integration

---

## 🚀 Next Steps

1. **Testing** (High Priority)
   - Unit tests for bot manager
   - Integration tests
   - GitHub Actions workflow testing

2. **Documentation** (Medium Priority)
   - Update integration plan
   - Add setup guide
   - Document bot selection logic

3. **Enhanced Features** (Medium Priority)
   - NIP-65 support
   - Zap goals
   - Profile publishing

4. **Node Telemetry** (Lower Priority)
   - Implement in bllvm-node
   - Opt-in service
   - Privacy-preserving design

---

**Status:** ✅ Multi-Bot Architecture Complete  
**Ready for:** Testing, Documentation, Enhanced Features

