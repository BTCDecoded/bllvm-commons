# Cross-Repo Release Pipeline Validation

**Date:** November 17, 2025  
**Status:** ⚠️ Issues Found - Needs Updates

---

## Current State Analysis

### ✅ What Exists

1. **Release Orchestrator** (`release_orchestrator.yml`)
   - Manual trigger only (`workflow_dispatch`)
   - Builds in dependency order:
     - consensus-proof (verify)
     - protocol-engine (build)
     - reference-node (build)
     - developer-sdk (build)
     - governance-app (docker image)
   - Sends deployment signal to governance-app

2. **Individual Build Workflows**
   - `build_lib_cached.yml` - Builds individual libraries
   - `build_docker.yml` - Builds Docker images
   - `prerelease.yml` - Creates prereleases
   - All run on `[self-hosted, Linux, X64]`

3. **Governance App CI**
   - Runs on push/PR to main/master
   - Tests, clippy, security, docs
   - Does NOT trigger orchestrator

### ❌ Missing Features

1. **No Automatic Triggers**
   - Builds in individual repos don't trigger downstream builds
   - No cascade from bllvm-consensus → bllvm-protocol → bllvm-node → bllvm → governance-app

2. **No Nightly Prerelease**
   - No scheduled workflow for nightly builds
   - No automatic prerelease creation

3. **No Cross-Repo Triggering**
   - Individual repo builds don't notify orchestrator
   - No `repository_dispatch` or `workflow_run` triggers

4. **Governance App Not Integrated**
   - governance-app CI doesn't trigger orchestrator
   - No automatic deployment on governance-app changes

---

## Required Changes

### 1. Add Nightly Schedule

**File:** `bllvm/.github/workflows/nightly-prerelease.yml` (NEW)

```yaml
name: Nightly Prerelease

on:
  schedule:
    # Run at 2 AM UTC daily
    - cron: '0 2 * * *'
  workflow_dispatch:  # Allow manual trigger

permissions:
  contents: write
  actions: read

jobs:
  create-nightly-prerelease:
    name: Create Nightly Prerelease
    runs-on: [self-hosted, Linux, X64]
    steps:
      - name: Generate nightly version tag
        id: version
        run: |
          DATE=$(date +%Y%m%d)
          COMMIT=$(git rev-parse --short HEAD)
          VERSION="nightly-${DATE}-${COMMIT}"
          echo "version=${VERSION}" >> $GITHUB_OUTPUT
      
      - name: Trigger prerelease
        uses: ./.github/workflows/prerelease.yml
        with:
          version_tag: ${{ steps.version.outputs.version }}
          platform: both
```

### 2. Add Cross-Repo Triggering

**Option A: Repository Dispatch (Recommended)**

When any repo builds successfully, dispatch to orchestrator:

**File:** `bllvm-consensus/.github/workflows/trigger-downstream.yml` (NEW)

```yaml
name: Trigger Downstream Builds

on:
  push:
    branches: [main, master]
  workflow_run:
    workflows: ["CI"]
    types: [completed]
    branches: [main, master]

jobs:
  trigger-orchestrator:
    if: github.event.workflow_run.conclusion == 'success'
    runs-on: ubuntu-latest
    steps:
      - name: Trigger orchestrator
        uses: actions/github-script@v7
        with:
          github-token: ${{ secrets.REPO_ACCESS_TOKEN || secrets.ORG_PAT }}
          script: |
            await github.rest.repos.createDispatchEvent({
              owner: 'BTCDecoded',
              repo: 'bllvm',
              event_type: 'build_consensus',
              client_payload: {
                ref: context.payload.workflow_run.head_branch,
                sha: context.payload.workflow_run.head_sha
              }
            })
```

**Option B: Workflow Run (Alternative)**

Listen for workflow completions in orchestrator:

**File:** `bllvm/.github/workflows/release_orchestrator.yml` (MODIFY)

```yaml
on:
  workflow_dispatch:
    # ... existing ...
  repository_dispatch:
    types: [build_consensus, build_protocol, build_node, build_sdk, build_governance]
  workflow_run:
    workflows: 
      - "CI"
      - "Build"
    types: [completed]
    branches: [main, master]
```

### 3. Add Cascading Build Logic

**File:** `bllvm/.github/workflows/release_orchestrator.yml` (MODIFY)

Add logic to determine which repos to rebuild based on trigger:

