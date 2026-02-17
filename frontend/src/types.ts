export type Portfolio = {
    id: string;
    name: string;
    created_at: string;
};

export type Position = {
    id: string;
    portfolio_id: string;
    ticker: string;
    shares: string; // BigDecimal from backend
    avg_buy_price: string; // BigDecimal from backend
    created_at: string;
};

export type PricePoint = {
    id: string;
    ticker: string;
    date: string; // YYYY-MM-DD
    close_price: string; // BigDecimal from backend
    created_at: string;
};

export type ChartPoint = {
    date: string;
    value: number;
    sma20: number | null;
    ema20: number | null;
    trend: number | null;
    // later: bollinger, drawdown
};

export type AllocationPoint = {
    ticker: string;
    value: number;
    weight: number;
};

export type AnalyticsResponse = {
    series: ChartPoint[];
    allocations: AllocationPoint[];
    meta: {
        points: number;
        start: string | null;
        end: string | null;
    };
};

export type TickerMatch = {
    symbol: string;
    name: string;
    _type: string;
    region: string;
    currency: string;
    matchScore: number;
};

export type Account = {
    id: string;
    portfolio_id: string;
    account_number: string;
    account_nickname: string;
    client_id: string | null;
    client_name: string | null;
    created_at: string;
};

export type HoldingSnapshot = {
    id: string;
    account_id: string;
    snapshot_date: string; // Date YYYY-MM-DD
    ticker: string;
    holding_name: string | null;
    asset_category: string | null;
    industry: string | null;
    quantity: string; // BigDecimal
    price: string; // BigDecimal
    average_cost: string; // BigDecimal
    book_value: string; // BigDecimal
    market_value: string; // BigDecimal
    fund: string | null;
    accrued_interest: string | null; // BigDecimal
    gain_loss: string | null; // BigDecimal
    gain_loss_pct: string | null; // BigDecimal
    percentage_of_assets: string | null; // BigDecimal
    created_at: string;
};

export type LatestAccountHolding = {
    id: string;
    account_id: string;
    account_nickname: string;
    account_number: string;
    ticker: string;
    holding_name: string | null;
    asset_category: string | null;
    industry: string | null;
    quantity: string; // BigDecimal
    price: string; // BigDecimal
    market_value: string; // BigDecimal
    gain_loss: string | null; // BigDecimal
    gain_loss_pct: string | null; // BigDecimal
    snapshot_date: string; // Date
};

export type AccountValueHistory = {
    account_id: string;
    snapshot_date: string; // Date
    total_value: string; // BigDecimal
    total_cost: string; // BigDecimal
    total_gain_loss: string | null; // BigDecimal (optional)
    total_gain_loss_pct: string | null; // BigDecimal (optional)
};

export type ImportResponse = {
    accounts_created: number;
    holdings_created: number;
    transactions_detected: number;
    errors: string[];
    snapshot_date: string;
};

export type CsvFileInfo = {
    name: string;
    path: string;
    date: string | null;
    file_type: "holdings" | "activities";
};

export type DetectedTransaction = {
    id: string;
    account_id: string;
    transaction_type: 'BUY' | 'SELL' | 'DIVIDEND' | 'SPLIT' | 'OTHER';
    ticker: string;
    quantity: string | null; // BigDecimal
    price: string | null; // BigDecimal
    amount: string | null; // BigDecimal
    transaction_date: string; // Date
    from_snapshot_date: string | null; // Date
    to_snapshot_date: string | null; // Date
    description: string | null;
    created_at: string;
};

export type CashFlow = {
    id: string;
    account_id: string;
    flow_type: 'DEPOSIT' | 'WITHDRAWAL';
    amount: string; // BigDecimal
    flow_date: string; // Date
    description: string | null;
    created_at: string;
};

export type AccountActivity = {
    account_id: string;
    activity_type: 'TRANSACTION' | 'CASH_FLOW';
    type_detail: string;
    ticker: string | null;
    quantity: string | null; // BigDecimal
    amount: string | null; // BigDecimal
    activity_date: string; // Date
    description: string | null;
};

export type AccountTruePerformance = {
    account_id: string;
    account_nickname: string;
    account_number: string;
    total_deposits: string; // BigDecimal
    total_withdrawals: string; // BigDecimal
    current_value: string; // BigDecimal
    book_value: string; // BigDecimal
    true_gain_loss: string; // BigDecimal
    true_gain_loss_pct: string; // BigDecimal
    as_of_date: string | null; // Date
};

