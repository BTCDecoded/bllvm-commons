//! Fee Forwarding Tracker
//!
//! Monitors blockchain for transactions to Commons address and records
//! fee forwarding contributions for governance.

use crate::governance::ContributionTracker;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use tracing::{debug, info, warn};

/// Fee forwarding tracker
pub struct FeeForwardingTracker {
    pool: SqlitePool,
    contribution_tracker: ContributionTracker,
    commons_addresses: Vec<String>,  // List of Commons addresses to monitor
}

impl FeeForwardingTracker {
    /// Create a new fee forwarding tracker
    pub fn new(pool: SqlitePool, commons_addresses: Vec<String>) -> Self {
        Self {
            pool: pool.clone(),
            contribution_tracker: ContributionTracker::new(pool),
            commons_addresses,
        }
    }
    
    /// Process a block and check for fee forwarding transactions
    /// This would be called when a new block is received
    pub async fn process_block(
        &self,
        block: &bllvm_protocol::Block,
        block_height: i32,
        contributor_id: &str,  // Miner/node identifier
    ) -> Result<Vec<FeeForwardingContribution>> {
        let mut contributions = Vec::new();
        
        // Check each transaction in the block
        for (tx_index, tx) in block.transactions.iter().enumerate() {
            // Skip coinbase (index 0)
            if tx_index == 0 {
                continue;
            }
            
            // Check each output for Commons address
            for output in &tx.outputs {
                // Decode script_pubkey to get address
                // For now, we'll check if the output value is sent to a Commons address
                // In production, this would decode the script_pubkey to get the address
                let address = self.decode_address_from_script(&output.script_pubkey)?;
                
                if let Some(address) = address {
                    if self.commons_addresses.contains(&address) {
                        // This is a fee forwarding transaction
                        let amount_btc = output.value as f64 / 100_000_000.0;  // Convert satoshis to BTC
                        
                        // Get transaction hash
                        let tx_hash = self.calculate_tx_hash(tx);
                        
                        // Check if we've already recorded this transaction
                        let existing: Option<i64> = sqlx::query_scalar(
                            r#"
                            SELECT id FROM fee_forwarding_contributions
                            WHERE tx_hash = ?
                            "#,
                        )
                        .bind(&tx_hash)
                        .fetch_optional(&self.pool)
                        .await?;
                        
                        if existing.is_some() {
                            continue;  // Already recorded
                        }
                        
                        // Record the contribution (this also records in unified_contributions)
                        self.contribution_tracker
                            .record_fee_forwarding_contribution(
                                contributor_id,
                                &tx_hash,
                                amount_btc,
                                &address,
                                block_height,
                                Utc::now(),
                            )
                            .await?;
                        
                        let tx_hash_clone = tx_hash.clone();
                        let address_clone = address.clone();
                        
                        contributions.push(FeeForwardingContribution {
                            contributor_id: contributor_id.to_string(),
                            tx_hash: tx_hash_clone.clone(),
                            block_height,
                            amount_btc,
                            commons_address: address_clone.clone(),
                            timestamp: Utc::now(),
                        });
                        
                        info!(
                            "Recorded fee forwarding: {} BTC (tx: {}) from {} to Commons address {}",
                            amount_btc, tx_hash_clone, contributor_id, address_clone
                        );
                    }
                }
            }
        }
        
        Ok(contributions)
    }
    
    /// Decode Bitcoin address from script_pubkey
    /// This is a simplified version - in production, use proper address decoding
    fn decode_address_from_script(&self, script_pubkey: &[u8]) -> Result<Option<String>> {
        // Simplified: For P2PKH, script is: OP_DUP OP_HASH160 <20-byte hash> OP_EQUALVERIFY OP_CHECKSIG
        // Pattern: 0x76 0xa9 0x14 <20 bytes> 0x88 0xac
        
        if script_pubkey.len() == 25
            && script_pubkey[0] == 0x76
            && script_pubkey[1] == 0xa9
            && script_pubkey[2] == 0x14
            && script_pubkey[23] == 0x88
            && script_pubkey[24] == 0xac
        {
            // Extract 20-byte hash
            let hash = &script_pubkey[3..23];
            // Convert to hex string (simplified - in production, use proper base58 encoding)
            let address = format!("{}", hex::encode(hash));
            Ok(Some(address))
        } else {
            // Not a P2PKH script, or we can't decode it
            // In production, would handle P2SH, P2WPKH, P2WSH, etc.
            Ok(None)
        }
    }
    
    /// Calculate transaction hash (simplified)
    fn calculate_tx_hash(&self, _tx: &bllvm_protocol::Transaction) -> String {
        // In production, this would serialize the transaction and hash it
        // For now, return a placeholder
        // TODO: Use proper transaction hashing
        format!("tx_hash_placeholder")
    }
    
    /// Get fee forwarding contributions for a contributor
    pub async fn get_contributor_contributions(
        &self,
        contributor_id: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<FeeForwardingContribution>> {
        #[derive(sqlx::FromRow)]
        struct FeeForwardingRow {
            contributor_id: String,
            tx_hash: String,
            block_height: i32,
            amount_btc: f64,
            commons_address: String,
            timestamp: DateTime<Utc>,
        }
        
        let rows = sqlx::query_as::<_, FeeForwardingRow>(
            r#"
            SELECT contributor_id, tx_hash, block_height, amount_btc, commons_address, timestamp
            FROM fee_forwarding_contributions
            WHERE contributor_id = ?
              AND timestamp >= ?
              AND timestamp <= ?
            ORDER BY timestamp DESC
            "#,
        )
        .bind(contributor_id)
        .bind(start_time)
        .bind(end_time)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| FeeForwardingContribution {
            contributor_id: row.contributor_id,
            tx_hash: row.tx_hash,
            block_height: row.block_height,
            amount_btc: row.amount_btc,
            commons_address: row.commons_address,
            timestamp: row.timestamp,
        }).collect())
    }
}

/// Fee forwarding contribution record
#[derive(Debug, Clone)]
pub struct FeeForwardingContribution {
    pub contributor_id: String,
    pub tx_hash: String,
    pub block_height: i32,
    pub amount_btc: f64,
    pub commons_address: String,
    pub timestamp: DateTime<Utc>,
}

