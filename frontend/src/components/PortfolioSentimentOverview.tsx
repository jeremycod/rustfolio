import { useQuery } from '@tanstack/react-query';
import {
    Box,
    Card,
    CardContent,
    Typography,
    Alert,
    CircularProgress,
    Table,
    TableBody,
    TableCell,
    TableContainer,
    TableHead,
    TableRow,
    Paper,
    Chip,
    Button,
    Grid,
} from '@mui/material';
import {
    TrendingUp,
    TrendingDown,
    TrendingFlat,
    Warning,
    Visibility,
} from '@mui/icons-material';
import { getPortfolioSentiment } from '../lib/endpoints';
import { ExperimentalBanner } from './ExperimentalBanner';
import type { SentimentTrend, MomentumTrend, DivergenceType } from '../types';

interface PortfolioSentimentOverviewProps {
    portfolioId: string;
}

export function PortfolioSentimentOverview({ portfolioId }: PortfolioSentimentOverviewProps) {
    const sentimentQuery = useQuery({
        queryKey: ['portfolio-sentiment', portfolioId],
        queryFn: () => getPortfolioSentiment(portfolioId),
        staleTime: 1000 * 60 * 60 * 6, // 6 hours
        retry: 1,
    });

    if (sentimentQuery.isLoading) {
        return (
            <Box display="flex" flexDirection="column" justifyContent="center" alignItems="center" minHeight="400px" gap={2}>
                <CircularProgress />
                <Typography variant="body2" color="text.secondary">
                    Analyzing portfolio sentiment...
                </Typography>
            </Box>
        );
    }

    if (sentimentQuery.isError) {
        const error = sentimentQuery.error as any;
        const errorMessage = error?.response?.data || error?.message || 'Unknown error';

        return (
            <Card elevation={2}>
                <CardContent>
                    <ExperimentalBanner feature="Portfolio Sentiment Analysis" />
                    <Alert severity="warning">
                        <Typography variant="body1" gutterBottom>
                            <strong>Portfolio Sentiment Analysis Not Available</strong>
                        </Typography>
                        <Typography variant="body2">
                            {errorMessage}
                        </Typography>
                        {errorMessage.includes('news service') && (
                            <Typography variant="body2" sx={{ mt: 2 }}>
                                This feature requires news data integration which is being completed in a future update.
                            </Typography>
                        )}
                    </Alert>
                </CardContent>
            </Card>
        );
    }

    if (!sentimentQuery.data) {
        return (
            <Alert severity="info">
                No sentiment data available for this portfolio.
            </Alert>
        );
    }

    const analysis = sentimentQuery.data;

    return (
        <Box>
            <ExperimentalBanner feature="Portfolio Sentiment Analysis" />

            {/* Summary Cards */}
            <Grid container spacing={2} mb={3}>
                <Grid item xs={12} md={4}>
                    <Paper elevation={1} sx={{ p: 2, textAlign: 'center' }}>
                        <Typography variant="caption" color="text.secondary">
                            Portfolio Avg Sentiment
                        </Typography>
                        <Typography
                            variant="h4"
                            color={getSentimentColor(analysis.portfolio_avg_sentiment)}
                        >
                            {analysis.portfolio_avg_sentiment.toFixed(2)}
                        </Typography>
                        <Typography variant="caption">
                            {getSentimentLabel(analysis.portfolio_avg_sentiment)}
                        </Typography>
                    </Paper>
                </Grid>

                <Grid item xs={12} md={4}>
                    <Paper elevation={1} sx={{ p: 2, textAlign: 'center' }}>
                        <Typography variant="caption" color="text.secondary">
                            Bullish Divergences
                        </Typography>
                        <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', my: 1 }}>
                            <TrendingUp color="success" />
                            <Typography variant="h4" sx={{ ml: 1 }}>
                                {analysis.bullish_divergences}
                            </Typography>
                        </Box>
                        <Typography variant="caption">
                            Potential buying opportunities
                        </Typography>
                    </Paper>
                </Grid>

                <Grid item xs={12} md={4}>
                    <Paper elevation={1} sx={{ p: 2, textAlign: 'center' }}>
                        <Typography variant="caption" color="text.secondary">
                            Bearish Divergences
                        </Typography>
                        <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', my: 1 }}>
                            <TrendingDown color="error" />
                            <Typography variant="h4" sx={{ ml: 1 }}>
                                {analysis.bearish_divergences}
                            </Typography>
                        </Box>
                        <Typography variant="caption">
                            Potential warning signals
                        </Typography>
                    </Paper>
                </Grid>
            </Grid>

            {/* Position Signals Table */}
            <Card elevation={2}>
                <CardContent>
                    <Typography variant="h6" gutterBottom>
                        Position Sentiment Signals
                    </Typography>

                    {analysis.signals.length > 0 ? (
                        <TableContainer>
                            <Table>
                                <TableHead>
                                    <TableRow>
                                        <TableCell><strong>Ticker</strong></TableCell>
                                        <TableCell align="center"><strong>Sentiment</strong></TableCell>
                                        <TableCell align="center"><strong>Trend</strong></TableCell>
                                        <TableCell align="center"><strong>Momentum</strong></TableCell>
                                        <TableCell align="center"><strong>Divergence</strong></TableCell>
                                        <TableCell align="center"><strong>Correlation</strong></TableCell>
                                        <TableCell align="center"><strong>Articles</strong></TableCell>
                                        <TableCell align="center"><strong>Actions</strong></TableCell>
                                    </TableRow>
                                </TableHead>
                                <TableBody>
                                    {analysis.signals.map((signal: any) => (
                                        <TableRow key={signal.ticker} hover>
                                            <TableCell>
                                                <Typography variant="body2" fontWeight={600}>
                                                    {signal.ticker}
                                                </Typography>
                                            </TableCell>
                                            <TableCell align="center">
                                                <Chip
                                                    label={signal.current_sentiment.toFixed(2)}
                                                    size="small"
                                                    sx={{
                                                        backgroundColor: getSentimentColor(signal.current_sentiment),
                                                        color: 'white',
                                                    }}
                                                />
                                            </TableCell>
                                            <TableCell align="center">
                                                <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
                                                    {getTrendIcon(signal.sentiment_trend)}
                                                    <Typography variant="caption" sx={{ ml: 0.5 }}>
                                                        {signal.sentiment_trend}
                                                    </Typography>
                                                </Box>
                                            </TableCell>
                                            <TableCell align="center">
                                                <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
                                                    {getMomentumIcon(signal.momentum_trend)}
                                                    <Typography variant="caption" sx={{ ml: 0.5 }}>
                                                        {signal.momentum_trend}
                                                    </Typography>
                                                </Box>
                                            </TableCell>
                                            <TableCell align="center">
                                                <Chip
                                                    label={signal.divergence}
                                                    size="small"
                                                    color={getDivergenceColor(signal.divergence)}
                                                    icon={signal.divergence === 'bearish' || signal.divergence === 'bullish' ? <Warning /> : undefined}
                                                />
                                            </TableCell>
                                            <TableCell align="center">
                                                {signal.sentiment_price_correlation ? (
                                                    <Typography variant="body2">
                                                        {signal.sentiment_price_correlation.toFixed(2)}
                                                        <br />
                                                        <Typography variant="caption" color="text.secondary">
                                                            ({signal.correlation_strength})
                                                        </Typography>
                                                    </Typography>
                                                ) : (
                                                    <Typography variant="caption" color="text.secondary">
                                                        N/A
                                                    </Typography>
                                                )}
                                            </TableCell>
                                            <TableCell align="center">
                                                <Typography variant="body2">
                                                    {signal.news_articles_analyzed}
                                                </Typography>
                                            </TableCell>
                                            <TableCell align="center">
                                                <Button
                                                    size="small"
                                                    startIcon={<Visibility />}
                                                    onClick={() => {
                                                        // TODO: Navigate to detailed sentiment view for this ticker
                                                        console.log('View details for', signal.ticker);
                                                    }}
                                                >
                                                    Details
                                                </Button>
                                            </TableCell>
                                        </TableRow>
                                    ))}
                                </TableBody>
                            </Table>
                        </TableContainer>
                    ) : (
                        <Alert severity="info">
                            No sentiment signals available for positions in this portfolio.
                        </Alert>
                    )}
                </CardContent>
            </Card>

            <Box sx={{ mt: 2 }}>
                <Typography variant="caption" color="text.secondary">
                    Last updated: {new Date(analysis.calculated_at).toLocaleString()}
                </Typography>
            </Box>
        </Box>
    );
}

