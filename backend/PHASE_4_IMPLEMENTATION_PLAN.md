Let# Phase 4 Implementation Plan: Advanced Analytics & AI Features

**Based on**: `docs/ANALYSIS.md` - Recommendations from Stock Research Sidekick Integration
**Start Date**: TBD
**Status**: Planning

---

## Executive Summary

This plan outlines the next evolution of Rustfolio from a portfolio tracker with risk management into an **intelligent portfolio assistant**. Drawing from the Stock Research Sidekick project, we'll add:
1. Advanced risk analytics (Sharpe, Beta, VaR, correlation analysis)
2. LLM-powered narrative insights and news analysis
3. Predictive analytics and forecasting
4. Interactive Q&A and educational features
5. Background processing and alerts

**Guiding Principles**:
- ✅ Educational tool, NOT investment advice
- ✅ Clearly label predictions as speculative
- ✅ Build on existing Phase 3 foundation
- ✅ Incremental delivery with user feedback loops
- ✅ Privacy-first (optional AI features, user consent)

---

## Phase 4A: Advanced Risk Analytics

**Duration**: 3-4 weeks
**Priority**: High
**Dependencies**: Phase 3 complete

### Goals
Enhance risk analysis with industry-standard metrics and correlation analysis to provide deeper portfolio insights.

### Sprint 9: Sharpe & Sortino Ratios

**Time Estimate**: 1 week

#### Backend Implementation

1. **Add Risk-Free Rate Configuration**
   - Environment variable: `RISK_FREE_RATE` (default: 4.5% for US Treasury)
   - Admin endpoint to update rate
   - Store in app state

2. **Compute Sharpe Ratio**
   ```rust
   // Formula: (portfolio_return - risk_free_rate) / portfolio_volatility
   fn calculate_sharpe_ratio(
       returns: &[f64],
       risk_free_rate: f64,
       period_days: i32
   ) -> Option<f64>
   ```

3. **Compute Sortino Ratio**
   ```rust
   // Like Sharpe but only uses downside volatility
   fn calculate_sortino_ratio(
       returns: &[f64],
       risk_free_rate: f64,
       target_return: f64
   ) -> Option<f64>
   ```

4. **Update RiskAssessment Model**
   ```rust
   pub struct PositionRisk {
       // ... existing fields
       pub sharpe_ratio: Option<f64>,
       pub sortino_ratio: Option<f64>,
       pub annualized_return: Option<f64>,
   }
   ```

#### Frontend Implementation

1. **Add to Risk Metrics Display**
   - Sharpe Ratio card with tooltip explanation
   - Sortino Ratio card with tooltip
   - Color coding: <1 (poor), 1-2 (good), >2 (excellent)

2. **Historical Sharpe Chart**
   - Add Sharpe ratio to RiskHistoryChart
   - Toggle for Sharpe/Sortino over time

**Deliverables**:
- [x] Sharpe ratio calculation (position + portfolio level)
- [x] Sortino ratio calculation
- [x] UI display with educational tooltips
- [x] Historical tracking

---

### Sprint 10: Value at Risk (VaR) & Expected Shortfall

**Time Estimate**: 1 week

#### Backend Implementation

1. **Parametric VaR (95% and 99%)**
   ```rust
   pub struct VaRMetrics {
       pub var_95: f64,      // 95% confidence level
       pub var_99: f64,      // 99% confidence level
       pub expected_shortfall: f64,  // Average loss beyond VaR
       pub methodology: String,  // "parametric", "historical", "monte_carlo"
   }
   ```

2. **Historical VaR**
   - Use actual return distribution
   - Calculate percentile losses

3. **Expected Shortfall (CVaR)**
   - Average of losses beyond VaR threshold
   - More conservative than VaR

4. **Position-Level VaR**
   - VaR for each holding
   - Contribution to portfolio VaR

#### Frontend Implementation

1. **VaR Dashboard Card**
   - Show 95% and 99% VaR
   - Expected Shortfall
   - Visual: histogram of returns with VaR cutoff line

2. **VaR Alerts**
   - Highlight if current drawdown exceeds VaR
   - "Portfolio is in a 1-in-20 loss event" warning

**Deliverables**:
- [x] VaR calculation (3 methodologies)
- [x] Expected Shortfall
- [x] VaR visualization
- [x] VaR-based alerts

---

### Sprint 11: Portfolio Correlation & Beta Analysis

**Time Estimate**: 1.5 weeks

#### Backend Implementation

