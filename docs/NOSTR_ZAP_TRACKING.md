# Nostr Zap Tracking for Contributor Qualification

## Overview

You can track Lightning zaps via Nostr events (NIP-57) without running a Lightning node or LNURL service. Zap receipts are published to Nostr relays as events, which you can subscribe to and track.

## How NIP-57 Zaps Work

### Zap Flow

1. **User wants to zap**: User clicks zap button in Nostr client
2. **Invoice generation**: Lightning service provider (LSP) generates invoice
3. **Payment**: User pays invoice via Lightning network
4. **Zap receipt published**: LSP publishes zap receipt event (kind 9735) to Nostr relays
5. **You subscribe**: Your system subscribes to zap events and tracks them

### Zap Receipt Event (Kind 9735)

```json
{
  "id": "zap_receipt_id",
  "pubkey": "lsp_pubkey",  // Lightning service provider
  "created_at": 1234567890,
  "kind": 9735,  // Zap receipt
  "tags": [
    ["p", "recipient_pubkey"],  // Your bot's pubkey
    ["e", "event_id"],  // Event being zapped (optional)
    ["bolt11", "lnbc1..."],  // Lightning invoice
    ["description", "{\"pubkey\":\"sender_pubkey\",\"content\":\"zap message\"}"],
    ["amount", "1000"]  // Amount in millisatoshis
  ],
  "content": "",  // Usually empty
  "sig": "signature"
}
```

### Key Information Available

