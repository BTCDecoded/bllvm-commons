# Fairness Analysis: What Should We Include?

## Fairness Concerns

### 1. Cooling-Off Periods (#3) - **HIGH PRIORITY**

**Problem**: Vote buying - someone makes a large contribution right before a vote to influence it.

**Example Attack**:
- Proposal comes up for vote
- Whale contributes 10 BTC right before vote
- Gets √10 = 3.16 votes immediately
- Influences the outcome unfairly

**Solution**: Require 30-day delay for large contributions before they count toward voting.

**Implementation Complexity**: **LOW**
- Just track contribution age
- Check if contribution >= threshold AND age >= 30 days
- Simple boolean check

**BTC Threshold**: Use 0.1 BTC (≈$9,000 at current prices) instead of USD

**Recommendation**: ✅ **INCLUDE** - Prevents vote buying, simple to implement

---

### 2. Diversity Requirements (#4) - **MEDIUM PRIORITY**

**Problem**: Single tier could dominate (e.g., only miners vote, community excluded).

**Solution**: Require at least 2 of 3 tiers for major decisions (Tier 4+).

**Implementation Complexity**: **MEDIUM**
- Need to track vote sources
- Check tier diversity before approval
- Could block legitimate proposals if one tier abstains

**Recommendation**: ⚠️ **DEFER** - Can add later if we see single-tier dominance. For MVP, quadratic weighting + caps should be enough.

---

### 3. Time Decay (#1) - **LOW PRIORITY FOR MVP**

**Problem**: Permanent power structures - old contributors never lose influence.

**Solution**: Contributions decay over time (180-365 days).

**Implementation Complexity**: **MEDIUM**
- Need to track contribution age
- Calculate decay factor
- More complex than cooling-off

**Recommendation**: ⚠️ **DEFER** - Can add later if power concentration becomes an issue. For MVP, weight caps should prevent worst-case scenarios.

---

## Recommendation

**Include**: **Cooling-Off Periods** with BTC threshold (0.1 BTC, 30 days)

**Why**:
- Prevents vote buying (real fairness issue)
- Simple to implement (just age check)
- Low overhead (track contribution timestamp)
- High impact (prevents timing attacks)

**Skip for now**:
- Diversity requirements (can add if needed)
- Time decay (can add if needed)

