# Sprint 11: Correlation & Beta Analysis - COMPLETE

**Status**: ✅ Complete
**Date**: February 14, 2026
**Sprint Duration**: 1 day

## Overview
Enhanced risk analytics with multi-benchmark beta analysis and systematic vs idiosyncratic risk decomposition.

## Completed Features

### 1. Multi-Benchmark Beta Analysis ✅
**Backend Implementation:**
- Added `compute_multi_benchmark_beta()` function in `risk_service.rs`
- Calculates beta against three major benchmarks:
  - **SPY** (S&P 500) - Broad market
  - **QQQ** (Nasdaq 100) - Tech-heavy index
  - **IWM** (Russell 2000) - Small-cap index
- Added fields to `PositionRisk` model:
  - `beta_spy: Option<f64>`
  - `beta_qqq: Option<f64>`
  - `beta_iwm: Option<f64>`

**Frontend Implementation:**
- Added 3 new metric cards in `RiskMetricsPanel.tsx`:
  - Beta vs S&P 500 (SPY)
  - Beta vs Nasdaq 100 (QQQ)
  - Beta vs Russell 2000 (IWM)
- Color-coded cards: green for beta < 1.2, orange for beta > 1.2
- Tooltips explain interpretation for each benchmark

### 2. Risk Decomposition (Systematic vs Idiosyncratic) ✅
**Backend Implementation:**
- Created `RiskDecomposition` struct in `models/risk.rs`:
  ```rust
  pub struct RiskDecomposition {
      pub systematic_risk: f64,     // Market-driven risk
      pub idiosyncratic_risk: f64,  // Stock-specific risk
      pub r_squared: f64,           // % variance explained by beta
      pub total_risk: f64,          // Total volatility
  }
  ```
- Added `compute_risk_decomposition()` function:
  - Uses correlation² (R²) to separate systematic from idiosyncratic risk
  - Formula: Systematic = √(R² × total_variance)
  - Formula: Idiosyncratic = √((1 - R²) × total_variance)
- Added `risk_decomposition: Option<RiskDecomposition>` to `PositionRisk`

**Frontend Implementation:**
- Added 3 new metric cards:
  1. **Systematic Risk** - Market-driven (blue, cannot be diversified away)
  2. **Idiosyncratic Risk** - Stock-specific (purple, can be diversified away)
  3. **R² Correlation** - Percentage of variance explained by market (teal)
- Cards show both absolute risk values and percentages
- Tooltips explain diversification implications

### 3. Correlation Matrix Enhancement ✅
**Backend Implementation:**
- Added `matrix_2d: Vec<Vec<f64>>` to `CorrelationMatrix` struct
- Implemented 2D matrix generation in `routes/risk.rs`:
  - Creates symmetric n×n matrix
  - Diagonal set to 1.0 (perfect self-correlation)
  - Fills from correlation pairs
- Ready for frontend heatmap visualization

## Technical Details

### Files Modified

**Backend:**
- `src/models/risk.rs` - Added RiskDecomposition struct, multi-benchmark beta fields
- `src/services/risk_service.rs` - Added compute_multi_benchmark_beta(), compute_risk_decomposition()
- `src/routes/risk.rs` - Added 2D matrix generation for correlation endpoint
- `src/services/risk_snapshot_service.rs` - Updated struct initializations
- `src/models/mod.rs` - Exported new types

**Frontend:**
- `src/types.ts` - Added RiskDecomposition type, updated PositionRisk with new fields
- `src/components/RiskMetricsPanel.tsx` - Added 7 new metric cards

### Build Status
```bash
Backend:  ✅ cargo check passed (0 errors, 26 warnings)
Frontend: ✅ npm run build passed (production build successful)
```

### Calculation Methods

**Multi-Benchmark Beta:**
```
Beta = Cov(asset_returns, benchmark_returns) / Var(benchmark_returns)
Calculated independently for SPY, QQQ, and IWM
```

**Risk Decomposition:**
```
R² = correlation²
Systematic Risk = √(R² × total_variance)
Idiosyncratic Risk = √((1 - R²) × total_variance)
Total Risk = √(Systematic² + Idiosyncratic²)
```

## Deferred Items

The following items are deferred to future sprints or enhancements:

1. **Interactive Correlation Heatmap** (Sprint 22 or later)
   - Requires React heatmap library integration
   - Color-coded correlation matrix visualization
   - Interactive tooltips and selection

2. **Beta Comparison Charts** (Future enhancement)
   - Side-by-side beta comparison across benchmarks
   - Rolling beta analysis (30/60/90 day windows)
   - Historical beta trend visualization

3. **Enhanced Diversification Score** (Future enhancement)
   - Correlation-adjusted portfolio diversification
   - Herfindahl index for concentration
   - Diversification effectiveness ratio

## User Impact

Users now have:
1. **Multi-benchmark perspective** - See how positions behave relative to different market segments
2. **Risk attribution** - Understand what portion of risk is market-driven vs stock-specific
3. **Diversification insights** - Identify which positions can benefit from diversification
4. **Better risk assessment** - More comprehensive risk analysis beyond single beta metric

## Next Steps