1. **Enhanced Correlation Matrix**
   - Already have basic correlation (Phase 3)
   - Add visualization data structure:
   ```rust
   pub struct CorrelationHeatmapData {
       pub tickers: Vec<String>,
       pub matrix: Vec<Vec<f64>>,  // 2D array
       pub clusters: Vec<PositionCluster>,  // Highly correlated groups
   }
   ```

2. **Beta Calculation Enhancement**
   - Already have beta vs SPY
   - Add multi-benchmark support (QQQ, IWM, bonds)
   - Rolling beta (30/60/90 day windows)

3. **Systematic vs Idiosyncratic Risk**
   ```rust
   pub struct RiskDecomposition {
       pub systematic_risk: f64,  // Beta * market_variance
       pub idiosyncratic_risk: f64,  // Stock-specific risk
       pub total_risk: f64,
   }
   ```

4. **Diversification Score Enhancement**
   - Current: basic Herfindahl index
   - Enhanced: account for correlations
   ```rust
   // True diversification benefit = 1 - sqrt(weighted_avg_correlation)
   fn calculate_effective_positions(
       positions: &[Position],
       correlations: &CorrelationMatrix
   ) -> f64
   ```

#### Frontend Implementation

1. **Correlation Heatmap**
   - Interactive heatmap (Recharts or D3)
   - Click cell to see scatter plot of two positions
   - Cluster visualization (group highly correlated assets)

2. **Beta Comparison Chart**
   - Multi-benchmark comparison
   - Rolling beta chart
   - Systematic vs idiosyncratic risk breakdown

3. **Diversification Dashboard**
   - Effective number of positions
   - Correlation-adjusted diversification score
   - Recommendations to reduce correlation

**Deliverables**:
- [x] Enhanced correlation analysis
- [x] Multi-benchmark beta
- [x] Risk decomposition
- [x] Interactive heatmap
- [x] Diversification improvements

---

## Phase 4B: AI-Powered Insights

**Duration**: 4-5 weeks
**Priority**: Medium-High
**Dependencies**: Phase 4A complete, LLM integration setup

### Sprint 12: LLM Integration Foundation

**Time Estimate**: 1 week

#### Backend Implementation

1. **LLM Service Architecture**
   ```rust
   // New service: src/services/llm_service.rs

   pub trait LlmProvider {
       async fn generate_completion(&self, prompt: String) -> Result<String, LlmError>;
       async fn generate_summary(&self, text: String, max_length: usize) -> Result<String, LlmError>;
   }

   pub struct OpenAiProvider {
       api_key: String,
       model: String,  // gpt-4o-mini for cost efficiency
   }
   ```

2. **Configuration**
   - Environment variables:
     - `OPENAI_API_KEY` (optional)
     - `LLM_ENABLED` (default: false)
     - `LLM_PROVIDER` (openai, anthropic, local)
   - Admin UI to enable/disable AI features
   - User consent tracking

3. **Rate Limiting & Caching**
   ```rust
   pub struct LlmCache {
       cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
       ttl: Duration,  // 1 hour
   }
   ```

4. **Error Handling**
   - Graceful degradation (show numeric data if LLM fails)
   - Retry logic with exponential backoff
   - Cost tracking (token usage logging)

#### Frontend Implementation

1. **AI Feature Toggle**
   - Settings page: "Enable AI-powered insights"
   - Privacy notice and consent form
   - Badge on AI-generated content

2. **Loading States**
   - Skeleton loaders for AI sections
   - "Generating insights..." with spinner

**Deliverables**:
- [x] LLM service abstraction
- [x] OpenAI integration
- [x] Rate limiting & caching
- [x] User consent UI
- [x] Cost tracking

---

### LLSprint 13: Narrative Portfolio Summaries

**Time Estimate**: 1 week

#### Backend Implementation

1. **Portfolio Performance Narrative Endpoint**
   ```rust
   // GET /api/analytics/portfolios/:id/narrative

   pub struct PortfolioNarrative {
       pub summary: String,  // 2-3 sentence overview
       pub performance_explanation: String,  // Why portfolio moved
       pub risk_highlights: Vec<String>,  // Key risk factors
       pub top_contributors: Vec<ContributorInsight>,
       pub generated_at: DateTime<Utc>,
   }
   ```

2. **Prompt Engineering**
   ```rust
   fn build_narrative_prompt(
       portfolio_data: &PortfolioRisk,
       performance: &AccountTruePerformance,
       time_period: &str
   ) -> String {
       format!(
           "Analyze this portfolio's {time_period} performance:\n\
            - Total value: ${}\n\
            - Return: {}%\n\
            - Risk score: {}/100\n\
            - Volatility: {}%\n\
            - Positions: {}\n\n\
            Provide:\n\
            1. A 2-3 sentence summary\n\
            2. Explanation of what drove performance\n\
            3. 3-4 key risk factors\n\
            4. Top 3 contributors (positive or negative)\n\n\
            IMPORTANT: Use educational language. Do NOT give buy/sell recommendations.",
           portfolio_data.total_value,
           performance.true_gain_loss_pct,
           portfolio_data.portfolio_risk_score,
           portfolio_data.portfolio_volatility,
           portfolio_data.position_risks.len()
       )
   }
   ```

