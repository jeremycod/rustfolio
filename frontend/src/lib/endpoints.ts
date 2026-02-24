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
    AccountTruePerformance,
    RiskAssessment,
    PortfolioRisk,
    PortfolioRiskWithViolations,
    CorrelationMatrix,
    CorrelationMatrixWithStats,
    RollingBetaAnalysis,
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
    PortfolioAnswer,
    PortfolioForecast,
    BetaForecast,
    ForecastMethod,
    SentimentSignal,
    PortfolioSentimentAnalysis,
    EnhancedSentimentSignal,
    JobRun,
    ScheduledJob,
    JobStats,
    CacheHealthStatus,
    AlertRule,
    CreateAlertRuleRequest,
    UpdateAlertRuleRequest,
    AlertHistory,
    Notification,
    NotificationCountResponse,
    NotificationPreferences,
    UpdateNotificationPreferences,
    AlertEvaluationResponse,
    TestAlertResponse,
    // Phase 1 & 2 types
    PortfolioDownsideRisk,
    MarketRegime,
    RegimeForecastResponse,
    VolatilityForecast,
    SignalResponse,
    SentimentAwareForecast,
    RiskPreferences,
    RiskProfile,
    // Phase 3 types
    ScreeningRequest,
    ScreeningResponse,
    RecommendationExplanation,
    Watchlist,
    WatchlistItem,
    WatchlistAlert,
    WatchlistThresholds,
    CreateWatchlistRequest,
    UpdateWatchlistRequest,
    AddWatchlistItemRequest,
    LongTermGuidance,
    InvestmentGoal,
    RiskAppetite,
    FactorType,
    FactorPortfolio,
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

export async function listPositions(portfolioId: string): Promise<Position[]> {
    const res = await api.get(`/api/portfolios/${portfolioId}/positions`);
    return res.data;
}

export async function createPosition(
    portfolioId: string,
    data: { ticker: string; shares: number; avg_buy_price: number }
): Promise<Position> {
    const res = await api.post(`/api/portfolios/${portfolioId}/positions`, data);
    return res.data;
}

export async function updatePosition(
    positionId: string,
    data: { shares: number; avg_buy_price: number }
): Promise<Position> {
    const res = await api.put(`/api/positions/${positionId}`, data);
    return res.data;
}

export async function deletePosition(positionId: string): Promise<void> {
    await api.delete(`/api/positions/${positionId}`);
}

export async function getAnalytics(portfolioId: string): Promise<AnalyticsResponse> {
    const res = await api.get(`/api/analytics/${portfolioId}`);
    return res.data;
}

export async function getPortfolioForecast(
    portfolioId: string,
    days?: number,
    method?: string
): Promise<PortfolioForecast> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());
    if (method) params.append('method', method);

    const queryString = params.toString();
    const url = `/api/analytics/${portfolioId}/forecast${queryString ? `?${queryString}` : ''}`;

    // Use longer timeout for forecast calculation (60 seconds)
    const res = await api.get(url, { timeout: 60000 });
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
    benchmark?: string,
    force?: boolean
): Promise<PortfolioRiskWithViolations> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());
    if (benchmark) params.append('benchmark', benchmark);
    if (force) params.append('force', 'true');

    const queryString = params.toString();
    const url = `/api/risk/portfolios/${portfolioId}${queryString ? `?${queryString}` : ''}`;

    // Use longer timeout for risk calculation (2 minutes)
    const res = await api.get(url, { timeout: 120000 });
    return res.data;
}

export async function getPortfolioCorrelations(
    portfolioId: string,
    days?: number,
    force?: boolean
): Promise<CorrelationMatrixWithStats> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());
    if (force) params.append('force', 'true');

    const queryString = params.toString();
    const url = `/api/risk/portfolios/${portfolioId}/correlations${queryString ? `?${queryString}` : ''}`;

    // Use longer timeout for correlation calculation (2 minutes)
    const res = await api.get(url, { timeout: 120000 });
    return res.data;
}

