# Base vs Experimental Builds - Implementation Complete

## ✅ All Changes Complete

### Summary
Successfully implemented base vs experimental build variants across both prerelease and production release workflows. The base variant is simply named "bllvm" (no suffix), while the experimental variant is clearly marked as "bllvm-experimental".

## ✅ Completed Updates

### 1. Build System (`build.sh`)
- ✅ Added `--variant` parameter (base or experimental)
- ✅ Base variant: `production` feature only
- ✅ Experimental variant: all experimental features (utxo-commitments, dandelion, ctv, stratum-v2, bip158, sigop)
- ✅ Uses `bllvm-commons` consistently (GitHub repo name)
- ✅ Variant-specific binary collection

### 2. Artifact Collection (`collect-artifacts.sh`)
- ✅ Variant parameter support
- ✅ Base: `binaries/` → `bllvm-{platform}.tar.gz`
- ✅ Experimental: `binaries-experimental/` → `bllvm-experimental-{platform}.tar.gz`
- ✅ Variant-specific checksums

### 3. Release Packaging (`create-release.sh`)
- ✅ Updated release notes for both variants
- ✅ Installation instructions for both
- ✅ Verification instructions for both

### 4. Workflows Updated
- ✅ `prerelease.yml` - Both variants
- ✅ `release_prod.yml` - Both variants  
- ✅ `release.yml` - Both variants

### 5. Naming Consistency
- ✅ All scripts use `bllvm-commons` (GitHub repo name)
- ✅ No references to `governance-app` in code
- ✅ Binary names match package name

## 📦 Release Artifacts

### Base Variant (Default)
- **Archive**: `bllvm-{version}-{platform}.tar.gz`
- **Checksums**: `SHA256SUMS-{platform}`
- **Features**: Production optimizations only
- **Use for**: Production deployments

### Experimental Variant
- **Archive**: `bllvm-experimental-{version}-{platform}.tar.gz`
- **Checksums**: `SHA256SUMS-experimental-{platform}`
- **Features**: All experimental features enabled
- **Use for**: Development, testing, advanced features

## 🎯 Feature Mapping

### Base Variant Features
- `production` - Performance optimizations (all repos)

### Experimental Variant Features
- `bllvm-consensus`: `production,utxo-commitments,ctv`
- `bllvm-protocol`: `production,utxo-commitments`
- `bllvm-node`: `production,utxo-commitments,dandelion,stratum-v2,bip158,sigop`
- `bllvm`: Inherits from bllvm-node
- `bllvm-sdk`: Default features
- `bllvm-commons`: Default features (no variants needed - it's a service)

## 📋 Files Changed

### Core Build System
- `build.sh` - Variant support
- `scripts/collect-artifacts.sh` - Variant support
- `scripts/create-release.sh` - Both variants

### Workflows
- `.github/workflows/prerelease.yml` - Both variants
- `.github/workflows/release_prod.yml` - Both variants
- `.github/workflows/release.yml` - Both variants

### Scripts (Naming)
- `scripts/generate-component-manifest.sh` - bllvm-commons
- `scripts/determine-build-requirements.sh` - bllvm-commons
- `scripts/setup-build-env.sh` - bllvm-commons
- `scripts/verify-versions.sh` - bllvm-commons
- `scripts/runner-status.sh` - bllvm-commons

## 🚀 Ready to Use

Both `prerelease.yml` and `release_prod.yml` are now ready to:
1. Build base and experimental variants
2. Collect artifacts for both
3. Create release packages for both
4. Publish both to GitHub releases

## 📝 Usage

### Local Development
```bash
# Build base variant
./build.sh --mode release --variant base

# Build experimental variant
./build.sh --mode release --variant experimental
```

### CI/CD
Both workflows now automatically build and release both variants.

