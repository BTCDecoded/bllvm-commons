#!/bin/bash
# Script to create all 47 community issues via GitHub API
# Usage: GITHUB_TOKEN=your_token ./create_all_issues.sh

set -e

GITHUB_TOKEN="${GITHUB_TOKEN:-}"
if [ -z "$GITHUB_TOKEN" ]; then
    echo "Error: GITHUB_TOKEN environment variable not set"
    exit 1
fi

API_BASE="https://api.github.com/repos/BTCDecoded"
HEADERS=(-H "Authorization: token $GITHUB_TOKEN" -H "Accept: application/vnd.github.v3+json" -H "Content-Type: application/json")

create_issue() {
    local repo=$1
    local title=$2
    local body=$3
    local labels=$4
    
    local json=$(cat <<EOF
{
  "title": "$title",
  "body": $(echo "$body" | jq -Rs .),
  "labels": $(echo "$labels" | jq -c .)
}
EOF
)
    
    curl -s -X POST "${API_BASE}/${repo}/issues" \
        "${HEADERS[@]}" \
        -d "$json" | jq -r '"✅ Issue #\(.number): \(.title) - \(.html_url)"'
}

# bllvm-consensus issues (continuing from #4)
echo "Creating bllvm-consensus issues..."

create_issue "bllvm-consensus" \
    "Improve error messages" \
    "## Description

Make consensus validation error messages more descriptive and helpful.

## Context

Clear error messages help developers understand what went wrong and how to fix it. Our consensus validation errors could be more descriptive.

## Acceptance Criteria

- [ ] Review existing error messages in consensus validation
- [ ] Improve error messages to be more descriptive
- [ ] Add context about what validation failed
- [ ] Ensure error messages are user-friendly

## Technical Details

