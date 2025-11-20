# Governance Weighting: Alternatives to Dollar Pegging

## Overview

Instead of pegging to dollars (fiat), we can use Bitcoin-native metrics or hybrid approaches that better align with Bitcoin values and avoid fiat dependency.

---

## Option 1: BTC with Moving Average (Recommended)

### Concept

Use BTC directly, but with a **moving average** to smooth volatility.

### Formula

```
contribution_btc = actual_contribution_btc
btc_price_ma = 30-day_moving_average(btc_price_usd)
contribution_value = contribution_btc * btc_price_ma
vote_weight = sqrt(contribution_value / normalization_factor)
```

### Advantages

- ✅ **Bitcoin-native**: No fiat dependency
- ✅ **Stable**: Moving average smooths volatility
- ✅ **Transparent**: BTC amounts are clear
- ✅ **Aligned**: Values Bitcoin as the unit of account

### Implementation

```rust
pub struct BtcMovingAverage {
    prices: VecDeque<(DateTime<Utc>, f64)>,  // (timestamp, btc_price_usd)
    window_days: u32,
}

impl BtcMovingAverage {
    /// Get 30-day moving average BTC price
    pub fn get_average_price(&self) -> f64 {
        let cutoff = Utc::now() - chrono::Duration::days(self.window_days as i64);
        let recent_prices: Vec<f64> = self.prices
            .iter()
            .filter(|(ts, _)| *ts >= cutoff)
            .map(|(_, price)| *price)
            .collect();
        
        if recent_prices.is_empty() {
            return 0.0;
        }
        
        recent_prices.iter().sum::<f64>() / recent_prices.len() as f64
    }
    
    /// Calculate contribution value using moving average
    pub fn calculate_contribution_value(&self, contribution_btc: f64) -> f64 {
        let avg_price = self.get_average_price();
        contribution_btc * avg_price
    }
}
```

### Example

- Contribution: 0.1 BTC
- 30-day MA: $60,000
- Value: 0.1 × $60,000 = $6,000
- Weight: √6,000 = 77.5 votes

**Even if BTC drops to $40,000 today, weight stays at $60,000 average**

---

## Option 2: Hashrate-Indexed (Bitcoin-Native)

### Concept

Peg to Bitcoin's hashrate as a measure of network value/security.

### Formula

```
contribution_btc = actual_contribution_btc
hashrate_index = current_hashrate / baseline_hashrate  // e.g., 2024 baseline
contribution_value = contribution_btc * baseline_price * hashrate_index
vote_weight = sqrt(contribution_value / normalization_factor)
```

### Advantages

- ✅ **Bitcoin-native**: Tied to network security
- ✅ **Network-aligned**: Grows with Bitcoin adoption
- ✅ **No fiat**: Pure Bitcoin metrics
- ✅ **Meaningful**: Hashrate reflects network value

### Implementation

```rust
pub struct HashrateIndex {
    baseline_hashrate: f64,  // e.g., 2024 average
    baseline_price_usd: f64,  // e.g., $60,000 at baseline
}

impl HashrateIndex {
    /// Get current hashrate index
    pub fn get_index(&self, current_hashrate: f64) -> f64 {
        current_hashrate / self.baseline_hashrate
    }
    
    /// Calculate contribution value
    pub fn calculate_contribution_value(
        &self,
        contribution_btc: f64,
        current_hashrate: f64,
    ) -> f64 {
        let index = self.get_index(current_hashrate);
        contribution_btc * self.baseline_price_usd * index
    }
}
```

### Example

- Contribution: 0.1 BTC
- Baseline: 500 EH/s at $60,000
- Current: 600 EH/s (20% growth)
- Value: 0.1 × $60,000 × 1.2 = $7,200
- Weight: √7,200 = 84.9 votes

**Network growth increases contribution value**

---

## Option 3: Block Subsidy Indexed

### Concept

Peg to Bitcoin's block subsidy as a measure of economic value.

### Formula

```
contribution_btc = actual_contribution_btc
subsidy_index = current_subsidy / initial_subsidy  // 50 BTC → 3.125 BTC = 0.0625
baseline_value = contribution_btc * baseline_price
contribution_value = baseline_value / subsidy_index  // Adjusts for halving
vote_weight = sqrt(contribution_value / normalization_factor)
```

