# Why Weight Caps Should Be Included

## The Problem

Even with quadratic weighting, a single whale could still dominate:

**Example**:
- Whale contributes 100 BTC → weight = √100 = **10 votes**
- Small contributor: 1 BTC → weight = √1 = **1 vote**
- Ratio: 10:1 (still significant!)

**Without caps**: If total system weight = 1000 votes, whale could have 10%+ influence
**With 5% cap**: Whale max = 50 votes (5% of 1000), ensuring minimum diversity

## Why It's Critical

1. **Prevents Absolute Dominance**: No single entity can control >5% of total weight
2. **Simple Implementation**: Just `min(calculated_weight, max_weight)` - very easy
3. **Low Overhead**: Only need to track total system weight, recalculate monthly
4. **Maintains Quadratic Benefits**: Still rewards contributions, just caps the maximum

## Implementation Complexity

**Very Low**:
```rust
fn apply_weight_cap(
    calculated_weight: f64,
    total_system_weight: f64,
    cap_percentage: f64,  // 0.05 = 5%
) -> f64 {
    let max_weight = total_system_weight * cap_percentage;
    calculated_weight.min(max_weight)
}
```

**That's it!** Just one line of logic.

## Why Skip Others

- **Time Decay**: Adds complexity, can add later if needed
- **Cooling-Off**: Prevents vote buying but adds tracking overhead
- **Dollar Conversion**: BTC volatility is a concern, but BTC-based is simpler for MVP
- **Multi-Round**: Only needed for major changes, can add later

## Recommendation

**Include Weight Caps** - It's simple, effective, and prevents a real attack vector.