export async function getRollingBeta(
    ticker: string,
    days: number = 180,
    benchmark: string = 'SPY',
    force: boolean = false
): Promise<RollingBetaAnalysis> {
    const params = new URLSearchParams();
    params.append('days', days.toString());
    params.append('benchmark', benchmark);
    if (force) {
        params.append('force', 'true');
    }

    const queryString = params.toString();
    const url = `/api/risk/positions/${ticker}/rolling-beta${queryString ? `?${queryString}` : ''}`;

    // Use longer timeout for rolling beta calculation (2 minutes)
    const res = await api.get(url, { timeout: 120000 });
    return res.data;
}

export async function getBetaForecast(
    ticker: string,
    days: number = 30,
    benchmark: string = 'SPY',
    method?: ForecastMethod
): Promise<BetaForecast> {
    const params = new URLSearchParams();
    params.append('days', days.toString());
    params.append('benchmark', benchmark);
    if (method) params.append('method', method);

    const queryString = params.toString();
    const url = `/api/risk/positions/${ticker}/beta-forecast${queryString ? `?${queryString}` : ''}`;

    // Longer timeout for forecast calculation (60 seconds)
    const res = await api.get(url, { timeout: 60000 });
    return res.data;
}

// Sentiment Analysis endpoints (Sprint 18)
export async function getPositionSentiment(
    ticker: string,
    days: number = 30
): Promise<SentimentSignal> {
    const res = await api.get(
        `/api/sentiment/positions/${ticker}/sentiment?days=${days}`,
        { timeout: 60000 }
    );
    return res.data;
}

export async function getPortfolioSentiment(
    portfolioId: string
): Promise<PortfolioSentimentAnalysis> {
    const res = await api.get(
        `/api/sentiment/portfolios/${portfolioId}/sentiment`,
        { timeout: 10000 } // 10 second timeout
    );
    return res.data;
}

export async function getPortfolioSentimentCacheStatus(
    portfolioId: string
): Promise<{
    portfolio_id: string;
    total_positions: number;
    cached_positions: number;
    missing_positions: number;
    expired_positions: number;
    missing_tickers: string[];
    is_complete: boolean;
    is_fresh: boolean;
    oldest_update: string | null;
    cache_age_hours: number;
    message: string;
    recommendation: string;
}> {
    const res = await api.get(
        `/api/sentiment/portfolios/${portfolioId}/cache-status`
    );
    return res.data;
}

export async function getEnhancedSentiment(
    ticker: string,
    days: number = 30
): Promise<EnhancedSentimentSignal> {
    const res = await api.get(
        `/api/sentiment/positions/${ticker}/enhanced-sentiment?days=${days}`,
        { timeout: 120000 }
    );
    return res.data;
}

export async function updatePriceHistory(ticker: string): Promise<void> {
    const res = await api.post(`/api/prices/${ticker}/update`, {}, { timeout: 60000 });
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
    timePeriod?: string,
    force?: boolean
): Promise<PortfolioNarrative> {
    const params = new URLSearchParams();
    if (timePeriod) params.append('time_period', timePeriod);
    if (force) params.append('force', 'true');

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
    const res = await api.get(`/api/optimization/portfolios/${portfolioId}`, {
        timeout: 10000 // 10 second timeout
    });
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
    days?: number,
    force?: boolean
): Promise<PortfolioNewsAnalysis> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());
    if (force) params.append('force', 'true');

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

// Job Scheduler Admin endpoints
export async function listJobs(): Promise<ScheduledJob[]> {
    const res = await api.get('/api/admin/jobs');
    return res.data;
}

export async function getRecentJobRuns(limit?: number): Promise<JobRun[]> {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());

    const queryString = params.toString();
    const url = `/api/admin/jobs/runs/recent${queryString ? `?${queryString}` : ''}`;

    const res = await api.get(url);
    return res.data;
}

export async function getJobHistory(jobName: string, limit?: number): Promise<JobRun[]> {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());

    const queryString = params.toString();
    const url = `/api/admin/jobs/${encodeURIComponent(jobName)}/history${queryString ? `?${queryString}` : ''}`;

    const res = await api.get(url);
    return res.data;
}

export async function getJobStats(jobName: string): Promise<JobStats> {
    const res = await api.get(`/api/admin/jobs/${encodeURIComponent(jobName)}/stats`);
    return res.data;
}

export async function triggerJob(jobName: string): Promise<{ success: boolean; message: string; job_id?: number }> {
    const res = await api.post(`/api/admin/jobs/${encodeURIComponent(jobName)}/trigger`);
    return res.data;
}

