# Build System Updates Summary

## Changes Made

### 1. Build Script (`build.sh`)
- ✅ Added `--variant` parameter (base or experimental)
- ✅ Base variant: builds with `production` feature only
- ✅ Experimental variant: builds with all experimental features
- ✅ Variant-specific binary collection directories:
  - Base: `artifacts/binaries/`
  - Experimental: `artifacts/binaries-experimental/`

### 2. Artifact Collection (`collect-artifacts.sh`)
- ✅ Added variant parameter support
- ✅ Variant-specific directory handling:
  - Base: `binaries/` and `binaries-windows/`
  - Experimental: `binaries-experimental/` and `binaries-experimental-windows/`
- ✅ Variant-specific checksum files:
  - Base: `SHA256SUMS-{platform}`
  - Experimental: `SHA256SUMS-experimental-{platform}`
- ✅ Variant-specific archive names:
  - Base: `bllvm-{platform}.tar.gz` / `bllvm-{platform}.zip`
  - Experimental: `bllvm-experimental-{platform}.tar.gz` / `bllvm-experimental-{platform}.zip`

### 3. Feature Mapping

#### Base Variant Features
- `production` - Performance optimizations (all repos)

#### Experimental Variant Features
- `bllvm-consensus`: `production,utxo-commitments,ctv`
- `bllvm-protocol`: `production,utxo-commitments`
- `bllvm-node`: `production,utxo-commitments,dandelion,stratum-v2,bip158,sigop`
- `bllvm`: Inherits from bllvm-node
- `bllvm-sdk`: Default features
- `governance-app`: Default features

## Next Steps

### 4. Update Workflows

#### `prerelease.yml`
- Build base variant (Linux + Windows)
- Build experimental variant (Linux + Windows)
- Collect artifacts for both variants
- Create release packages for both variants

#### `release_prod.yml`
- Same as prerelease.yml but for production releases

### 5. Update Release Packaging

#### `create-release.sh`
- Handle both variants when creating release packages
- Update release notes to document both variants
- Include both archive types in GitHub release

## Usage Examples

### Local Development
```bash
# Build base variant
./build.sh --mode release --variant base

# Build experimental variant
./build.sh --mode release --variant experimental
```

### Artifact Collection
```bash
# Collect base artifacts
./scripts/collect-artifacts.sh linux-x86_64 base

# Collect experimental artifacts
./scripts/collect-artifacts.sh linux-x86_64 experimental
```

## File Naming Convention

### Base Variant (Default)
- Binaries: `binaries/` and `binaries-windows/`
- Checksums: `SHA256SUMS-{platform}`
- Archives: `bllvm-{version}-{platform}.tar.gz`

### Experimental Variant
- Binaries: `binaries-experimental/` and `binaries-experimental-windows/`
- Checksums: `SHA256SUMS-experimental-{platform}`
- Archives: `bllvm-experimental-{version}-{platform}.tar.gz`

## Testing Checklist

- [ ] Base variant builds successfully
- [ ] Experimental variant builds successfully
- [ ] Base variant binaries work correctly
- [ ] Experimental variant binaries work correctly
- [ ] Artifact collection works for both variants
- [ ] Release packaging includes both variants
- [ ] GitHub release includes both archive types

