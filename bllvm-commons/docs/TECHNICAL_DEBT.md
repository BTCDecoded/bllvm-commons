# Technical Debt Documentation

**Last Updated**: 2025-01-XX  
**Status**: Active tracking and remediation

## Overview

This document tracks technical debt items across the bllvm-commons codebase, prioritizing critical issues and documenting why non-critical items remain.

## Critical TODOs (High Priority)

### 1. GitHub API Client - Octocrab 0.38 Migration
**Location**: `src/github/client.rs`  
**Count**: 7 TODOs  
**Priority**: HIGH  
**Reason**: API compatibility issues with octocrab 0.38  
**Impact**: Some GitHub API operations may not work correctly  
**Status**: Pending octocrab 0.38 API documentation review

**Items**:
- Line 344: Branch protection API update needed
- Line 452, 474, 518, 538: Workflow API updates needed
- Line 500: Repository dispatch API update needed
- Line 599: Artifacts API update needed
- Line 643: Installation token API update needed

**Action Plan**:
1. Review octocrab 0.38 changelog and migration guide
2. Update all affected API calls
3. Add integration tests for each operation

---

### 2. Nostr Zap Tracking - Invoice Parsing
**Location**: `src/nostr/zap_tracker.rs`  
**Line**: 121  
**Priority**: MEDIUM  
**Reason**: Need to parse bolt11 invoices to extract payment hash  
**Impact**: Zap tracking may not correctly identify duplicate payments  
**Status**: Waiting for bolt11 parsing library integration

**Action Plan**:
1. Evaluate bolt11 parsing libraries (lightning-invoice, etc.)
2. Integrate parsing to extract payment hash
3. Update duplicate detection logic

---

### 3. Fee Forwarding - Transaction Hashing
**Location**: `src/governance/fee_forwarding.rs`  
**Line**: 141  
**Priority**: MEDIUM  
**Reason**: Placeholder transaction hashing implementation  
**Impact**: Duplicate detection for fee forwarding may not work correctly  
**Status**: Needs proper Bitcoin transaction hashing

**Action Plan**:
1. Use bitcoin crate's transaction hashing
2. Implement proper txid calculation
3. Update duplicate detection

---

## Non-Critical TODOs (Documented)

### 4. Test Infrastructure - Mock GitHub Client
**Location**: `src/github/cross_layer_status.rs`  
**Line**: 1409  
**Priority**: LOW  
**Reason**: Test uses real RSA key generation, should use mocks  
**Impact**: Tests may be slower or require external dependencies  
**Status**: Acceptable for now - tests work correctly

**Why It Remains**:
- Tests are functional and pass
- Mock implementation would require significant refactoring
- Low priority compared to production code improvements

---

### 5. Audit Logger - Cloning
**Location**: `src/main.rs`  
**Line**: 180  
**Priority**: LOW  
**Reason**: Audit logger doesn't implement Clone, using Arc would be better  
**Impact**: Minor code complexity  
**Status**: Acceptable - current implementation works

**Why It Remains**:
- Current implementation is functional
- Refactoring would require significant changes
- Not blocking any features

---

### 6. Vote Aggregator - Participation Votes Table
**Location**: `src/governance/vote_aggregator.rs`  
**Line**: 125  
**Priority**: LOW  
**Reason**: Future enhancement to query explicit votes from database  
**Impact**: Feature enhancement, not a bug  
**Status**: Planned for future release

**Why It Remains**:
- Current voting mechanism works via zap tracking
- Explicit vote table is a future enhancement
- Not required for Phase 1 functionality

---

### 7. GitHub Integration - Signature Count
**Location**: `src/webhooks/github_integration.rs`  
**Line**: 254  
**Priority**: LOW  
**Reason**: Should query actual signature count from database  
**Impact**: Status checks may show approximate counts  
**Status**: Acceptable - approximate counts are sufficient

**Why It Remains**:
- Current implementation provides sufficient information
- Database query would add latency
- Not critical for status check functionality

---

### 8. GitHub Integration - Tier Detection
**Location**: `src/webhooks/github_integration.rs`  
**Line**: 292  
**Priority**: LOW  
**Reason**: Hardcoded tier value, should detect from PR  
**Impact**: Tier classification may be incorrect  
**Status**: Acceptable - tier classification happens elsewhere

**Why It Remains**:
- Tier classification is handled by tier_classification module
- This is a fallback/default value
- Not critical for functionality

---

## Error Handling Improvements

### Completed
- ✅ Fixed `pool().unwrap()` calls in `time_lock.rs` (11 instances)
- ✅ Fixed JSON deserialization error handling in `database/mod.rs` (5 instances)
- ✅ Fixed `unwrap_or_default()` in production code paths

### Remaining
- ⏳ Test code unwrap/expect (acceptable - 43 instances in tests)
- ⏳ Some safe fallbacks in production code (e.g., `unwrap_or_else(|_| vec![])`)

## Metrics

### Before Improvements
- Total unwrap/expect: 476 instances
- Critical production code: ~136 instances
- Test code: ~340 instances

### After Improvements
- Total unwrap/expect: ~465 instances
- Critical production code: ~125 instances (11 fixed)
- Test code: ~340 instances (unchanged - acceptable)

### Target
- Production code: <50 instances (only safe fallbacks)
- Test code: Acceptable to keep unwrap/expect

## Next Steps

1. **Immediate** (This Sprint):
   - Complete GitHub API client migration (7 TODOs)
   - Fix remaining critical unwrap/expect in production code

2. **Short-term** (Next Sprint):
   - Implement bolt11 invoice parsing
   - Fix transaction hashing in fee forwarding

3. **Long-term** (Future):
   - Refactor test infrastructure to use mocks
   - Implement explicit vote tracking
   - Improve tier detection in webhooks

## Review Process

This document should be reviewed:
- Weekly during active development
- Before each release
- When addressing specific technical debt items

## Notes

- Test code unwrap/expect is acceptable as tests should fail fast
- Safe fallbacks (e.g., `unwrap_or_else(|_| vec![])`) are acceptable for non-critical paths
- All critical error paths should use proper error handling