export async function triggerAllJobs(): Promise<{
    total_jobs: number;
    successful: number;
    failed: number;
    job_results: Array<{
        job_name: string;
        status: string;
        message: string;
        duration_ms?: number;
        error_message?: string;
    }>;
    total_duration_ms: number;
}> {
    // Longer timeout for running all jobs (10 minutes)
    const res = await api.post('/api/admin/jobs/trigger-all', {}, { timeout: 600000 });
    return res.data;
}

export async function getCacheHealth(): Promise<CacheHealthStatus> {
    const res = await api.get('/api/admin/cache-health');
    return res.data;
}

// Alert Rules Endpoints
export async function listAlertRules(): Promise<AlertRule[]> {
    const res = await api.get('/api/alerts/rules');
    return res.data;
}

export async function getAlertRule(ruleId: string): Promise<AlertRule> {
    const res = await api.get(`/api/alerts/rules/${ruleId}`);
    return res.data;
}

export async function createAlertRule(data: CreateAlertRuleRequest): Promise<AlertRule> {
    const res = await api.post('/api/alerts/rules', data);
    return res.data;
}

export async function updateAlertRule(ruleId: string, data: UpdateAlertRuleRequest): Promise<AlertRule> {
    const res = await api.put(`/api/alerts/rules/${ruleId}`, data);
    return res.data;
}

export async function deleteAlertRule(ruleId: string): Promise<void> {
    await api.delete(`/api/alerts/rules/${ruleId}`);
}

export async function enableAlertRule(ruleId: string): Promise<AlertRule> {
    const res = await api.post(`/api/alerts/rules/${ruleId}/enable`);
    return res.data;
}

export async function disableAlertRule(ruleId: string): Promise<AlertRule> {
    const res = await api.post(`/api/alerts/rules/${ruleId}/disable`);
    return res.data;
}

export async function testAlertRule(ruleId: string): Promise<TestAlertResponse> {
    const res = await api.post(`/api/alerts/rules/${ruleId}/test`);
    return res.data;
}

// Alert History Endpoints
export async function getAlertHistory(limit?: number, offset?: number): Promise<AlertHistory[]> {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());
    if (offset) params.append('offset', offset.toString());
    const url = `/api/alerts/history${params.toString() ? '?' + params.toString() : ''}`;
    const res = await api.get(url);
    return res.data;
}

export async function getAlertHistoryById(id: string): Promise<AlertHistory> {
    const res = await api.get(`/api/alerts/history/${id}`);
    return res.data;
}

export async function getRuleHistory(ruleId: string, limit?: number): Promise<AlertHistory[]> {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());
    const url = `/api/alerts/rules/${ruleId}/history${params.toString() ? '?' + params.toString() : ''}`;
    const res = await api.get(url);
    return res.data;
}

// Notifications Endpoints
export async function getNotifications(limit?: number, offset?: number): Promise<Notification[]> {
    const params = new URLSearchParams();
    if (limit) params.append('limit', limit.toString());
    if (offset) params.append('offset', offset.toString());
    const url = `/api/notifications${params.toString() ? '?' + params.toString() : ''}`;
    const res = await api.get(url);
    return res.data;
}

export async function getUnreadNotificationCount(): Promise<NotificationCountResponse> {
    const res = await api.get('/api/notifications/unread');
    return res.data;
}

export async function markNotificationRead(notificationId: string): Promise<void> {
    await api.post(`/api/notifications/${notificationId}/read`);
}

export async function markAllNotificationsRead(): Promise<void> {
    await api.post('/api/notifications/mark-all-read');
}

export async function deleteNotification(notificationId: string): Promise<void> {
    await api.delete(`/api/notifications/${notificationId}`);
}

// Notification Preferences Endpoints
export async function getNotificationPreferences(): Promise<NotificationPreferences> {
    const res = await api.get('/api/notifications/preferences');
    return res.data;
}

export async function updateNotificationPreferences(data: UpdateNotificationPreferences): Promise<NotificationPreferences> {
    const res = await api.put('/api/notifications/preferences', data);
    return res.data;
}

export async function sendTestEmail(): Promise<{ message: string }> {
    const res = await api.post('/api/notifications/test-email');
    return res.data;
}