### Advantages

- ✅ **Bitcoin-native**: Tied to Bitcoin's monetary policy
- ✅ **Halving-aware**: Adjusts for supply reduction
- ✅ **Economic alignment**: Reflects Bitcoin's scarcity

### Implementation

```rust
use bllvm_consensus::economic::get_block_subsidy;

pub struct SubsidyIndex {
    baseline_price_usd: f64,
    baseline_height: u64,  // e.g., height at launch
}

impl SubsidyIndex {
    /// Calculate contribution value adjusted for subsidy
    pub fn calculate_contribution_value(
        &self,
        contribution_btc: f64,
        current_height: u64,
    ) -> f64 {
        let current_subsidy = get_block_subsidy(current_height) as f64 / 100_000_000.0;
        let baseline_subsidy = get_block_subsidy(self.baseline_height) as f64 / 100_000_000.0;
        let subsidy_ratio = current_subsidy / baseline_subsidy;
        
        // As subsidy decreases, same BTC contribution is worth more
        let adjusted_value = (contribution_btc * self.baseline_price_usd) / subsidy_ratio;
        adjusted_value
    }
}
```

### Example

- Contribution: 0.1 BTC
- Baseline: 6.25 BTC subsidy at $60,000
- Current: 3.125 BTC subsidy (halved)
- Value: 0.1 × $60,000 × (6.25 / 3.125) = $12,000
- Weight: √12,000 = 109.5 votes

**Halving increases value of contributions (scarcity premium)**

---

## Option 4: Multi-Asset Basket

### Concept

Use a basket of assets: BTC, energy cost, hashrate, etc.

### Formula

```
contribution_btc = actual_contribution_btc

// Weighted basket
btc_component = contribution_btc * btc_price_usd * 0.5
energy_component = (contribution_btc * energy_cost_per_btc) * 0.3
hashrate_component = (contribution_btc * hashrate_value) * 0.2

contribution_value = btc_component + energy_component + hashrate_component
vote_weight = sqrt(contribution_value / normalization_factor)
```

### Advantages

- ✅ **Diversified**: Not dependent on single metric
- ✅ **Robust**: Multiple value signals
- ✅ **Bitcoin-aligned**: All components Bitcoin-related

### Disadvantages

- ❌ **Complex**: Harder to understand
- ❌ **Arbitrary weights**: 0.5/0.3/0.2 is subjective

---

## Option 5: Energy-Indexed (Most Bitcoin-Native)

### Concept

Peg to energy cost of producing Bitcoin (proof-of-work value).

### Formula

```
contribution_btc = actual_contribution_btc
energy_per_btc = network_energy_consumption / btc_mined_per_period
energy_cost_per_btc = energy_per_btc * electricity_price
contribution_value = contribution_btc * energy_cost_per_btc
vote_weight = sqrt(contribution_value / normalization_factor)
```

### Advantages

- ✅ **Bitcoin-native**: Tied to proof-of-work
- ✅ **Economic reality**: Reflects actual cost
- ✅ **No fiat dependency**: Can use energy units directly

### Implementation

```rust
pub struct EnergyIndex {
    network_hashrate: f64,  // EH/s
    energy_efficiency: f64,  // J/TH (joules per terahash)
    electricity_price_per_kwh: f64,  // Optional: for USD conversion
}

impl EnergyIndex {
    /// Calculate energy cost per BTC
    pub fn calculate_energy_per_btc(&self, blocks_per_period: f64) -> f64 {
        // Network energy = hashrate * efficiency
        let network_energy_j = self.network_hashrate * 1_000_000.0 * self.energy_efficiency;
        let network_energy_kwh = network_energy_j / 3_600_000.0;
        
        // Energy per BTC = total energy / BTC mined
        let btc_mined = blocks_per_period * 6.25;  // Current subsidy
        network_energy_kwh / btc_mined
    }
    
    /// Calculate contribution value in energy terms
    pub fn calculate_contribution_value(
        &self,
        contribution_btc: f64,
        blocks_per_period: f64,
    ) -> f64 {
        let energy_per_btc = self.calculate_energy_per_btc(blocks_per_period);
        // Value = energy cost (can use USD or keep in kWh)
        contribution_btc * energy_per_btc * self.electricity_price_per_kwh
    }
}
```