3. **Caching Strategy**
   - Cache narratives for 1 hour
   - Invalidate on portfolio changes
   - Store in Redis or PostgreSQL

#### Frontend Implementation

1. **Narrative Panel**
   - Expandable card on Portfolio Risk Overview
   - "AI Insights" section with brain icon
   - Disclosure: "AI-generated educational content"

2. **Narrative Display**
   ```tsx
   <Card>
     <CardHeader>
       <Psychology /> AI Portfolio Summary
       <Chip label="Experimental" size="small" />
     </CardHeader>
     <CardContent>
       <Typography variant="body1">{narrative.summary}</Typography>

       <Typography variant="h6">What Drove Performance?</Typography>
       <Typography variant="body2">{narrative.performance_explanation}</Typography>

       <Typography variant="h6">Key Risk Factors</Typography>
       <List>
         {narrative.risk_highlights.map(risk => <ListItem>{risk}</ListItem>)}
       </List>

       <Alert severity="info">
         This is AI-generated educational content, not investment advice.
       </Alert>
     </CardContent>
   </Card>
   ```

**Deliverables**:
- [x] Narrative generation endpoint
- [x] Prompt engineering for portfolio analysis
- [x] Narrative UI component
- [x] Disclaimers and badges

---

### Sprint 14: News Aggregation & Theme Clustering

**Time Estimate**: 1.5 weeks

#### Backend Implementation

1. **News Service**
   ```rust
   // src/services/news_service.rs

   pub struct NewsArticle {
       pub title: String,
       pub url: String,
       pub source: String,
       pub published_at: DateTime<Utc>,
       pub snippet: String,
   }

   pub trait NewsProvider {
       async fn fetch_news(&self, ticker: String, days: i32) -> Result<Vec<NewsArticle>>;
   }

   pub struct SerperProvider {
       api_key: String,
   }
   ```

2. **News Clustering with LLM**
   ```rust
   pub struct NewsTheme {
       pub theme_name: String,
       pub summary: String,  // 2-3 sentences
       pub sentiment: Sentiment,  // Positive, Neutral, Negative
       pub articles: Vec<NewsArticle>,
       pub relevance_score: f64,
   }

   async fn cluster_news_themes(
       articles: Vec<NewsArticle>,
       llm: &dyn LlmProvider
   ) -> Result<Vec<NewsTheme>>
   ```

3. **Portfolio News Endpoint**
   ```rust
   // GET /api/analytics/portfolios/:id/news?days=7

   pub struct PortfolioNewsAnalysis {
       pub portfolio_id: String,
       pub themes: Vec<NewsTheme>,
       pub position_news: HashMap<String, Vec<NewsTheme>>,  // ticker -> themes
       pub overall_sentiment: Sentiment,
       pub fetched_at: DateTime<Utc>,
   }
   ```

4. **Background Job**
   - Fetch news for all positions nightly
   - Cache results in database
   - Incremental updates during day

#### Frontend Implementation

1. **News Dashboard Tab**
   - New tab: "News & Insights"
   - Timeline view of themes
   - Filter by sentiment, ticker, date

2. **Theme Cards**
   ```tsx
   <Card>
     <CardHeader>
       <Badge color={sentiment}>
         {theme.theme_name}
       </Badge>
       <Typography variant="caption">
         {theme.articles.length} articles
       </Typography>
     </CardHeader>
     <CardContent>
       <Typography>{theme.summary}</Typography>
       <Accordion>
         <AccordionSummary>View Articles</AccordionSummary>
         <AccordionDetails>
           {theme.articles.map(article => (
             <Link href={article.url} target="_blank">
               {article.title} - {article.source}
             </Link>
           ))}
         </AccordionDetails>
       </Accordion>
     </CardContent>
   </Card>
   ```

3. **Sentiment Indicators**
   - Portfolio sentiment score
   - Per-position sentiment badges
   - Sentiment trend chart over time

**Deliverables**:
- [x] News fetching service
- [x] LLM-based theme clustering
- [x] Portfolio news analysis
- [x] News dashboard UI
- [x] Background job scheduler

---

### Sprint 15: Interactive Q&A Assistant

**Time Estimate**: 1 week

#### Backend Implementation

