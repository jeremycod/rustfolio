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
    Divider,
    List,
    ListItem,
    ListItemText,
    IconButton,
    Collapse,
} from '@mui/material';
import {
    TrendingUp,
    TrendingDown,
    Info,
    Warning,
    CheckCircle,
    ExpandMore,
    OpenInNew,
    Article,
    Business,
    PersonOutline,
    HelpOutline,
} from '@mui/icons-material';
import { getEnhancedSentiment } from '../lib/endpoints';
import { ExperimentalBanner } from './ExperimentalBanner';
import { MetricHelpDialog } from './MetricHelpDialog';
import type {
    EnhancedSentimentSignal,
    MaterialEvent,
    InsiderTransaction,
    EventImportance,
    ConfidenceLevel,
} from '../types';

interface EnhancedSentimentDashboardProps {
    ticker: string;
}

export function EnhancedSentimentDashboard({ ticker }: EnhancedSentimentDashboardProps) {
    const [days] = useState(30);
    const [expandedNewsArticles, setExpandedNewsArticles] = useState(false);
    const [expandedMaterialEvents, setExpandedMaterialEvents] = useState(false);
    const [expandedInsiderActivity, setExpandedInsiderActivity] = useState(false);
    const [helpOpen, setHelpOpen] = useState<string | null>(null);

    const sentimentQuery = useQuery({
        queryKey: ['enhanced-sentiment', ticker, days],
        queryFn: () => getEnhancedSentiment(ticker, days),
        staleTime: 1000 * 60 * 60 * 6, // 6 hours
        retry: 1,
    });

    if (sentimentQuery.isLoading) {
        return (
            <Box display="flex" flexDirection="column" justifyContent="center" alignItems="center" minHeight="400px" gap={2}>
                <CircularProgress />
                <Typography variant="body2" color="text.secondary">
                    Analyzing multi-source sentiment for {ticker}...
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
                    <ExperimentalBanner feature="Enhanced Sentiment Analysis" />
                    <Alert severity="warning">
                        <Typography variant="body1" gutterBottom>
                            <strong>Enhanced Sentiment Analysis Not Available</strong>
                        </Typography>
                        <Typography variant="body2">
                            {errorMessage}
                        </Typography>
                    </Alert>
                </CardContent>
            </Card>
        );
    }

    const signal = sentimentQuery.data!;

    return (
        <Box sx={{ width: '100%', p: 2 }}>
            <ExperimentalBanner feature="Enhanced Sentiment Analysis (Multi-Source)" />

            {/* Combined Sentiment Overview */}
            <Paper elevation={3} sx={{ p: 3, mb: 3, bgcolor: 'background.paper' }}>
                <Box display="flex" alignItems="center" gap={1} mb={2}>
                    <Typography variant="h5">
                        Combined Sentiment Analysis
                    </Typography>
                    <IconButton
                        size="small"
                        onClick={() => setHelpOpen('combined_sentiment')}
                        sx={{
                            p: 0.5,
                            color: 'text.secondary',
                            '&:hover': {
                                color: 'primary.main',
                                backgroundColor: 'primary.50',
                            },
                        }}
                    >
                        <HelpOutline sx={{ fontSize: 18 }} />
                    </IconButton>
                </Box>
                <Grid container spacing={2} alignItems="center">
                    <Grid item xs={12} md={6}>
                        <Stack direction="row" spacing={2} alignItems="center">
                            <Box>
                                {getSentimentIcon(signal.combined_sentiment)}
                            </Box>
                            <Box>
                                <Typography variant="h3" color={getSentimentColor(signal.combined_sentiment)}>
                                    {(signal.combined_sentiment * 100).toFixed(1)}
                                </Typography>
                                <Typography variant="caption" color="text.secondary">
                                    Combined Score (-100 to +100)
                                </Typography>
                            </Box>
                        </Stack>
                    </Grid>
                    <Grid item xs={12} md={6}>
                        <Stack spacing={1}>
                            <Box display="flex" alignItems="center" gap={0.5}>
                                <Chip
                                    label={`Confidence: ${formatConfidence(signal.confidence_level)}`}
                                    color={getConfidenceColor(signal.confidence_level)}
                                    icon={<CheckCircle />}
                                />
                                <IconButton
                                    size="small"
                                    onClick={() => setHelpOpen('combined_confidence')}
                                    sx={{
                                        p: 0.5,
                                        color: 'text.secondary',
                                        '&:hover': {
                                            color: 'primary.main',
                                            backgroundColor: 'primary.50',
                                        },
                                    }}
                                >
                                    <HelpOutline sx={{ fontSize: 16 }} />
                                </IconButton>
                            </Box>
                            <Typography variant="caption" color="text.secondary">
                                Based on: News, SEC Filings, and Insider Activity
                            </Typography>
                        </Stack>
                    </Grid>
                </Grid>

                {/* Divergence Flags */}
                {signal.divergence_flags.length > 0 && (
                    <Box sx={{ mt: 2 }}>
                        <Alert severity="warning" icon={<Warning />}>
                            <Typography variant="subtitle2" gutterBottom>
                                <strong>Divergence Detected</strong>
                            </Typography>
                            <Stack spacing={0.5}>
                                {signal.divergence_flags.map((flag: string, idx: number) => (
                                    <Typography key={idx} variant="body2">
                                        {flag}
                                    </Typography>
                                ))}
                            </Stack>
                        </Alert>
                    </Box>
                )}
            </Paper>

            {/* Source Breakdown */}
            <Grid container spacing={2} sx={{ mb: 3 }}>
                {/* News Sentiment */}
                <Grid item xs={12} md={4}>
                    <Card elevation={2}>
                        <CardContent>
                            <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
                                <Stack direction="row" spacing={1} alignItems="center">
                                    <Article color="primary" />
                                    <Typography variant="h6">News Sentiment</Typography>
                                </Stack>
                                <IconButton
                                    size="small"
                                    onClick={() => setHelpOpen('news_sentiment_enhanced')}
                                    sx={{
                                        p: 0.5,
                                        color: 'text.secondary',
                                        '&:hover': {
                                            color: 'primary.main',
                                            backgroundColor: 'primary.50',
                                        },
                                    }}
                                >
                                    <HelpOutline sx={{ fontSize: 16 }} />
                                </IconButton>
                            </Stack>
                            <Typography variant="h4" color={getSentimentColor(signal.news_sentiment)}>
                                {(signal.news_sentiment * 100).toFixed(1)}
                            </Typography>
                            <Typography variant="caption" color="text.secondary">
                                Confidence: {signal.news_confidence}
                            </Typography>
                            <Typography variant="body2" sx={{ mt: 1 }} color="text.secondary">
                                Based on news article analysis
                            </Typography>
                        </CardContent>
                    </Card>
                </Grid>

                {/* SEC Filing Sentiment */}
                <Grid item xs={12} md={4}>
                    <Card elevation={2}>
                        <CardContent>
                            <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
                                <Stack direction="row" spacing={1} alignItems="center">
                                    <Business color="secondary" />
                                    <Typography variant="h6">SEC Filings</Typography>
                                </Stack>
                                <IconButton
                                    size="small"
                                    onClick={() => setHelpOpen('sec_filing_sentiment')}
                                    sx={{
                                        p: 0.5,
                                        color: 'text.secondary',
                                        '&:hover': {
                                            color: 'primary.main',
                                            backgroundColor: 'primary.50',
                                        },
                                    }}
                                >
                                    <HelpOutline sx={{ fontSize: 16 }} />
                                </IconButton>
                            </Stack>
                            <Typography variant="h4" color={signal.sec_filing_score !== undefined ? getSentimentColor(signal.sec_filing_score) : 'text.disabled'}>
                                {signal.sec_filing_score !== undefined
                                    ? (signal.sec_filing_score * 100).toFixed(1)
                                    : 'N/A'}
                            </Typography>
                            <Typography variant="caption" color="text.secondary">
                                {signal.material_events.length} Material Events
                            </Typography>
                            <Typography variant="body2" sx={{ mt: 1 }} color="text.secondary">
                                8-K filings and material events
                            </Typography>
                        </CardContent>
                    </Card>
                </Grid>

                {/* Insider Activity */}
                <Grid item xs={12} md={4}>
                    <Card elevation={2}>
                        <CardContent>
                            <Stack direction="row" spacing={1} alignItems="center" justifyContent="space-between" sx={{ mb: 2 }}>
                                <Stack direction="row" spacing={1} alignItems="center">
                                    <PersonOutline color="action" />
                                    <Typography variant="h6">Insider Activity</Typography>
                                </Stack>
                                <IconButton
                                    size="small"
                                    onClick={() => setHelpOpen('insider_sentiment')}
                                    sx={{
                                        p: 0.5,
                                        color: 'text.secondary',
                                        '&:hover': {
                                            color: 'primary.main',
                                            backgroundColor: 'primary.50',
                                        },
                                    }}
                                >
                                    <HelpOutline sx={{ fontSize: 16 }} />
                                </IconButton>
                            </Stack>
                            <Typography variant="h4" color={getSentimentColor(signal.insider_sentiment.sentiment_score)}>
                                {(signal.insider_sentiment.sentiment_score * 100).toFixed(1)}
                            </Typography>
                            <Typography variant="caption" color="text.secondary">
                                {signal.insider_sentiment.total_transactions} Transactions
                            </Typography>
                            <Stack direction="row" spacing={1} sx={{ mt: 1 }}>
                                <Chip
                                    size="small"
                                    label={`Buy: ${signal.insider_sentiment.buying_transactions}`}
                                    color="success"
                                    variant="outlined"
                                />
                                <Chip
                                    size="small"
                                    label={`Sell: ${signal.insider_sentiment.selling_transactions}`}
                                    color="error"
                                    variant="outlined"
                                />
                            </Stack>
                        </CardContent>
                    </Card>
                </Grid>
            </Grid>

            {/* News Articles Section */}
            {signal.news_articles && signal.news_articles.length > 0 && (
                <Card elevation={2} sx={{ mb: 2 }}>
                    <CardContent>
                        <Stack
                            direction="row"
                            justifyContent="space-between"
                            alignItems="center"
                            sx={{ cursor: 'pointer' }}
                            onClick={() => setExpandedNewsArticles(!expandedNewsArticles)}
                        >
                            <Typography variant="h6">
                                News Articles ({signal.news_articles.length})
                            </Typography>
                            <IconButton size="small">
                                <ExpandMore
                                    sx={{
                                        transform: expandedNewsArticles ? 'rotate(180deg)' : 'rotate(0deg)',
                                        transition: 'transform 0.3s',
                                    }}
                                />
                            </IconButton>
                        </Stack>
                        <Collapse in={expandedNewsArticles}>
                            <Divider sx={{ my: 2 }} />
                            <List>
                                {signal.news_articles.map((article: any, idx: number) => (
                                    <ListItem
                                        key={idx}
                                        sx={{
                                            bgcolor: 'background.default',
                                            mb: 1,
                                            borderRadius: 1,
                                        }}
                                    >
                                        <ListItemText
                                            primaryTypographyProps={{ component: 'div' }}
                                            secondaryTypographyProps={{ component: 'div' }}
                                            primary={
                                                <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap">
                                                    <Article color="primary" fontSize="small" />
                                                    <Typography variant="subtitle2" sx={{ flex: 1 }}>
                                                        {article.title}
                                                    </Typography>
                                                    <Typography variant="caption" color="text.secondary">
                                                        {new Date(article.published_at).toLocaleDateString()}
                                                    </Typography>
                                                </Stack>
                                            }
                                            secondary={
                                                <>
                                                    <Typography variant="body2" sx={{ mt: 1 }}>
                                                        {article.snippet}
                                                    </Typography>
                                                    <Stack direction="row" spacing={1} alignItems="center" sx={{ mt: 1 }}>
                                                        <Chip
                                                            label={article.source}
                                                            size="small"
                                                            variant="outlined"
                                                        />
                                                        <IconButton
                                                            size="small"
                                                            href={article.url}
                                                            target="_blank"
                                                            rel="noopener noreferrer"
                                                        >
                                                            <OpenInNew fontSize="small" />
                                                        </IconButton>
                                                    </Stack>
                                                </>
                                            }
                                        />
                                    </ListItem>
                                ))}
                            </List>
                        </Collapse>
                    </CardContent>
                </Card>
            )}

            {/* Material Events Section */}
            {signal.material_events.length > 0 && (
                <Card elevation={2} sx={{ mb: 2 }}>
                    <CardContent>
                        <Stack
                            direction="row"
                            justifyContent="space-between"
                            alignItems="center"
                            sx={{ cursor: 'pointer' }}
                            onClick={() => setExpandedMaterialEvents(!expandedMaterialEvents)}
                        >
                            <Typography variant="h6">
                                Material Events ({signal.material_events.length})
                            </Typography>
                            <IconButton size="small">
                                <ExpandMore
                                    sx={{
                                        transform: expandedMaterialEvents ? 'rotate(180deg)' : 'rotate(0deg)',
                                        transition: 'transform 0.3s',
                                    }}
                                />
                            </IconButton>
                        </Stack>
                        <Collapse in={expandedMaterialEvents}>
                            <Divider sx={{ my: 2 }} />
                            <List>
                                {signal.material_events.map((event: any, idx: number) => (
                                    <ListItem
                                        key={idx}
                                        sx={{
                                            bgcolor: 'background.default',
                                            mb: 1,
                                            borderRadius: 1,
                                        }}
                                    >
                                        <ListItemText
                                            primaryTypographyProps={{ component: 'div' }}
                                            secondaryTypographyProps={{ component: 'div' }}
                                            primary={
                                                <Stack direction="row" spacing={1} alignItems="center">
                                                    <Chip
                                                        label={event.importance.toUpperCase()}
                                                        size="small"
                                                        color={getImportanceColor(event.importance)}
                                                    />
                                                    <Typography variant="subtitle2">
                                                        {event.event_type}
                                                    </Typography>
                                                    <Typography variant="caption" color="text.secondary">
                                                        {new Date(event.event_date).toLocaleDateString()}
                                                    </Typography>
                                                </Stack>
                                            }
                                            secondary={
                                                <>
                                                    <Typography variant="body2" sx={{ mt: 1 }}>
                                                        {event.summary}
                                                    </Typography>
                                                    <Stack direction="row" spacing={1} alignItems="center" sx={{ mt: 1 }}>
                                                        <Chip
                                                            label={`Sentiment: ${(event.sentiment_score * 100).toFixed(0)}`}
                                                            size="small"
                                                            color={getSentimentChipColor(event.sentiment_score)}
                                                            variant="outlined"
                                                        />
                                                        <IconButton
                                                            size="small"
                                                            href={event.filing_url}
                                                            target="_blank"
                                                            rel="noopener noreferrer"
                                                        >
                                                            <OpenInNew fontSize="small" />
                                                        </IconButton>
                                                    </Stack>
                                                </>
                                            }
                                        />
                                    </ListItem>
                                ))}
                            </List>
                        </Collapse>
                    </CardContent>
                </Card>
            )}

            {/* Notable Insider Transactions */}
            {signal.insider_sentiment.notable_transactions.length > 0 && (
                <Card elevation={2}>
                    <CardContent>
                        <Stack
                            direction="row"
                            justifyContent="space-between"
                            alignItems="center"
                            sx={{ cursor: 'pointer' }}
                            onClick={() => setExpandedInsiderActivity(!expandedInsiderActivity)}
                        >
                            <Typography variant="h6">
                                Notable Insider Transactions ({signal.insider_sentiment.notable_transactions.length})
                            </Typography>
                            <IconButton size="small">
                                <ExpandMore
                                    sx={{
                                        transform: expandedInsiderActivity ? 'rotate(180deg)' : 'rotate(0deg)',
                                        transition: 'transform 0.3s',
                                    }}
                                />
                            </IconButton>
                        </Stack>
                        <Collapse in={expandedInsiderActivity}>
                            <Divider sx={{ my: 2 }} />
                            <List>
                                {signal.insider_sentiment.notable_transactions.map((txn: any, idx: number) => (
                                    <ListItem
                                        key={idx}
                                        sx={{
                                            bgcolor: 'background.default',
                                            mb: 1,
                                            borderRadius: 1,
                                        }}
                                    >
                                        <ListItemText
                                            primaryTypographyProps={{ component: 'div' }}
                                            secondaryTypographyProps={{ component: 'div' }}
                                            primary={
                                                <Stack direction="row" spacing={1} alignItems="center">
                                                    <Chip
                                                        label={txn.transaction_type.toUpperCase()}
                                                        size="small"
                                                        color={txn.transaction_type === 'purchase' ? 'success' : 'error'}
                                                    />
                                                    <Typography variant="subtitle2">
                                                        {txn.reporting_person}
                                                    </Typography>
                                                    {txn.title && (
                                                        <Typography variant="caption" color="text.secondary">
                                                            ({txn.title})
                                                        </Typography>
                                                    )}
                                                </Stack>
                                            }
                                            secondary={
                                                <>
                                                    <Typography variant="body2" sx={{ mt: 1 }}>
                                                        <strong>{txn.shares.toLocaleString()}</strong> shares on{' '}
                                                        {new Date(txn.transaction_date).toLocaleDateString()}
                                                    </Typography>
                                                    {txn.price_per_share && (
                                                        <Typography variant="caption" color="text.secondary">
                                                            Price: ${txn.price_per_share}
                                                        </Typography>
                                                    )}
                                                </>
                                            }
                                        />
                                    </ListItem>
                                ))}
                            </List>
                        </Collapse>
                    </CardContent>
                </Card>
            )}

            {/* Footer */}
            <Box sx={{ mt: 2, textAlign: 'center' }}>
                <Typography variant="caption" color="text.secondary">
                    Last updated: {new Date(signal.calculated_at).toLocaleString()}
                </Typography>
            </Box>

            {/* Help Dialog */}
            {helpOpen && (
                <MetricHelpDialog
                    open={true}
                    onClose={() => setHelpOpen(null)}
                    metricKey={helpOpen}
                />
            )}
        </Box>
    );
}

