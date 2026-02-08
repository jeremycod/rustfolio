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
    errors: string[];
    snapshot_date: string;
};

export type CsvFileInfo = {
    name: string;
    path: string;
    date: string | null;
};