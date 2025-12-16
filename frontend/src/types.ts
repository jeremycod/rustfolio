export type Portfolio = {
    id: string;
    name: string;
    created_at: string;
};

export type Position = {
    id: string;
    portfolio_id: string;
    ticker: string;
    shares: number;
    avg_buy_price: number;
    created_at: string;
};

export type PricePoint = {
    id: string;
    ticker: string;
    date: string; // YYYY-MM-DD
    close_price: number;
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