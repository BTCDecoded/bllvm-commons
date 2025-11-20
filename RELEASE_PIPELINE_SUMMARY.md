# Release Pipeline Updates Summary

**Date:** November 17, 2025  
**Status:** ✅ Updates Complete

---

## ✅ Changes Made

### 1. Nightly Prerelease Workflow

**File:** `bllvm/.github/workflows/nightly-prerelease.yml` (NEW)

- **Schedule:** Runs daily at 2 AM UTC
- **Manual Trigger:** Available via workflow_dispatch
- **Action:** Triggers orchestrator with `build_all` event type
- **Result:** Orchestrator builds all repos and creates prerelease automatically

### 2. Orchestrator Enhancements

**File:** `bllvm/.github/workflows/release_orchestrator.yml` (MODIFIED)

**Added Triggers:**
- `repository_dispatch` - For cross-repo triggering
  - Types: `build_consensus`, `build_protocol`, `build_node`, `build_sdk`, `build_governance`, `build_all`
- `workflow_run` - For workflow completion triggers
  - Watches: `CI`, `Build`, `Release` workflows
  - Branches: `main`, `master`

**Added Job:**
- `trigger-prerelease` - Automatically creates prerelease after successful build
  - Generates nightly version tag: `nightly-YYYYMMDD-COMMIT`
  - Triggers `prerelease.yml` workflow
  - Runs after `build-governance-app-image` completes

### 3. Governance App Integration

**File:** `governance-app/.github/workflows/governance-app-ci.yml` (MODIFIED)

**Added Job:**
- `trigger-orchestrator` - Triggers orchestrator on successful push to main/master
  - Runs after `test`, `clippy`, `security` jobs
  - Sends `build_governance` event to orchestrator
  - Includes ref, sha, and repo info in payload

---

## 🔄 Pipeline Flow

### Nightly Flow
```
Cron Schedule (2 AM UTC)
    ↓
nightly-prerelease.yml
    ↓
Trigger orchestrator (build_all)
    ↓
Orchestrator builds all repos
    ↓
trigger-prerelease job
    ↓
Create prerelease (nightly-YYYYMMDD-COMMIT)
```

### Cross-Repo Flow
```
Any Repo Push/PR Merge
    ↓
Individual Repo CI
    ↓ (on success)
Repository Dispatch → orchestrator
    ↓
Orchestrator determines scope
    ↓
Build downstream repos
    ↓
Build governance-app
    ↓
trigger-prerelease job
    ↓
Create prerelease
```

### Governance App Flow
```
governance-app push to main
    ↓
governance-app CI (test, clippy, security)
    ↓ (on success)
trigger-orchestrator job
    ↓
Repository Dispatch → orchestrator (build_governance)
    ↓
Orchestrator builds governance-app
    ↓
trigger-prerelease job
    ↓
Create prerelease
```

---

## 📋 Next Steps

### For Individual Repos

To enable cross-repo triggering, add this to each repo's CI workflow:

```yaml
trigger-orchestrator:
  needs: [test, build]  # Adjust based on your jobs
  if: |
    (github.ref == 'refs/heads/main' || github.ref == 'refs/heads/master') &&
    github.event_name == 'push' &&
    always()
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
            event_type: 'build_consensus',  # Change per repo
            client_payload: {
              ref: context.ref,
              sha: context.sha,
              repo: 'bllvm-consensus'  # Change per repo
            }
          })
```

**Repos to update:**
- `bllvm-consensus` → `build_consensus`
- `bllvm-protocol` → `build_protocol`
- `bllvm-node` → `build_node`
- `bllvm-sdk` → `build_sdk`
- `governance-app` → `build_governance` (✅ Already done)

---

## ✅ Validation

### What Works Now

1. ✅ **Nightly Prereleases** - Automatic daily builds
2. ✅ **Governance App Integration** - Triggers orchestrator on push
3. ✅ **Automatic Prerelease** - Created after orchestrator completes
4. ✅ **Cross-Repo Triggering** - Ready for individual repos

### What Needs Individual Repo Updates

1. ⏳ **Individual Repo Triggers** - Need to add trigger jobs to each repo
2. ⏳ **Cascading Logic** - Orchestrator currently builds all repos (could optimize)

---

## 🔧 Configuration

### Required Secrets

- `REPO_ACCESS_TOKEN` or `ORG_PAT` - For cross-repo triggering
- `GITHUB_TOKEN` - Default (may have limited permissions)

### Self-Hosted Runner

All workflows run on: `[self-hosted, Linux, X64]`

---

**Status:** ✅ Core Pipeline Updated  
**Ready for:** Individual repo integration
