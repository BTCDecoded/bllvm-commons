# Build System Updates - Complete Summary

## ✅ Completed Changes

### 1. Build Script (`build.sh`)
- ✅ Added `--variant` parameter (base or experimental)
- ✅ Base variant: `production` feature only
- ✅ Experimental variant: all experimental features
- ✅ Updated to use `bllvm-commons` instead of `governance-app`
- ✅ Variant-specific binary collection directories

### 2. Artifact Collection (`collect-artifacts.sh`)
- ✅ Added variant parameter support
- ✅ Base: `binaries/` and `SHA256SUMS-{platform}`
- ✅ Experimental: `binaries-experimental/` and `SHA256SUMS-experimental-{platform}`
- ✅ Archive naming: `bllvm-{platform}` vs `bllvm-experimental-{platform}`

### 3. Prerelease Workflow (`prerelease.yml`)
- ✅ Builds both base and experimental variants
- ✅ Collects artifacts for both variants
- ✅ Validates both variants
- ✅ Updated Windows cross-compilation for both variants

### 4. Naming Consistency
- ✅ Updated `build.sh` to use `bllvm-commons` consistently
- ✅ Binary names match package name (`bllvm-commons`)
- ⚠️ Directory name is still `governance-app` (acceptable - just a path)

## ⚠️ Remaining Work

### 5. Production Release Workflow (`release_prod.yml`)
**Status**: Needs update for base/experimental variants

**Changes Needed**:
- Split build steps for base and experimental variants
- Update artifact collection for both variants
- Update validation for both variants
- Update Windows cross-compilation for both variants

### 6. Simple Release Workflow (`release.yml`)
**Status**: Needs update or deprecation decision

**Options**:
- Update for base/experimental variants (similar to prerelease.yml)
- Deprecate in favor of `release_prod.yml`
- Document when to use each

### 7. Release Packaging (`create-release.sh`)
**Status**: Needs update to handle both variants

**Changes Needed**:
- Create release packages for both variants
- Update release notes to document both variants
- Include both archive types in GitHub release

## 📋 Release Pipeline Analysis

### Current Workflows
1. **`prerelease.yml`** - ✅ Updated
   - Prerelease builds (alpha, beta, rc)
   - Both base and experimental variants

2. **`release_prod.yml`** - ⚠️ Needs update
   - Production releases with artifact reuse
   - More sophisticated than prerelease
   - Needs base/experimental support

3. **`release.yml`** - ⚠️ Needs update or deprecation
   - Simple unified release
   - Similar to prerelease but for production
   - Consider consolidating with release_prod.yml

4. **`release_orchestrator.yml`** - ✅ No changes needed
   - Orchestrates build chain
   - Calls other workflows

## 🔧 bllvm-commons (governance-app) Status

### Naming
- **Directory**: `governance-app/` (legacy, acceptable)
- **Package**: `bllvm-commons` (current)
- **Binary**: `bllvm-commons` (current)
- **Build Script**: ✅ Updated to use `bllvm-commons`

### Feature Variants
- **No variants needed**: bllvm-commons is a service, not a node
- **No experimental features**: It's just governance enforcement
- **Always builds the same**: No feature flags to enable/disable

## 📝 Next Steps

1. Update `release_prod.yml` for base/experimental variants
2. Update `release.yml` or document deprecation
3. Update `create-release.sh` to package both variants
4. Test both variants in CI
5. Update documentation

## 🎯 Feature Mapping Summary

### Base Variant
- `production` feature only
- Stable, minimal features
- Binary: `bllvm-{version}-{platform}.tar.gz`

### Experimental Variant
- All features: `production,utxo-commitments,dandelion,ctv,stratum-v2,bip158,sigop`
- Full-featured build
- Binary: `bllvm-experimental-{version}-{platform}.tar.gz`

### bllvm-commons
- No variants (service, not node)
- Always builds the same
- Included in both base and experimental releases

