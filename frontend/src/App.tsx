import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listPortfolios } from "./lib/endpoints";
import { Layout } from "./components/Layout";
import { Dashboard } from "./components/Dashboard";
import { PortfolioOverview } from "./components/PortfolioOverview";
import { Analytics } from "./components/Analytics";
import { Settings } from "./components/Settings";
import { Accounts } from "./components/Accounts";
import { AccountDetail } from "./components/AccountDetail";
import { RiskAnalysis } from "./components/RiskAnalysis";

export default function App() {
    const [selectedPortfolioId, setSelectedPortfolioId] = useState<string | null>(null);
    const [currentPage, setCurrentPage] = useState('dashboard');
    const [selectedAccountId, setSelectedAccountId] = useState<string | null>(null);
    const [selectedTicker, setSelectedTicker] = useState<string | null>(null);

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

    const renderPage = () => {
        switch (currentPage) {
            case 'dashboard':
                return (
                    <Dashboard
                        selectedPortfolioId={selectedPortfolioId}
                        onPortfolioChange={setSelectedPortfolioId}
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
                    />
                );
            case 'risk':
                return <RiskAnalysis selectedTicker={selectedTicker} />;
            case 'settings':
                return <Settings />;
            default:
                return (
                    <Dashboard
                        selectedPortfolioId={selectedPortfolioId}
                        onPortfolioChange={setSelectedPortfolioId}
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
        // Clear ticker selection when navigating away from risk page
        if (page !== 'risk') {
            setSelectedTicker(null);
        }
    };

    const handleTickerNavigate = (ticker: string) => {
        setSelectedTicker(ticker);
        setCurrentPage('risk');
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