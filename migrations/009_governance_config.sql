-- Governance-controlled configuration registry
-- Tracks all configurable parameters that require governance approval to change

CREATE TABLE IF NOT EXISTS governance_config_registry (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_key TEXT NOT NULL UNIQUE,
    config_category TEXT NOT NULL, -- 'feature_flags', 'thresholds', 'time_windows', 'limits', etc.
    current_value TEXT NOT NULL, -- JSON-encoded value
    default_value TEXT NOT NULL, -- JSON-encoded default value
    description TEXT,
    tier_requirement INTEGER NOT NULL DEFAULT 5, -- Tier required to change (default: Tier 5)
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    created_by TEXT, -- User/entity that created this config
    notes TEXT
);

CREATE INDEX IF NOT EXISTS idx_governance_config_category ON governance_config_registry(config_category);
CREATE INDEX IF NOT EXISTS idx_governance_config_key ON governance_config_registry(config_key);

-- Pending configuration changes (require governance approval)
CREATE TABLE IF NOT EXISTS governance_config_changes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_key TEXT NOT NULL,
    proposed_value TEXT NOT NULL, -- JSON-encoded proposed value
    current_value TEXT NOT NULL, -- JSON-encoded current value (snapshot at proposal time)
    change_reason TEXT,
    proposed_by TEXT NOT NULL, -- User/entity proposing the change
    proposed_at INTEGER NOT NULL,
    tier_requirement INTEGER NOT NULL DEFAULT 5,
    status TEXT NOT NULL DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'activated', 'cancelled'
    approval_pr_id INTEGER, -- PR that approved this change (if applicable)
    approved_at INTEGER,
    approved_by TEXT,
    activated_at INTEGER,
    activation_pr_id INTEGER, -- PR that activated this change
    notes TEXT,
    FOREIGN KEY (config_key) REFERENCES governance_config_registry(config_key)
);

CREATE INDEX IF NOT EXISTS idx_governance_config_changes_key ON governance_config_changes(config_key);
CREATE INDEX IF NOT EXISTS idx_governance_config_changes_status ON governance_config_changes(status);
CREATE INDEX IF NOT EXISTS idx_governance_config_changes_pr ON governance_config_changes(approval_pr_id);

-- Configuration change history (audit trail)
CREATE TABLE IF NOT EXISTS governance_config_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    config_key TEXT NOT NULL,
    old_value TEXT NOT NULL, -- JSON-encoded old value
    new_value TEXT NOT NULL, -- JSON-encoded new value
    changed_at INTEGER NOT NULL,
    changed_by TEXT NOT NULL,
    change_id INTEGER, -- Reference to governance_config_changes.id if applicable
    activation_method TEXT NOT NULL, -- 'governance_approved', 'emergency', 'manual', 'default'
    notes TEXT,
    FOREIGN KEY (config_key) REFERENCES governance_config_registry(config_key),
    FOREIGN KEY (change_id) REFERENCES governance_config_changes(id)
);

CREATE INDEX IF NOT EXISTS idx_governance_config_history_key ON governance_config_history(config_key);
CREATE INDEX IF NOT EXISTS idx_governance_config_history_change ON governance_config_history(change_id);



