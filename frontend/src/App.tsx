import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import {
    createPortfolio,
    createPosition,
    getAnalytics,
    listPortfolios,
    listPositions,
    updatePrices,
} from "./lib/endpoints";

import { PortfolioChart } from "./components/PortfolioChart";

export default function App() {
    const qc = useQueryClient();

    const [selectedPortfolioId, setSelectedPortfolioId] = useState<string | null>(null);

    // 1) Portfolios
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

    // 2) Positions for selected portfolio
    const positionsQ = useQuery({
        queryKey: ["positions", selectedPortfolioId],
        queryFn: () => listPositions(selectedPortfolioId!),
        enabled: !!selectedPortfolioId,
    });

    // 3) Analytics for selected portfolio
    const analyticsQ = useQuery({
        queryKey: ["analytics", selectedPortfolioId],
        queryFn: () => getAnalytics(selectedPortfolioId!),
        enabled: !!selectedPortfolioId,
    });

    // Mutations
    const createPortfolioM = useMutation({
        mutationFn: (name: string) => createPortfolio(name),
        onSuccess: async () => {
            await qc.invalidateQueries({ queryKey: ["portfolios"] });
        },
    });

    const createPositionM = useMutation({
        mutationFn: (args: {
            portfolioId: string;
            ticker: string;
            shares: number;
            avg_buy_price: number;
        }) =>
            createPosition(args.portfolioId, {
                ticker: args.ticker,
                shares: args.shares,
                avg_buy_price: args.avg_buy_price,
            }),
        onSuccess: async () => {
            await qc.invalidateQueries({ queryKey: ["positions", selectedPortfolioId] });
            await qc.invalidateQueries({ queryKey: ["analytics", selectedPortfolioId] });
        },
    });

    const updatePricesM = useMutation({
        mutationFn: (ticker: string) => updatePrices(ticker),
        onSuccess: async () => {
            await qc.invalidateQueries({ queryKey: ["analytics", selectedPortfolioId] });
        },
        onError: (err: any) => {
            const status = err?.response?.status;
            if (status === 429) {
                alert("Rate limited by provider. Try again in a minute.");
            } else {
                alert("Price update failed. Check backend logs.");
            }
        },
    });

    // Derived convenience values
    const tickers = useMemo(
        () => (positionsQ.data ?? []).map((p) => p.ticker),
        [positionsQ.data]
    );

    return (
        <div style={{ fontFamily: "system-ui", padding: 16, maxWidth: 1100, margin: "0 auto" }}>
            <h1 style={{ margin: 0 }}>Rustfolio</h1>
            <p style={{ marginTop: 6, color: "#666" }}>
                Frontend integration (Chapter 8): portfolios, holdings, analytics chart
            </p>

            {/* Portfolio selector + create */}
            <section style={{ display: "flex", gap: 12, alignItems: "center", marginTop: 12 }}>
                <label style={{ fontWeight: 600 }}>Portfolio:</label>

                {portfoliosQ.isLoading ? (
                    <span>Loading…</span>
                ) : portfoliosQ.isError ? (
                    <span style={{ color: "crimson" }}>Failed to load portfolios</span>
                ) : (
                    <select
                        value={selectedPortfolioId ?? ""}
                        onChange={(e) => setSelectedPortfolioId(e.target.value)}
                    >
                        {(portfoliosQ.data ?? []).map((p) => (
                            <option key={p.id} value={p.id}>
                                {p.name}
                            </option>
                        ))}
                    </select>
                )}

                <button
                    onClick={() => createPortfolioM.mutate(`New Portfolio ${new Date().toISOString()}`)}
                    disabled={createPortfolioM.isPending}
                >
                    + New
                </button>
            </section>

            <hr style={{ margin: "16px 0" }} />

            {/* Holdings */}
            <section>
                <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
                    <h2 style={{ margin: 0 }}>Holdings</h2>

                    {selectedPortfolioId ? (
                        <button
                            onClick={() =>
                                createPositionM.mutate({
                                    portfolioId: selectedPortfolioId,
                                    ticker: "AAPL",
                                    shares: 1,
                                    avg_buy_price: 150,
                                })
                            }
                            disabled={createPositionM.isPending}
                            title="Demo helper: adds AAPL position"
                        >
                            + Add AAPL (demo)
                        </button>
                    ) : null}
                </div>

                {positionsQ.isLoading ? <p>Loading holdings…</p> : null}
                {positionsQ.isError ? (
                    <p style={{ color: "crimson" }}>Failed to load holdings</p>
                ) : null}

                <ul style={{ paddingLeft: 18 }}>
                    {(positionsQ.data ?? []).map((pos) => (
                        <li key={pos.id} style={{ display: "flex", gap: 10, alignItems: "center" }}>
                            <b style={{ width: 70 }}>{pos.ticker}</b>
                            <span>shares: {pos.shares}</span>
                            <span>avg: {pos.avg_buy_price}</span>
                            <button
                                onClick={() => updatePricesM.mutate(pos.ticker)}
                                disabled={updatePricesM.isPending}
                            >
                                Update price
                            </button>
                        </li>
                    ))}
                </ul>

                {tickers.length === 0 ? (
                    <p style={{ color: "#666" }}>
                        Add at least one position, then generate prices using backend:
                        <code style={{ marginLeft: 6 }}>POST /api/prices/AAPL/mock</code>
                    </p>
                ) : null}
            </section>

            <hr style={{ margin: "16px 0" }} />

            {/* Analytics */}
            <section>
                <h2 style={{ marginTop: 0 }}>Analytics</h2>

                {analyticsQ.isLoading ? <p>Loading analytics…</p> : null}
                {analyticsQ.isError ? (
                    <p style={{ color: "crimson" }}>
                        Failed to load analytics. Make sure you have price_points data for tickers in this
                        portfolio.
                    </p>
                ) : null}

                {/* ✅ This is the part you asked for, in context */}
                {analyticsQ.data ? (
                    <div>
                        <div style={{ display: "flex", gap: 16, flexWrap: "wrap", marginBottom: 10 }}>
                            <div>
                                <b>Points:</b> {analyticsQ.data.meta.points}
                            </div>
                            <div>
                                <b>Range:</b> {analyticsQ.data.meta.start ?? "—"} →{" "}
                                {analyticsQ.data.meta.end ?? "—"}
                            </div>
                        </div>

                        <PortfolioChart series={analyticsQ.data.series} />
                    </div>
                ) : null}
            </section>

            <hr style={{ margin: "16px 0" }} />

            {/* Debug */}
            <section>
                <h3 style={{ marginTop: 0 }}>Debug</h3>
                <div>Selected portfolio: {selectedPortfolioId ?? "(none)"}</div>
                <div>Tickers: {tickers.join(", ") || "(none)"}</div>
            </section>
        </div>
    );
}