1. **Q&A Endpoint**
   ```rust
   // POST /api/analytics/portfolios/:id/ask

   pub struct PortfolioQuestion {
       pub question: String,
       pub context: Option<String>,  // Optional context from UI
   }

   pub struct PortfolioAnswer {
       pub answer: String,
       pub sources: Vec<String>,  // Which data informed the answer
       pub confidence: Confidence,  // High, Medium, Low
       pub follow_up_questions: Vec<String>,
   }
   ```

2. **Context Building**
   ```rust
   async fn build_qa_context(
       portfolio_id: Uuid,
       pool: &PgPool
   ) -> Result<String> {
       // Gather:
       // - Portfolio metrics
       // - Recent performance
       // - Risk data
       // - News themes
       // - Optimization recommendations

       format!(
           "Portfolio Context:\n\
            Value: ${}\n\
            Return (30d): {}%\n\
            Risk Score: {}/100\n\
            Top Holdings: {}\n\
            Recent News: {}\n\
            Risk Issues: {}",
           // ... data
       )
   }
   ```

3. **Prompt with RAG (Retrieval-Augmented Generation)**
   ```rust
   fn build_qa_prompt(
       question: &str,
       context: &str
   ) -> String {
       format!(
           "You are a portfolio analysis assistant. Answer the user's question \
            using ONLY the provided portfolio data. If you don't have enough \
            information, say so. Do NOT give buy/sell recommendations.\n\n\
            Portfolio Data:\n{context}\n\n\
            Question: {question}\n\n\
            Answer:",
           context = context,
           question = question
       )
   }
   ```

4. **Safety Guardrails**
   - Filter out buy/sell recommendation attempts
   - Detect and reject questions about other users' portfolios
   - Rate limit: 10 questions per hour per user

#### Frontend Implementation

1. **Q&A Panel**
   - Floating chat button on Portfolio Risk page
   - Slide-out panel with chat interface
   - Pre-populated example questions

2. **Chat Interface**
   ```tsx
   <Box sx={{ height: 500, display: 'flex', flexDirection: 'column' }}>
     <Box sx={{ flex: 1, overflow: 'auto', p: 2 }}>
       {messages.map(msg => (
         <Message
           content={msg.content}
           role={msg.role}
           sources={msg.sources}
           confidence={msg.confidence}
         />
       ))}
     </Box>

     <TextField
       placeholder="Ask about your portfolio..."
       value={question}
       onChange={(e) => setQuestion(e.target.value)}
       onKeyPress={(e) => e.key === 'Enter' && handleAsk()}
     />

     <Box>
       <Chip label="Why did my portfolio drop?" onClick={() => askExample(0)} />
       <Chip label="Which position is riskiest?" onClick={() => askExample(1)} />
       <Chip label="What's my diversification score?" onClick={() => askExample(2)} />
     </Box>
   </Box>
   ```

3. **Conversation History**
   - Store last 10 Q&A pairs
   - Allow user to export conversation

**Deliverables**:
- [x] Q&A endpoint with RAG
- [x] Context building from portfolio data
- [x] Safety guardrails
- [x] Chat UI component
- [x] Example questions

---

## Phase 4C: Predictive Analytics

**Duration**: 3-4 weeks
**Priority**: Medium
**Dependencies**: Phase 4A complete

### Sprint 16: Time-Series Forecasting

**Time Estimate**: 1.5 weeks

#### Backend Implementation

1. **Python Micro-Service**
   ```python
   # forecasting_service.py

   from fastapi import FastAPI
   from prophet import Prophet
   import pandas as pd

   app = FastAPI()

   @app.post("/forecast")
   async def forecast_timeseries(
       data: TimeSeriesData,
       periods: int = 30
   ):
       df = pd.DataFrame({
           'ds': data.dates,
           'y': data.values
       })

       model = Prophet()
       model.fit(df)

       future = model.make_future_dataframe(periods=periods)
       forecast = model.predict(future)

       return {
           'predictions': forecast['yhat'].tolist(),
           'lower_bound': forecast['yhat_lower'].tolist(),
           'upper_bound': forecast['yhat_upper'].tolist(),
           'confidence': 0.95
       }
   ```

2. **Rust Client**
   ```rust
   // src/services/forecasting_service.rs

   pub struct ForecastingService {
       base_url: String,
   }

   impl ForecastingService {
       pub async fn forecast_portfolio_value(
           &self,
           historical_data: &[f64],
           days_ahead: i32
       ) -> Result<Forecast> {
           // Call Python service
       }
   }
   ```

