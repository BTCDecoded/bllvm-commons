# Monetization System User Guide

## How to Participate in Governance

### 1. Zap to Vote

**What is Zap Voting?**
- Send Lightning payments (zaps) to governance proposals
- Your zap amount determines your vote weight (quadratic: √amount)
- Zaps can be "support", "veto", or "abstain" (via message)

**How to Zap a Proposal**:

1. **Find the Proposal**:
   - Proposals are published to Nostr with zap addresses
   - Look for events with tag `["d", "governance-proposal"]`
   - Check the `zap` tag for the Lightning address

2. **Send a Zap**:
   - Use a Nostr client that supports zaps (e.g., Damus, Amethyst)
   - Zap the proposal event with your vote amount
   - Include message: "support", "veto", or "abstain" (optional)

3. **Vote Weight**:
   - Your vote weight = √(zap_amount_btc)
   - Example: 1 BTC zap = 1.0 weight, 4 BTC zap = 2.0 weight
   - Maximum: 5% of total system weight (prevents whale dominance)

**Example**:
```
Zap 0.01 BTC to proposal → Vote weight = √0.01 = 0.1
Zap 1.0 BTC to proposal → Vote weight = √1.0 = 1.0
Zap 4.0 BTC to proposal → Vote weight = √4.0 = 2.0
```

### 2. Merge Mining

**What is Merge Mining?**
- Mine secondary chains (e.g., RSK, Namecoin) alongside Bitcoin
- 1% of secondary chain rewards go to Commons
- Your contribution = 1% of rewards you generate

**How to Participate**:
1. Set up merge mining with Stratum V2
2. Configure secondary chains
3. Contributions tracked automatically (30-day rolling)

**Vote Weight**:
- Your participation weight = √(total_contributions_btc)
- Updated monthly (30-day rolling window)

### 3. Fee Forwarding

**What is Fee Forwarding?**
- Forward transaction fees to Commons address
- Tracked on-chain (verified)
- Your contribution = fees you forward

**How to Participate**:
1. Configure Commons address in node config:
   ```toml
   [fee_forwarding]
   enabled = true
   commons_address = "bc1qcommons..."
   forwarding_percentage = 10  # 10% of fees
   contributor_id = "your_node_id"
   ```

2. Fees automatically forwarded when mining blocks
3. Contributions tracked automatically (30-day rolling)

**Vote Weight**:
- Your participation weight = √(total_contributions_btc)
- Updated monthly (30-day rolling window)

### 4. General Zaps

**What are General Zaps?**
- Zap the Commons bot directly (not tied to a proposal)
- Cumulative contribution (all-time)
- Builds your participation weight over time

**How to Zap**:
1. Find the Commons bot on Nostr
2. Zap the bot with any amount
3. Your cumulative zaps count toward participation weight

**Vote Weight**:
- Your participation weight = √(cumulative_zaps_btc)
- No time limit (cumulative, all-time)

## Voting Rules

### Vote Weight Calculation

**For Proposals**:
- Use the higher of:
  - Proposal zap weight: √(proposal_zap_btc)
  - 10% of participation weight: participation_weight × 0.1

**For Participation**:
- Total contribution = merge_mining + fee_forwarding + cumulative_zaps
- Weight = √(total_contribution_btc)
- Capped at 5% of total system weight

### Cooling-Off Period

**Large Contributions** (≥0.1 BTC):
- Must wait 30 days before counting toward votes
- Prevents vote buying and timing attacks

**Small Contributions** (<0.1 BTC):
- Count immediately
- No cooling-off period

### Weight Caps

**Per Contributor**:
- Maximum: 5% of total system weight
- Prevents whale dominance
- Ensures fair distribution

## Veto Mechanisms

### Economic Node Veto (Tier 3+)
- **Mining Veto**: 30%+ of network hashpower
- **Economic Veto**: 40%+ of economic activity
- Either threshold blocks the proposal

### Zap Vote Veto (All Tiers)
- **Threshold**: 40% of total zap vote weight
- If 40%+ of zap votes are "veto", proposal is blocked

## Checking Your Status

### View Your Contributions

```bash
# Query database (if you have access)
sqlite3 governance.db "
SELECT 
    contributor_id,
    contribution_type,
    SUM(amount_btc) as total_btc
FROM unified_contributions
WHERE contributor_id = 'your_id'
GROUP BY contribution_type;
"
```

### View Your Weight

```bash
sqlite3 governance.db "
SELECT 
    contributor_id,
    total_contribution_btc,
    base_weight,
    capped_weight
FROM participation_weights
WHERE contributor_id = 'your_id';
"
```

### View Proposal Votes

```bash
sqlite3 governance.db "
SELECT 
    pr_id,
    vote_type,
    SUM(vote_weight) as total_weight,
    COUNT(*) as vote_count
FROM proposal_zap_votes
WHERE pr_id = 123
GROUP BY vote_type;
"
```

## Best Practices

1. **Start Small**: Build participation weight over time
2. **Be Consistent**: Regular contributions build weight
3. **Understand Proposals**: Read before voting
4. **Use Messages**: Include "support" or "veto" in zap messages
5. **Monitor Your Weight**: Check periodically to understand your influence

## Troubleshooting

### My Zap Wasn't Counted

**Check**:
1. Was it sent to the correct bot pubkey?
2. Was it a valid NIP-57 zap receipt?
3. Check logs: `journalctl -u bllvm-commons | grep zap`

### My Weight Isn't Updating

**Check**:
1. Is weight update task running? (Check logs)
2. Are contributions verified? (Check `verified` column)
3. Are contributions in cooling-off? (Check `contribution_age_days`)

### Proposal Vote Not Showing

**Check**:
1. Was zap sent to the proposal event?
2. Does zap have `zapped_event_id` matching proposal?
3. Check `proposal_zap_votes` table

## Support

For issues or questions:
- GitHub Issues: https://github.com/BTCDecoded/bllvm-commons/issues
- Nostr: @BTCCommons_Gov
- Email: governance@btcdecoded.org

