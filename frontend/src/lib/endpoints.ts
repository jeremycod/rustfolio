import { api } from "./api";
import type {
    AnalyticsResponse,
    Portfolio,
    Position,
    PricePoint,
    TickerMatch,
    Account,
    LatestAccountHolding,
    AccountValueHistory,
    ImportResponse,
    CsvFileInfo,
    DetectedTransaction,
    CashFlow,
    AccountActivity,
    AccountTruePerformance
} from "../types";

export async function listPortfolios(): Promise<Portfolio[]> {
    const res = await api.get("/api/portfolios");
    return res.data;
}

export async function createPortfolio(name: string): Promise<Portfolio> {
    const res = await api.post("/api/portfolios", { name });
    return res.data;
}

export async function listPositions(portfolioId: string): Promise<Position[]> {
    const res = await api.get(`/api/portfolios/${portfolioId}/positions`);
    return res.data;
}

export async function createPosition(portfolioId: string, payload: {
    ticker: string;
    shares: number;
    avg_buy_price: number;
}): Promise<Position> {
    const res = await api.post(`/api/portfolios/${portfolioId}/positions`, payload);
    return res.data;
}

export async function getAnalytics(portfolioId: string): Promise<AnalyticsResponse> {
    const res = await api.get(`/api/analytics/${portfolioId}`);
    return res.data;
}

export async function updatePrices(ticker: string): Promise<void> {
    await api.post(`/api/prices/${ticker}/update`);
}

export async function getLatestPrice(ticker: string): Promise<PricePoint> {
    const res = await api.get(`/api/prices/${ticker}/latest`);
    return res.data;
}

export async function deletePosition(positionId: string): Promise<void> {
    await api.delete(`/api/positions/${positionId}`);
}

export async function updatePosition(positionId: string, payload: {
    shares: number;
    avg_buy_price: number;
}): Promise<Position> {
    const res = await api.put(`/api/positions/${positionId}`, payload);
    return res.data;
}

export async function searchTickers(keyword: string): Promise<TickerMatch[]> {
    const res = await api.get(`/api/prices/search/${encodeURIComponent(keyword)}`);
    return res.data;
}

// Account endpoints
export async function listAccounts(portfolioId: string): Promise<Account[]> {
    const res = await api.get(`/api/portfolios/${portfolioId}/accounts`);
    return res.data;
}

export async function getAccount(accountId: string): Promise<Account> {
    const res = await api.get(`/api/accounts/${accountId}`);
    return res.data;
}

export async function getLatestHoldings(accountId: string): Promise<LatestAccountHolding[]> {
    const res = await api.get(`/api/accounts/${accountId}/holdings`);
    return res.data;
}

export async function getAccountHistory(accountId: string): Promise<AccountValueHistory[]> {
    const res = await api.get(`/api/accounts/${accountId}/history`);
    return res.data;
}

export async function getPortfolioHistory(portfolioId: string): Promise<AccountValueHistory[]> {
    const res = await api.get(`/api/portfolios/${portfolioId}/history`);
    return res.data;
}

// Import endpoints
export async function listCsvFiles(): Promise<CsvFileInfo[]> {
    const res = await api.get('/api/import/files');
    return res.data;
}

export async function importCSV(portfolioId: string, filePath: string): Promise<ImportResponse> {
    const res = await api.post(`/api/portfolios/${portfolioId}/import`, { file_path: filePath });
    return res.data;
}

// Transaction endpoints
export async function getAccountTransactions(accountId: string): Promise<DetectedTransaction[]> {
    const res = await api.get(`/api/accounts/${accountId}/transactions`);
    return res.data;
}

export async function getAccountActivity(accountId: string): Promise<AccountActivity[]> {
    const res = await api.get(`/api/accounts/${accountId}/activity`);
    return res.data;
}

export async function getAccountTruePerformance(accountId: string): Promise<AccountTruePerformance> {
    const res = await api.get(`/api/accounts/${accountId}/true-performance`);
    return res.data;
}

export async function getPortfolioTruePerformance(portfolioId: string): Promise<AccountTruePerformance[]> {
    const res = await api.get(`/api/portfolios/${portfolioId}/true-performance`);
    return res.data;
}

// Cash flow endpoints
export async function createCashFlow(accountId: string, payload: {
    flow_type: 'DEPOSIT' | 'WITHDRAWAL';
    amount: number;
    flow_date: string; // YYYY-MM-DD
    description?: string;
}): Promise<CashFlow> {
    const res = await api.post(`/api/accounts/${accountId}/cash-flows`, payload);
    return res.data;
}

export async function listCashFlows(accountId: string): Promise<CashFlow[]> {
    const res = await api.get(`/api/accounts/${accountId}/cash-flows`);
    return res.data;
}

// Admin endpoints
export async function resetAllData(): Promise<{ message: string; tables_cleared: string[] }> {
    const res = await api.post('/api/admin/reset-all-data');
    return res.data;
}