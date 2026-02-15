# Sprint 11: Correlation & Beta Analysis - In Progress

**Status**: 🟡 Implementation started
**Start Date**: 2026-02-14
**Estimate**: 1.5 weeks (scaled down to core features)

## Overview

Enhance correlation analysis and beta calculations with multi-benchmark support, risk decomposition, and improved visualizations.

**Current State**: Basic correlation matrix exists, beta vs SPY only
**Target State**: Enhanced correlation with heatmap data, multi-benchmark beta, risk decomposition

## Scope for This Sprint

Given the complexity, we'll focus on **core backend functionality** and **basic frontend updates**. Advanced visualizations (interactive heatmap, scatter plots) can be added in future enhancements.

## Tasks Breakdown

### Backend Tasks (Core Features)

#### 1. Enhanced Correlation Matrix
- [ ] Add `matrix_2d: Vec<Vec<f64>>` to CorrelationMatrix model
- [ ] Convert correlation pairs to 2D matrix format
- [ ] Ensure proper ordering (same ticker order for rows/columns)
- [ ] Keep backward compatibility with correlation pairs

#### 2. Multi-Benchmark Beta
- [ ] Add `beta_spy`, `beta_qqq`, `beta_iwm` fields to PositionRisk
- [ ] Compute beta against multiple benchmarks
- [ ] Add benchmark parameter to risk calculations
- [ ] Return all betas in API response

#### 3. Systematic vs Idiosyncratic Risk
- [ ] Create `RiskDecomposition` model
- [ ] Calculate systematic risk (beta-related)
- [ ] Calculate idiosyncratic risk (stock-specific)
- [ ] Add to risk assessment response

#### 4. Enhanced Diversification
- [ ] Calculate correlation-adjusted diversification score
- [ ] Compute effective number of positions
- [ ] Account for correlation in diversification benefit

### Frontend Tasks (Basic Updates)

#### 1. TypeScript Types
- [ ] Add `matrix_2d` to CorrelationMatrix type
- [ ] Add multi-benchmark beta fields
- [ ] Add RiskDecomposition type

#### 2. Display Updates
- [ ] Show multi-benchmark betas in RiskMetricsPanel
- [ ] Display risk decomposition (systematic vs idiosyncratic)
- [ ] Show correlation-adjusted diversification score

#### 3. Advanced Visualizations (Deferred)
- [ ] Interactive correlation heatmap (Sprint 22 or future)
- [ ] Beta comparison chart across benchmarks (Sprint 22 or future)
- [ ] Scatter plot for correlation pairs (Sprint 22 or future)

## Technical Details

### Multi-Benchmark Beta
```rust
pub struct PositionRisk {
    // ... existing fields ...
    pub beta: Option<f64>,  // Keep for backward compatibility (SPY)
    pub beta_spy: Option<f64>,  // Explicit SPY beta
    pub beta_qqq: Option<f64>,  // Nasdaq 100 beta
    pub beta_iwm: Option<f64>,  // Russell 2000 beta
}
```

### Risk Decomposition
```rust
pub struct RiskDecomposition {
    pub systematic_risk: f64,     // Variance explained by market
    pub idiosyncratic_risk: f64,  // Stock-specific variance
    pub r_squared: f64,           // % variance explained by beta
}
```

Formula:
- Systematic Risk = β² × σ²_market
- Idiosyncratic Risk = σ²_total - σ²_systematic
- R² from regression

### Correlation-Adjusted Diversification
```
Effective N = 1 / (avg_weight² + (1 - avg_weight) × avg_correlation)
```

Accounts for both concentration and correlation.

## Files to Modify

### Backend
1. `src/models/risk.rs` - Add RiskDecomposition, enhance PositionRisk
2. `src/services/risk_service.rs` - Multi-benchmark beta, risk decomposition
3. `src/routes/risk.rs` - Update correlation matrix generation

### Frontend
4. `src/types.ts` - Update types
5. `src/components/RiskMetricsPanel.tsx` - Display multi-benchmark betas
6. `src/components/PortfolioRiskOverview.tsx` - Show risk decomposition

## Success Criteria

- [ ] Backend compiles with no errors
- [ ] Multi-benchmark beta calculated (SPY, QQQ, IWM)
- [ ] Systematic vs idiosyncratic risk computed
- [ ] Correlation matrix includes 2D array format
- [ ] Frontend displays new beta metrics
- [ ] Risk decomposition shown in UI

## Deferred to Future Sprints

- Interactive correlation heatmap with D3/Recharts
- Scatter plot visualization for correlation pairs
- Cluster analysis for highly correlated groups
- Rolling beta charts (30/60/90 day windows)
- Beta comparison across different time periods

---

**Note**: Focusing on analytical foundation. Visualizations can be enhanced iteratively.
