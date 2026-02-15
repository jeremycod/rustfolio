# Rustfolio Implementation Roadmap

**Document Version:** 1.0
**Created:** February 14, 2026
**Status:** Active Development

---

## Overview

This roadmap outlines the implementation plan to complete Phase 3 & 3C of Rustfolio's risk management system. The focus is on the three critical missing features that will bring the system from 72% to 95%+ completion.

---

## Sprint 1: Portfolio-Level Risk Aggregation (P0)

**Duration:** 2-3 days
**Effort:** 4-6 hours
**Priority:** CRITICAL - This is the highest-value missing feature

### Objective
Enable users to see portfolio-wide risk metrics, not just individual position risk. This is essential for proper portfolio management and diversification analysis.

### Backend Implementation

#### 1. Database Schema (if needed)
**File:** `backend/migrations/[timestamp]_add_portfolio_risk_aggregation.sql`

No new tables needed - will aggregate from existing `risk_snapshots` table.

#### 2. Models
**File:** `backend/src/models/risk.rs`

Add new structs:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortfolioRiskAggregation {
    pub portfolio_id: String,
    pub as_of_date: NaiveDate,

    // Aggregated metrics
    pub total_value: f64,
    pub weighted_volatility: f64,
    pub portfolio_beta: f64,
    pub portfolio_sharpe_ratio: Option<f64>,
    pub max_drawdown: f64,
    pub portfolio_var_95: f64,

    // Risk score
    pub overall_risk_score: f64,
    pub overall_risk_level: RiskLevel,

    // Contribution analysis
    pub position_contributions: Vec<PositionRiskContribution>,

    // Metadata
    pub positions_analyzed: i32,
    pub positions_excluded: i32,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionRiskContribution {
    pub ticker: String,
    pub holding_name: Option<String>,
    pub position_value: f64,
    pub weight: f64,

    // Risk metrics
    pub volatility: f64,
    pub beta: Option<f64>,
    pub risk_score: f64,

    // Contribution to portfolio risk
    pub volatility_contribution: f64,
    pub beta_contribution: Option<f64>,
    pub risk_contribution_score: f64,

    // Percentage of total portfolio risk
    pub risk_contribution_pct: f64,
}
```

#### 3. Service Layer
**File:** `backend/src/services/risk_aggregation_service.rs` (NEW)

Functions needed:
```rust
/// Calculate portfolio-level risk aggregation
pub async fn aggregate_portfolio_risk(
    pool: &PgPool,
    portfolio_id: &str,
) -> Result<PortfolioRiskAggregation, AppError>

/// Calculate weighted average volatility
fn calculate_weighted_volatility(
    positions: &[PositionRisk]
) -> f64

/// Calculate portfolio beta vs market
fn calculate_portfolio_beta(
    positions: &[PositionRisk]
) -> f64

/// Calculate risk contribution for each position
fn calculate_risk_contributions(
    positions: &[PositionRisk],
    portfolio_volatility: f64,
) -> Vec<PositionRiskContribution>

/// Calculate portfolio Sharpe ratio
fn calculate_portfolio_sharpe(
    positions: &[PositionRisk],
    risk_free_rate: f64,
) -> Option<f64>
```

**Algorithm Notes:**
- **Weighted Volatility:** `Σ(weight_i × volatility_i)`
- **Portfolio Beta:** `Σ(weight_i × beta_i)`
- **Risk Contribution:** `(position_volatility × position_weight) / portfolio_volatility`
- **Sharpe Ratio:** `(portfolio_return - risk_free_rate) / portfolio_volatility`

#### 4. API Routes
**File:** `backend/src/routes/risk.rs`

Add route:
```rust
// Get portfolio-level risk aggregation
// GET /api/risk/portfolios/:portfolio_id/aggregate
async fn get_portfolio_risk_aggregation(
    State(state): State<AppState>,
    Path(portfolio_id): Path<String>,
) -> Result<Json<PortfolioRiskAggregation>, AppError>
```

#### 5. Module Registration
**Files to update:**
- `backend/src/services/mod.rs` - Add `pub mod risk_aggregation_service;`
- `backend/src/routes/risk.rs` - Register new route

### Frontend Implementation

#### 1. TypeScript Types
**File:** `frontend/src/types.ts`

Add interfaces matching backend models:
```typescript
export interface PortfolioRiskAggregation {
  portfolio_id: string;
  as_of_date: string;
  total_value: number;
  weighted_volatility: number;
  portfolio_beta: number;
  portfolio_sharpe_ratio: number | null;
  max_drawdown: number;
  portfolio_var_95: number;
  overall_risk_score: number;
  overall_risk_level: RiskLevel;
  position_contributions: PositionRiskContribution[];
  positions_analyzed: number;
  positions_excluded: number;
  last_updated: string;
}

export interface PositionRiskContribution {
  ticker: string;
  holding_name: string | null;
  position_value: number;
  weight: number;
  volatility: number;
  beta: number | null;
  risk_score: number;
  volatility_contribution: number;
  beta_contribution: number | null;
  risk_contribution_score: number;
  risk_contribution_pct: number;
}
```

#### 2. API Client
**File:** `frontend/src/lib/endpoints.ts`

Add function:
```typescript
export async function getPortfolioRiskAggregation(
  portfolioId: string
): Promise<PortfolioRiskAggregation> {
  const response = await api.get(
    `/api/risk/portfolios/${portfolioId}/aggregate`
  );
  return response.data;
}
```

#### 3. Component: PortfolioRiskOverview.tsx
**File:** `frontend/src/components/PortfolioRiskOverview.tsx` (NEW)

**Structure:**
```tsx
export default function PortfolioRiskOverview({ portfolioId }: Props) {
  // Fetch data with TanStack Query
  const { data, isLoading, error } = useQuery({
    queryKey: ['portfolioRiskAggregation', portfolioId],
    queryFn: () => getPortfolioRiskAggregation(portfolioId),
    enabled: !!portfolioId,
  });

  return (
    <Box>
      {/* Summary Card */}
      <Card>
        <CardHeader title="Portfolio Risk Overview" />
        <CardContent>
          <Grid container spacing={3}>
            {/* Overall Risk Badge */}
            <Grid item xs={12} md={4}>
              <RiskScoreGauge
                score={data.overall_risk_score}
                level={data.overall_risk_level}
              />
            </Grid>

            {/* Key Metrics */}
            <Grid item xs={12} md={8}>
              <MetricsGrid>
                <MetricCard label="Weighted Volatility" value={...} />
                <MetricCard label="Portfolio Beta" value={...} />
                <MetricCard label="Max Drawdown" value={...} />
                <MetricCard label="Sharpe Ratio" value={...} />
                <MetricCard label="Value at Risk (95%)" value={...} />
              </MetricsGrid>
            </Grid>
          </Grid>
        </CardContent>
      </Card>

      {/* Risk Contribution Chart */}
      <Card sx={{ mt: 2 }}>
        <CardHeader title="Risk Contribution by Position" />
        <CardContent>
          <ResponsiveContainer width="100%" height={400}>
            <BarChart data={data.position_contributions}>
              <XAxis dataKey="ticker" />
              <YAxis label="Risk Contribution %" />
              <Tooltip />
              <Bar dataKey="risk_contribution_pct" fill="#ff6b6b" />
            </BarChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>

      {/* Top Risk Contributors Table */}
      <Card sx={{ mt: 2 }}>
        <CardHeader title="Top Risk Contributors" />
        <CardContent>
          <TableContainer>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Ticker</TableCell>
                  <TableCell>Weight</TableCell>
                  <TableCell>Volatility</TableCell>
                  <TableCell>Risk Score</TableCell>
                  <TableCell>Risk Contribution</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {sortedContributions.map(position => (
                  <TableRow key={position.ticker}>
                    <TableCell>{position.ticker}</TableCell>
                    <TableCell>{formatPercent(position.weight)}</TableCell>
                    <TableCell>{formatPercent(position.volatility)}</TableCell>
                    <TableCell>
                      <RiskBadge level={getRiskLevel(position.risk_score)} />
                    </TableCell>
                    <TableCell>
                      {formatPercent(position.risk_contribution_pct)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </CardContent>
      </Card>
    </Box>
  );
}
```

#### 4. Component: RiskScoreGauge.tsx
**File:** `frontend/src/components/RiskScoreGauge.tsx` (NEW)

A circular gauge visualization showing risk score 0-100 with color zones:
- 0-30: Green (Low risk)
- 31-60: Yellow (Moderate risk)
- 61-100: Red (High risk)

Use Recharts RadialBarChart or create custom SVG gauge.

#### 5. Integration
**File:** `frontend/src/components/PortfolioOverview.tsx` or Dashboard

Add new tab or section for "Portfolio Risk":
```tsx
<Tab label="Portfolio Risk" value="risk" />

{tabValue === 'risk' && (
  <PortfolioRiskOverview portfolioId={selectedPortfolioId} />
)}
```

### Testing Checklist

#### Backend
- [ ] Empty portfolio (no positions)
- [ ] Single position portfolio
- [ ] Portfolio with 10+ positions
- [ ] Portfolio with only mutual funds (no risk data)
- [ ] Portfolio with mixed equities and excluded assets
- [ ] Verify weighted calculations are correct
- [ ] Verify risk contributions sum to 100%
- [ ] API response time <500ms for 50 positions

#### Frontend
- [ ] Loading state displays correctly
- [ ] Error state handles API failures gracefully
- [ ] Data displays with correct formatting
- [ ] Chart renders properly with various data sizes
- [ ] Table sorts by risk contribution correctly
- [ ] Responsive layout on mobile/tablet
- [ ] Risk gauge animates smoothly
- [ ] Color coding matches risk levels

### Success Criteria
- ✅ Users can view overall portfolio risk score
- ✅ Users can see which positions contribute most to risk
- ✅ Metrics are calculated correctly (weighted averages)
- ✅ UI is intuitive and visually clear
- ✅ Performance is acceptable (<500ms backend, <100ms frontend render)

---

## Sprint 2: Risk Threshold Settings (P1)

**Duration:** 1-2 days
**Effort:** 3-4 hours
**Priority:** HIGH - Enables personalization

### Objective
Allow users to customize risk thresholds per portfolio based on their risk tolerance. Provide preview of which positions would trigger warnings at current thresholds.

### Backend Implementation

#### 1. Database Migration
**File:** `backend/migrations/[timestamp]_add_risk_threshold_settings.sql`

```sql
CREATE TABLE risk_threshold_settings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,

    -- Thresholds
    volatility_warning_threshold NUMERIC(5,2) NOT NULL DEFAULT 30.0,
    volatility_critical_threshold NUMERIC(5,2) NOT NULL DEFAULT 50.0,

    drawdown_warning_threshold NUMERIC(5,2) NOT NULL DEFAULT -20.0,
    drawdown_critical_threshold NUMERIC(5,2) NOT NULL DEFAULT -35.0,

    beta_warning_threshold NUMERIC(4,2) NOT NULL DEFAULT 1.5,
    beta_critical_threshold NUMERIC(4,2) NOT NULL DEFAULT 2.0,

    risk_score_warning_threshold NUMERIC(5,2) NOT NULL DEFAULT 60.0,
    risk_score_critical_threshold NUMERIC(5,2) NOT NULL DEFAULT 80.0,

    var_warning_threshold NUMERIC(5,2) NOT NULL DEFAULT -5.0,
    var_critical_threshold NUMERIC(5,2) NOT NULL DEFAULT -10.0,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    UNIQUE(portfolio_id)
);

CREATE INDEX idx_risk_thresholds_portfolio ON risk_threshold_settings(portfolio_id);
```

#### 2. Models
**File:** `backend/src/models/risk_thresholds.rs` (NEW or add to risk.rs)

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskThresholdSettings {
    pub id: String,
    pub portfolio_id: String,

    pub volatility_warning_threshold: f64,
    pub volatility_critical_threshold: f64,

    pub drawdown_warning_threshold: f64,
    pub drawdown_critical_threshold: f64,

    pub beta_warning_threshold: f64,
    pub beta_critical_threshold: f64,

    pub risk_score_warning_threshold: f64,
    pub risk_score_critical_threshold: f64,

    pub var_warning_threshold: f64,
    pub var_critical_threshold: f64,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateRiskThresholds {
    pub volatility_warning_threshold: Option<f64>,
    pub volatility_critical_threshold: Option<f64>,
    // ... other optional fields
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdViolation {
    pub ticker: String,
    pub holding_name: Option<String>,
    pub metric_name: String,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub severity: ViolationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ViolationSeverity {
    Warning,
    Critical,
}
```

#### 3. Database Queries
**File:** `backend/src/db/risk_threshold_queries.rs` (NEW)

```rust
pub async fn get_thresholds(
    pool: &PgPool,
    portfolio_id: &str,
) -> Result<RiskThresholdSettings, sqlx::Error>

pub async fn upsert_thresholds(
    pool: &PgPool,
    portfolio_id: &str,
    thresholds: &UpdateRiskThresholds,
) -> Result<RiskThresholdSettings, sqlx::Error>

pub async fn get_or_create_default_thresholds(
    pool: &PgPool,
    portfolio_id: &str,
) -> Result<RiskThresholdSettings, sqlx::Error>
```

#### 4. Service Layer
**File:** `backend/src/services/risk_threshold_service.rs` (NEW)

```rust
/// Check which positions violate thresholds
pub async fn check_threshold_violations(
    pool: &PgPool,
    portfolio_id: &str,
) -> Result<Vec<ThresholdViolation>, AppError>

/// Get default thresholds
pub fn get_default_thresholds() -> RiskThresholdSettings
```

#### 5. API Routes
**File:** `backend/src/routes/risk.rs`

```rust
// GET /api/risk/portfolios/:portfolio_id/thresholds
async fn get_risk_thresholds(...)

// POST /api/risk/portfolios/:portfolio_id/thresholds
async fn update_risk_thresholds(...)

// GET /api/risk/portfolios/:portfolio_id/threshold-violations
async fn get_threshold_violations(...)
```

### Frontend Implementation

#### 1. TypeScript Types
**File:** `frontend/src/types.ts`

```typescript
export interface RiskThresholdSettings {
  id: string;
  portfolio_id: string;
  volatility_warning_threshold: number;
  volatility_critical_threshold: number;
  drawdown_warning_threshold: number;
  drawdown_critical_threshold: number;
  beta_warning_threshold: number;
  beta_critical_threshold: number;
  risk_score_warning_threshold: number;
  risk_score_critical_threshold: number;
  var_warning_threshold: number;
  var_critical_threshold: number;
  created_at: string;
  updated_at: string;
}

export interface ThresholdViolation {
  ticker: string;
  holding_name: string | null;
  metric_name: string;
  metric_value: number;
  threshold_value: number;
  severity: 'warning' | 'critical';
}
```

#### 2. API Client
**File:** `frontend/src/lib/endpoints.ts`

```typescript
export async function getRiskThresholds(portfolioId: string) {
  const response = await api.get(`/api/risk/portfolios/${portfolioId}/thresholds`);
  return response.data;
}

export async function updateRiskThresholds(
  portfolioId: string,
  thresholds: Partial<RiskThresholdSettings>
) {
  const response = await api.post(
    `/api/risk/portfolios/${portfolioId}/thresholds`,
    thresholds
  );
  return response.data;
}

export async function getThresholdViolations(portfolioId: string) {
  const response = await api.get(
    `/api/risk/portfolios/${portfolioId}/threshold-violations`
  );
  return response.data;
}
```

#### 3. Component: RiskThresholdSettings.tsx
**File:** `frontend/src/components/RiskThresholdSettings.tsx` (NEW)

```tsx
export default function RiskThresholdSettings({ portfolioId }: Props) {
  const [thresholds, setThresholds] = useState<RiskThresholdSettings | null>(null);

  // Fetch current thresholds
  const { data } = useQuery({
    queryKey: ['riskThresholds', portfolioId],
    queryFn: () => getRiskThresholds(portfolioId),
  });

  // Mutation to update thresholds
  const updateMutation = useMutation({
    mutationFn: (newThresholds: Partial<RiskThresholdSettings>) =>
      updateRiskThresholds(portfolioId, newThresholds),
    onSuccess: () => {
      queryClient.invalidateQueries(['riskThresholds', portfolioId]);
      queryClient.invalidateQueries(['thresholdViolations', portfolioId]);
    },
  });

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
      <DialogTitle>Risk Threshold Settings</DialogTitle>
      <DialogContent>
        <Grid container spacing={3}>
          {/* Volatility Thresholds */}
          <Grid item xs={12}>
            <Typography variant="h6">Volatility</Typography>
            <Box sx={{ px: 2 }}>
              <Typography>Warning: {volatilityWarning}%</Typography>
              <Slider
                value={volatilityWarning}
                onChange={(e, val) => setVolatilityWarning(val as number)}
                min={10}
                max={100}
                marks={[
                  { value: 20, label: '20%' },
                  { value: 50, label: '50%' },
                  { value: 80, label: '80%' },
                ]}
              />

              <Typography>Critical: {volatilityCritical}%</Typography>
              <Slider
                value={volatilityCritical}
                onChange={(e, val) => setVolatilityCritical(val as number)}
                min={20}
                max={150}
              />
            </Box>
          </Grid>

          {/* Similar sections for Drawdown, Beta, Risk Score, VaR */}

        </Grid>

        {/* Preview Section */}
        <Box sx={{ mt: 3 }}>
          <Typography variant="h6">Positions Exceeding Thresholds</Typography>
          <ThresholdViolationsPreview
            portfolioId={portfolioId}
            thresholds={currentThresholds}
          />
        </Box>
      </DialogContent>

      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button onClick={handleReset}>Reset to Defaults</Button>
        <Button onClick={handleSave} variant="contained">
          Save Changes
        </Button>
      </DialogActions>
    </Dialog>
  );
}
```

#### 4. Integration
Add settings icon/button to Portfolio Overview or Risk Analysis page:
```tsx
<IconButton onClick={() => setSettingsOpen(true)}>
  <SettingsIcon />
</IconButton>

<RiskThresholdSettings
  portfolioId={portfolioId}
  open={settingsOpen}
  onClose={() => setSettingsOpen(false)}
/>
```

### Testing Checklist
- [ ] Default thresholds created on first access
- [ ] Slider changes update preview immediately
- [ ] Save persists to database
- [ ] Reset to defaults works correctly
- [ ] Violations preview shows correct positions
- [ ] Multiple portfolios have independent settings
- [ ] Validation prevents invalid threshold combinations

### Success Criteria
- ✅ Users can customize all risk thresholds
- ✅ Preview shows which positions exceed thresholds
- ✅ Settings persist across sessions
- ✅ Each portfolio has independent thresholds
- ✅ Reset to defaults option available

---

## Sprint 3: Code Cleanup (P2)

**Duration:** 0.5-1 day
**Effort:** 2-3 hours
**Priority:** MEDIUM - Technical debt

### Objective
Remove all compiler warnings and unused code to improve codebase quality and maintainability.

### Tasks

#### 1. Remove Unused Imports
- [ ] position_service.rs:7 - Remove `CreatePortfolio`, `Portfolio`, `UpdatePortfolio`
- [ ] errors.rs:2 - Remove `axum::Json`
- [ ] risk_service.rs:10 - Remove `warn`
- [ ] holding_snapshot_queries.rs:1 - Remove `bigdecimal::BigDecimal`

#### 2. Remove Dead Code
- [ ] price_queries.rs:7 - Remove or use `insert_many` function
- [ ] main.rs:87 - Remove or use `health_check` function
- [ ] main.rs:91 - Remove or use `root` function
- [ ] position_service.rs - Remove unused functions: `create`, `list`, `fetch_one`, `delete`, `update`
- [ ] holding_snapshot_queries.rs:7 - Remove or use `create` function

#### 3. Fix Unused Variables
- [ ] alphavantage.rs:151 - Prefix `note` with underscore: `_note`

#### 4. Fix Naming Conventions
- [ ] alphavantage.rs:42 - Rename `marketOpen` → `market_open`
- [ ] alphavantage.rs:44 - Rename `marketClose` → `market_close`
- [ ] alphavantage.rs:50 - Rename `matchScore` → `match_score`
- [ ] alphavantage.rs:100 - Rename `responseWrapper` → `response_wrapper`

#### 5. Remove Unused Enum Variants
- [ ] errors.rs:21 - Remove `AppError::Unauthorized` or use it

### Verification
Run after cleanup:
```bash
cargo clippy --all-targets --all-features
cargo build
```

Should have 0 warnings.

---

## Sprint 4 (Optional): Downloadable Reports (P1)

**Duration:** 2-3 days
**Effort:** 5-7 hours
**Priority:** HIGH VALUE - User requested feature

See PHASE3_CONSOLIDATED_STATUS.md for detailed implementation plan.

---

## Success Metrics

### Sprint 1 Success
- [ ] Portfolio risk aggregation endpoint returns valid data
- [ ] Frontend component renders without errors
- [ ] Risk contribution calculations are accurate
- [ ] Performance meets targets (<500ms API, smooth rendering)

### Sprint 2 Success
- [ ] Threshold settings persist correctly
- [ ] Preview shows correct violations
- [ ] UI is intuitive and responsive
- [ ] Each portfolio has independent settings

### Sprint 3 Success
- [ ] Zero compiler warnings
- [ ] Zero clippy warnings
- [ ] All tests pass
- [ ] Code coverage maintained or improved

### Overall Project Success
- Phase 3 Original: 60% → 85%+
- Phase 3C: 67% → 100% (after Sprint 4)
- Combined: 72% → 92%+
- User satisfaction with risk management features
- Adoption of portfolio risk aggregation feature

---

## Timeline Estimate

| Sprint | Duration | Completion Date |
|--------|----------|-----------------|
| Sprint 1: Portfolio Aggregation | 2-3 days | Feb 17-19, 2026 |
| Sprint 2: Threshold Settings | 1-2 days | Feb 20-21, 2026 |
| Sprint 3: Code Cleanup | 0.5-1 day | Feb 22, 2026 |
| Sprint 4: Reports (Optional) | 2-3 days | Feb 23-26, 2026 |

**Target Completion:** February 22-26, 2026

---

## Risk Mitigation

### Technical Risks
1. **Complex calculations may have bugs**
   - Mitigation: Write unit tests for all formulas
   - Add integration tests with known portfolios

2. **Performance issues with large portfolios**
   - Mitigation: Test with 100+ position portfolios
   - Add pagination if needed
   - Implement caching layer

3. **Frontend state management complexity**
   - Mitigation: Use TanStack Query for all API calls
   - Keep component state simple
   - Add loading/error boundaries

### Process Risks
1. **Scope creep during implementation**
   - Mitigation: Stick to roadmap, add enhancements to backlog

2. **Testing taking longer than expected**
   - Mitigation: Automate tests where possible
   - Focus on critical paths first

---

## Next Steps

1. Review this roadmap with stakeholders
2. Begin Sprint 1: Portfolio-Level Risk Aggregation
3. Set up tracking for sprint progress
4. Schedule sprint review after each sprint

**Ready to begin implementation? Start with Sprint 1!**
