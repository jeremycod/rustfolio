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

export type PositionRisk = {
    volatility: number; // Annualized volatility as percentage
    max_drawdown: number; // Maximum drawdown as negative percentage
    beta: number | null; // Beta coefficient relative to benchmark
    sharpe: number | null; // Sharpe ratio (risk-adjusted return)
    value_at_risk: number | null; // 5% VaR as negative percentage
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

export type CorrelationPair = {
    ticker1: string;
    ticker2: string;
    correlation: number; // -1.0 to 1.0
};

export type CorrelationMatrix = {
    portfolio_id: string;
    tickers: string[];
    correlations: CorrelationPair[];
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
    value_at_risk?: number | string; // BigDecimal from backend (comes as string)
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
