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
    signals: SentimentSignal[];
    portfolio_avg_sentiment: number;
    bullish_divergences: number;
    bearish_divergences: number;
    calculated_at: string;
};

// Enhanced Sentiment Analysis Types (Sprint 18 Extension - Multi-Source)
export type EventImportance = 'critical' | 'high' | 'medium' | 'low';
export type InsiderTransactionType = 'purchase' | 'sale' | 'grant' | 'exercise';
export type InsiderConfidence = 'high' | 'medium' | 'low' | 'none';
export type ConfidenceLevel = 'very_high' | 'high' | 'medium' | 'low';

export type MaterialEvent = {
    ticker: string;
    event_date: string;
    event_type: string;
    sentiment_score: number;
    summary: string;
    importance: EventImportance;
    filing_url: string;
};

export type InsiderTransaction = {
    ticker: string;
    transaction_date: string;
    reporting_person: string;
    title?: string;
    transaction_type: InsiderTransactionType;
    shares: number;
    price_per_share?: number;
    ownership_after?: number;
};

export type InsiderSentiment = {
    ticker: string;
    period_days: number;
    net_shares_traded: number;
    total_transactions: number;
    buying_transactions: number;
    selling_transactions: number;
    sentiment_score: number;
    confidence: InsiderConfidence;
    notable_transactions: InsiderTransaction[];
};

export type NewsArticle = {
    title: string;
    url: string;
    source: string;
    published_at: string;
    snippet: string;
};

export type EnhancedSentimentSignal = {
    ticker: string;
    news_sentiment: number;
    news_confidence: string;
    news_articles: NewsArticle[];
    material_events: MaterialEvent[];
    sec_filing_score?: number;
    insider_sentiment: InsiderSentiment;
    combined_sentiment: number;
    confidence_level: ConfidenceLevel;
    divergence_flags: string[];
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
    percentage_change: number;
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
    action_type: string;
    ticker: string;
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
    narrative: string;
    confidence: number;
    time_period: string;
    summary?: string;
    performance_explanation?: string;
    risk_highlights?: string[];
    top_contributors?: string[];
    generated_at: string; // ISO 8601 timestamp
};

// News & Sentiment Types
export type Sentiment = 'Positive' | 'Neutral' | 'Negative' | 'positive' | 'neutral' | 'negative';

