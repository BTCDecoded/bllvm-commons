# Governance App (bllvm-commons) Integration Validation

**Date:** November 17, 2025  
**Status:** ⚠️ Issues Found - Needs Fixes

---

## Current State Analysis

### ✅ What's Working

1. **Governance App CI Integration**
   - ✅ `trigger-orchestrator` job added to `governance-app-ci.yml`
   - ✅ Triggers orchestrator on successful push to main/master
   - ✅ Sends `build_governance` event type
   - ✅ Includes ref, sha, and repo info

2. **Orchestrator Builds Governance App**
   - ✅ `build-governance-app-image` job exists
   - ✅ Builds Docker image for governance-app
   - ✅ Uses version from `versions.toml` (reads `governance-app` key)
   - ✅ Runs after `build-developer-sdk` (correct dependency order)

3. **Prerelease Integration**
   - ✅ `trigger-prerelease` job runs after governance-app build
   - ✅ Creates prerelease automatically
   - ✅ Generates nightly version tag

4. **Deployment Signal**
   - ✅ Sends deployment signal to governance-app
   - ✅ Includes tag and image info

### ❌ Issues Found

1. **Repository Name Mismatch**
   - **Problem:** Orchestrator references `governance-app` but repo is `bllvm-commons`
   - **Location:** `release_orchestrator.yml` line 103, 143
   - **Impact:** Build will fail if repo name doesn't match

2. **Version Key Mismatch**
   - **Problem:** `versions.toml` uses `governance-app` key but repo is `bllvm-commons`
   - **Location:** `versions.toml` line 19, `release_orchestrator.yml` line 43
   - **Impact:** Version lookup will fail

3. **Event Type Consistency**
   - **Problem:** Governance app sends `build_governance` but orchestrator might expect different name
   - **Location:** `governance-app-ci.yml` line 183
   - **Impact:** Orchestrator might not trigger correctly

4. **Missing Repository Dispatch Handler**
   - **Problem:** Orchestrator accepts `repository_dispatch` but doesn't handle `build_governance` event
   - **Location:** `release_orchestrator.yml` - no conditional logic for event types
   - **Impact:** All dispatches trigger full build (inefficient)

---

## Required Fixes

### Fix 1: Update Repository Name

**File:** `bllvm/.github/workflows/release_orchestrator.yml`

**Change:**
```yaml
# Line 103
repo: governance-app  # Change to: bllvm-commons

# Line 143
repo: 'governance-app'  # Change to: 'bllvm-commons'
```

**Also update:**
- `build_docker.yml` default (if applicable)
- Any other references

### Fix 2: Update Version Key

**Option A: Keep `governance-app` key in versions.toml** (Recommended)
- Update orchestrator to map `governance-app` → `bllvm-commons` repo
- Keeps backward compatibility

**Option B: Change versions.toml key to `bllvm-commons`**
- Update orchestrator parsing
- More consistent naming

**Recommended:** Option A (map in orchestrator)

### Fix 3: Add Event Type Handling

**File:** `bllvm/.github/workflows/release_orchestrator.yml`

**Add conditional logic:**
```yaml
jobs:
  determine-scope:
    runs-on: ubuntu-latest
    outputs:
      build_all: ${{ steps.scope.outputs.build_all }}
      build_governance_only: ${{ steps.scope.outputs.build_governance_only }}
    steps:
      - name: Determine build scope
        id: scope
        run: |
          if [ "${{ github.event_name }}" = "repository_dispatch" ]; then
            case "${{ github.event.action }}" in
              build_governance)
                echo "build_governance_only=true" >> $GITHUB_OUTPUT
                ;;
              build_all)
                echo "build_all=true" >> $GITHUB_OUTPUT
                ;;
            esac
          else
            echo "build_all=true" >> $GITHUB_OUTPUT
          fi

  # Then conditionally run jobs based on scope
  build-governance-app-image:
    needs: [read-versions, determine-scope]
    if: |
      needs.determine-scope.outputs.build_all == 'true' ||
      needs.determine-scope.outputs.build_governance_only == 'true'
    # ... rest of job
```

### Fix 4: Update Version Parsing

**File:** `bllvm/.github/workflows/release_orchestrator.yml`

**Update parsing to handle both names:**
```yaml
- name: Parse versions.toml
  id: parse
  run: |
    # Try governance-app first, fallback to bllvm-commons
    GA=$(grep -E '^(governance-app|bllvm-commons)' versions.toml | head -1 | awk -F '="' '{print $2}' | tr -d '"')
    if [ -z "$GA" ]; then
      echo "⚠️  No governance-app or bllvm-commons version found"
      GA="main"  # Fallback to main
    fi
    echo "ga=$GA" >> $GITHUB_OUTPUT
```

---

## Integration Flow Validation

### Current Flow (After Fixes)

```
governance-app (bllvm-commons) push to main
    ↓
governance-app-ci.yml runs
    ↓
test, clippy, security jobs
    ↓ (on success)
trigger-orchestrator job
    ↓
Repository Dispatch: build_governance → bllvm orchestrator
    ↓
Orchestrator reads versions.toml
    ↓
determine-scope: build_governance_only = true
    ↓
Skip: verify-consensus, build-protocol-engine, build-reference-node, build-developer-sdk
    ↓
build-governance-app-image (only governance-app)
    ↓
trigger-prerelease
    ↓
Create prerelease
    ↓
deploy-signal → governance-app
```

### Nightly Flow

```
Cron (2 AM UTC)
    ↓
nightly-prerelease.yml
    ↓
Repository Dispatch: build_all → orchestrator
    ↓
determine-scope: build_all = true
    ↓
Build all repos (consensus → protocol → node → sdk → governance)
    ↓
trigger-prerelease
    ↓
Create prerelease
```

---

## Validation Checklist

### ✅ Integration Points

- [x] Governance app CI triggers orchestrator
- [x] Orchestrator builds governance app
- [x] Prerelease created after build
- [x] Deployment signal sent
- [ ] Repository name matches (needs fix)
- [ ] Version key matches (needs fix)
- [ ] Event type handling (needs fix)

### ✅ Workflow Dependencies

- [x] Governance app builds after developer-sdk
- [x] Prerelease triggers after governance app build
- [x] Deployment signal after prerelease
- [x] All run on self-hosted runner

### ✅ Configuration

- [x] Versions.toml has governance-app entry
- [x] Orchestrator reads versions.toml
- [x] Docker image name configured
- [ ] Repository name consistency (needs fix)

---

## Recommended Actions

### Immediate Fixes

1. **Update repository name** in orchestrator to `bllvm-commons`
2. **Add version key mapping** (governance-app → bllvm-commons)
3. **Add event type handling** for conditional builds
4. **Test integration** with actual push

### Testing

1. Push to governance-app main branch
2. Verify orchestrator triggers
3. Verify only governance-app builds (not full stack)
4. Verify prerelease created
5. Verify deployment signal sent

---

## Summary

**Status:** ⚠️ Partially Integrated - Needs Fixes

**Working:**
- ✅ Governance app CI integration
- ✅ Orchestrator builds governance app
- ✅ Prerelease creation
- ✅ Deployment signals

**Needs Fixes:**
- ❌ Repository name mismatch (governance-app vs bllvm-commons)
- ❌ Version key mapping
- ❌ Event type conditional logic
- ❌ Efficient build scope (currently builds all repos)

**Priority:** High - Core integration functionality

