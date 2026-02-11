# Portfolio Optimization Suggestions - Implementation Specification

**Feature:** Portfolio Optimization Suggestions
**Effort:** 10+ hours
**Status:** In Progress
**Started:** 2026-02-10

---

## Overview

Portfolio Optimization Suggestions is an intelligent advisory system that analyzes portfolios and recommends specific actions to improve risk/return profiles. It provides data-driven rebalancing recommendations with "what-if" scenario modeling.

---

## User Experience

### What Users See

Users receive actionable portfolio recommendations like:

```
⚠️ High Concentration Risk
Your TSLA position is 35% of your portfolio (risk score: 78/100)

Recommendation: Reduce TSLA by 15% ($25,000)
Expected Impact:
  • Portfolio risk: 68 → 58 (-15%)
  • Portfolio volatility: 28% → 23%
  • Maintain similar returns with lower risk

Suggested Action: Sell 50 shares of TSLA, reinvest in VTI or QQQ
```

### Key Features for End Users

1. **Automatic Portfolio Analysis**
   - Concentration risk detection
   - Risk imbalance identification
   - Diversification gap analysis
   - Risk/return efficiency scoring

2. **Actionable Recommendations**
   - Specific position adjustments
   - Expected impact metrics
   - Clear reasoning and education
   - Alternative suggestions

3. **"What-If" Calculator**
   - Interactive sliders for position sizes
   - Real-time impact preview
   - Multiple scenario comparison
   - Save/load scenarios

4. **Risk Analysis**
   - Before/after comparison
   - Risk contribution breakdown
   - Correlation impact
   - Historical backtesting

---

## Real-World Example

**Current Portfolio:** $300K
- TSLA: $100K (33%) - Risk Score: 82
- AAPL: $60K (20%) - Risk Score: 58
- MSFT: $50K (17%) - Risk Score: 52
- NVDA: $40K (13%) - Risk Score: 79
- Cash: $50K (17%)

**System Analysis:**
1. ⚠️ Concentration Risk: TSLA at 33% (recommended max: 15%)
2. ⚠️ Tech Heavy: 83% in tech stocks - highly correlated
3. ⚠️ High Risk Positions: TSLA and NVDA both very volatile

**Optimization Suggestions:**

```
🎯 Recommended Changes (3 actions)

1. Reduce TSLA position
   Sell: $70K TSLA → New allocation: 10% ($30K)
   Why: Lower concentration risk, reduce volatility

2. Trim NVDA
   Sell: $15K NVDA → New allocation: 8% ($25K)
   Why: Second highest risk contributor

3. Diversify into defensive assets
   Buy: $40K BND (Bond ETF)
   Buy: $25K VTI (Total Market)
   Buy: $20K XLE (Energy sector)

Expected Results:
├─ Portfolio Risk: 72 → 54 (-25%)
├─ Volatility: 31% → 21% (-32%)
├─ Max Drawdown: -38% → -24% (-37%)
├─ Sharpe Ratio: 0.78 → 1.24 (+59%)
└─ Diversification Score: 3.2 → 7.8

[Preview in What-If Calculator] [Apply All] [Customize]
```

---

## Technical Implementation

### Architecture

```
┌─────────────────────────────────────────┐
│         Frontend (React/TS)             │
│  ┌────────────────────────────────────┐ │
│  │ PortfolioOptimizer Component       │ │
│  │  - Display recommendations         │ │
│  │  - What-if calculator UI           │ │
│  │  - Scenario builder                │ │
│  └────────────────────────────────────┘ │
└─────────────────┬───────────────────────┘
                  │
                  │ REST API
                  ▼
┌─────────────────────────────────────────┐
│         Backend (Rust)                  │
│  ┌────────────────────────────────────┐ │
│  │ Optimization Service                │ │
│  │  - Analyze portfolio                │ │
│  │  - Calculate optimal allocations    │ │
│  │  - Generate recommendations         │ │
│  │  - Simulate scenarios               │ │
│  └────────────────────────────────────┘ │
│  ┌────────────────────────────────────┐ │
│  │ Algorithms                          │ │
│  │  - Modern Portfolio Theory          │ │
│  │  - Risk Parity                      │ │
│  │  - Concentration detection          │ │
│  │  - Correlation analysis             │ │
│  └────────────────────────────────────┘ │
└─────────────────────────────────────────┘
```

### Core Algorithms

#### 1. Concentration Risk Detection
```rust
fn detect_concentration_risk(portfolio: &Portfolio) -> Vec<ConcentrationAlert> {
    // Check individual position sizes
    // Threshold: Any position > 15% of portfolio
    // Severity: 15-20% = warning, 20-30% = high, >30% = critical
}
```

#### 2. Risk Contribution Analysis
```rust
fn calculate_risk_contribution(portfolio: &Portfolio) -> Vec<RiskContribution> {
    // Calculate each position's contribution to total portfolio risk
    // Risk contribution = position_weight × position_volatility × correlation
    // Flag positions contributing >20% of total risk
}
```

