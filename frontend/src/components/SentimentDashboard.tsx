import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
    Box,
    Card,
    CardContent,
    Typography,
    Alert,
    CircularProgress,
    Chip,
    Stack,
    Grid,
    Paper,
    Tooltip,
} from '@mui/material';
import {
    TrendingUp,
    TrendingDown,
    TrendingFlat,
    Warning,
    CheckCircle,
    Error,
    Info,
} from '@mui/icons-material';
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip as RechartsTooltip,
    Legend,
    ResponsiveContainer,
    ReferenceLine,
} from 'recharts';
import { getPositionSentiment } from '../lib/endpoints';
import { ExperimentalBanner } from './ExperimentalBanner';
import type { SentimentSignal, SentimentTrend, MomentumTrend, DivergenceType } from '../types';

interface SentimentDashboardProps {
    ticker: string;
}

export function SentimentDashboard({ ticker }: SentimentDashboardProps) {
    const [days] = useState(30);

    const sentimentQuery = useQuery({
        queryKey: ['sentiment', ticker, days],
        queryFn: () => getPositionSentiment(ticker, days),
        staleTime: 1000 * 60 * 60 * 6, // 6 hours
        retry: 1,
    });

    if (sentimentQuery.isLoading) {
        return (
            <Box display="flex" flexDirection="column" justifyContent="center" alignItems="center" minHeight="400px" gap={2}>
                <CircularProgress />
                <Typography variant="body2" color="text.secondary">
                    Analyzing sentiment for {ticker}...
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
                    <ExperimentalBanner feature="Sentiment Analysis" />
                    <Alert severity="warning">
                        <Typography variant="body1" gutterBottom>
                            <strong>Sentiment Analysis Not Available</strong>
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
                No sentiment data available for {ticker}.
            </Alert>
        );
    }

    const signal: SentimentSignal = sentimentQuery.data;

    // Transform data for chart
    const chartData = signal.historical_sentiment.map((point) => ({
        date: new Date(point.date).toLocaleDateString(),
        sentiment: point.sentiment_score,
        price: point.price,
        newsVolume: point.news_volume,
    }));

    return (
        <Card elevation={2}>
            <CardContent>
                <ExperimentalBanner feature="Sentiment Analysis" />

                {/* Header */}
                <Box display="flex" justifyContent="space-between" alignItems="center" mb={3}>
                    <Typography variant="h6">
                        Sentiment Analysis: {ticker}
                    </Typography>
                    <Typography variant="caption" color="text.secondary">
                        Last updated: {new Date(signal.calculated_at).toLocaleString()}
                    </Typography>
                </Box>

                {/* Key Metrics */}
                <Grid container spacing={2} mb={3}>
                    {/* Sentiment Score */}
                    <Grid item xs={12} md={3}>
                        <Paper elevation={1} sx={{ p: 2, textAlign: 'center' }}>
                            <Typography variant="caption" color="text.secondary">
                                Current Sentiment
                            </Typography>
                            <Typography variant="h4" color={getSentimentColor(signal.current_sentiment)}>
                                {signal.current_sentiment.toFixed(2)}
                            </Typography>
                            <Typography variant="caption">
                                {getSentimentLabel(signal.current_sentiment)}
                            </Typography>
                        </Paper>
                    </Grid>

                    {/* Sentiment Trend */}
                    <Grid item xs={12} md={3}>
                        <Paper elevation={1} sx={{ p: 2, textAlign: 'center' }}>
                            <Typography variant="caption" color="text.secondary">
                                Sentiment Trend
                            </Typography>
                            <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', my: 1 }}>
                                {getTrendIcon(signal.sentiment_trend)}
                                <Typography variant="h6" sx={{ ml: 1 }}>
                                    {signal.sentiment_trend}
                                </Typography>
                            </Box>
                        </Paper>
                    </Grid>

                    {/* Momentum Trend */}
                    <Grid item xs={12} md={3}>
                        <Paper elevation={1} sx={{ p: 2, textAlign: 'center' }}>
                            <Typography variant="caption" color="text.secondary">
                                Price Momentum
                            </Typography>
                            <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', my: 1 }}>
                                {getMomentumIcon(signal.momentum_trend)}
                                <Typography variant="h6" sx={{ ml: 1 }}>
                                    {signal.momentum_trend}
                                </Typography>
                            </Box>
                        </Paper>
                    </Grid>

                    {/* Divergence */}
                    <Grid item xs={12} md={3}>
                        <Paper elevation={1} sx={{ p: 2, textAlign: 'center' }}>
                            <Typography variant="caption" color="text.secondary">
                                Divergence Signal
                            </Typography>
                            <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', my: 1 }}>
                                {getDivergenceIcon(signal.divergence)}
                                <Typography variant="h6" sx={{ ml: 1 }}>
                                    {signal.divergence}
                                </Typography>
                            </Box>
                        </Paper>
                    </Grid>
                </Grid>

                {/* Correlation Info */}
                {signal.sentiment_price_correlation && (
                    <Alert severity="info" sx={{ mb: 3 }}>
                        <Typography variant="body2">
                            <strong>Correlation:</strong> {signal.sentiment_price_correlation.toFixed(3)} (
                            {signal.correlation_strength})
                            {signal.correlation_lag_days !== undefined && signal.correlation_lag_days > 0 && (
                                <> • Sentiment leads price by {signal.correlation_lag_days} day(s)</>
                            )}
                        </Typography>
                        <Typography variant="caption">
                            Based on {signal.news_articles_analyzed} news articles analyzed
                        </Typography>
                    </Alert>
                )}

                {/* Warnings */}
                {signal.warnings.length > 0 && (
                    <Alert severity="warning" sx={{ mb: 3 }}>
                        <Typography variant="body2" fontWeight={600} gutterBottom>
                            Data Quality Warnings:
                        </Typography>
                        {signal.warnings.map((warning, idx) => (
                            <Typography key={idx} variant="body2">
                                • {warning}
                            </Typography>
                        ))}
                    </Alert>
                )}

                {/* Historical Chart */}
                <Box mb={3}>
                    <Typography variant="subtitle1" gutterBottom>
                        Sentiment History (30 days)
                    </Typography>
                    {chartData.length > 0 ? (
                        <ResponsiveContainer width="100%" height={300}>
                            <LineChart data={chartData} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
                                <CartesianGrid strokeDasharray="3 3" />
                                <XAxis
                                    dataKey="date"
                                    angle={-45}
                                    textAnchor="end"
                                    height={80}
                                    tick={{ fontSize: 12 }}
                                />
                                <YAxis
                                    yAxisId="left"
                                    domain={[-1, 1]}
                                    label={{ value: 'Sentiment Score', angle: -90, position: 'insideLeft' }}
                                />
                                <RechartsTooltip
                                    content={({ active, payload }) => {
                                        if (active && payload && payload.length) {
                                            return (
                                                <Box
                                                    sx={{
                                                        backgroundColor: 'white',
                                                        border: '1px solid #ccc',
                                                        padding: 2,
                                                        borderRadius: 1,
                                                    }}
                                                >
                                                    <Typography variant="body2">
                                                        <strong>Date:</strong> {payload[0].payload.date}
                                                    </Typography>
                                                    <Typography variant="body2" color="primary">
                                                        <strong>Sentiment:</strong> {(payload[0].payload.sentiment as number).toFixed(3)}
                                                    </Typography>
                                                    <Typography variant="body2">
                                                        <strong>News Volume:</strong> {payload[0].payload.newsVolume} articles
                                                    </Typography>
                                                </Box>
                                            );
                                        }
                                        return null;
                                    }}
                                />
                                <Legend />
                                <ReferenceLine
                                    yAxisId="left"
                                    y={0}
                                    stroke="#666"
                                    strokeDasharray="3 3"
                                    label={{ value: 'Neutral (0.0)', position: 'right' }}
                                />
                                <Line
                                    yAxisId="left"
                                    type="monotone"
                                    dataKey="sentiment"
                                    stroke="#1976d2"
                                    strokeWidth={2}
                                    dot={false}
                                    name="Sentiment Score"
                                />
                            </LineChart>
                        </ResponsiveContainer>
                    ) : (
                        <Alert severity="info">
                            Insufficient historical data for chart visualization.
                        </Alert>
                    )}
                </Box>

                {/* Methodology Link */}
                <Box sx={{ mt: 3, pt: 2, borderTop: '1px solid #e0e0e0' }}>
                    <Typography variant="caption" color="text.secondary">
                        <Info fontSize="small" sx={{ verticalAlign: 'middle', mr: 0.5 }} />
                        Sentiment scores are calculated from news article themes, weighted by relevance and recency.
                        Correlation analysis uses Pearson correlation with lag detection.
                    </Typography>
                </Box>
            </CardContent>
        </Card>
    );
}

// Helper functions
function getSentimentColor(score: number): string {
    if (score > 0.3) return 'success.main';
    if (score < -0.3) return 'error.main';
    return 'text.secondary';
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
            return <TrendingUp color="success" />;
        case 'deteriorating':
            return <TrendingDown color="error" />;
        default:
            return <TrendingFlat color="action" />;
    }
}

function getMomentumIcon(momentum: MomentumTrend) {
    switch (momentum) {
        case 'bullish':
            return <TrendingUp color="success" />;
        case 'bearish':
            return <TrendingDown color="error" />;
        default:
            return <TrendingFlat color="action" />;
    }
}

function getDivergenceIcon(divergence: DivergenceType) {
    switch (divergence) {
        case 'bullish':
            return <CheckCircle color="success" />;
        case 'bearish':
            return <Warning color="warning" />;
        case 'confirmed':
            return <CheckCircle color="primary" />;
        default:
            return <TrendingFlat color="action" />;
    }
}