---

## Option 6: Hybrid: BTC + Moving Average + Hashrate

### Concept (Recommended Hybrid)

Combine BTC moving average with hashrate growth for stability + network alignment.

### Formula

```
contribution_btc = actual_contribution_btc
btc_price_ma = 30-day_moving_average(btc_price_usd)
hashrate_growth = current_hashrate / baseline_hashrate
contribution_value = contribution_btc * btc_price_ma * hashrate_growth^0.5
vote_weight = sqrt(contribution_value / normalization_factor)
```

### Advantages

- ✅ **Stable**: Moving average smooths volatility
- ✅ **Network-aligned**: Hashrate growth factor
- ✅ **Bitcoin-native**: No pure fiat dependency
- ✅ **Balanced**: Combines price stability with network growth

### Implementation

```rust
pub struct HybridIndex {
    btc_ma: BtcMovingAverage,
    hashrate_index: HashrateIndex,
    hashrate_weight: f64,  // 0.0-1.0, e.g., 0.3
}

impl HybridIndex {
    /// Calculate contribution value
    pub fn calculate_contribution_value(
        &self,
        contribution_btc: f64,
        current_hashrate: f64,
    ) -> f64 {
        let btc_price = self.btc_ma.get_average_price();
        let hashrate_growth = self.hashrate_index.get_index(current_hashrate);
        
        // Weighted combination
        let base_value = contribution_btc * btc_price;
        let hashrate_adjustment = hashrate_growth.powf(self.hashrate_weight);
        
        base_value * hashrate_adjustment
    }
}
```

---

## Comparison Matrix

| Option | Stability | Bitcoin-Native | Complexity | Fiat Dependency |
|--------|-----------|---------------|------------|-----------------|
| **Dollar** | ✅ High | ❌ No | ✅ Low | ❌ Full |
| **BTC + MA** | ✅ Medium | ✅ Yes | ✅ Low | ⚠️ Partial (for MA) |
| **Hashrate** | ⚠️ Medium | ✅ Yes | ✅ Medium | ❌ None |
| **Subsidy** | ✅ High | ✅ Yes | ✅ Low | ⚠️ Partial (baseline) |
| **Energy** | ⚠️ Medium | ✅ Yes | ❌ High | ⚠️ Partial (optional) |
| **Hybrid** | ✅ High | ✅ Yes | ⚠️ Medium | ⚠️ Partial (for MA) |

---

## Recommended: BTC with Moving Average

### Why This Works Best

1. **Bitcoin-Native**: Uses BTC as unit of account
2. **Stable**: 30-day MA smooths volatility
3. **Simple**: Easy to understand and verify
4. **Transparent**: BTC amounts are clear
5. **Minimal Fiat**: Only for moving average calculation (can use multiple sources)

### Implementation

```rust
// bllvm-commons/src/governance/weight_calculator.rs

pub struct BtcBasedWeightCalculator {
    btc_price_service: BtcPriceService,
    ma_window_days: u32,  // 30 days
}

impl BtcBasedWeightCalculator {
    /// Calculate weight using BTC with moving average
    pub async fn calculate_weight(
        &self,
        contribution_btc: f64,
    ) -> Result<f64> {
        // Get 30-day moving average BTC price
        let btc_price_ma = self.btc_price_service
            .get_moving_average(self.ma_window_days)
            .await?;
        
        // Calculate contribution value
        let contribution_value = contribution_btc * btc_price_ma;
        
        // Apply quadratic formula
        let normalization_factor = 1.0;  // $1 = 1.0 weight
        Ok((contribution_value / normalization_factor).sqrt())
    }
    
    /// Calculate weight for multiple contribution types
    pub async fn calculate_unified_weight(
        &self,
        merge_mining_btc: f64,
        fee_forwarding_btc: f64,
        zaps_btc: f64,
    ) -> Result<f64> {
        let total_btc = merge_mining_btc + fee_forwarding_btc + zaps_btc;
        self.calculate_weight(total_btc).await
    }
}
```

