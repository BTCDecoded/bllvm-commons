-- Migration 006: Remove Economic Nodes, Veto System, and Fee Forwarding
-- This migration removes deprecated governance systems:
-- - Economic nodes and veto system (removed - maintainer-only governance)
-- - Fee forwarding (removed - not part of monetization model)
-- - Cleans up contribution data
-- - Updates participation_weights table structure

-- Drop economic nodes and veto system tables
DROP TABLE IF EXISTS veto_signals;
DROP TABLE IF EXISTS economic_nodes;
DROP TABLE IF EXISTS pr_veto_state;

-- Drop fee forwarding table
DROP TABLE IF EXISTS fee_forwarding_contributions;

-- Clean up unified_contributions
-- Remove fee forwarding contributions (no longer tracked)
DELETE FROM unified_contributions WHERE contribution_type = 'fee_forwarding';

-- Remove merge mining contributions (now module revenue, not governance)
DELETE FROM unified_contributions WHERE contribution_type LIKE 'merge_mining:%';

-- Note: Keep zap contributions for reporting/transparency (but they don't affect governance)

-- Update participation_weights table structure
-- Remove old fields that are no longer used (merge_mining_btc, fee_forwarding_btc, cumulative_zaps_btc)
-- Since governance is maintainer-only, all weights are 0.0

-- Create new simplified participation_weights table
CREATE TABLE IF NOT EXISTS participation_weights_new (
    contributor_id TEXT PRIMARY KEY,
    contributor_type TEXT NOT NULL,
    total_contribution_btc REAL DEFAULT 0.0,  -- Always 0.0 (maintainer-only governance)
    base_weight REAL DEFAULT 0.0,            -- Always 0.0 (maintainer-only governance)
    capped_weight REAL DEFAULT 0.0,          -- Always 0.0 (maintainer-only governance)
    total_system_weight REAL DEFAULT 0.0,    -- Always 0.0 (maintainer-only governance)
    last_updated TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Migrate existing data (set all weights to 0.0)
INSERT INTO participation_weights_new (contributor_id, contributor_type, total_contribution_btc, base_weight, capped_weight, total_system_weight, last_updated)
SELECT 
    contributor_id,
    contributor_type,
    0.0 as total_contribution_btc,
    0.0 as base_weight,
    0.0 as capped_weight,
    0.0 as total_system_weight,
    last_updated
FROM participation_weights;

-- Drop old table
DROP TABLE IF EXISTS participation_weights;

-- Rename new table
ALTER TABLE participation_weights_new RENAME TO participation_weights;

-- Note: proposal_zap_votes table is kept for reporting/transparency
-- Zaps are tracked but don't affect governance (maintainer-only multisig)