3. **Forecast Endpoint**
   ```rust
   // GET /api/analytics/portfolios/:id/forecast?days=30

   pub struct PortfolioForecast {
       pub predictions: Vec<ForecastPoint>,
       pub methodology: String,  // "Prophet time-series model"
       pub confidence_level: f64,
       pub warnings: Vec<String>,
   }

   pub struct ForecastPoint {
       pub date: String,
       pub predicted_value: f64,
       pub lower_bound: f64,
       pub upper_bound: f64,
   }
   ```

#### Frontend Implementation

1. **Forecast Chart**
   - Add to Analytics page
   - Show historical data + forecast with confidence bands
   - Clear disclaimer: "This is a statistical projection, not a guarantee"

2. **Scenario Analysis**
   - Show best/worst case scenarios
   - Probability distribution visualization

**Deliverables**:
- [x] Python forecasting micro-service
- [x] Prophet-based time-series forecasting
- [x] Rust client for forecasting service
- [x] Forecast UI with confidence bands
- [x] Disclaimers

---

### Sprint 17: Rolling Regression & Beta Forecasting

**Time Estimate**: 1 week

#### Backend Implementation

1. **Rolling Regression**
   ```rust
   pub struct RollingBeta {
       pub window: i32,  // days
       pub data_points: Vec<BetaPoint>,
   }

   pub struct BetaPoint {
       pub date: String,
       pub beta: f64,
       pub r_squared: f64,
       pub alpha: f64,  // Jensen's alpha
   }
   ```

2. **Beta Forecast**
   - Use historical beta trend to forecast
   - Confidence intervals based on beta stability

3. **Risk Contribution Forecast**
   - Project which positions will contribute most risk
   - Based on forecasted betas and volatilities

#### Frontend Implementation

1. **Rolling Beta Chart**
   - Show beta evolution over time
   - Highlight periods of high/low beta

2. **Risk Contribution Forecast**
   - Bar chart: current vs forecasted risk contribution
   - Recommendations for rebalancing

**Deliverables**:
- [x] Rolling regression implementation
- [x] Beta forecasting
- [x] Risk contribution forecasts
- [x] Visualization

---

### Sprint 18: Sentiment-Aware Signals (Experimental)

**Time Estimate**: 1.5 weeks

#### Backend Implementation

1. **Sentiment Extraction from News**
   ```rust
   pub struct SentimentSignal {
       pub ticker: String,
       pub sentiment_score: f64,  // -1 (negative) to +1 (positive)
       pub sentiment_trend: Trend,  // Improving, Stable, Deteriorating
       pub news_volume: i32,
       pub calculated_at: DateTime<Utc>,
   }
   ```

2. **Sentiment-Price Correlation**
   - Analyze if sentiment changes precede price moves
   - Calculate correlation lag
   - Provide insights (experimental only)

3. **Momentum Indicators**
   - Combine price momentum with sentiment momentum
   - Flag positions with diverging signals

#### Frontend Implementation

1. **Sentiment Dashboard**
   - Sentiment score per position
   - Sentiment trend indicators
   - Correlation insights

2. **Experimental Badge**
   - Clearly mark as experimental
   - Link to methodology explanation

**Deliverables**:
- [x] Sentiment extraction from news
- [x] Sentiment-price correlation analysis
- [x] Momentum indicators
- [x] Experimental dashboard

---

## Phase 4D: System Enhancements

**Duration**: 2 weeks
**Priority**: High (for reliability)
**Dependencies**: Phase 4B complete

### Sprint 19: Background Jobs & Scheduling

**Time Estimate**: 1 week

#### Backend Implementation

1. **Task Queue System**
   ```rust
   // Use tokio-cron-scheduler or similar

   pub struct JobScheduler {
       jobs: Vec<Job>,
   }

   pub enum Job {
       RefreshPrices,
       FetchNews,
       GenerateNarratives,
       CalculateForecasts,
       SendAlerts,
   }
   ```

2. **Background Jobs**
   - **Nightly**:
     - Refresh all price data
     - Fetch news for all positions
     - Generate portfolio narratives
     - Calculate forecasts
   - **Hourly**:
     - Check for threshold violations
     - Generate alerts
   - **Weekly**:
     - Clean up old cache entries
     - Archive historical data

3. **Job Status Dashboard**
   - Admin endpoint: `/api/admin/jobs`
   - View job history, success/failure rates
   - Manual job triggers

#### Frontend Implementation

1. **Admin Dashboard**
   - Jobs status table
   - Manual job triggers
   - Job logs viewer

**Deliverables**:
- [x] Task scheduler
- [x] Background jobs implementation
- [x] Job monitoring dashboard
- [x] Error handling & retries

---

### Sprint 20: Alerts & Notifications System

**Time Estimate**: 1 week

#### Backend Implementation

