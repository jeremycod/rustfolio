import { useEffect, useMemo, useState } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
    Container,
    Typography,
    Box,
    FormControl,
    InputLabel,
    Select,
    MenuItem,
    Button,
    Divider,
    List,
    ListItem,
    ListItemText,
    Chip,
    Alert,
    CircularProgress,
    Paper,
    Grid,
} from '@mui/material';
import { Add, Refresh } from '@mui/icons-material';

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
        <Container maxWidth="lg" sx={{ py: 3 }}>
            <Typography variant="h1" gutterBottom>
                Rustfolio
            </Typography>
            <Typography variant="body1" color="text.secondary" gutterBottom>
                Frontend integration (Chapter 8): portfolios, holdings, analytics chart
            </Typography>

            {/* Portfolio selector + create */}
            <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', my: 3 }}>
                <FormControl sx={{ minWidth: 200 }}>
                    <InputLabel>Portfolio</InputLabel>
                    {portfoliosQ.isLoading ? (
                        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            <CircularProgress size={20} />
                            <Typography>Loading…</Typography>
                        </Box>
                    ) : portfoliosQ.isError ? (
                        <Alert severity="error">Failed to load portfolios</Alert>
                    ) : (
                        <Select
                            value={selectedPortfolioId ?? ""}
                            onChange={(e) => setSelectedPortfolioId(e.target.value)}
                            label="Portfolio"
                        >
                            {(portfoliosQ.data ?? []).map((p) => (
                                <MenuItem key={p.id} value={p.id}>
                                    {p.name}
                                </MenuItem>
                            ))}
                        </Select>
                    )}
                </FormControl>

                <Button
                    variant="contained"
                    startIcon={<Add />}
                    onClick={() => createPortfolioM.mutate(`New Portfolio ${new Date().toISOString()}`)}
                    disabled={createPortfolioM.isPending}
                >
                    New Portfolio
                </Button>
            </Box>

            <Divider sx={{ my: 2 }} />

            {/* Holdings */}
            <Paper sx={{ p: 3, mb: 3 }}>
                <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
                    <Typography variant="h2">Holdings</Typography>

                    {selectedPortfolioId && (
                        <Button
                            variant="outlined"
                            startIcon={<Add />}
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
                            Add AAPL (demo)
                        </Button>
                    )}
                </Box>

                {positionsQ.isLoading && (
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                        <CircularProgress size={20} />
                        <Typography>Loading holdings…</Typography>
                    </Box>
                )}
                {positionsQ.isError && (
                    <Alert severity="error">Failed to load holdings</Alert>
                )}

                <List>
                    {(positionsQ.data ?? []).map((pos) => (
                        <ListItem key={pos.id} sx={{ px: 0 }}>
                            <ListItemText
                                primary={
                                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                                        <Chip label={pos.ticker} color="primary" sx={{ minWidth: 80 }} />
                                        <Typography>Shares: {pos.shares}</Typography>
                                        <Typography>Avg: ${pos.avg_buy_price}</Typography>
                                    </Box>
                                }
                            />
                            <Button
                                variant="outlined"
                                size="small"
                                startIcon={<Refresh />}
                                onClick={() => updatePricesM.mutate(pos.ticker)}
                                disabled={updatePricesM.isPending}
                            >
                                Update Price
                            </Button>
                        </ListItem>
                    ))}
                </List>

                {tickers.length === 0 && (
                    <Alert severity="info">
                        Add at least one position, then generate prices using backend:
                        <Box component="code" sx={{ ml: 1, fontFamily: 'monospace' }}>
                            POST /api/prices/AAPL/mock
                        </Box>
                    </Alert>
                )}
            </Paper>

            {/* Analytics */}
            <Paper sx={{ p: 3, mb: 3 }}>
                <Typography variant="h2" gutterBottom>
                    Analytics
                </Typography>

                {analyticsQ.isLoading && (
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                        <CircularProgress size={20} />
                        <Typography>Loading analytics…</Typography>
                    </Box>
                )}
                {analyticsQ.isError && (
                    <Alert severity="error">
                        Failed to load analytics. Make sure you have price_points data for tickers in this
                        portfolio.
                    </Alert>
                )}

                {analyticsQ.data && (
                    <Box>
                        <Grid container spacing={2} sx={{ mb: 2 }}>
                            <Grid item>
                                <Chip 
                                    label={`Points: ${analyticsQ.data.meta.points}`} 
                                    variant="outlined" 
                                />
                            </Grid>
                            <Grid item>
                                <Chip 
                                    label={`Range: ${analyticsQ.data.meta.start ?? "—"} → ${analyticsQ.data.meta.end ?? "—"}`}
                                    variant="outlined"
                                />
                            </Grid>
                        </Grid>

                        <PortfolioChart series={analyticsQ.data.series} />
                    </Box>
                )}
            </Paper>

            {/* Debug */}
            <Paper sx={{ p: 2 }}>
                <Typography variant="h6" gutterBottom>
                    Debug
                </Typography>
                <Typography variant="body2" color="text.secondary">
                    Selected portfolio: {selectedPortfolioId ?? "(none)"}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                    Tickers: {tickers.join(", ") || "(none)"}
                </Typography>
            </Paper>
        </Container>
    );
}