// Helper functions
function getSentimentIcon(score: number) {
    if (score > 0.2) return <TrendingUp fontSize="large" color="success" />;
    if (score < -0.2) return <TrendingDown fontSize="large" color="error" />;
    return <Info fontSize="large" color="action" />;
}

function getSentimentColor(score: number): string {
    if (score > 0.2) return 'success.main';
    if (score < -0.2) return 'error.main';
    return 'text.secondary';
}

function getSentimentChipColor(score: number): 'success' | 'error' | 'default' {
    if (score > 0.2) return 'success';
    if (score < -0.2) return 'error';
    return 'default';
}

function formatConfidence(level: ConfidenceLevel): string {
    return level.replace('_', ' ').toUpperCase();
}

function getConfidenceColor(level: ConfidenceLevel): 'success' | 'info' | 'warning' | 'default' {
    switch (level) {
        case 'very_high':
            return 'success';
        case 'high':
            return 'info';
        case 'medium':
            return 'warning';
        default:
            return 'default';
    }
}

function getImportanceColor(importance: EventImportance): 'error' | 'warning' | 'info' | 'default' {
    switch (importance) {
        case 'critical':
            return 'error';
        case 'high':
            return 'warning';
        case 'medium':
            return 'info';
        default:
            return 'default';
    }
}