// Helper functions
function getSentimentColor(score: number): string {
    if (score > 0.3) return '#4caf50'; // green
    if (score < -0.3) return '#f44336'; // red
    return '#9e9e9e'; // grey
}

function getSentimentLabel(score: number): string {
    if (score > 0.5) return 'Very Positive';
    if (score > 0.2) return 'Positive';
    if (score > -0.2) return 'Neutral';
    if (score > -0.5) return 'Negative';
    return 'Very Negative';
}

function getTrendIcon(trend: SentimentTrend) {
    switch (trend) {
        case 'improving':
            return <TrendingUp color="success" fontSize="small" />;
        case 'deteriorating':
            return <TrendingDown color="error" fontSize="small" />;
        default:
            return <TrendingFlat color="action" fontSize="small" />;
    }
}

function getMomentumIcon(momentum: MomentumTrend) {
    switch (momentum) {
        case 'bullish':
            return <TrendingUp color="success" fontSize="small" />;
        case 'bearish':
            return <TrendingDown color="error" fontSize="small" />;
        default:
            return <TrendingFlat color="action" fontSize="small" />;
    }
}

function getDivergenceColor(divergence: DivergenceType): 'success' | 'warning' | 'default' | 'primary' {
    switch (divergence) {
        case 'bullish':
            return 'success';
        case 'bearish':
            return 'warning';
        case 'confirmed':
            return 'primary';
        default:
            return 'default';
    }
}