1. **Alert Rules Engine**
   ```rust
   pub struct AlertRule {
       pub id: String,
       pub user_id: String,
       pub portfolio_id: Option<String>,
       pub rule_type: AlertType,
       pub threshold: f64,
       pub enabled: bool,
   }

   pub enum AlertType {
       PriceChange { ticker: String, percentage: f64 },
       VolatilitySpike { ticker: String, threshold: f64 },
       DrawdownExceeded { percentage: f64 },
       ThresholdViolation { metric: String },
       NewsSentimentChange { ticker: String },
   }
   ```

2. **Notification Channels**
   - Email (via SendGrid or similar)
   - In-app notifications
   - Webhook support (future: SMS, Slack)

3. **Alert History**
   - Store all triggered alerts
   - Endpoint: `/api/alerts?days=30`

#### Frontend Implementation

1. **Alert Configuration UI**
   - Settings page: "My Alerts"
   - Create/edit/delete alert rules
   - Test alert functionality

2. **In-App Notification Center**
   - Bell icon with badge count
   - Notification list with "mark as read"
   - Link to relevant portfolio/position

3. **Alert History**
   - View past alerts
   - Filter by type, date, portfolio

**Deliverables**:
- [x] Alert rules engine
- [x] Email notifications
- [x] In-app notification center
- [x] Alert configuration UI
- [x] Alert history

---

## Phase 4E: User Experience Polish

**Duration**: 1-2 weeks
**Priority**: Medium
**Dependencies**: All Phase 4 features implemented

### Sprint 21: Educational Content & Tooltips

**Time Estimate**: 1 week

#### Implementation

1. **Educational Tooltips**
   - Generate via LLM once, store in database
   - Cover all risk metrics (volatility, drawdown, Sharpe, etc.)
   - Simple language, 2-3 sentences

2. **Help Center**
   - "/help" page with:
     - Glossary of terms
     - Metric explanations
     - FAQ
     - Video tutorials (future)

3. **Onboarding Flow**
   - First-time user tutorial
   - Feature highlights
   - Sample portfolio option

4. **Contextual Help**
   - Question mark icons next to complex features
   - Sidebar help panel
   - "Learn more" links

**Deliverables**:
- [x] 50+ educational tooltips
- [x] Help center page
- [x] Onboarding flow
- [x] Contextual help system

---

### Sprint 22: Mobile Optimization & PWA

**Time Estimate**: 1 week

#### Implementation

1. **Responsive Design Improvements**
   - Optimize all Phase 4 components for mobile
   - Touch-friendly controls
   - Simplified mobile layouts

2. **Progressive Web App**
   - Service worker for offline support
   - Add to homescreen prompt
   - Push notification support (future)

3. **Mobile-Specific Features**
   - Swipe gestures
   - Bottom navigation
   - Mobile-optimized charts

**Deliverables**:
- [x] Mobile-responsive all features
- [x] PWA setup
- [x] Touch optimizations

---

## Technology Stack Additions

### Backend
- **LLM Integration**: `openai` crate (or `anthropic-rs`)
- **Task Scheduling**: `tokio-cron-scheduler`
- **Email**: `lettre` (SMTP client)
- **News API**: HTTP client to Serper or NewsAPI
- **WebSocket**: `axum-ws` (for real-time notifications)

### Frontend
- **Chat UI**: Existing MUI components
- **Heatmap**: `recharts` (already using)
- **Notifications**: `notistack` (toast notifications)

### Infrastructure
- **Python Micro-Service**: FastAPI + Prophet + scikit-learn
  - Deployed separately (Docker container)
  - Rust communicates via HTTP
- **Redis** (optional): For caching LLM responses
- **Message Queue** (optional): For background jobs (RabbitMQ or AWS SQS)

---

## Cost Considerations

### LLM API Costs
- **GPT-4o-mini**: ~$0.15 per 1M input tokens, ~$0.60 per 1M output tokens
- **Estimated Monthly Cost** (100 active users):
  - Narratives: 100 users × 4 narratives/month × 2000 tokens = ~$0.12
  - News clustering: 100 users × 10 tickers × 4 weeks × 5000 tokens = ~$2.00
  - Q&A: 100 users × 10 questions × 3000 tokens = ~$0.36
  - **Total: ~$2.50/month for 100 users**

### News API Costs
- Serper: $50/month for 5,000 searches (100 users × 10 tickers = 1,000 searches/day)
- Alternative: Free NewsAPI (100 requests/day) for MVP

### Infrastructure
- Python forecasting service: ~$10/month (small VM or serverless)
- Redis (optional): ~$5/month (small instance)

