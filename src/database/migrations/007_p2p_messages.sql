-- Migration: P2P Message Deduplication
-- Creates table to track received P2P governance messages for deduplication

CREATE TABLE IF NOT EXISTS received_p2p_messages (
    message_id TEXT NOT NULL PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_received_p2p_messages_timestamp ON received_p2p_messages(timestamp);

