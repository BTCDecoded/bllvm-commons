# Build System Updates - Final Status

## ✅ Completed

### 1. Build Script (`build.sh`)
- ✅ Added `--variant` parameter (base or experimental)
- ✅ Base variant: `production` feature only
- ✅ Experimental variant: all experimental features
- ✅ Uses `bllvm-commons` consistently (not governance-app)
- ✅ Variant-specific binary collection directories

### 2. Artifact Collection (`collect-artifacts.sh`)
- ✅ Added variant parameter support
- ✅ Base: `binaries/` and `SHA256SUMS-{platform}`
- ✅ Experimental: `binaries-experimental/` and `SHA256SUMS-experimental-{platform}`
- ✅ Archive naming: `bllvm-{platform}` vs `bllvm-experimental-{platform}`

### 3. Release Packaging (`create-release.sh`)
- ✅ Updated release notes to document both variants
- ✅ Installation instructions for both variants
- ✅ Verification instructions for both variants

### 4. Prerelease Workflow (`prerelease.yml`)
- ✅ Builds both base and experimental variants
- ✅ Collects artifacts for both variants
- ✅ Validates both variants
- ✅ Updated Windows cross-compilation for both variants

### 5. Production Release Workflow (`release_prod.yml`)
- ✅ Builds both base and experimental variants
- ✅ Collects artifacts for both variants
- ✅ Validates both variants
- ✅ Updated Windows cross-compilation for both variants
- ✅ Updated GitHub release to include both archive types

### 6. Naming Consistency
- ✅ All scripts use `bllvm-commons` (not `governance-app`)
- ✅ GitHub repo references use `bllvm-commons`
- ✅ Binary names match package name (`bllvm-commons`)

## 📋 Release Pipeline Status

### Workflows Updated
1. **`prerelease.yml`** - ✅ Complete
2. **`release_prod.yml`** - ✅ Complete

### Workflows Not Updated
3. **`release.yml`** - ⚠️ Simple release workflow (consider updating or deprecating)
4. **`release_orchestrator.yml`** - ✅ No changes needed (orchestration only)

## 🎯 Feature Mapping

### Base Variant
- **Features**: `production` only
- **Binary**: `bllvm-{version}-{platform}.tar.gz`
- **Use for**: Production deployments, stability priority

### Experimental Variant
- **Features**: `production,utxo-commitments,dandelion,ctv,stratum-v2,bip158,sigop`
- **Binary**: `bllvm-experimental-{version}-{platform}.tar.gz`
- **Use for**: Development, testing, advanced features

### bllvm-commons
- **No variants**: Service, not a node
- **Always builds the same**: No feature flags
- **Included in both**: Base and experimental releases

## 📝 Next Steps

1. ⚠️ Update `release.yml` or document deprecation
2. ✅ Test both variants in CI
3. ✅ Update documentation

## 🔍 Files Changed

### Core Build System
- `build.sh` - Added variant support
- `scripts/collect-artifacts.sh` - Added variant support
- `scripts/create-release.sh` - Updated for both variants

### Workflows
- `.github/workflows/prerelease.yml` - Both variants
- `.github/workflows/release_prod.yml` - Both variants

### Scripts (Naming Consistency)
- `scripts/generate-component-manifest.sh` - bllvm-commons
- `scripts/determine-build-requirements.sh` - bllvm-commons
- `scripts/setup-build-env.sh` - bllvm-commons
- `scripts/verify-versions.sh` - bllvm-commons
- `scripts/runner-status.sh` - bllvm-commons

## ✅ Ready for Testing

Both `prerelease.yml` and `release_prod.yml` are now ready to:
- Build base and experimental variants
- Collect artifacts for both
- Create release packages for both
- Publish both to GitHub releases

