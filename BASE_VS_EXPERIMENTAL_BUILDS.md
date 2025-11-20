# Base vs Experimental Builds Plan

## Overview

This document outlines the plan to differentiate between "base" and "experimental" builds for both the prerelease and production release workflows.

## Build Variants

### Base Build
**Purpose**: Minimal, stable release with core functionality only

**Features Included:**
- Core `bllvm` binary
- Production optimizations (`production` feature)
- Standard storage backends (`redb`, `sysinfo`)
- Process sandboxing (`nix`, `libc`)

**Features Excluded:**
- ❌ `utxo-commitments` - Experimental UTXO commitment system
- ❌ `dandelion` - Privacy-preserving transaction relay
- ❌ `ctv` - BIP119 CheckTemplateVerify (proposed soft fork)
- ❌ `stratum-v2` - Stratum V2 mining protocol
- ❌ `bip158` - Compact block filters
- ❌ `sigop` - Signature operations counting

**Binary Naming:**
- `bllvm-{version}-{platform}.tar.gz`
- `bllvm-{version}-{platform}.zip` (Windows)

### Experimental Build
**Purpose**: Full-featured build with all experimental features enabled

**Features Included:**
- All base features
- ✅ `utxo-commitments` - UTXO commitment system
- ✅ `dandelion` - Dandelion++ privacy relay
- ✅ `ctv` - BIP119 CheckTemplateVerify
- ✅ `stratum-v2` - Stratum V2 mining
- ✅ `bip158` - Compact block filters
- ✅ `sigop` - Signature operations counting

**Binary Naming:**
- `bllvm-experimental-{version}-{platform}.tar.gz`
- `bllvm-experimental-{version}-{platform}.zip` (Windows)

## Feature Mapping

### bllvm-consensus
- `production` - ✅ Base + Experimental
- `utxo-commitments` - ❌ Base, ✅ Experimental
- `ctv` - ❌ Base, ✅ Experimental

### bllvm-protocol
- `production` - ✅ Base + Experimental
- `utxo-commitments` - ❌ Base, ✅ Experimental (passed through)

### bllvm-node
- `production` - ✅ Base + Experimental
- `utxo-commitments` - ❌ Base, ✅ Experimental
- `dandelion` - ❌ Base, ✅ Experimental
- `stratum-v2` - ❌ Base, ✅ Experimental
- `bip158` - ❌ Base, ✅ Experimental
- `sigop` - ❌ Base, ✅ Experimental

### bllvm (binary)
- Inherits features from bllvm-node

## Implementation Plan

### 1. Update Build Scripts

#### `build.sh`
- Add `--variant` parameter: `base` or `experimental`
- Build with appropriate feature flags based on variant
- Default to `base` for safety

#### Feature Flag Logic
```bash
# Base build
BASE_FEATURES="production"

# Experimental build
EXPERIMENTAL_FEATURES="production,utxo-commitments,dandelion,ctv,stratum-v2,bip158,sigop"
```

### 2. Update Workflows

#### `prerelease.yml`
- Build both variants in parallel jobs
- Collect artifacts separately
- Create separate release packages

#### `release_prod.yml`
- Build both variants in parallel jobs
- Collect artifacts separately
- Create separate release packages

### 3. Update Artifact Collection

#### `collect-artifacts.sh`
- Accept variant parameter (`base` or `experimental`)
- Use variant-specific directory names:
  - `binaries/` and `binaries-windows/` (base variant)
  - `binaries-experimental/` and `binaries-experimental-windows/` (experimental variant)
- Generate variant-specific checksums:
  - `SHA256SUMS-{platform}` (base variant)
  - `SHA256SUMS-experimental-{platform}` (experimental variant)

### 4. Update Release Packaging

#### `create-release.sh`
- Handle both variants
- Create separate archives:
  - `bllvm-{version}-{platform}.tar.gz` (base variant)
  - `bllvm-experimental-{version}-{platform}.tar.gz` (experimental variant)
- Update release notes to document both variants

### 5. Release Notes Updates

Document both variants clearly:
- Base: Stable, minimal features
- Experimental: All features, including proposed soft forks

## Build Command Examples

### Local Development
```bash
# Build base variant
./build.sh --mode release --variant base

# Build experimental variant
./build.sh --mode release --variant experimental
```

### CI/CD
```yaml
# In workflow
- name: Build base variant
  run: commons/build.sh --mode release --variant base

- name: Build experimental variant
  run: commons/build.sh --mode release --variant experimental
```

## Artifact Structure

```
artifacts/
├── binaries/
│   ├── bllvm
│   ├── bllvm-keygen
│   ├── bllvm-sign
│   └── bllvm-verify
├── binaries-windows/
│   ├── bllvm.exe
│   └── ...
├── binaries-experimental/
│   ├── bllvm
│   └── ...
├── binaries-experimental-windows/
│   ├── bllvm.exe
│   └── ...
├── SHA256SUMS-linux-x86_64
├── SHA256SUMS-windows-x86_64
├── SHA256SUMS-experimental-linux-x86_64
├── SHA256SUMS-experimental-windows-x86_64
├── bllvm-{version}-linux-x86_64.tar.gz
├── bllvm-{version}-windows-x86_64.zip
├── bllvm-experimental-{version}-linux-x86_64.tar.gz
└── bllvm-experimental-{version}-windows-x86_64.zip
```

## Testing Strategy

1. **Base Build Tests**
   - Verify core functionality works
   - Verify experimental features are disabled
   - Test that experimental feature flags fail gracefully

2. **Experimental Build Tests**
   - Verify all features are enabled
   - Test experimental feature functionality
   - Verify feature interactions

3. **Integration Tests**
   - Test both variants can run on same network
   - Verify feature compatibility

## Migration Notes

- Existing releases will continue to work
- New releases will include both variants
- Users can choose based on their needs:
  - **Base**: Production use, stability priority
  - **Experimental**: Development, testing, advanced features

## Future Considerations

- Consider adding more granular feature sets (e.g., "privacy" variant with just dandelion)
- Consider feature flags at runtime (some features already support this)
- Consider deprecating experimental features as they mature