**Recommended Next Sprint Options:**
1. **Sprint 12**: Machine Learning - Predictive risk modeling
2. **Sprint 13**: Sector Analysis - Industry-level risk aggregation
3. **Sprint 22**: Advanced Visualizations - Interactive heatmaps and charts
4. **Diversification Enhancement**: Complete correlation-adjusted scoring

## Notes

- Multi-benchmark beta calculations run in parallel (efficient)
- Risk decomposition only calculated when beta is available
- All new fields are optional (backward compatible)
- TypeScript types fully updated and aligned with backend models
- All metric cards include educational tooltips

---

**Sprint 11 successfully completed in 1 day** ✅

---

# Sprint 12: LLM Integration Foundation - TODO

**Status**: ⚪ Not Started
**Estimate**: 1 week
**Phase**: 4B - AI-Powered Insights

## Overview
Set up the foundation for AI-powered features by integrating Large Language Model (LLM) capabilities into Rustfolio. This sprint focuses on infrastructure, not user-facing features yet.

## Backend Tasks

### 1. LLM Service Architecture
- [ ] Create `src/services/llm_service.rs`
- [ ] Define `LlmProvider` trait
  ```rust
  pub trait LlmProvider {
      async fn generate_completion(&self, prompt: String) -> Result<String, LlmError>;
      async fn generate_summary(&self, text: String, max_length: usize) -> Result<String, LlmError>;
      async fn get_embedding(&self, text: String) -> Result<Vec<f32>, LlmError>;
  }
  ```
- [ ] Implement `OpenAiProvider` struct
  - [ ] Use `gpt-4o-mini` model (cost-effective)
  - [ ] Implement async API calls with `reqwest`
  - [ ] Add timeout handling (30 seconds)
  - [ ] Parse OpenAI API responses
- [ ] Add error types in `src/errors.rs`
  - [ ] `LlmError::ApiError(String)`
  - [ ] `LlmError::RateLimited`
  - [ ] `LlmError::Disabled`
  - [ ] `LlmError::InvalidResponse`

### 2. Configuration & Environment
- [ ] Add environment variables to `.env`:
  - [ ] `OPENAI_API_KEY` (optional, default: empty)
  - [ ] `LLM_ENABLED` (default: false)
  - [ ] `LLM_PROVIDER` (default: openai)
  - [ ] `LLM_MAX_TOKENS` (default: 500)
  - [ ] `LLM_TEMPERATURE` (default: 0.7)
- [ ] Update `AppState` to include LLM service
- [ ] Add configuration struct:
  ```rust
  pub struct LlmConfig {
      pub enabled: bool,
      pub provider: String,
      pub api_key: Option<String>,
      pub max_tokens: usize,
      pub temperature: f32,
  }
  ```
- [ ] Load config on startup with validation

### 3. Rate Limiting & Caching
- [ ] Create `LlmCache` struct
  ```rust
  pub struct LlmCache {
      cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
      ttl: Duration,  // 1 hour default
  }
  ```
- [ ] Implement cache methods:
  - [ ] `get(key: &str) -> Option<String>`
  - [ ] `set(key: &str, value: String)`
  - [ ] `clear_expired()`
- [ ] Add per-user rate limiting
  - [ ] Track requests per user per hour
  - [ ] Return 429 when limit exceeded
  - [ ] Default: 50 requests/user/hour
- [ ] Implement cache key generation (hash of prompt)

### 4. Token Usage Tracking
- [ ] Create `llm_usage` database table
  ```sql
  CREATE TABLE llm_usage (
      id UUID PRIMARY KEY,
      user_id UUID,
      portfolio_id UUID,
      model VARCHAR(50),
      prompt_tokens INT,
      completion_tokens INT,
      total_cost NUMERIC(10, 6),
      created_at TIMESTAMPTZ DEFAULT NOW()
  );
  ```
- [ ] Add migration file
- [ ] Create queries in `src/db/llm_queries.rs`
  - [ ] `log_usage()`
  - [ ] `get_user_usage(user_id, start_date, end_date)`
  - [ ] `get_total_cost()`
- [ ] Calculate costs based on OpenAI pricing

### 5. User Consent & Settings
- [ ] Create `user_preferences` table
  ```sql
  CREATE TABLE user_preferences (
      id UUID PRIMARY KEY,
      user_id UUID UNIQUE,
      llm_enabled BOOLEAN DEFAULT false,
      consent_given_at TIMESTAMPTZ,
      created_at TIMESTAMPTZ DEFAULT NOW(),
      updated_at TIMESTAMPTZ DEFAULT NOW()
  );
  ```
- [ ] Add migration
- [ ] Create endpoints:
  - [ ] `GET /api/users/:id/preferences`
  - [ ] `PUT /api/users/:id/preferences`
  - [ ] `POST /api/users/:id/llm-consent`

### 6. Error Handling & Fallbacks
- [ ] Implement retry logic with exponential backoff
  - [ ] Max 3 retries
  - [ ] Delays: 1s, 2s, 4s
- [ ] Add graceful degradation
  - [ ] Return numeric data if LLM fails
  - [ ] Show "AI features unavailable" message
