import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { listPortfolios } from "./lib/endpoints";
import { Layout } from "./components/Layout";
import { Dashboard } from "./components/Dashboard";
import { Holdings } from "./components/Holdings";
import { Analytics } from "./components/Analytics";
import { Settings } from "./components/Settings";

export default function App() {
    const [selectedPortfolioId, setSelectedPortfolioId] = useState<string | null>(null);
    const [currentPage, setCurrentPage] = useState('dashboard');

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
                    />
                );
            case 'holdings':
                return (
                    <Holdings 
                        selectedPortfolioId={selectedPortfolioId}
                        onPortfolioChange={setSelectedPortfolioId}
                    />
                );
            case 'analytics':
                return <Analytics selectedPortfolioId={selectedPortfolioId} />;
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

    return (
        <Layout 
            currentPage={currentPage} 
            onPageChange={setCurrentPage}
        >
            {renderPage()}
        </Layout>
    );
}