**Total Additional Monthly Cost**: ~$65-70/month for 100 active users

---

## Phased Rollout Strategy

### Phase 4A (3-4 weeks)
1. Sprint 9: Sharpe & Sortino
2. Sprint 10: VaR & Expected Shortfall
3. Sprint 11: Correlation & Beta enhancements
**Milestone**: Advanced risk analytics complete

### Phase 4B (4-5 weeks)
1. Sprint 12: LLM foundation
2. Sprint 13: Narrative summaries
3. Sprint 14: News aggregation
4. Sprint 15: Interactive Q&A
**Milestone**: AI insights available (beta)

### Phase 4C (3-4 weeks)
1. Sprint 16: Time-series forecasting
2. Sprint 17: Rolling regression
3. Sprint 18: Sentiment signals
**Milestone**: Predictive analytics (experimental)

### Phase 4D (2 weeks)
1. Sprint 19: Background jobs
2. Sprint 20: Alerts system
**Milestone**: Reliability & automation

### Phase 4E (1-2 weeks)
1. Sprint 21: Educational content
2. Sprint 22: Mobile optimization
**Milestone**: Polish & accessibility

**Total Duration**: 13-17 weeks (3-4 months)

---

## Success Metrics

### User Engagement
- [ ] 60%+ users enable AI features
- [ ] 10+ Q&A questions per user per month
- [ ] 80%+ users view portfolio narratives

### Technical
- [ ] <2s response time for narratives
- [ ] <5s for news clustering
- [ ] 99%+ uptime for background jobs
- [ ] <$0.10 LLM cost per active user per month

### Business Value
- [ ] User satisfaction score >4.5/5 for AI features
- [ ] 30%+ increase in daily active users
- [ ] Feature differentiation vs competitors

---

## Risk Mitigation

### Technical Risks
- **LLM hallucinations**: Implement fact-checking, cite sources, clear disclaimers
- **API costs**: Implement caching, rate limiting, budget alerts
- **Python service reliability**: Deploy with auto-restart, health checks, fallback to basic forecasting

### Regulatory Risks
- **Not financial advice**: Prominent disclaimers on all AI content
- **Data privacy**: User consent, GDPR compliance, no PII to LLM
- **Investment recommendations**: Filter and reject buy/sell suggestions

### User Experience Risks
- **Over-reliance on AI**: Educational tooltips, encourage critical thinking
- **Feature complexity**: Progressive disclosure, hide advanced features by default
- **Information overload**: Configurable views, hide/show sections

---

## Dependencies & Prerequisites

### Before Starting Phase 4
- [x] Phase 3 complete (risk management foundation)
- [ ] User feedback on Phase 3 features
- [ ] Legal review of AI feature disclaimers
- [ ] OpenAI API key (or alternative LLM)
- [ ] News API key (Serper or similar)
- [ ] Python deployment environment setup

### Infrastructure Requirements
- PostgreSQL (already have)
- Redis (optional, for caching)
- Python runtime (3.10+, for forecasting service)
- Docker (for Python service deployment)
- Increased API rate limits (Alpha Vantage, LLM, News)

---

## Testing Strategy

### Unit Tests
- Risk calculation functions (Sharpe, VaR, etc.)
- LLM prompt building
- News clustering logic

### Integration Tests
- LLM service communication
- Python forecasting service
- Background job execution

### User Acceptance Testing
- Beta user group (10-20 users)
- A/B testing for AI features (control group without AI)
- Feedback surveys after each sprint

### Performance Testing
- Load testing for LLM endpoints (concurrent users)
- Background job stress testing
- Forecast generation latency

---

## Documentation Requirements

### User Documentation
- [x] Help center with metric explanations
- [x] AI feature user guide
- [x] FAQ for predictive analytics
- [x] Video tutorials (optional)

### Developer Documentation
- [x] LLM integration guide
- [x] Python service deployment
- [x] Background job architecture
- [x] API documentation updates

### Compliance Documentation
- [x] AI usage policy
- [x] Data privacy statement
- [x] Disclaimer templates
- [x] Regulatory compliance checklist

---

## Next Steps

1. **Stakeholder Review** (1 week)
   - Review plan with team
   - Legal review of AI disclaimers
   - Budget approval

2. **Phase 4A Kickoff** (Week 2)
   - Set up development environment
   - Sprint 9 planning
   - Design reviews

3. **Beta User Recruitment** (Ongoing)
   - Identify 10-20 beta users for AI features
   - Set up feedback channels
   - Create testing protocol

4. **Vendor Setup** (Week 1-2)
   - OpenAI API account
   - News API account
   - Python service infrastructure

---

## Conclusion