// Risk Management Types
export type RiskLevel = 'low' | 'moderate' | 'high';

export type RiskDecomposition = {
    systematic_risk: number;     // Variance explained by market
    idiosyncratic_risk: number;  // Stock-specific variance
    r_squared: number;           // % variance explained by beta
    total_risk: number;          // Total volatility
};

export type PositionRisk = {
    volatility: number; // Annualized volatility as percentage
    max_drawdown: number; // Maximum drawdown as negative percentage
    beta: number | null; // Beta coefficient relative to SPY (legacy)
    beta_spy: number | null; // Beta vs SPY
    beta_qqq: number | null; // Beta vs Nasdaq 100
    beta_iwm: number | null; // Beta vs Russell 2000
    risk_decomposition: RiskDecomposition | null; // Systematic vs idiosyncratic risk
    sharpe: number | null; // Sharpe ratio (risk-adjusted return)
    sortino: number | null; // Sortino ratio (downside risk-adjusted return)
    annualized_return: number | null; // Annualized return as percentage
    value_at_risk: number | null; // 5% VaR as negative percentage (legacy)
    var_95: number | null; // 95% confidence VaR (5% chance of exceeding)
    var_99: number | null; // 99% confidence VaR (1% chance of exceeding)
    expected_shortfall_95: number | null; // Expected Shortfall at 95% confidence (CVaR)
    expected_shortfall_99: number | null; // Expected Shortfall at 99% confidence (CVaR)
};

export type RiskAssessment = {
    ticker: string;
    metrics: PositionRisk;
    risk_score: number; // 0-100 risk score
    risk_level: RiskLevel; // low/moderate/high classification
};

export type RiskThresholds = {
    volatility_threshold: number | null;
    drawdown_threshold: number | null;
    beta_threshold: number | null;
    var_threshold: number | null;
    risk_score_threshold: number | null;
};

export type PositionRiskContribution = {
    ticker: string;
    market_value: number;
    weight: number; // 0-1 (position weight in portfolio)
    risk_assessment: RiskAssessment;
};

export type PortfolioRisk = {
    portfolio_id: string;
    total_value: number;
    portfolio_volatility: number;
    portfolio_max_drawdown: number;
    portfolio_beta: number | null;
    portfolio_sharpe: number | null;
    portfolio_risk_score: number;
    risk_level: RiskLevel;
    position_risks: PositionRiskContribution[];
};

export type ViolationSeverity = 'warning' | 'critical';

export type ThresholdViolation = {
    ticker: string;
    holding_name: string | null;
    metric_name: string;
    metric_value: number;
    threshold_value: number;
    threshold_type: ViolationSeverity;
};

// Note: Backend uses #[serde(flatten)] on portfolio_risk, so PortfolioRisk fields
// are at the top level of the JSON, not nested under a "portfolio_risk" key
export type PortfolioRiskWithViolations = PortfolioRisk & {
    thresholds: RiskThresholdSettings;
    violations: ThresholdViolation[];
};

export type CorrelationPair = {
    ticker1: string;
    ticker2: string;
    correlation: number; // -1.0 to 1.0
};

export type CorrelationMatrix = {
    portfolio_id: string;
    tickers: string[];
    correlations: CorrelationPair[];
    matrix_2d: number[][];
};

export type CorrelationStatistics = {
    average_correlation: number;
    max_correlation: number;
    min_correlation: number;
    correlation_std_dev: number;
    high_correlation_pairs: number;
    adjusted_diversification_score: number;
};

export type CorrelationMatrixWithStats = {
    portfolio_id: string;
    tickers: string[];
    correlations: CorrelationPair[];
    matrix_2d: number[][];
    statistics: CorrelationStatistics;
};

// Rolling Beta Analysis Types
export type BetaPoint = {
    date: string;
    beta: number;
    r_squared: number;
    alpha?: number;
};

export type RollingBetaAnalysis = {
    ticker: string;
    benchmark: string;
    beta_30d: BetaPoint[];
    beta_60d: BetaPoint[];
    beta_90d: BetaPoint[];
    current_beta: number;
    beta_volatility: number;
};

// Beta Forecast Types
export type BetaForecastPoint = {
    date: string;
    predicted_beta: number;
    lower_bound: number;
    upper_bound: number;
    confidence_level: number;
};