- **Recipient**: `p` tag (your bot's pubkey)
- **Amount**: `amount` tag (millisatoshis)
- **Sender**: In `description` tag JSON (if available)
- **Timestamp**: `created_at`
- **Invoice**: `bolt11` tag (can verify payment hash)

## Implementation

### Add Zap Subscription to NostrClient

```rust
// bllvm-commons/src/nostr/client.rs

impl NostrClient {
    /// Subscribe to zap events for this pubkey
    pub async fn subscribe_to_zaps(
        &self,
        recipient_pubkey: &str,
    ) -> Result<tokio::sync::mpsc::Receiver<ZapEvent>> {
        use nostr_sdk::prelude::*;
        
        // Create filter for zap receipts (kind 9735) to this pubkey
        let filter = Filter::new()
            .kind(Kind::ZapReceipt)  // Kind 9735
            .pubkey(XOnlyPublicKey::from_str(recipient_pubkey)?);
        
        // Subscribe to events
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        
        // Spawn task to handle incoming zap events
        let client = self.client.clone();
        tokio::spawn(async move {
            let mut notifications = client.notifications();
            while let Ok(notification) = notifications.recv().await {
                if let RelayPoolNotification::Event { event, .. } = notification {
                    if event.kind == Kind::ZapReceipt {
                        // Parse zap event
                        if let Ok(zap) = parse_zap_event(&event) {
                            let _ = tx.send(zap).await;
                        }
                    }
                }
            }
        });
        
        Ok(rx)
    }
}

/// Parsed zap event
#[derive(Debug, Clone)]
pub struct ZapEvent {
    pub recipient_pubkey: String,
    pub sender_pubkey: Option<String>,
    pub amount_msat: u64,
    pub timestamp: i64,
    pub invoice: Option<String>,
    pub message: Option<String>,
    pub zapped_event_id: Option<String>,
}

fn parse_zap_event(event: &Event) -> Result<ZapEvent> {
    // Extract recipient (p tag)
    let recipient = event
        .tags
        .iter()
        .find(|tag| tag.as_vec()[0] == "p")
        .and_then(|tag| tag.as_vec().get(1))
        .ok_or_else(|| anyhow!("Missing p tag"))?;
    
    // Extract amount (amount tag)
    let amount_msat = event
        .tags
        .iter()
        .find(|tag| tag.as_vec()[0] == "amount")
        .and_then(|tag| tag.as_vec().get(1))
        .and_then(|amt| amt.parse::<u64>().ok())
        .unwrap_or(0);
    
    // Extract invoice (bolt11 tag)
    let invoice = event
        .tags
        .iter()
        .find(|tag| tag.as_vec()[0] == "bolt11")
        .and_then(|tag| tag.as_vec().get(1))
        .map(|s| s.to_string());
    
    // Extract description (contains sender info)
    let (sender_pubkey, message) = event
        .tags
        .iter()
        .find(|tag| tag.as_vec()[0] == "description")
        .and_then(|tag| tag.as_vec().get(1))
        .and_then(|desc| {
            serde_json::from_str::<serde_json::Value>(desc).ok()
        })
        .map(|desc| {
            let sender = desc.get("pubkey")
                .and_then(|p| p.as_str())
                .map(|s| s.to_string());
            let msg = desc.get("content")
                .and_then(|c| c.as_str())
                .map(|s| s.to_string());
            (sender, msg)
        })
        .unwrap_or((None, None));
    
    // Extract zapped event (e tag)
    let zapped_event_id = event
        .tags
        .iter()
        .find(|tag| tag.as_vec()[0] == "e")
        .and_then(|tag| tag.as_vec().get(1))
        .map(|s| s.to_string());
    
    Ok(ZapEvent {
        recipient_pubkey: recipient.to_string(),
        sender_pubkey,
        amount_msat,
        timestamp: event.created_at.as_i64(),
        invoice,
        message,
        zapped_event_id,
    })
}
```

### Zap Tracking Service

```rust
// bllvm-commons/src/nostr/zap_tracker.rs

use crate::database::Database;
use crate::nostr::client::NostrClient;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::SqlitePool;
use tracing::{info, warn};

pub struct ZapTracker {
    pool: SqlitePool,
    nostr_client: NostrClient,
    bot_pubkeys: Vec<String>,  // All bot pubkeys to track
}

impl ZapTracker {
    pub fn new(pool: SqlitePool, nostr_client: NostrClient, bot_pubkeys: Vec<String>) -> Self {
        Self {
            pool,
            nostr_client,
            bot_pubkeys,
        }
    }
    
    /// Start tracking zaps for all bot pubkeys
    pub async fn start_tracking(&self) -> Result<()> {
        // Subscribe to zaps for each bot pubkey
        for pubkey in &self.bot_pubkeys {
            let mut zap_rx = self.nostr_client.subscribe_to_zaps(pubkey).await?;
            
            // Spawn task to process zaps for this pubkey
            let pool = self.pool.clone();
            let pubkey_clone = pubkey.clone();
            tokio::spawn(async move {
                while let Some(zap) = zap_rx.recv().await {
                    if let Err(e) = Self::process_zap(&pool, &pubkey_clone, zap).await {
                        warn!("Failed to process zap: {}", e);
                    }
                }
            });
        }
        
        info!("Started tracking zaps for {} bot pubkeys", self.bot_pubkeys.len());
        Ok(())
    }
    
    /// Process a zap event
    async fn process_zap(
        pool: &SqlitePool,
        recipient_pubkey: &str,
        zap: crate::nostr::client::ZapEvent,
    ) -> Result<()> {
        // Convert millisatoshis to BTC
        let amount_btc = zap.amount_msat as f64 / 100_000_000_000.0;
        
        // Record zap in database
        sqlx::query!(
            r#"
            INSERT INTO zap_contributions
            (recipient_pubkey, sender_pubkey, amount_msat, amount_btc, timestamp, invoice_hash, message)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
            recipient_pubkey,
            zap.sender_pubkey,
            zap.amount_msat as i64,
            amount_btc,
            DateTime::from_timestamp(zap.timestamp, 0),
            zap.invoice.as_ref().map(|i| Self::extract_payment_hash(i)),
            zap.message
        )
        .execute(pool)
        .await?;
        
        info!(
            "Recorded zap: {} msat ({:.8} BTC) to {} from {}",
            zap.amount_msat,
            amount_btc,
            recipient_pubkey,
            zap.sender_pubkey.as_deref().unwrap_or("unknown")
        );
        
        Ok(())
    }
    
    /// Extract payment hash from invoice (for verification)
    fn extract_payment_hash(invoice: &str) -> Option<String> {
        // Parse bolt11 invoice to extract payment hash
        // This is optional - just for additional verification
        // You can use a bolt11 parsing library
        None  // Placeholder
    }
    
    /// Get total zaps for a pubkey in time period
    pub async fn get_total_zaps(
        &self,
        pubkey: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<f64> {
        let result = sqlx::query_scalar!(
            r#"
            SELECT SUM(amount_btc) as total
            FROM zap_contributions
            WHERE recipient_pubkey = ? 
              AND timestamp >= ? 
              AND timestamp <= ?
            "#,
            pubkey,
            start_time,
            end_time
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(result.unwrap_or(0.0))
    }
    
    /// Get zaps by sender (for contributor qualification)
    pub async fn get_zaps_by_sender(
        &self,
        sender_pubkey: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> Result<Vec<ZapContribution>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, recipient_pubkey, sender_pubkey, amount_msat, amount_btc, timestamp, invoice_hash, message
            FROM zap_contributions
            WHERE sender_pubkey = ?
              AND timestamp >= ?
              AND timestamp <= ?
            ORDER BY timestamp DESC
            "#,
            sender_pubkey,
            start_time,
            end_time
        )
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows.into_iter().map(|row| ZapContribution {
            id: row.id,
            recipient_pubkey: row.recipient_pubkey,
            sender_pubkey: row.sender_pubkey,
            amount_msat: row.amount_msat as u64,
            amount_btc: row.amount_btc,
            timestamp: row.timestamp,
            invoice_hash: row.invoice_hash,
            message: row.message,
        }).collect())
    }
}

#[derive(Debug, Clone)]
pub struct ZapContribution {
    pub id: i32,
    pub recipient_pubkey: String,
    pub sender_pubkey: Option<String>,
    pub amount_msat: u64,
    pub amount_btc: f64,
    pub timestamp: DateTime<Utc>,
    pub invoice_hash: Option<String>,
    pub message: Option<String>,
}
```

### Database Schema

```sql
-- Migration: Add zap tracking table
CREATE TABLE zap_contributions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    recipient_pubkey TEXT NOT NULL,  -- Bot pubkey that received zap
    sender_pubkey TEXT,  -- Sender pubkey (if available in description)
    amount_msat INTEGER NOT NULL,  -- Amount in millisatoshis
    amount_btc REAL NOT NULL,  -- Amount in BTC
    timestamp DATETIME NOT NULL,
    invoice_hash TEXT,  -- Payment hash from invoice (for verification)
    message TEXT,  -- Optional zap message
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for efficient queries
CREATE INDEX idx_zap_recipient ON zap_contributions(recipient_pubkey);
CREATE INDEX idx_zap_sender ON zap_contributions(sender_pubkey);
CREATE INDEX idx_zap_timestamp ON zap_contributions(timestamp);
CREATE INDEX idx_zap_recipient_time ON zap_contributions(recipient_pubkey, timestamp);
```

### Integration with Contributor Qualification

```rust
// bllvm-commons/src/economic_nodes/verification.rs

impl ContributionVerifier {
    /// Verify Lightning zap contribution using Nostr zap events
    pub async fn verify_lightning_zap_nostr(
        &self,
        sender_pubkey: &str,
        proof: &LightningZapProof,
    ) -> Result<bool, GovernanceError> {
        // Get zap tracker
        let zap_tracker = self.get_zap_tracker().await?;
        
        // Calculate time window (90 days)
        let end_time = Utc::now();
        let start_time = end_time - chrono::Duration::days(90);
        
        // Get all zaps from this sender in time window
        let zaps = zap_tracker
            .get_zaps_by_sender(sender_pubkey, start_time, end_time)
            .await?;
        
        // Verify minimum threshold (0.05 BTC over 90 days)
        let total_btc: f64 = zaps.iter().map(|z| z.amount_btc).sum();
        if total_btc < 0.05 {
            return Ok(false);
        }
        
        // Verify minimum contribution count (3+ contributions)
        if zaps.len() < 3 {
            return Ok(false);
        }
        
        // Verify total matches proof
        if (total_btc - proof.total_zaps_btc).abs() > 0.0001 {
            return Ok(false);
        }
        
        // Verify all zaps are to Commons bot pubkeys
        let commons_pubkeys = self.get_commons_bot_pubkeys().await?;
        for zap in &zaps {
            if !commons_pubkeys.contains(&zap.recipient_pubkey) {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}
```

## Advantages

1. **No Lightning Node Required**: Just subscribe to Nostr relays
2. **No LNURL Service**: Zap receipts are published automatically
3. **Public Verification**: Anyone can verify zap events on relays
4. **Simple Implementation**: Uses existing Nostr client infrastructure
5. **Real-time Tracking**: Zaps appear immediately on relays

## Limitations

1. **Trust in LSP**: Zap receipts are published by Lightning service providers, not senders
2. **Missing Sender Info**: Some zaps don't include sender pubkey in description
3. **Relay Reliability**: Depends on relays to store and serve zap events
4. **No Payment Verification**: Can't verify payment actually went through (only receipt)

## Mitigations

1. **Multiple Relays**: Subscribe to multiple relays for redundancy
2. **Historical Queries**: Query historical zap events when verifying
3. **Invoice Hash Verification**: Extract payment hash from invoice for additional verification
4. **Sender Pubkey Optional**: Allow qualification even if sender pubkey missing (use invoice hash as identifier)

## Configuration

```toml
# bllvm-commons/config/app.toml

[nostr]
enabled = true
relays = [
    "wss://relay.damus.io",
    "wss://nos.lol",
    "wss://relay.nostr.band"
]

# Bot pubkeys to track zaps for
[nostr.zap_tracking]
enabled = true
bot_pubkeys = [
    "npub1...",  # @BTCCommons_Gov
    "npub1...",  # @BTCCommons_Dev
    "npub1...",  # @BTCCommons_Network
]

# Zap qualification thresholds
[nostr.zap_tracking.qualification]
minimum_contribution_btc = 0.05
minimum_contributions = 3
measurement_period_days = 90
```

## Summary

**Yes, you can use Nostr integration to track zaps without a Lightning node!**

- Subscribe to zap receipt events (kind 9735) on Nostr relays
- Filter for zaps to your bot pubkeys
- Extract amount, sender, timestamp from zap events
- Track in database for contributor qualification
- Much simpler than running a Lightning node

The only requirement is that zap receipts are published to Nostr relays (which they are by default when using standard Lightning service providers).