Phase 4 transforms Rustfolio from a portfolio tracker into an **intelligent portfolio assistant**, incorporating advanced risk analytics, AI-powered insights, and predictive capabilities. By building incrementally and maintaining clear disclaimers, we can deliver significant value while managing technical and regulatory risks.

The phased approach allows us to:
1. Validate each feature with user feedback
2. Manage costs (start with low-volume features)
3. Ensure quality (one sprint at a time)
4. Pivot if needed (based on user adoption)

**Recommended Start**: After Phase 3 user feedback period (2-4 weeks post-Phase 3 completion)

---

## Outstanding Items from Sprint 11 (Deferred)

The following items from Sprint 11 were partially implemented or deferred to future work:

### 1. Complete Multi-Benchmark Beta Calculations
**Status**: Infrastructure in place, calculations return None
**Priority**: Medium
**Estimate**: 2-3 hours

The `compute_multi_benchmark_beta()` function exists and has the correct structure, but currently returns `(None, None)` for QQQ and IWM benchmarks. Need to:
- Implement actual QQQ beta calculation (Nasdaq 100)
- Implement actual IWM beta calculation (Russell 2000)
- Add error handling for missing benchmark data
- Test with real data

**Location**: `src/services/risk_service.rs:compute_multi_benchmark_beta()`

### 2. Complete Risk Decomposition Calculations
**Status**: Model defined, calculation stub exists
**Priority**: Medium
**Estimate**: 2-3 hours

The `RiskDecomposition` struct is defined and the `compute_risk_decomposition()` function has the correct signature, but needs full implementation:
- Calculate systematic risk using beta and market variance
- Calculate idiosyncratic risk (residual variance)
- Verify R² calculation accuracy
- Add validation for edge cases (zero volatility, perfect correlation)

**Location**: `src/services/risk_service.rs:compute_risk_decomposition()`

### 3. Enhanced Diversification Score (Correlation-Adjusted)
**Status**: Not started
**Priority**: Low
**Estimate**: 4-6 hours

Current diversification score uses basic Herfindahl index. Enhance to account for correlations:
```rust
// True diversification benefit = 1 - sqrt(weighted_avg_correlation)
fn calculate_effective_positions(
    positions: &[Position],
    correlations: &CorrelationMatrix
) -> f64 {
    // Weight each position by portfolio percentage
    // Calculate average pairwise correlation
    // Adjust effective position count based on correlation
}
```

This provides a more realistic measure of diversification benefit.

### 4. Interactive Correlation Heatmap (Sprint 22)
**Status**: Backend ready (2D matrix structure exists), UI deferred
**Priority**: Low (deferred to Sprint 22)
**Estimate**: 1 week

Backend provides `matrix_2d: Vec<Vec<f64>>` in CorrelationMatrix. Frontend needs:
- Heatmap visualization library integration (Recharts or D3)
- Color-coded cells (red = high correlation, blue = low/negative)
- Interactive tooltips showing exact correlation values
- Click cell to see scatter plot of two positions
- Cluster visualization (group highly correlated assets)

**Dependencies**: React heatmap library selection and integration

### 5. Beta Comparison Charts
**Status**: Not started
**Priority**: Low (deferred)
**Estimate**: 3-5 days

Visualizations for multi-benchmark beta analysis:
- Side-by-side bar chart: Beta vs SPY, QQQ, IWM
- Rolling beta chart (30/60/90 day windows)
- Historical beta trend with confidence bands
- Systematic vs idiosyncratic risk breakdown visualization

**Dependencies**: Complete multi-benchmark beta calculations (#1)

### 6. Rolling Beta Windows
**Status**: Not started
**Priority**: Low (deferred)
**Estimate**: 1 week

Extend beta calculations to support time windows:
```rust
pub struct RollingBeta {
    pub window: i32,  // days (30, 60, 90)
    pub data_points: Vec<BetaPoint>,
}

pub struct BetaPoint {
    pub date: String,
    pub beta: f64,
    pub r_squared: f64,
    pub alpha: f64,  // Jensen's alpha
}
```

Shows how beta stability changes over time, useful for identifying regime changes.

**Note**: This overlaps with Sprint 17 (Rolling Regression & Beta Forecasting) in Phase 4C.

---

**To complete Sprint 11 fully**: Focus on items #1 and #2 (complete multi-benchmark beta and risk decomposition). Items #3-6 are lower priority enhancements that can be addressed in future sprints or when user demand warrants.

---

*Based on Stock Research Sidekick integration ideas from `docs/ANALYSIS.md`*
*Created: 2026-02-14*
*Updated: 2026-02-14 (Added Sprint 11 outstanding items)*
*Version: 1.1*
