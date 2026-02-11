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
    total_gain_loss: string; // BigDecimal
    total_gain_loss_pct: string; // BigDecimal
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
    volatility: number;
    max_drawdown: number;
    beta?: number;
    sharpe?: number;
    value_at_risk?: number;
    risk_score: number;
    risk_level: RiskLevel;
    total_value?: number; // For portfolio snapshots
    market_value?: number; // For position snapshots
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