```yaml
jobs:
  determine-build-scope:
    runs-on: ubuntu-latest
    outputs:
      build_consensus: ${{ steps.scope.outputs.build_consensus }}
      build_protocol: ${{ steps.scope.outputs.build_protocol }}
      build_node: ${{ steps.scope.outputs.build_node }}
      build_sdk: ${{ steps.scope.outputs.build_sdk }}
      build_governance: ${{ steps.scope.outputs.build_governance }}
    steps:
      - name: Determine build scope
        id: scope
        run: |
          # If consensus changed, rebuild everything
          if [ "${{ github.event.action }}" = "build_consensus" ]; then
            echo "build_consensus=true" >> $GITHUB_OUTPUT
            echo "build_protocol=true" >> $GITHUB_OUTPUT
            echo "build_node=true" >> $GITHUB_OUTPUT
            echo "build_sdk=true" >> $GITHUB_OUTPUT
            echo "build_governance=true" >> $GITHUB_OUTPUT
          # If protocol changed, rebuild protocol and downstream
          elif [ "${{ github.event.action }}" = "build_protocol" ]; then
            echo "build_protocol=true" >> $GITHUB_OUTPUT
            echo "build_node=true" >> $GITHUB_OUTPUT
            echo "build_sdk=true" >> $GITHUB_OUTPUT
            echo "build_governance=true" >> $GITHUB_OUTPUT
          # ... etc
          fi
```

### 4. Integrate Governance App

**File:** `governance-app/.github/workflows/governance-app-ci.yml` (MODIFY)

Add step to trigger orchestrator on successful build:

```yaml
jobs:
  # ... existing jobs ...
  
  trigger-orchestrator:
    needs: [test, clippy, security]
    if: github.ref == 'refs/heads/main' || github.ref == 'refs/heads/master'
    runs-on: ubuntu-latest
    steps:
      - name: Trigger orchestrator
        uses: actions/github-script@v7
        with:
          github-token: ${{ secrets.REPO_ACCESS_TOKEN || secrets.ORG_PAT }}
          script: |
            await github.rest.repos.createDispatchEvent({
              owner: 'BTCDecoded',
              repo: 'bllvm',
              event_type: 'build_governance',
              client_payload: {
                ref: context.ref,
                sha: context.sha
              }
            })
```

### 5. Add Prerelease Trigger

**File:** `bllvm/.github/workflows/release_orchestrator.yml` (MODIFY)

After successful build, trigger prerelease:

```yaml
jobs:
  # ... existing build jobs ...
  
  trigger-prerelease:
    needs: [build-governance-app-image]
    if: success()
    runs-on: [self-hosted, Linux, X64]
    steps:
      - name: Generate prerelease version
        id: version
        run: |
          DATE=$(date +%Y%m%d)
          COMMIT=$(git rev-parse --short HEAD)
          VERSION="nightly-${DATE}-${COMMIT}"
          echo "version=${VERSION}" >> $GITHUB_OUTPUT
      
      - name: Trigger prerelease
        uses: ./.github/workflows/prerelease.yml
        with:
          version_tag: ${{ steps.version.outputs.version }}
          platform: both
```

---

## Proposed Architecture

### Trigger Flow

```
Any Repo Push/PR Merge
    ↓
Individual Repo CI (bllvm-consensus, bllvm-protocol, etc.)
    ↓ (on success)
Repository Dispatch → bllvm orchestrator
    ↓
Orchestrator determines scope
    ↓
Build downstream repos
    ↓
Build governance-app
    ↓
Trigger prerelease
    ↓
Create nightly prerelease
```

### Nightly Flow

```
Cron Schedule (2 AM UTC)
    ↓
Nightly Prerelease Workflow
    ↓
Trigger Orchestrator
    ↓
Build all repos (latest main)
    ↓
Create prerelease
```

---

## Implementation Checklist

### Phase 1: Nightly Prerelease
- [ ] Create `nightly-prerelease.yml` workflow
- [ ] Add cron schedule
- [ ] Test manual trigger

### Phase 2: Cross-Repo Triggering
- [ ] Add `repository_dispatch` to orchestrator
- [ ] Add trigger workflows to individual repos
- [ ] Test cascade from consensus → protocol → node → sdk → governance

### Phase 3: Governance App Integration
- [ ] Add orchestrator trigger to governance-app CI
- [ ] Test governance-app build → orchestrator → prerelease

### Phase 4: Cascading Logic
- [ ] Add build scope determination
- [ ] Only rebuild downstream repos
- [ ] Optimize build times

---

## Validation Questions

1. **Should every push trigger a full build?**
   - Current: No automatic triggers
   - Proposed: Yes, but only rebuild downstream

2. **Should nightly builds use latest main or specific versions?**
   - Current: Uses versions.toml
   - Proposed: Use latest main for nightly, versions.toml for releases

3. **Should governance-app changes trigger full stack rebuild?**
   - Current: No
   - Proposed: Yes, rebuild governance-app only (it's at the end)

4. **Should failed builds block downstream?**
   - Current: Uses `needs:` (yes)
   - Proposed: Keep this behavior

---

## Next Steps

1. **Review this validation** with team
2. **Decide on trigger strategy** (repository_dispatch vs workflow_run)
3. **Implement nightly prerelease** first (simplest)
4. **Add cross-repo triggering** incrementally
5. **Test cascade** from each repo
6. **Monitor build times** and optimize

---

**Status:** ⚠️ Needs Implementation  
**Priority:** High (Core CI/CD functionality)