export type BetaRegimeChange = {
    date: string;
    beta_before: number;
    beta_after: number;
    z_score: number;
    regime_type: string;
};

export type BetaForecast = {
    ticker: string;
    benchmark: string;
    current_beta: number;
    beta_volatility: number;
    forecast_points: BetaForecastPoint[];
    methodology: ForecastMethod;
    confidence_level: number;
    regime_changes: BetaRegimeChange[];
    warnings: string[];
    generated_at: string;
};

// Sentiment Analysis Types (Sprint 18)
export type SentimentTrend = 'improving' | 'stable' | 'deteriorating';
export type MomentumTrend = 'bullish' | 'neutral' | 'bearish';
export type DivergenceType = 'bullish' | 'bearish' | 'confirmed' | 'none';

export type SentimentDataPoint = {
    date: string;
    sentiment_score: number; // -1.0 to +1.0
    news_volume: number;
    price?: number;
};

export type SentimentSignal = {
    ticker: string;
    current_sentiment: number; // -1.0 to +1.0
    sentiment_trend: SentimentTrend;
    momentum_trend: MomentumTrend;
    divergence: DivergenceType;
    sentiment_price_correlation?: number; // -1 to +1
    correlation_lag_days?: number;
    correlation_strength?: string; // "strong", "moderate", "weak"
    historical_sentiment: SentimentDataPoint[];
    news_articles_analyzed: number;
    calculated_at: string;
    warnings: string[];
};

export type PortfolioSentimentAnalysis = {
    portfolio_id: string;
    signals: SentimentSignal[];
    portfolio_avg_sentiment: number;
    bullish_divergences: number;
    bearish_divergences: number;
    calculated_at: string;
};

// Historical Risk Tracking Types
export type RiskSnapshot = {
    id: string;
    portfolio_id: string;
    ticker?: string;
    snapshot_date: string; // Date YYYY-MM-DD
    snapshot_type: 'portfolio' | 'position';
    volatility: number | string; // BigDecimal from backend (comes as string)
    max_drawdown: number | string; // BigDecimal from backend (comes as string)
    beta?: number | string; // BigDecimal from backend (comes as string)
    sharpe?: number | string; // BigDecimal from backend (comes as string)
    sortino?: number | string; // BigDecimal from backend (comes as string)
    annualized_return?: number | string; // BigDecimal from backend (comes as string)
    value_at_risk?: number | string; // BigDecimal from backend (comes as string)
    var_95?: number | string; // BigDecimal from backend (comes as string)
    var_99?: number | string; // BigDecimal from backend (comes as string)
    expected_shortfall_95?: number | string; // BigDecimal from backend (comes as string)
    expected_shortfall_99?: number | string; // BigDecimal from backend (comes as string)
    risk_score: number | string; // BigDecimal from backend (comes as string)
    risk_level: RiskLevel;
    total_value?: number | string; // BigDecimal from backend (comes as string)
    market_value?: number | string; // BigDecimal from backend (comes as string)
    created_at: string;
};

export type RiskAlert = {
    portfolio_id: string;
    ticker?: string;
    alert_type: string; // "risk_increase", "threshold_breach"
    previous_value: number;
    current_value: number;
    change_percent: number;
    date: string; // Date YYYY-MM-DD
    metric_name: string; // "risk_score", "volatility", etc.
};

// Portfolio Optimization types
export type RecommendationType =
    | 'reduce_concentration'
    | 'rebalance_sectors'
    | 'reduce_risk'
    | 'improve_efficiency'
    | 'increase_diversification';

export type Severity = 'info' | 'warning' | 'high' | 'critical';

export type AdjustmentAction = 'BUY' | 'SELL' | 'HOLD';

export type PositionAdjustment = {
    ticker: string;
    holding_name?: string;
    current_value: number;
    current_weight: number;
    recommended_value: number;
    recommended_weight: number;
    action: AdjustmentAction;
    amount_change: number;
    shares_change?: number;
};

export type ExpectedImpact = {
    risk_score_before: number;
    risk_score_after: number;
    risk_score_change: number;
    volatility_before: number;
    volatility_after: number;
    volatility_change: number;
    sharpe_before?: number;
    sharpe_after?: number;
    sharpe_change?: number;
    diversification_before: number;
    diversification_after: number;
    diversification_change: number;
    max_drawdown_before: number;
    max_drawdown_after: number;
};