### Price Source Options

**Option A: Multiple Exchange Average**
```rust
// Average of multiple exchanges
let prices = vec![
    get_price_from_exchange("coinbase").await?,
    get_price_from_exchange("kraken").await?,
    get_price_from_exchange("binance").await?,
];
let avg_price = prices.iter().sum::<f64>() / prices.len() as f64;
```

**Option B: On-Chain Oracle**
```rust
// Use on-chain price oracle (e.g., DLC, oracles)
let price = get_price_from_oracle().await?;
```

**Option C: Community Consensus**
```rust
// Use median of multiple sources
let sources = vec![
    get_price_from_api("coingecko").await?,
    get_price_from_api("coinmarketcap").await?,
    get_price_from_oracle().await?,
];
let median_price = calculate_median(sources);
```

---

## Alternative: Pure BTC (No Conversion)

### Concept

Use BTC directly without any conversion, but with normalization.

### Formula

```
contribution_btc = actual_contribution_btc
normalization_factor = 1.0 BTC  // 1 BTC = 1.0 weight baseline
vote_weight = sqrt(contribution_btc / normalization_factor)
```

### Advantages

- ✅ **Pure Bitcoin**: No fiat at all
- ✅ **Simple**: No price lookups needed
- ✅ **Transparent**: BTC amounts are clear
- ✅ **Volatility**: Accepts BTC volatility as feature

### Disadvantages

- ❌ **Volatile**: Weights change with BTC price swings
- ❌ **Unstable**: Harder to set fixed thresholds

### Mitigation: Time-Weighted BTC

```rust
// Use time-weighted BTC (recent contributions weighted more)
pub fn calculate_time_weighted_btc(
    contributions: &[(DateTime<Utc>, f64)],  // (timestamp, btc_amount)
) -> f64 {
    let now = Utc::now();
    contributions.iter()
        .map(|(ts, amount)| {
            let age_days = (now - *ts).num_days();
            let weight = (-age_days as f64 / 90.0).exp();  // Exponential decay
            amount * weight
        })
        .sum()
}
```

---

## Recommendation: BTC with 30-Day Moving Average

### Why This Is Best

1. **Bitcoin-Native**: BTC is the unit of account
2. **Stable Enough**: 30-day MA smooths short-term volatility
3. **Simple**: Easy to understand and implement
4. **Transparent**: All calculations verifiable
5. **Minimal Fiat Dependency**: Only for MA calculation (can use multiple sources or oracles)

### Formula

```
contribution_btc = actual_contribution_btc
btc_price_ma_30d = 30_day_moving_average(btc_price_usd)
contribution_value = contribution_btc * btc_price_ma_30d
vote_weight = sqrt(contribution_value / 1.0)
```

### Example

**Month 1:**
- Contribution: 0.1 BTC
- BTC price: $50,000
- 30-day MA: $55,000
- Value: 0.1 × $55,000 = $5,500
- Weight: √5,500 = 74.2 votes

**Month 2 (BTC drops to $40,000):**
- Contribution: 0.1 BTC
- BTC price: $40,000
- 30-day MA: $52,000 (still high from previous month)
- Value: 0.1 × $52,000 = $5,200
- Weight: √5,200 = 72.1 votes

**Stability**: Weight only drops 2.8% despite 20% price drop

---

## Alternative: Pure BTC with Volatility Acceptance

### Concept

Accept BTC volatility as a feature, not a bug. Governance adapts to Bitcoin's value.

### Formula

```
contribution_btc = actual_contribution_btc
vote_weight = sqrt(contribution_btc / 1.0)
```

### Advantages

- ✅ **Pure Bitcoin**: No fiat dependency whatsoever
- ✅ **Simple**: No price lookups
- ✅ **Bitcoin-aligned**: Governance value = Bitcoin value

### Thresholds Adjust Automatically

If BTC price doubles:
- Same contribution = 2x weight
- But thresholds stay in BTC terms
- **Result**: More participation needed (natural scaling)

### Example