- **Files to modify**: \`src/**/*.rs\`
- **Dependencies**: None
- **References**: See existing error types

## Skills Required

- Rust
- UX design
- Error handling

## Difficulty

- [x] Good First Issue
- [ ] Intermediate
- [ ] Advanced

## Priority

- [ ] P0 (Critical - Blocks production)
- [ ] P1 (High - Important feature)
- [ ] P2 (Medium - Nice to have)
- [x] P3 (Low - Future enhancement)

## Getting Started

1. Search for error messages in consensus code
2. Identify messages that could be clearer
3. Improve error messages with more context
4. Test error messages appear correctly
5. Submit PR with improvements" \
    '["good-first-issue", "type:enhancement", "area:consensus", "priority:low"]'

create_issue "bllvm-consensus" \
    "Implement missing Kani proofs" \
    "## Description

Add formal verification proofs for uncovered consensus rules.

## Context

We have 176+ Kani proofs, but there are still consensus rules that could benefit from formal verification. Adding more proofs increases confidence in correctness.

## Acceptance Criteria

- [ ] Identify consensus rules without Kani proofs
- [ ] Implement Kani proofs for identified rules
- [ ] Verify proofs pass
- [ ] Document the proofs

## Technical Details

- **Files to modify**: \`src/**/*.rs\` (add \`#[cfg(kani)]\` blocks)
- **Dependencies**: Kani (see docs for setup)
- **References**: \`docs/FORMAL_VERIFICATION_PLAN.md\`

## Skills Required

- Formal verification
- Kani model checker
- Rust
- Bitcoin consensus rules

## Difficulty

- [ ] Good First Issue
- [x] Intermediate
- [ ] Advanced

## Priority

- [ ] P0 (Critical - Blocks production)
- [x] P1 (High - Important feature)
- [ ] P2 (Medium - Nice to have)
- [ ] P3 (Low - Future enhancement)

## Getting Started

1. Review \`docs/FORMAL_VERIFICATION_PLAN.md\`
2. Identify consensus rules needing proofs
3. Study existing Kani proofs for patterns
4. Implement new proofs
5. Run: \`cargo kani --features verify\`
6. Submit PR with new proofs" \
    '["intermediate", "type:feature", "area:consensus", "priority:high"]'

create_issue "bllvm-consensus" \
    "Add fuzzing targets" \
    "## Description

Create new fuzzing targets for consensus validation.

## Context

Fuzzing helps discover bugs by testing with random inputs. We have some fuzzing targets but could expand coverage.

## Acceptance Criteria

- [ ] Identify areas needing fuzzing coverage
- [ ] Create new fuzzing targets
- [ ] Ensure fuzzing targets are well-structured
- [ ] Document fuzzing targets

## Technical Details

- **Files to modify**: \`fuzz/fuzz_targets/\`
- **Dependencies**: libfuzzer (see fuzzing setup)
- **References**: See existing fuzzing targets

## Skills Required

- Fuzzing
- Rust
- Bitcoin protocol

## Difficulty

- [ ] Good First Issue
- [x] Intermediate
- [ ] Advanced

## Priority

- [ ] P0 (Critical - Blocks production)
- [ ] P1 (High - Important feature)
- [x] P2 (Medium - Nice to have)
- [ ] P3 (Low - Future enhancement)

## Getting Started

1. Review existing fuzzing targets in \`fuzz/fuzz_targets/\`
2. Identify consensus functions that would benefit from fuzzing
3. Create new fuzzing targets
4. Run fuzzing: \`cargo fuzz run <target>\`
5. Submit PR with new fuzzing targets" \
    '["intermediate", "type:testing", "area:consensus", "priority:medium"]'

create_issue "bllvm-consensus" \
    "Performance optimization" \
    "## Description

Optimize hot paths in consensus validation.

## Context

Performance is important for consensus validation, especially in hot paths. There may be opportunities to optimize without sacrificing correctness.

## Acceptance Criteria

- [ ] Profile consensus validation code
- [ ] Identify hot paths and bottlenecks
- [ ] Optimize identified areas
- [ ] Ensure optimizations don't break correctness
- [ ] Benchmark improvements

## Technical Details

- **Files to modify**: \`src/**/*.rs\`
- **Dependencies**: Benchmarking tools
- **References**: See \`bllvm-consensus/LOW_HANGING_FRUIT_OPTIMIZATIONS.md\`

## Skills Required

- Performance profiling
- Rust optimization
- Benchmarking

## Difficulty

- [ ] Good First Issue
- [x] Intermediate
- [ ] Advanced

## Priority

- [ ] P0 (Critical - Blocks production)
- [ ] P1 (High - Important feature)
- [x] P2 (Medium - Nice to have)
- [ ] P3 (Low - Future enhancement)

## Getting Started

1. Profile consensus code to find hot paths
2. Review \`LOW_HANGING_FRUIT_OPTIMIZATIONS.md\` for ideas
3. Implement optimizations carefully
4. Benchmark before/after
5. Ensure all tests still pass
6. Submit PR with optimizations" \
    '["intermediate", "type:enhancement", "area:consensus", "priority:medium"]'

create_issue "bllvm-consensus" \
    "Implement UTXO commitment verification" \
    "## Description

Complete UTXO commitment verification logic.

## Context

UTXO commitments allow efficient verification of UTXO set state. The verification logic needs to be completed.

## Acceptance Criteria

- [ ] Review existing UTXO commitment code
- [ ] Implement verification logic
- [ ] Add tests for verification
- [ ] Document the verification process

## Technical Details

- **Files to modify**: \`src/utxo_commitments/verification.rs\`
- **Dependencies**: Merkle tree implementation
- **References**: \`IMPORTANT_PLACEHOLDERS_AND_TODOS.md\`

## Skills Required

- Cryptography
- Merkle trees
- Bitcoin protocol

## Difficulty

- [ ] Good First Issue
- [ ] Intermediate
- [x] Advanced

## Priority

- [ ] P0 (Critical - Blocks production)
- [x] P1 (High - Important feature)
- [ ] P2 (Medium - Nice to have)
- [ ] P3 (Low - Future enhancement)

## Getting Started

1. Review \`src/utxo_commitments/verification.rs\`
2. Study UTXO commitment specification
3. Implement verification logic
4. Add comprehensive tests
5. Submit PR with implementation" \
    '["advanced", "type:feature", "area:consensus", "priority:high"]'

create_issue "bllvm-consensus" \
    "Add consensus rule tests from Bitcoin Core" \
    "## Description

Port additional test vectors from Bitcoin Core.

## Context

Bitcoin Core has extensive test vectors that validate consensus rules. Porting these tests helps ensure our implementation matches Bitcoin Core behavior.

## Acceptance Criteria

- [ ] Identify test vectors to port from Bitcoin Core
- [ ] Port test vectors to our test format
- [ ] Ensure all ported tests pass
- [ ] Document source of test vectors

## Technical Details

- **Files to modify**: \`tests/core_test_vectors/\`
- **Dependencies**: Bitcoin Core test data
- **References**: See existing core test vectors

## Skills Required

- Bitcoin Core knowledge
- Test porting
- Bitcoin consensus rules

## Difficulty

- [ ] Good First Issue
- [ ] Intermediate
- [x] Advanced

## Priority

- [ ] P0 (Critical - Blocks production)
- [x] P1 (High - Important feature)
- [ ] P2 (Medium - Nice to have)
- [ ] P3 (Low - Future enhancement)

## Getting Started

1. Review existing core test vectors
2. Study Bitcoin Core test suite
3. Identify test vectors to port
4. Port tests to our format
5. Run tests: \`cargo test --test core_test_vectors\`
6. Submit PR with new test vectors" \
    '["advanced", "type:testing", "area:consensus", "priority:high"]'

echo ""
echo "✅ Completed bllvm-consensus issues (9 total)"
echo ""