export type OptimizationRecommendation = {
    id: string;
    recommendation_type: RecommendationType;
    severity: Severity;
    title: string;
    rationale: string;
    affected_positions: PositionAdjustment[];
    expected_impact: ExpectedImpact;
    suggested_actions: string[];
};

export type CurrentMetrics = {
    risk_score: number;
    volatility: number;
    max_drawdown: number;
    sharpe_ratio?: number;
    diversification_score: number;
    correlation_adjusted_diversification_score?: number;
    average_correlation?: number;
    position_count: number;
    largest_position_weight: number;
    top_3_concentration: number;
};

export type PortfolioHealth = 'excellent' | 'good' | 'fair' | 'poor' | 'critical';

export type AnalysisSummary = {
    total_recommendations: number;
    critical_issues: number;
    high_priority: number;
    warnings: number;
    overall_health: PortfolioHealth;
    key_findings: string[];
};

export type OptimizationAnalysis = {
    portfolio_id: string;
    portfolio_name: string;
    total_value: number;
    analysis_date: string;
    current_metrics: CurrentMetrics;
    recommendations: OptimizationRecommendation[];
    summary: AnalysisSummary;
};
// Risk Threshold Settings
export type RiskThresholdSettings = {
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
};

export type UpdateRiskThresholds = {
    volatility_warning_threshold?: number;
    volatility_critical_threshold?: number;
    drawdown_warning_threshold?: number;
    drawdown_critical_threshold?: number;
    beta_warning_threshold?: number;
    beta_critical_threshold?: number;
    risk_score_warning_threshold?: number;
    risk_score_critical_threshold?: number;
    var_warning_threshold?: number;
    var_critical_threshold?: number;
};

export type ViolationSeverity = 'warning' | 'critical';

export type ThresholdViolation = {
    ticker: string;
    holding_name: string | null;
    metric_name: string;
    metric_value: number;
    threshold_value: number;
    threshold_type: ViolationSeverity;
};

// LLM / AI Features
export type UserPreferences = {
    id: string;
    user_id: string;
    llm_enabled: boolean;
    consent_given_at: string | null;
    narrative_cache_hours: number;
    created_at: string;
    updated_at: string;
};

export type UpdateUserPreferences = {
    llm_enabled: boolean;
    narrative_cache_hours?: number;
};

export type LlmUsageStats = {
    total_requests: number;
    total_prompt_tokens: number;
    total_completion_tokens: number;
    total_cost: string; // BigDecimal from backend
    current_month_cost: string; // BigDecimal from backend
};

export type PortfolioNarrative = {
    summary: string;
    performance_explanation: string;
    risk_highlights: string[];
    top_contributors: string[];
    generated_at: string; // ISO 8601 timestamp
};

// News & Sentiment Types
export type NewsArticle = {
    title: string;
    url: string;
    source: string;
    published_at: string; // ISO 8601 timestamp
    snippet: string;
};

export type Sentiment = 'positive' | 'neutral' | 'negative';

export type NewsTheme = {
    theme_name: string;
    summary: string;
    sentiment: Sentiment;
    articles: NewsArticle[];
    relevance_score: number;
};

export type PortfolioNewsAnalysis = {
    portfolio_id: string;
    themes: NewsTheme[];
    position_news: Record<string, NewsTheme[]>; // ticker -> themes
    overall_sentiment: Sentiment;
    fetched_at: string; // ISO 8601 timestamp
};

// Q&A Types
export type Confidence = 'high' | 'medium' | 'low';

export type PortfolioQuestion = {
    question: string;
    context_hint?: string;
};

export type PortfolioAnswer = {
    answer: string;
    sources: string[];
    confidence: Confidence;
    follow_up_questions: string[];
    generated_at: string; // ISO 8601 timestamp
};

export type QAConversation = {
    question: PortfolioQuestion;
    answer: PortfolioAnswer;
};

// Forecasting Types
export type ForecastPoint = {
    date: string;
    predicted_value: number;
    lower_bound: number;
    upper_bound: number;
    confidence_level: number; // e.g., 0.95 for 95%
};

export type ForecastMethod = 'linear_regression' | 'exponential_smoothing' | 'moving_average' | 'ensemble';

export type PortfolioForecast = {
    portfolio_id: string;
    current_value: number;
    forecast_points: ForecastPoint[];
    methodology: ForecastMethod;
    confidence_level: number;
    warnings: string[];
    generated_at: string; // ISO 8601 timestamp
};
