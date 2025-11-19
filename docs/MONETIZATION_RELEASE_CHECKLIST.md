# Monetization System Release Checklist

## Pre-Release Checklist

### ✅ Core Implementation
- [x] Zap subscription and tracking
- [x] Merge mining contribution tracking
- [x] Fee forwarding tracking
- [x] Weight calculation (quadratic + caps + cooling-off)
- [x] Vote aggregation
- [x] Economic node veto integration
- [x] Database migrations
- [x] Mathematical specifications
- [x] Kani formal verification
- [x] Integration tests

### 🔧 Configuration & Setup

#### 1. Database Migration
```bash
# Verify migration 005 runs successfully
cd bllvm-commons
sqlx migrate run
# Should create:
# - unified_contributions
# - zap_contributions
# - fee_forwarding_contributions
# - participation_weights
# - proposal_zap_votes
```

#### 2. Configuration Setup

**Required Environment Variables**:
```bash
# Nostr Configuration
NOSTR_ENABLED=true
NOSTR_SERVER_NSEC_PATH=/etc/governance/server.nsec
NOSTR_RELAYS=wss://relay.damus.io,wss://nos.lol
NOSTR_ZAP_ADDRESS=donations@btcdecoded.org  # Legacy (optional)
GOVERNANCE_CONFIG=commons_mainnet

# Governance Configuration
GOVERNANCE_COMMONS_ADDRESSES=bc1qcommons1,bc1qcommons2  # Comma-separated
GOVERNANCE_CONTRIBUTION_TRACKING_ENABLED=true
GOVERNANCE_WEIGHT_UPDATES_ENABLED=true
GOVERNANCE_WEIGHT_UPDATE_INTERVAL_SECS=86400  # Daily
```

**Or in `config/production.toml`**:
```toml
[governance]
commons_addresses = ["bc1qcommons1", "bc1qcommons2"]
contribution_tracking_enabled = true
weight_updates_enabled = true
weight_update_interval_secs = 86400
```

#### 3. Bot Configuration (Multi-Bot Support)

```toml
[nostr.bots.governance_bot]
nsec_path = "/etc/governance/bots/governance.nsec"
npub = "npub1..."
lightning_address = "donations@btcdecoded.org"

[nostr.bots.governance_bot.profile]
name = "@BTCCommons_Gov"
about = "Bitcoin Commons Governance Bot"
picture = "https://btcdecoded.org/assets/logo.png"
```

### 🚀 Service Initialization

#### Automatic Startup (main.rs)
The following services start automatically:

1. **Zap Tracker**: Starts if `NOSTR_ENABLED=true` and bot pubkeys configured
2. **Weight Update Task**: Runs daily (or configured interval) if `weight_updates_enabled=true`
3. **Database Migrations**: Run automatically on startup

#### Manual Verification

```bash
# Check logs for initialization
journalctl -u bllvm-commons -f | grep -i "governance\|zap\|weight"

# Expected log messages:
# "Zap tracker started"
# "Periodic weight update task started"
# "Database migrations completed"
```

### 📊 Health Checks

#### Status Endpoint
```bash
curl http://localhost:3000/status | jq '.features.governance'
```

**Expected Response**:
```json
{
  "enabled": true,
  "tables_exist": true,
  "contributor_count": 0,
  "weight_updates_enabled": true,
  "commons_addresses_count": 2
}
```

#### Database Verification
```bash
# Check tables exist
sqlite3 governance.db ".tables" | grep -E "contributions|weights|votes"

# Check migration applied
sqlite3 governance.db "SELECT version FROM _sqlx_migrations WHERE version = 5;"
```

### 🔒 Security Checklist

- [ ] **Nostr Keys**: Securely stored, not in git
- [ ] **Database**: Proper access controls
- [ ] **Commons Addresses**: Verified and correct
- [ ] **Rate Limiting**: Enabled for webhooks
- [ ] **Backup**: Automated backups configured
- [ ] **Monitoring**: Health checks configured

### 📝 Documentation