- [ ] Log all LLM errors to monitoring
- [ ] Add circuit breaker pattern (optional)

### 7. Testing & Validation
- [ ] Unit tests for `LlmProvider`
  - [ ] Mock OpenAI API responses
  - [ ] Test error handling
  - [ ] Test rate limiting
- [ ] Integration test: End-to-end LLM call
- [ ] Test cache expiration
- [ ] Test cost calculation accuracy
- [ ] Load test: Multiple concurrent requests

## Frontend Tasks

### 1. AI Feature Toggle UI
- [ ] Create `Settings.tsx` page (if doesn't exist)
- [ ] Add "AI Features" section
  - [ ] Toggle switch: "Enable AI-powered insights"
  - [ ] Description text
  - [ ] Privacy notice link
- [ ] Create consent dialog
  - [ ] Privacy policy summary
  - [ ] Data usage explanation
  - [ ] "I understand and consent" checkbox
  - [ ] Accept/Decline buttons
- [ ] Show badge on AI-generated content
  - [ ] Create `<AIBadge />` component
  - [ ] Icon: Brain or Sparkle
  - [ ] Text: "AI Generated" or "Experimental"

### 2. Loading States
- [ ] Create `<AILoadingState />` component
  - [ ] Skeleton loader for text content
  - [ ] Animated spinner
  - [ ] "Generating insights..." message
- [ ] Add loading state to future AI sections
  - [ ] Placeholder cards
  - [ ] Progress indicator

### 3. API Integration
- [ ] Add endpoints to `src/lib/endpoints.ts`:
  ```typescript
  export async function getUserPreferences(userId: string): Promise<UserPreferences>
  export async function updateLlmConsent(userId: string, consent: boolean): Promise<void>
  export async function getLlmUsage(userId: string): Promise<LlmUsageStats>
  ```
- [ ] Add TypeScript types:
  ```typescript
  export type UserPreferences = {
      id: string;
      user_id: string;
      llm_enabled: boolean;
      consent_given_at: string | null;
  };

  export type LlmUsageStats = {
      total_requests: number;
      total_cost: number;
      current_month_cost: number;
  };
  ```
- [ ] Add error handling for LLM API calls

### 4. Settings Page Updates
- [ ] Create `<LlmSettings />` component
  - [ ] Display consent status
  - [ ] Show usage statistics
  - [ ] Cost breakdown (if admin)
  - [ ] Revoke consent button
- [ ] Add navigation link to settings

## Documentation

### 1. Code Documentation
- [ ] Add rustdoc comments to `llm_service.rs`
- [ ] Document LLM prompt engineering guidelines
- [ ] Add README for AI features

### 2. User Documentation
- [ ] Privacy policy update
  - [ ] Explain data sent to OpenAI
  - [ ] Retention policies
  - [ ] User rights
- [ ] FAQ about AI features
  - [ ] What data is shared?
  - [ ] How much does it cost?
  - [ ] Can I opt out?

### 3. Developer Documentation
- [ ] Guide: Adding new LLM providers
- [ ] Guide: Writing effective prompts
- [ ] Cost estimation spreadsheet

## Deliverables Checklist

- [ ] **Backend Infrastructure**
  - [ ] LLM service with OpenAI integration
  - [ ] Rate limiting and caching
  - [ ] Token usage tracking
  - [ ] Error handling with graceful fallbacks

- [ ] **Database Schema**
  - [ ] `llm_usage` table for cost tracking
  - [ ] `user_preferences` table for consent

- [ ] **API Endpoints**
  - [ ] GET/PUT user preferences
  - [ ] POST consent endpoint
  - [ ] GET usage statistics

- [ ] **Frontend UI**
  - [ ] Settings page with AI toggle
  - [ ] Consent dialog
  - [ ] AI content badges
  - [ ] Loading states

- [ ] **Testing**
  - [ ] Unit tests (>80% coverage)
  - [ ] Integration tests
  - [ ] Load tests for rate limiting

- [ ] **Documentation**
  - [ ] Code documentation
  - [ ] User privacy policy
  - [ ] Developer guides

## Success Criteria

✅ Sprint 12 is complete when:
1. LLM service can successfully call OpenAI API
2. Rate limiting prevents abuse
3. Costs are accurately tracked in database
4. Users can enable/disable AI features
5. Graceful degradation works when LLM fails
6. All tests pass
7. Documentation is complete

## Notes

- **Cost Management**: Start with conservative rate limits (50 requests/user/hour)
- **Privacy First**: AI features are opt-in, disabled by default
- **Monitoring**: Set up alerts for high LLM costs (>$10/day)
- **Model Choice**: gpt-4o-mini balances cost ($0.15/1M input tokens) with quality
- **Future Providers**: Architecture supports Anthropic Claude, local models (Ollama)

## Estimated Hours Breakdown

- Backend LLM service: 8 hours
- Rate limiting & caching: 4 hours
- Database schema & queries: 4 hours
- Frontend settings UI: 6 hours
- Consent flow: 4 hours
- Testing: 6 hours
- Documentation: 3 hours

**Total**: ~35 hours (~1 week)

---

**Ready to begin Sprint 12!** 🚀