// Evaluation Endpoint
export async function evaluateAllAlerts(): Promise<AlertEvaluationResponse> {
    const res = await api.post('/api/alerts/evaluate-all');
    return res.data;
}

// Optimization Generation Endpoint
export async function generateOptimizationAnalysis(portfolioId: string): Promise<{ message: string; portfolio_id: string }> {
    const res = await api.post(`/api/optimization/portfolios/${portfolioId}/generate`);
    return res.data;
}

// ============================================================================
// Phase 1 & Phase 2 Enhanced Features Endpoints
// ============================================================================

// Phase 1: Downside Risk Analysis
export async function getPortfolioDownsideRisk(
    portfolioId: string,
    days: number = 90,
    benchmark: string = 'SPY',
    force: boolean = false
): Promise<any> {
    const params = new URLSearchParams();
    params.append('days', days.toString());
    params.append('benchmark', benchmark);
    if (force) {
        params.append('force', 'true');
    }

    const url = `/api/risk/portfolios/${portfolioId}/downside?${params.toString()}`;
    const res = await api.get(url, { timeout: 120000 });
    // Return full response including cache metadata
    return res.data;
}

// Phase 1: Market Regime Detection
export async function getMarketRegime(): Promise<MarketRegime> {
    const res = await api.get('/api/market/regime');
    return res.data;
}

export async function getRegimeForecast(days: number = 30): Promise<RegimeForecastResponse> {
    const res = await api.get(`/api/market/regime/forecast?days=${days}`);
    return res.data;
}

export async function getMarketRegimeHistory(days: number = 90): Promise<MarketRegime[]> {
    const res = await api.get(`/api/market/regime/history?days=${days}`);
    return res.data;
}

// Phase 2: Volatility Forecasting (GARCH)
export async function getVolatilityForecast(
    ticker: string,
    days: number = 30,
    confidence_level: number = 0.95
): Promise<VolatilityForecast> {
    const params = new URLSearchParams();
    params.append('days', days.toString());
    params.append('confidence_level', confidence_level.toString());

    const url = `/api/risk/positions/${ticker}/volatility-forecast?${params.toString()}`;
    const res = await api.get(url, { timeout: 60000 });
    return res.data;
}

// Phase 2: Trading Signals
export async function getTradingSignals(
    symbol: string,
    horizon: number = 3,
    signalTypes?: string[],
    minProbability?: number
): Promise<SignalResponse> {
    const params = new URLSearchParams();
    params.append('horizon', horizon.toString());
    if (signalTypes && signalTypes.length > 0) {
        params.append('signal_types', signalTypes.join(','));
    }
    if (minProbability !== undefined) {
        params.append('min_probability', minProbability.toString());
    }

    const url = `/api/stocks/${symbol}/signals?${params.toString()}`;
    const res = await api.get(url, { timeout: 60000 });
    return res.data;
}

export async function getSignalHistory(
    symbol: string,
    days: number = 30
): Promise<any[]> {
    const url = `/api/stocks/${symbol}/signals/history?days=${days}`;
    const res = await api.get(url);
    return res.data;
}

// Phase 2: Sentiment Forecasting
export async function getSentimentForecast(
    ticker: string,
    days: number = 30
): Promise<SentimentAwareForecast> {
    const url = `/api/sentiment/positions/${ticker}/sentiment-forecast?days=${days}`;
    const res = await api.get(url, { timeout: 120000 });
    return res.data;
}

// Phase 2: User Risk Preferences
export async function getUserRiskPreferences(userId: string): Promise<RiskPreferences> {
    const res = await api.get(`/api/users/${userId}/preferences`);
    return res.data;
}

export async function updateUserRiskPreferences(
    userId: string,
    preferences: Partial<RiskPreferences>
): Promise<RiskPreferences> {
    const res = await api.put(`/api/users/${userId}/preferences`, preferences);
    return res.data;
}

export async function resetUserRiskPreferences(userId: string): Promise<RiskPreferences> {
    const res = await api.post(`/api/users/${userId}/preferences/reset`);
    return res.data;
}

export async function getUserRiskProfile(userId: string): Promise<RiskProfile> {
    const res = await api.get(`/api/users/${userId}/risk-profile`);
    return res.data;
}