**Low BTC Price ($30,000):**
- 0.1 BTC contribution = 0.316 weight
- Threshold: 100 votes = 10 BTC total needed

**High BTC Price ($90,000):**
- 0.1 BTC contribution = 0.316 weight (same!)
- Threshold: 100 votes = 10 BTC total needed (same!)

**Key**: Thresholds in BTC terms, not dollar terms

---

## Hybrid Recommendation

### Best of Both Worlds

**Use BTC with Moving Average, but make it optional:**

```rust
pub enum WeightingMethod {
    /// Pure BTC (no conversion)
    PureBtc { normalization_btc: f64 },
    
    /// BTC with moving average
    BtcWithMA { ma_window_days: u32, price_source: PriceSource },
    
    /// Hashrate-indexed
    HashrateIndexed { baseline_hashrate: f64 },
    
    /// Hybrid (BTC MA + hashrate)
    Hybrid { ma_window_days: u32, hashrate_weight: f64 },
}

impl WeightingMethod {
    pub async fn calculate_weight(
        &self,
        contribution_btc: f64,
    ) -> Result<f64> {
        match self {
            Self::PureBtc { normalization_btc } => {
                Ok((contribution_btc / normalization_btc).sqrt())
            }
            Self::BtcWithMA { ma_window_days, price_source } => {
                let price_ma = price_source.get_moving_average(*ma_window_days).await?;
                let value = contribution_btc * price_ma;
                Ok((value / 1.0).sqrt())
            }
            Self::HashrateIndexed { baseline_hashrate } => {
                let current_hashrate = get_current_hashrate().await?;
                let index = current_hashrate / baseline_hashrate;
                let value = contribution_btc * 60_000.0 * index;  // Baseline $60k
                Ok((value / 1.0).sqrt())
            }
            Self::Hybrid { ma_window_days, hashrate_weight } => {
                let price_ma = get_price_ma(*ma_window_days).await?;
                let hashrate_growth = get_hashrate_growth().await?;
                let value = contribution_btc * price_ma * hashrate_growth.powf(*hashrate_weight);
                Ok((value / 1.0).sqrt())
            }
        }
    }
}
```

### Configuration

```yaml
# governance/config/governance-weighting.yml

weighting_method: "btc_with_ma"  # or "pure_btc", "hashrate", "hybrid"

btc_with_ma:
  ma_window_days: 30
  price_sources:
    - "coinbase"
    - "kraken"
    - "binance"
  use_median: true  # Use median of sources

pure_btc:
  normalization_btc: 1.0  # 1 BTC = 1.0 weight baseline

hashrate_indexed:
  baseline_hashrate_eh: 500.0  # 500 EH/s
  baseline_price_usd: 60000.0

hybrid:
  ma_window_days: 30
  hashrate_weight: 0.3  # 30% weight on hashrate growth
```

---

## Final Recommendation

### Primary: BTC with 30-Day Moving Average

**Why:**
- Bitcoin-native (BTC is unit of account)
- Stable enough (30-day MA smooths volatility)
- Simple to implement and verify
- Minimal fiat dependency (only for MA, can use oracles)

**Fallback: Pure BTC**
- If price sources unavailable
- Accept volatility as feature
- Thresholds in BTC terms

### Implementation Priority

1. **Phase 1**: Pure BTC (simplest, no dependencies)
2. **Phase 2**: Add BTC with MA (stability)
3. **Phase 3**: Add hashrate indexing (network alignment)
4. **Phase 4**: Hybrid model (best of all)

---

## Summary

**Alternatives to Dollar Pegging:**

1. ✅ **BTC + Moving Average** (Recommended)
   - Bitcoin-native, stable, simple

2. ✅ **Pure BTC** (Simplest)
   - No fiat dependency, accept volatility

3. ✅ **Hashrate-Indexed** (Network-aligned)
   - Tied to Bitcoin security, grows with network

4. ✅ **Subsidy-Indexed** (Halving-aware)
   - Reflects Bitcoin scarcity

5. ✅ **Hybrid** (Best of all)
   - Combines stability + network alignment

**Recommendation**: Start with **BTC + 30-day MA**, with fallback to **Pure BTC** if price sources unavailable.