- [x] Mathematical specifications (Orange Paper)
- [x] Kani proofs
- [x] Integration tests
- [ ] **User Guide**: How to zap to vote
- [ ] **Operator Guide**: How to configure and monitor
- [ ] **Deployment Guide**: Step-by-step setup

### 🧪 Testing

#### Pre-Release Testing
```bash
# Run all governance tests
cd bllvm-commons
cargo test --test governance_tests
cargo test --test governance_integration_test
cargo test --test governance_e2e_integration_test

# Run Kani proofs (if available)
cargo kani --features verify --tests verification::governance_kani_proofs

# Verify migrations
sqlx migrate info
```

#### Production-Like Testing
1. **Test Zap Tracking**:
   - Send test zap to bot pubkey
   - Verify it appears in `zap_contributions` table
   - Verify it's recorded in `unified_contributions`

2. **Test Weight Calculation**:
   - Create test contributions
   - Run weight update manually
   - Verify weights calculated correctly

3. **Test Vote Aggregation**:
   - Create test proposal
   - Add zap votes
   - Verify aggregation works

### 📋 Deployment Steps

1. **Database Setup**:
   ```bash
   # Ensure database exists
   sqlite3 governance.db "SELECT 1;"
   
   # Run migrations
   sqlx migrate run
   ```

2. **Configuration**:
   ```bash
   # Copy production config
   cp config/production.toml.example config/production.toml
   
   # Edit and set:
   # - Commons addresses
   # - Nostr bot pubkeys
   # - Zap addresses
   ```

3. **Start Service**:
   ```bash
   # Build
   cargo build --release
   
   # Start
   systemctl start bllvm-commons
   
   # Verify
   systemctl status bllvm-commons
   curl http://localhost:3000/health
   ```

4. **Monitor**:
   ```bash
   # Watch logs
   journalctl -u bllvm-commons -f
   
   # Check status
   curl http://localhost:3000/status | jq
   ```

### ⚠️ Known Limitations

1. **Address Decoding**: Fee forwarding tracker uses simplified address decoding (P2PKH only)
   - **Workaround**: Use P2PKH addresses for Commons addresses initially
   - **Future**: Add full address decoding (P2SH, P2WPKH, P2WSH, P2TR)

2. **Transaction Hashing**: Fee forwarding tracker uses placeholder hash
   - **Workaround**: Will be fixed in next iteration
   - **Impact**: Duplicate detection may not work perfectly

3. **Merge Mining Integration**: Placeholder in merge mining coordinator
   - **Workaround**: Contributions tracked via event system or manual entry
   - **Future**: Full integration with merge mining coordinator

### 🎯 Post-Release Monitoring

#### Key Metrics to Monitor

1. **Zap Tracking**:
   - Zaps received per day
   - Failed zap processing
   - Duplicate zap detection

2. **Weight Updates**:
   - Update success rate
   - Update duration
   - Contributor count growth

3. **Vote Aggregation**:
   - Proposals created
   - Votes cast
   - Veto activations

4. **Database Performance**:
   - Query performance
   - Table sizes
   - Index usage

#### Alerting

Set up alerts for:
- Weight update failures
- Zap tracking failures
- Database connection issues
- High error rates

### 📚 Quick Reference

**Configuration Files**:
- `config/production.toml` - Main config
- `.env` - Environment variables (not in git)

**Database Tables**:
- `unified_contributions` - All contributions
- `zap_contributions` - Zap events
- `fee_forwarding_contributions` - Fee forwarding
- `participation_weights` - Calculated weights
- `proposal_zap_votes` - Processed votes

**Key Services**:
- `ZapTracker` - Tracks zaps from Nostr
- `ContributionTracker` - Records all contributions
- `WeightCalculator` - Calculates participation weights
- `VoteAggregator` - Aggregates votes for proposals

**Endpoints**:
- `GET /health` - Health check
- `GET /status` - Detailed status (includes governance)

---

## Release Sign-Off

- [ ] All tests passing
- [ ] Migrations verified
- [ ] Configuration documented
- [ ] Health checks working
- [ ] Monitoring configured
- [ ] Documentation complete
- [ ] Security review complete

**Ready for Release**: ✅ / ❌

