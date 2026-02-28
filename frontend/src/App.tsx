import { useEffect, useState } from "react";
import { useQuery, useQueryClient } from "@tanstack/react-query";
import { Box, Typography, Alert } from "@mui/material";
import { listPortfolios } from "./lib/endpoints";
import { Layout } from "./components/Layout";
import { Dashboard } from "./components/Dashboard";
import { PortfolioOverview } from "./components/PortfolioOverview";
import { Analytics } from "./components/Analytics";
import { Settings } from "./components/Settings";
import { Accounts } from "./components/Accounts";
import { AccountDetail } from "./components/AccountDetail";
import { RiskAnalysis } from "./components/RiskAnalysis";
import { PortfolioRiskOverview } from "./components/PortfolioRiskOverview";
import { RiskComparison } from "./components/RiskComparison";
import { CorrelationHeatmap } from "./components/CorrelationHeatmap";
import { RollingBetaPage } from "./components/RollingBetaPage";
import { AdminDashboard } from "./components/AdminDashboard";
import { usePreferences } from "./contexts/PreferencesContext";
import NotificationsPage from "./components/NotificationsPage";
import AlertRulesPage from "./components/AlertRulesPage";
import AlertHistoryPage from "./components/AlertHistoryPage";
// Phase 1 & 2 components
import { CVaRAnalysis } from "./components/CVaRAnalysis";
import { DownsideRiskAnalysis } from "./components/DownsideRiskAnalysis";
import { MarketRegimePage } from "./components/MarketRegimePage";
import { VolatilityForecasting } from "./components/VolatilityForecasting";
import { TradingSignalsPage } from "./components/TradingSignalsPage";
import { SentimentForecasting } from "./components/SentimentForecasting";
// Phase 3 components
import { ScreeningPage } from "./components/ScreeningPage";
import { WatchlistPage } from "./components/WatchlistPage";
import { LongTermGuidancePage } from "./components/LongTermGuidancePage";
import { FactorPortfolioPage } from "./components/FactorPortfolioPage";