// ============================================================================
// Phase 3: AI-Powered Recommendations, Screening & Watchlists
// ============================================================================

// Screening Endpoints
export async function screenStocks(request: ScreeningRequest): Promise<ScreeningResponse> {
    const res = await api.post('/api/recommendations/screen', request, { timeout: 30000 });
    return res.data;
}

// Recommendation Explanation Endpoint
export async function getRecommendationExplanation(symbol: string): Promise<RecommendationExplanation> {
    const res = await api.get(`/api/recommendations/${symbol}/explanation`, { timeout: 30000 });
    return res.data;
}

// Long-Term Guidance Endpoint
export async function getLongTermGuidance(
    portfolioId: string,
    goal: InvestmentGoal,
    horizon: number,
    risk_tolerance: RiskAppetite
): Promise<LongTermGuidance> {
    const params = new URLSearchParams();
    params.append('goal', goal);
    params.append('horizon', horizon.toString());
    params.append('risk_tolerance', risk_tolerance);
    const url = `/api/recommendations/long-term/${portfolioId}?${params.toString()}`;
    const res = await api.get(url, { timeout: 30000 });
    return res.data;
}

// Factor-Based Recommendations Endpoint
export async function getFactorRecommendations(
    portfolioId: string,
    days?: number,
    includeBacktest?: boolean,
    includeEtfs?: boolean
): Promise<FactorPortfolio> {
    const params = new URLSearchParams();
    if (days) params.append('days', days.toString());
    if (includeBacktest !== undefined) params.append('include_backtest', includeBacktest.toString());
    if (includeEtfs !== undefined) params.append('include_etfs', includeEtfs.toString());
    const url = `/api/recommendations/factors/${portfolioId}?${params.toString()}`;
    const res = await api.get(url, { timeout: 30000 });
    return res.data;
}

// Watchlist Endpoints
export async function listWatchlists(): Promise<Watchlist[]> {
    const res = await api.get('/api/watchlists');
    return res.data;
}

export async function createWatchlist(data: CreateWatchlistRequest): Promise<Watchlist> {
    const res = await api.post('/api/watchlists', data);
    return res.data;
}

export async function getWatchlist(watchlistId: string): Promise<Watchlist> {
    const res = await api.get(`/api/watchlists/${watchlistId}`);
    return res.data;
}

export async function updateWatchlist(watchlistId: string, data: UpdateWatchlistRequest): Promise<Watchlist> {
    const res = await api.put(`/api/watchlists/${watchlistId}`, data);
    return res.data;
}

export async function deleteWatchlist(watchlistId: string): Promise<void> {
    await api.delete(`/api/watchlists/${watchlistId}`);
}

export async function getWatchlistItems(watchlistId: string): Promise<WatchlistItem[]> {
    const res = await api.get(`/api/watchlists/${watchlistId}/items`);
    return res.data;
}

export async function addWatchlistItem(watchlistId: string, data: AddWatchlistItemRequest): Promise<WatchlistItem> {
    const res = await api.post(`/api/watchlists/${watchlistId}/items`, data);
    return res.data;
}

export async function removeWatchlistItem(watchlistId: string, symbol: string): Promise<void> {
    await api.delete(`/api/watchlists/${watchlistId}/items/${symbol}`);
}

export async function updateWatchlistThresholds(
    watchlistId: string,
    symbol: string,
    thresholds: WatchlistThresholds
): Promise<WatchlistItem> {
    const res = await api.put(`/api/watchlists/${watchlistId}/items/${symbol}/thresholds`, thresholds);
    return res.data;
}

export async function getWatchlistAlerts(watchlistId: string): Promise<WatchlistAlert[]> {
    const res = await api.get(`/api/watchlists/${watchlistId}/alerts`);
    return res.data;
}

// Export screening results to CSV
export async function exportScreeningCSV(request: ScreeningRequest): Promise<void> {
    const res = await api.post('/api/recommendations/screen/export', request, {
        responseType: 'blob',
        timeout: 30000,
    });

    const blob = new Blob([res.data], { type: 'text/csv' });
    const downloadUrl = window.URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = downloadUrl;
    link.download = `screening_results_${new Date().toISOString().slice(0, 10)}.csv`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
    window.URL.revokeObjectURL(downloadUrl);
}