#### 3. Diversification Score
```rust
fn calculate_diversification_score(portfolio: &Portfolio) -> f64 {
    // Score from 0-10 based on:
    // - Number of positions (more is better, up to 20)
    // - Sector diversity (Herfindahl index)
    // - Correlation structure (lower average correlation is better)
    // - Asset class mix (stocks, bonds, commodities)
}
```

#### 4. Optimization Algorithm
```rust
fn optimize_portfolio(
    current: &Portfolio,
    constraints: &Constraints,
    objective: ObjectiveFunction
) -> OptimizationResult {
    // Use quadratic programming to minimize:
    // minimize: portfolio_variance
    // subject to:
    //   - expected_return >= target_return
    //   - sum(weights) = 1.0
    //   - weight[i] >= min_weight[i]
    //   - weight[i] <= max_weight[i]
    //   - position_count <= max_positions
}
```

### Data Structures

```rust
// Backend models
pub struct OptimizationRecommendation {
    pub recommendation_type: RecommendationType,
    pub severity: Severity,
    pub affected_positions: Vec<PositionAdjustment>,
    pub rationale: String,
    pub expected_impact: ExpectedImpact,
    pub suggested_actions: Vec<Action>,
}

pub struct PositionAdjustment {
    pub ticker: String,
    pub current_value: f64,
    pub current_weight: f64,
    pub recommended_value: f64,
    pub recommended_weight: f64,
    pub action: AdjustmentAction, // Buy, Sell, Hold
    pub amount_change: f64,
}

pub struct ExpectedImpact {
    pub risk_score_before: f64,
    pub risk_score_after: f64,
    pub volatility_before: f64,
    pub volatility_after: f64,
    pub sharpe_before: f64,
    pub sharpe_after: f64,
    pub diversification_before: f64,
    pub diversification_after: f64,
}

pub enum RecommendationType {
    ReduceConcentration,
    RebalanceSectors,
    ReduceRisk,
    ImproveEfficiency,
    IncreaseDiversification,
}

pub enum Severity {
    Info,      // FYI, no urgency
    Warning,   // Should address soon
    High,      // Address within week
    Critical,  // Address immediately
}
```

### API Endpoints

```rust
// GET /api/optimization/portfolios/:portfolio_id
// Returns: OptimizationAnalysis with recommendations

// POST /api/optimization/portfolios/:portfolio_id/simulate
// Body: { adjustments: [{ ticker, new_weight }] }
// Returns: SimulationResult with projected metrics

// GET /api/optimization/portfolios/:portfolio_id/scenarios
// Returns: List of saved optimization scenarios

// POST /api/optimization/portfolios/:portfolio_id/scenarios
// Body: { name, adjustments }
// Returns: Saved scenario
```

---

## Implementation Phases

### Phase 1: Core Analysis (3-4 hours)
- Concentration risk detection
- Risk contribution analysis
- Diversification scoring
- Basic recommendations

### Phase 2: Optimization Engine (3-4 hours)
- Portfolio optimization algorithm
- Constraint handling
- Multiple objective functions
- Rebalancing suggestions

### Phase 3: What-If Calculator (2-3 hours)
- Scenario simulation API
- Frontend interactive calculator
- Real-time impact preview
- Scenario save/load

### Phase 4: UI & Integration (2-3 hours)
- Recommendations display component
- Action cards and alerts
- Integration with Portfolio Overview
- User preference settings

---

## User Benefits

1. **No Guesswork**: Data-driven recommendations instead of emotional decisions
2. **Risk Management**: Proactive alerts before problems become losses
3. **Education**: Understand WHY changes are suggested
4. **Confidence**: See projected outcomes before making trades
5. **Cost Savings**: Avoid concentration mistakes that lead to big losses

---

## When Users Would Use This

- **Monthly review**: Check for optimization opportunities
- **After big gains**: When one stock grows too large
- **Market volatility**: Rebalance during turbulent times
- **Before retirement**: Shift to lower-risk allocation
- **After adding money**: Efficiently deploy new capital

---

## Success Metrics

- Users receive actionable recommendations
- What-if calculator shows accurate projections
- Recommendations include clear rationale
- Impact metrics are easy to understand
- Users can save and compare scenarios

---

## Technical Dependencies

### Backend (Rust)
- `ndarray` - Matrix operations for optimization
- `optimization` or `argmin` - Quadratic programming
- Existing risk metrics from Phase 1-3

### Frontend (TypeScript/React)
- Recharts - Visualization
- Material-UI Sliders - What-if calculator
- Existing portfolio data

---

## Future Enhancements

- **Machine Learning**: Predict optimal rebalancing timing
- **Tax Optimization**: Minimize capital gains tax impact
- **Goal-Based**: Optimize for retirement date, income needs
- **Automated Rebalancing**: One-click apply recommendations
- **Backtesting**: Show how suggestions would have performed historically
- **Monte Carlo Simulation**: Stress test optimization under various market scenarios

---

## Notes

- Start with rule-based recommendations (simpler, faster)
- Add optimization algorithms in Phase 2
- Focus on user education - explain every suggestion
- Use real portfolio data for testing
- Consider trading costs in recommendations
- Respect user preferences and constraints
