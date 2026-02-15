import { api } from "./api";
import type {
    AnalyticsResponse,
    Portfolio,
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
    AccountTruePerformance,
    RiskAssessment,
    PortfolioRisk,
    PortfolioRiskWithViolations,
    CorrelationMatrix,
    RiskSnapshot,
    RiskAlert,
    RiskThresholdSettings,
    UpdateRiskThresholds,
    OptimizationAnalysis,
    UserPreferences,
    UpdateUserPreferences,
    LlmUsageStats,
    PortfolioNarrative,
    PortfolioNewsAnalysis,
    NewsTheme,
    PortfolioQuestion,
    PortfolioAnswer
} from "../types";

export async function listPortfolios(): Promise<Portfolio[]> {
    const res = await api.get("/api/portfolios");
    return res.data;
}

export async function createPortfolio(name: string): Promise<Portfolio> {
    const res = await api.post("/api/portfolios", { name });
    return res.data;
}

export async function deletePortfolio(portfolioId: string): Promise<void> {
    await api.delete(`/api/portfolios/${portfolioId}`);
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

export async function getPriceHistory(ticker: string): Promise<PricePoint[]> {
    const res = await api.get(`/api/prices/${ticker}`);
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

// Risk endpoints
export async function getPositionRisk(
    ticker: string,
    days?: number,
    benchmark?: string
): Promise<RiskAssessment> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());
    if (benchmark) params.append('benchmark', benchmark);

    const queryString = params.toString();
    const url = `/api/risk/positions/${ticker}${queryString ? `?${queryString}` : ''}`;

    const res = await api.get(url);
    return res.data;
}

export async function getPortfolioRisk(
    portfolioId: string,
    days?: number,
    benchmark?: string
): Promise<PortfolioRiskWithViolations> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());
    if (benchmark) params.append('benchmark', benchmark);

    const queryString = params.toString();
    const url = `/api/risk/portfolios/${portfolioId}${queryString ? `?${queryString}` : ''}`;

    const res = await api.get(url);
    return res.data;
}

export async function getPortfolioCorrelations(
    portfolioId: string,
    days?: number
): Promise<CorrelationMatrix> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());

    const queryString = params.toString();
    const url = `/api/risk/portfolios/${portfolioId}/correlations${queryString ? `?${queryString}` : ''}`;

    // Use longer timeout for correlation calculation (2 minutes)
    const res = await api.get(url, { timeout: 120000 });
    return res.data;
}

// Risk snapshot endpoints
export async function createRiskSnapshot(
    portfolioId: string
): Promise<RiskSnapshot[]> {
    const res = await api.post(`/api/risk/portfolios/${portfolioId}/snapshot`);
    return res.data;
}

export async function getRiskHistory(
    portfolioId: string,
    ticker?: string,
    days?: number
): Promise<RiskSnapshot[]> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());
    if (ticker) params.append('ticker', ticker);

    const queryString = params.toString();
    const url = `/api/risk/portfolios/${portfolioId}/history${queryString ? `?${queryString}` : ''}`;

    const res = await api.get(url);
    return res.data;
}

export async function getRiskAlerts(
    portfolioId: string,
    days?: number,
    threshold?: number
): Promise<RiskAlert[]> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());
    if (threshold) params.append('threshold', threshold.toString());

    const queryString = params.toString();
    const url = `/api/risk/portfolios/${portfolioId}/alerts${queryString ? `?${queryString}` : ''}`;

    const res = await api.get(url);
    return res.data;
}

// Risk threshold settings endpoints
export async function getRiskThresholds(
    portfolioId: string
): Promise<RiskThresholdSettings> {
    const res = await api.get(`/api/risk/portfolios/${portfolioId}/thresholds`);
    return res.data;
}

export async function updateRiskThresholds(
    portfolioId: string,
    thresholds: UpdateRiskThresholds
): Promise<RiskThresholdSettings> {
    const res = await api.post(`/api/risk/portfolios/${portfolioId}/thresholds`, thresholds);
    return res.data;
}

// Portfolio narrative endpoint
export async function getPortfolioNarrative(
    portfolioId: string,
    timePeriod?: string
): Promise<PortfolioNarrative> {
    const params = new URLSearchParams();
    if (timePeriod) params.append('time_period', timePeriod);

    const queryString = params.toString();
    const url = `/api/risk/portfolios/${portfolioId}/narrative${queryString ? `?${queryString}` : ''}`;

    const res = await api.get(url);
    return res.data;
}

// Risk export endpoints
export async function exportPortfolioRiskCSV(
    portfolioId: string,
    days?: number,
    benchmark?: string
): Promise<void> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());
    if (benchmark) params.append('benchmark', benchmark);

    const queryString = params.toString();
    const url = `/api/risk/portfolios/${portfolioId}/export/csv${queryString ? `?${queryString}` : ''}`;

    // Download CSV file
    const res = await api.get(url, {
        responseType: 'blob',
        timeout: 120000, // 2 minutes for large portfolios
    });

    // Extract filename from Content-Disposition header
    const contentDisposition = res.headers['content-disposition'];
    let filename = 'portfolio_risk_export.csv';
    if (contentDisposition) {
        const match = contentDisposition.match(/filename="?([^"]+)"?/);
        if (match) {
            filename = match[1];
        }
    }

    // Create download link
    const blob = new Blob([res.data], { type: 'text/csv' });
    const downloadUrl = window.URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = downloadUrl;
    link.download = filename;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(downloadUrl);
}

// Portfolio optimization endpoints
export async function getPortfolioOptimization(
    portfolioId: string
): Promise<OptimizationAnalysis> {
    const res = await api.get(`/api/optimization/portfolios/${portfolioId}`);
    return res.data;
}
// LLM / AI Features endpoints
export async function getUserPreferences(userId: string): Promise<UserPreferences> {
    const res = await api.get(`/api/llm/users/${userId}/preferences`);
    return res.data;
}

export async function updateUserPreferences(
    userId: string,
    preferences: UpdateUserPreferences
): Promise<UserPreferences> {
    const res = await api.put(`/api/llm/users/${userId}/preferences`, preferences);
    return res.data;
}

export async function updateLlmConsent(
    userId: string,
    consent: boolean
): Promise<UserPreferences> {
    const res = await api.post(`/api/llm/users/${userId}/llm-consent`, { consent });
    return res.data;
}

export async function getLlmUsageStats(userId: string): Promise<LlmUsageStats> {
    const res = await api.get(`/api/llm/users/${userId}/usage`);
    return res.data;
}

// News endpoints
export async function getPortfolioNews(
    portfolioId: string,
    days?: number
): Promise<PortfolioNewsAnalysis> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());

    const queryString = params.toString();
    const url = `/api/news/portfolios/${portfolioId}/news${queryString ? `?${queryString}` : ''}`;

    const res = await api.get(url);
    return res.data;
}

export async function getTickerNews(
    ticker: string,
    days?: number
): Promise<NewsTheme[]> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());

    const queryString = params.toString();
    const url = `/api/news/positions/${ticker}/news${queryString ? `?${queryString}` : ''}`;

    const res = await api.get(url);
    return res.data;
}

// Q&A endpoint
export async function askPortfolioQuestion(
    portfolioId: string,
    question: PortfolioQuestion
): Promise<PortfolioAnswer> {
    const res = await api.post(`/api/qa/portfolios/${portfolioId}/ask`, question);
    return res.data;
}