export default function App() {
    const [selectedPortfolioId, setSelectedPortfolioId] = useState<string | null>(null);
    const [currentPage, setCurrentPage] = useState('dashboard');
    const [selectedAccountId, setSelectedAccountId] = useState<string | null>(null);
    const [selectedTicker, setSelectedTicker] = useState<string | null>(null);

    const queryClient = useQueryClient();
    const { autoRefresh } = usePreferences();

    const portfoliosQ = useQuery({
        queryKey: ["portfolios"],
        queryFn: listPortfolios,
    });

    // Auto-select first portfolio once loaded
    useEffect(() => {
        if (!selectedPortfolioId && portfoliosQ.data?.length) {
            setSelectedPortfolioId(portfoliosQ.data[0].id);
        }
    }, [portfoliosQ.data, selectedPortfolioId]);

    // Auto-refresh data when enabled
    useEffect(() => {
        if (!autoRefresh) return;

        const interval = setInterval(() => {
            queryClient.invalidateQueries();
        }, 60000); // Refresh every 60 seconds

        return () => clearInterval(interval);
    }, [autoRefresh, queryClient]);

    const renderPage = () => {
        switch (currentPage) {
            case 'dashboard':
                return (
                    <Dashboard
                        selectedPortfolioId={selectedPortfolioId}
                        onPortfolioChange={setSelectedPortfolioId}
                        onNavigate={handlePageChange}
                        onTickerNavigate={handleTickerNavigate}
                    />
                );
            case 'accounts':
                if (selectedAccountId) {
                    return (
                        <AccountDetail
                            accountId={selectedAccountId}
                            onBack={() => setSelectedAccountId(null)}
                            onTickerNavigate={handleTickerNavigate}
                        />
                    );
                }
                return (
                    <Accounts
                        selectedPortfolioId={selectedPortfolioId}
                        onPortfolioChange={setSelectedPortfolioId}
                        onAccountSelect={(accountId) => setSelectedAccountId(accountId)}
                    />
                );
            case 'holdings':
                return (
                    <PortfolioOverview
                        selectedPortfolioId={selectedPortfolioId}
                        onPortfolioChange={setSelectedPortfolioId}
                        onTickerNavigate={handleTickerNavigate}
                    />
                );
            case 'analytics':
                return (
                    <Analytics
                        selectedPortfolioId={selectedPortfolioId}
                        onPortfolioChange={setSelectedPortfolioId}
                        onTickerNavigate={handleTickerNavigate}
                    />
                );
            case 'risk':
                return <RiskAnalysis selectedTicker={selectedTicker} />;
            case 'portfolio-risk':
                return (
                    <PortfolioRiskOverview
                        selectedPortfolioId={selectedPortfolioId}
                        onPortfolioChange={setSelectedPortfolioId}
                        onTickerNavigate={handleTickerNavigate}
                    />
                );
            case 'risk-comparison':
                return <RiskComparison onTickerNavigate={handleTickerNavigate} />;
            case 'correlations':
                return selectedPortfolioId ? (
                    <CorrelationHeatmap
                        portfolioId={selectedPortfolioId}
                        onTickerNavigate={handleTickerNavigate}
                    />
                ) : null;
            case 'rolling-beta':
                return (
                    <RollingBetaPage
                        selectedPortfolioId={selectedPortfolioId}
                        onPortfolioChange={setSelectedPortfolioId}
                        initialTicker={selectedTicker || undefined}
                    />
                );
            case 'settings':
                return <Settings />;
            case 'admin':
                return <AdminDashboard />;
            case 'notifications':
                return <NotificationsPage />;
            case 'alerts':
                return <AlertRulesPage />;
            case 'alert-history':
                return <AlertHistoryPage />;
            // Phase 1 features
            case 'cvar':
                return <CVaRAnalysis
                    selectedPortfolioId={selectedPortfolioId}
                    onTickerNavigate={handleTickerNavigate}
                />;
            case 'downside-risk':
                return selectedPortfolioId ?
                    <DownsideRiskAnalysis
                        portfolioId={selectedPortfolioId}
                        onTickerNavigate={handleTickerNavigate}
                    /> : null;
            case 'market-regime':
                return <MarketRegimePage />;
            // Phase 2 features
            case 'volatility-forecast':
                return <VolatilityForecasting initialTicker={selectedTicker || undefined} />;
            case 'trading-signals':
                return <TradingSignalsPage initialTicker={selectedTicker || undefined} />;
            case 'sentiment-forecast':
                return <SentimentForecasting initialTicker={selectedTicker || undefined} />;
            // Phase 3 features
            case 'screening':
                return <ScreeningPage onTickerNavigate={handleTickerNavigate} />;
            case 'watchlists':
                return <WatchlistPage onTickerNavigate={handleTickerNavigate} />;
            case 'long-term-guidance':
                return <LongTermGuidancePage />;
            case 'factor-portfolio':
                return <FactorPortfolioPage />;
            default:
                return (
                    <Dashboard
                        selectedPortfolioId={selectedPortfolioId}
                        onPortfolioChange={setSelectedPortfolioId}
                        onNavigate={handlePageChange}
                    />
                );
        }
    };

    const handlePageChange = (page: string) => {
        setCurrentPage(page);
        // Clear account selection when navigating away from accounts page
        if (page !== 'accounts') {
            setSelectedAccountId(null);
        }
        // Clear ticker selection when navigating away from ticker-specific pages
        const tickerPages = ['risk', 'volatility-forecast', 'trading-signals', 'sentiment-forecast', 'rolling-beta', 'downside-risk'];
        if (!tickerPages.includes(page)) {
            setSelectedTicker(null);
        }
    };

    const handleTickerNavigate = (ticker: string, page?: string) => {
        setSelectedTicker(ticker);
        setCurrentPage(page || 'risk');
    };

    return (
        <Layout
            currentPage={currentPage}
            onPageChange={handlePageChange}
        >
            {renderPage()}
        </Layout>
    );
}