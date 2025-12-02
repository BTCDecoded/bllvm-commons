# Release Pipelines Analysis

## Current Release Workflows

### 1. `prerelease.yml`
**Purpose**: Create prerelease builds (alpha, beta, rc)
**Status**: ✅ Updated for base/experimental variants
**Key Features**:
- Builds both base and experimental variants
- Collects artifacts for both variants
- Creates prerelease on GitHub

### 2. `release_prod.yml`
**Purpose**: Create production releases with artifact reuse
**Status**: ⚠️ Needs update for base/experimental variants
**Key Features**:
- Determines which repos need building
- Downloads existing artifacts when possible
- Generates component manifests
- More sophisticated than prerelease.yml

### 3. `release.yml`
**Purpose**: Simple unified release (legacy?)
**Status**: ⚠️ Needs update for base/experimental variants
**Key Features**:
- Simpler than release_prod.yml
- No artifact reuse logic
- Similar to prerelease.yml but for production

### 4. `release_orchestrator.yml`
**Purpose**: Orchestrates the full build chain
**Status**: ✅ No changes needed (calls other workflows)
**Key Features**:
- Chains verification and build workflows
- Sends deployment signals

## Recommendations

### Option 1: Consolidate (Recommended)
- **Keep**: `prerelease.yml` and `release_prod.yml`
- **Deprecate**: `release.yml` (merge into release_prod.yml if needed)
- **Keep**: `release_orchestrator.yml` (orchestration only)

### Option 2: Keep All
- Update all three release workflows for base/experimental
- Document when to use each

## bllvm-commons (governance-app) Consistency

### Current State
- **Directory name**: `governance-app/` (legacy)
- **Package name**: `bllvm-commons` (current)
- **Binary name**: `bllvm-commons` (current)

### Feature Flags
- **No feature flags**: bllvm-commons is a service, not a node
- **No base/experimental variants needed**: It's just governance enforcement
- **Always builds the same**: No experimental features to enable/disable

### Build Script Updates Needed
- ✅ Updated `build.sh` to use `bllvm-commons` instead of `governance-app`
- ✅ Binary names already correct in `collect-artifacts.sh`
- ⚠️ Directory name is still `governance-app` (acceptable - just a path)

## Action Items

1. ✅ Update `prerelease.yml` for base/experimental (DONE)
2. ⚠️ Update `release_prod.yml` for base/experimental (IN PROGRESS)
3. ⚠️ Update `release.yml` for base/experimental (or deprecate)
4. ✅ Fix `build.sh` naming consistency (DONE)
5. ⚠️ Update `create-release.sh` to handle both variants
6. ✅ Document bllvm-commons doesn't need variants (DONE)

