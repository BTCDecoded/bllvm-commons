# Feature Placement Fixes Applied

## Summary

Fixed feature placement issues to ensure:
1. ✅ ZMQ is available in base variant (our differentiator!)
2. ✅ Infrastructure features are explicitly in base
3. ✅ utxo-commitments removed from default (experimental only)
4. ✅ All experimental features properly categorized
5. ✅ quinn added to experimental variant

## Changes Made

### 1. Updated `bllvm/build.sh` - Base Variant

**Before:**
```bash
features="production"
```

**After:**
```bash
# For bllvm-node and bllvm:
features="sysinfo,redb,nix,libc,production,governance,zmq"
```

**Rationale:**
- `sysinfo,redb,nix,libc` - Core infrastructure (required for node operation)
- `production` - Performance optimizations
- `governance` - Core governance feature
- `zmq` - **Differentiator!** Real-time notifications enabled by default

### 2. Updated `bllvm/build.sh` - Experimental Variant

**Before:**
```bash
features="production,utxo-commitments,ctv,dandelion,stratum-v2,bip158,sigop,iroh"
```

**After:**
```bash
features="sysinfo,redb,nix,libc,production,governance,zmq,utxo-commitments,ctv,dandelion,stratum-v2,bip158,sigop,iroh,quinn"
```

**Rationale:**
- Includes all base features
- Adds all experimental features
- Added `quinn` (was missing)

### 3. Updated `bllvm-node/Cargo.toml` - Default Features

**Before:**
```toml
default = ["sysinfo", "redb", "nix", "libc", "utxo-commitments", "production", "governance", "zmq"]
```

**After:**
```toml
default = ["sysinfo", "redb", "nix", "libc", "production", "governance", "zmq"]
# Note: utxo-commitments removed from default - it's experimental only
```

**Rationale:**
- Removed `utxo-commitments` from default (experimental only)
- Kept infrastructure and core features in default

## Feature Placement Summary

### Base Variant (Consensus-Only)
**bllvm-consensus & bllvm-protocol:**
- `production` only

**bllvm-node & bllvm:**
- `sysinfo` - System information
- `redb` - Storage backend
- `nix` - Process sandboxing
- `libc` - System calls
- `production` - Performance optimizations
- `governance` - Governance webhooks
- `zmq` - **Real-time notifications (differentiator)**

### Experimental Variant
**bllvm-consensus:**
- `production,utxo-commitments,ctv`

**bllvm-protocol:**
- `production,utxo-commitments,ctv`

**bllvm-node & bllvm:**
- All base features PLUS:
- `utxo-commitments` - UTXO commitment system
- `ctv` - BIP119 CheckTemplateVerify
- `dandelion` - Privacy-preserving relay
- `stratum-v2` - Stratum V2 mining
- `bip158` - Compact block filters
- `sigop` - Signature operations counting
- `iroh` - Iroh transport
- `quinn` - Quinn QUIC transport

## Verification

### Base Build Should Include:
- ✅ ZMQ notifications (differentiator)
- ✅ Governance webhooks
- ✅ Core infrastructure
- ✅ Production optimizations
- ❌ NO experimental features

### Experimental Build Should Include:
- ✅ Everything from base
- ✅ All experimental features
- ✅ All transport options (iroh, quinn)

## Impact

1. **ZMQ Now Available in Base**: Our differentiator is now properly included in the consensus-only build
2. **Clear Separation**: Base = stable core, Experimental = all features
3. **No Accidental Experimental Features**: utxo-commitments removed from default
4. **Complete Experimental Build**: quinn transport now included

