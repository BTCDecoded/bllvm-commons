# Feature Placement Analysis: Base vs Experimental

## Current State

### Default Features in Cargo.toml
```toml
default = ["sysinfo", "redb", "nix", "libc", "utxo-commitments", "production", "governance", "zmq"]
```

### Base Variant (build.sh)
```bash
features="production"
```

### Experimental Variant (build.sh)
```bash
features="production,utxo-commitments,ctv,dandelion,stratum-v2,bip158,sigop,iroh"
```

## Issues Found

### ❌ Problem 1: ZMQ Missing from Base
- **Status**: ZMQ is in default but NOT in base variant
- **Impact**: Base builds won't have ZMQ (our differentiator!)
- **Fix**: Add `zmq` to base variant

### ❌ Problem 2: utxo-commitments in Default
- **Status**: utxo-commitments is in default features
- **Impact**: Base builds might get it via default (unintended)
- **Fix**: Remove from default, keep only in experimental

### ❌ Problem 3: Infrastructure Features Not Explicit
- **Status**: sysinfo, redb, nix, libc are in default but not explicitly in base
- **Impact**: Base relies on default features (fragile)
- **Fix**: Explicitly include in base variant

### ❌ Problem 4: governance Not Explicit
- **Status**: governance is in default but not in build.sh
- **Impact**: Unclear if it's in base or experimental
- **Fix**: Decide placement and make explicit

### ⚠️ Problem 5: quinn Not in Build Script
- **Status**: quinn feature exists but not in build.sh
- **Impact**: Not included in either variant
- **Fix**: Add to experimental if needed

## Feature Categorization

### Core Infrastructure (Should be in BASE)
- ✅ `sysinfo` - System information for monitoring
- ✅ `redb` - Storage backend (core functionality)
- ✅ `nix` - Process sandboxing (security)
- ✅ `libc` - System calls (required for nix)
- ✅ `production` - Performance optimizations
- ✅ `zmq` - **Differentiator!** Should be in base
- ✅ `governance` - Governance webhooks (core feature)

### Experimental Features (Should be in EXPERIMENTAL only)
- ✅ `utxo-commitments` - Experimental UTXO commitment system
- ✅ `ctv` - BIP119 CheckTemplateVerify (proposed soft fork)
- ✅ `dandelion` - Privacy-preserving transaction relay
- ✅ `stratum-v2` - Stratum V2 mining protocol
- ✅ `bip158` - Compact block filters
- ✅ `sigop` - Signature operations counting
- ✅ `iroh` - Iroh transport (experimental networking)
- ✅ `quinn` - Quinn QUIC transport (experimental)

### Optional/Advanced (Could be either)
- `mimalloc` - Memory allocator optimization (performance, could be base)
- `json-logging` - Structured logging (infrastructure, could be base)
- `sled` - Alternative storage backend (optional)
- `verify` - Kani verification (dev only, not for releases)

## Recommended Fixes

### Base Variant Should Include:
```bash
features="sysinfo,redb,nix,libc,production,governance,zmq"
```

### Experimental Variant Should Include:
```bash
features="sysinfo,redb,nix,libc,production,governance,zmq,utxo-commitments,ctv,dandelion,stratum-v2,bip158,sigop,iroh,quinn"
```

### Default Features Should Be:
```toml
default = ["sysinfo", "redb", "nix", "libc", "production", "governance", "zmq"]
# Note: utxo-commitments removed from default
```

## Rationale

### Why ZMQ in Base?
- **Differentiator**: ZMQ is enabled by default to differentiate BLLVM
- **Core Feature**: Real-time notifications are a core node feature
- **Stable**: ZMQ is a mature, stable technology
- **No Experimental Risk**: ZMQ doesn't affect consensus or protocol

### Why governance in Base?
- **Core Feature**: Governance is a core BLLVM feature
- **Production Ready**: Governance system is production-ready
- **Not Experimental**: Governance enforcement is stable

### Why utxo-commitments NOT in Base?
- **Experimental**: UTXO commitments are still experimental
- **Consensus-Adjacent**: Could affect consensus validation
- **Optional**: Not required for core Bitcoin functionality

### Why Infrastructure Features in Base?
- **Required**: sysinfo, redb, nix, libc are core infrastructure
- **Stable**: These are mature, stable dependencies
- **Security**: nix/libc provide security features

## Implementation Plan

1. **Update build.sh** - Add infrastructure features to base
2. **Update build.sh** - Add zmq and governance to base
3. **Update build.sh** - Add quinn to experimental
4. **Update Cargo.toml** - Remove utxo-commitments from default
5. **Update documentation** - Document feature placement rationale