export type NewsTheme = {
    theme: string;
    summary: string;
    sentiment: Sentiment;
    articles: NewsArticle[];
    relevance_score: number;
    article_count: number;
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

export type ForecastMethod = 'linear_regression' | 'exponential_smoothing' | 'moving_average' | 'ensemble' | 'mean_reversion';

export type PortfolioForecast = {
    portfolio_id: string;
    current_value: number;
    forecast_points: ForecastPoint[];
    methodology: ForecastMethod;
    confidence_level: number;
    warnings: string[];
    generated_at: string; // ISO 8601 timestamp
};

// Job Scheduler types
export type JobStatus = 'running' | 'success' | 'failed' | 'cancelled';

export type JobRun = {
    id: number;
    job_name: string;
    started_at: string;
    completed_at: string | null;
    status: JobStatus;
    items_processed: number | null;
    items_failed: number | null;
    error_message: string | null;
    duration_ms: number | null;
};

export type ScheduledJob = {
    name: string;
    schedule: string;
    description: string;
    last_run: string | null;
    last_status: JobStatus | null;
    next_run: string | null;
};

export type JobStats = {
    job_name: string;
    total_runs: number;
    successful_runs: number;
    failed_runs: number;
    avg_duration_ms: number;
    last_run_at: string | null;
    last_status: JobStatus | null;
};

// Cache Health types
export type CacheHealthLevel = 'healthy' | 'degraded' | 'critical';

export type CacheTableHealth = {
    table_name: string;
    total_entries: number;
    fresh_entries: number;
    stale_entries: number;
    calculating_entries: number;
    error_entries: number;
    hit_rate_pct: number | null;
    avg_age_hours: number | null;
};

export type CacheHealthSummary = {
    total_entries: number;
    total_fresh: number;
    total_stale: number;
    total_calculating: number;
    total_errors: number;
    freshness_pct: number;
    error_rate_pct: number;
};

export type CacheHealthStatus = {
    checked_at: string;
    status: CacheHealthLevel;
    tables: CacheTableHealth[];
    summary: CacheHealthSummary;
};

// Alerts & Notifications Types

// Alert Rules
export type AlertRule = {
    id: string;
    user_id: string;
    portfolio_id: string | null;
    ticker: string | null;
    rule_type: string;
    threshold: number;
    comparison: string;
    enabled: boolean;
    name: string;
    description: string | null;
    notification_channels: string[];
    cooldown_hours: number;
    last_triggered_at: string | null;
    created_at: string;
    updated_at: string;
};

export type CreateAlertRuleRequest = {
    portfolio_id?: string;
    ticker?: string;
    rule_type: any; // Tagged union - will be { type: string, config: object }
    threshold: number;
    comparison: Comparison;
    name: string;
    description?: string;
    notification_channels?: NotificationChannel[];
    cooldown_hours?: number;
};

export type UpdateAlertRuleRequest = {
    threshold?: number;
    comparison?: Comparison;
    enabled?: boolean;
    name?: string;
    description?: string;
    notification_channels?: NotificationChannel[];
    cooldown_hours?: number;
};

// Alert Enums
export type Comparison = 'gt' | 'lt' | 'gte' | 'lte' | 'eq';
export type Direction = 'up' | 'down' | 'either';
export type Timeframe = 'intraday' | 'daily' | 'weekly' | 'monthly';
export type RiskMetric = 'risk_score' | 'volatility' | 'sharpe_ratio' | 'max_drawdown' | 'var_95' | 'cvar_95';
export type NotificationChannel = 'email' | 'in_app' | 'webhook';
export type AlertSeverity = 'low' | 'medium' | 'high' | 'critical';

// Alert Types (used for rule_type string in AlertRule)
export type AlertRuleType =
    | 'price_change'
    | 'volatility_spike'
    | 'drawdown_exceeded'
    | 'risk_threshold'
    | 'sentiment_change'
    | 'divergence';

// Alert History
export type AlertHistory = {
    id: string;
    alert_rule_id: string;
    user_id: string;
    portfolio_id: string | null;
    ticker: string | null;
    rule_type: string;
    threshold: number;
    actual_value: number;
    message: string;
    severity: string;
    metadata: Record<string, any>;
    triggered_at: string;
};

// Notifications
export type Notification = {
    id: string;
    user_id: string;
    title: string;
    message: string;
    notification_type: string;
    read: boolean;
    read_at: string | null;
    related_alert_id: string | null;
    created_at: string;
};

export type NotificationCountResponse = {
    total: number;
    unread: number;
};

// Notification Preferences
export type NotificationPreferences = {
    id: string;
    user_id: string;
    email_enabled: boolean;
    in_app_enabled: boolean;
    webhook_enabled: boolean;
    webhook_url: string | null;
    quiet_hours_start: string | null;
    quiet_hours_end: string | null;
    timezone: string;
    max_daily_emails: number;
    created_at: string;
    updated_at: string;
};

export type UpdateNotificationPreferences = {
    email_enabled?: boolean;
    in_app_enabled?: boolean;
    webhook_enabled?: boolean;
    webhook_url?: string;
    quiet_hours_start?: string;
    quiet_hours_end?: string;
    timezone?: string;
    max_daily_emails?: number;
};

// Evaluation
export type AlertEvaluationResult = {
    rule_id: string;
    triggered: boolean;
    actual_value: number;
    threshold: number;
    message: string;
    severity: string;
    metadata: Record<string, any>;
};

export type AlertEvaluationResponse = {
    evaluated_rules: number;
    triggered_alerts: number;
    results: AlertEvaluationResult[];
};

export type TestAlertResponse = {
    rule: AlertRule;
    evaluation: AlertEvaluationResult | null;
    would_trigger: boolean;
};

// ============================================================================
// Phase 1 & Phase 2 Enhanced Features Types
// ============================================================================

// Downside Risk Analysis Types (Phase 1)
export type DownsideRiskMetrics = {
    downside_deviation: number;
    sortino_ratio: number;
    mar: number; // Minimum Acceptable Return
    sharpe_ratio: number; // For comparison
    interpretation: {
        downside_risk_level: 'Low' | 'Moderate' | 'High' | 'Very High';
        sortino_rating: 'Excellent' | 'Good' | 'Fair' | 'Poor';
        sortino_vs_sharpe: string;
        summary: string;
    };
};

export type PositionDownsideContribution = {
    ticker: string;
    weight: number;
    downside_metrics: DownsideRiskMetrics;
};

export type PortfolioDownsideRisk = {
    portfolio_id: string;
    portfolio_metrics: DownsideRiskMetrics;
    position_downside_risks: PositionDownsideContribution[];
    days: number;
    benchmark: string;
};

// Correlation Clustering Types (Phase 1)
export type AssetCluster = {
    cluster_id: number;
    tickers: string[];
    avg_correlation: number;
    color: string; // Hex color for visualization
    name: string; // "Cluster A (3 assets)"
};

export type CorrelationMatrixEnhanced = CorrelationMatrixWithStats & {
    clusters?: AssetCluster[];
    cluster_labels?: Record<string, number>; // ticker -> cluster_id
    inter_cluster_correlations?: number[][];
};

// Market Regime Types (Phase 1)
export type RegimeType = 'Bull' | 'Bear' | 'HighVolatility' | 'Normal';

export type StateProbabilities = {
    bull: number;
    bear: number;
    high_volatility: number;
    normal: number;
};

export type MarketRegime = {
    date: string;
    regime_type: RegimeType;
    volatility_level: number;
    market_return: number;
    confidence: number;
    threshold_multiplier: number;
    hmm_probabilities?: StateProbabilities;
    predicted_regime?: string;
    transition_probability?: number;
    ensemble_confidence?: number;
};

export type RegimeForecast = {
    days_ahead: number;
    predicted_regime: string;
    confidence: number;
    state_probabilities: StateProbabilities;
    transition_probability: number;
};

export type RegimeForecastResponse = {
    forecasts: RegimeForecast[];
};

// GARCH Volatility Forecasting Types (Phase 2)
export type GarchParameters = {
    omega: number;
    alpha: number;
    beta: number;
    persistence: number;
    long_run_volatility: number;
};

export type VolatilityForecastPoint = {
    day: number;
    predicted_volatility: number;
    confidence_lower: number;
    confidence_upper: number;
};

export type VolatilityForecast = {
    ticker: string;
    current_volatility: number;
    forecast_days: number;
    confidence_level: number;
    garch_parameters: GarchParameters;
    forecasts: VolatilityForecastPoint[];
    warnings: string[];
};

// Trading Signals Types (Phase 2)
export type SignalType = 'Momentum' | 'MeanReversion' | 'Trend' | 'Combined';
export type SignalDirection = 'Bullish' | 'Bearish' | 'Neutral';
export type SignalConfidence = 'High' | 'Medium' | 'Low';

export type SignalFactor = {
    indicator: string;
    value: number;
    weight: number;
    direction: 'bullish' | 'bearish' | 'neutral';
    interpretation: string;
};

export type ContributingFactors = {
    factors: SignalFactor[];
    bullish_score: number;
    bearish_score: number;
    total_factors: number;
};

export type TradingSignal = {
    signal_type: SignalType;
    probability: number; // 0.0-1.0
    direction: SignalDirection;
    confidence: SignalConfidence;
    explanation: string;
    contributing_factors: ContributingFactors;
};

export type OverallRecommendation = {
    action: 'Buy' | 'Sell' | 'Hold';
    strength: 'Strong' | 'Weak';
    probability: number;
    rationale: string;
};

export type SignalResponse = {
    ticker: string;
    horizon_months: number;
    signals: TradingSignal[];
    overall_recommendation?: OverallRecommendation;
};

// Sentiment Forecasting Types (Phase 2)
export type SentimentFactors = {
    news_sentiment?: number;
    sec_sentiment?: number;
    insider_sentiment?: number;
    combined_sentiment?: number;
};

export type SentimentMomentum = {
    change_7d?: number;
    change_30d?: number;
    acceleration?: number;
};

export type SentimentDivergence = {
    detected: boolean;
    type?: 'Bullish' | 'Bearish';
    score?: number;
    reversal_probability?: number;
    explanation?: string;
};

export type SentimentAwareForecast = {
    ticker: string;
    days: number;
    base_forecast: ForecastPoint[];
    adjusted_forecast: ForecastPoint[];
    sentiment_factors: SentimentFactors;
    sentiment_momentum: SentimentMomentum;
    sentiment_spike: {
        detected: boolean;
        z_score: number;
    };
    divergence: SentimentDivergence;
    interpretation: string;
};

// User Risk Preferences Types (Phase 2)
export type RiskAppetite = 'Conservative' | 'Balanced' | 'Aggressive';
export type SignalSensitivity = 'Low' | 'Medium' | 'High';

export type RiskPreferences = {
    id: string;
    user_id: string;
    risk_appetite: RiskAppetite;
    forecast_horizon_preference: number; // 1-24 months
    signal_sensitivity: SignalSensitivity;
    sentiment_weight: number; // 0.0-1.0
    technical_weight: number;
    fundamental_weight: number;
    custom_settings?: Record<string, any>;
    created_at: string;
    updated_at: string;
};

export type RiskProfile = {
    risk_appetite: string;
    description: string;
    risk_threshold_multiplier: number;
    signal_confidence_threshold: number;
    forecast_horizon_days: number;
    characteristics: string[];
};
