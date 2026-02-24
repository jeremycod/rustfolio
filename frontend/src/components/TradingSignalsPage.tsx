import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  TextField,
  Button,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Grid,
  Card,
  CardContent,
  Alert,
  Chip,
  CircularProgress,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Divider,
  IconButton,
} from '@mui/material';
import {
  TrendingUp,
  TrendingDown,
  TrendingFlat,
  Search,
  ExpandMore,
  Info as InfoIcon,
  HelpOutline,
} from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getTradingSignals, searchTickers } from '../lib/endpoints';
import type { SignalDirection, SignalFactor } from '../types';
import { MetricHelpDialog } from './MetricHelpDialog';

export function TradingSignalsPage() {
  const [ticker, setTicker] = useState('');
  const [searchTicker, setSearchTicker] = useState('');
  const [horizon, setHorizon] = useState(3);
  const [signalTypeFilter, setSignalTypeFilter] = useState<string>('all');
  const [helpOpen, setHelpOpen] = useState(false);
  const [selectedMetric, setSelectedMetric] = useState<string>('');

  // Fetch company name
  const companyInfoQ = useQuery({
    queryKey: ['companyInfo', searchTicker],
    queryFn: () => searchTickers(searchTicker),
    enabled: !!searchTicker,
    staleTime: 1000 * 60 * 60,
  });

  const companyName = companyInfoQ.data?.[0]?.name || null;

  // Fetch trading signals
  const signalsQ = useQuery({
    queryKey: ['tradingSignals', searchTicker, horizon, signalTypeFilter],
    queryFn: () => {
      const types = signalTypeFilter === 'all' ? undefined : [signalTypeFilter];
      return getTradingSignals(searchTicker, horizon, types);
    },
    enabled: !!searchTicker,
  });

  const handleSearch = () => {
    if (ticker.trim()) {
      setSearchTicker(ticker.trim().toUpperCase());
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  const getDirectionIcon = (direction: SignalDirection) => {
    switch (direction) {
      case 'Bullish':
        return <TrendingUp sx={{ color: 'success.main' }} />;
      case 'Bearish':
        return <TrendingDown sx={{ color: 'error.main' }} />;
      case 'Neutral':
        return <TrendingFlat sx={{ color: 'text.secondary' }} />;
    }
  };

  const getDirectionColor = (direction: SignalDirection): "success" | "error" | "default" => {
    switch (direction) {
      case 'Bullish':
        return 'success';
      case 'Bearish':
        return 'error';
      case 'Neutral':
        return 'default';
    }
  };

  const getConfidenceColor = (confidence: string): "success" | "warning" | "error" => {
    if (!confidence) return 'warning';
    switch (confidence.toLowerCase()) {
      case 'high':
        return 'success';
      case 'medium':
        return 'warning';
      case 'low':
        return 'error';
      default:
        return 'warning';
    }
  };

  const formatPercent = (value: number): string => {
    return `${(value * 100).toFixed(0)}%`;
  };

  // Map indicator names to help keys
  const getIndicatorHelpKey = (indicatorName: string): string | null => {
    const normalized = indicatorName.toLowerCase().replace(/\s+/g, '_');

    // Map common indicator names to help keys
    const mappings: Record<string, string> = {
      // Basic indicators
      'rsi_14': 'rsi_14',
      'rsi_meanreversion': 'rsi_meanreversion',
      'rsi': 'rsi_14',
      'macd': 'macd',
      'momentum_20d': 'momentum_20d',
      'momentum': 'momentum_20d',
      'bollinger_bands': 'bollinger_band_percent_b',
      'bollinger': 'bollinger_band_percent_b',
      'price_deviation_sma50': 'price_deviation_sma50',
      'price_deviation': 'price_deviation_sma50',
      'moving_average_cross': 'moving_average_cross',
      'ma_cross': 'moving_average_cross',
      'sma_50_200_cross': 'moving_average_cross',
      'volume_trend': 'volume_trend',
      'volume': 'volume_trend',
      'ema_alignment': 'ema_alignment',
      'ema': 'ema_alignment',
      // Meta-signals (used in Combined Signal)
      'momentum_signal': 'momentum_signal',
      'meanreversion_signal': 'meanreversion_signal',
      'trend_signal': 'trend_signal',
    };

    return mappings[normalized] || null;
  };

  const renderOverallRecommendation = () => {
    if (!signalsQ.data?.overall_recommendation) return null;

    const rec = signalsQ.data.overall_recommendation;
    const actionColor = rec.action === 'Buy' ? 'success' : rec.action === 'Sell' ? 'error' : 'info';

    return (
      <Paper sx={{ p: 3, mb: 3, bgcolor: `${actionColor}.50` }}>
        <Box display="flex" alignItems="center" gap={1} mb={2}>
          <Typography variant="h6">
            Overall Recommendation
          </Typography>
          <IconButton
            size="small"
            onClick={(e) => {
              e.stopPropagation();
              setSelectedMetric('trading_signal_overall');
              setHelpOpen(true);
            }}
            sx={{
              p: 0.5,
              color: 'text.secondary',
              '&:hover': {
                color: 'primary.main',
                backgroundColor: 'primary.50',
              },
            }}
          >
            <HelpOutline fontSize="small" />
          </IconButton>
        </Box>
        <Grid container spacing={2} alignItems="center">
          <Grid item xs={12} md={4}>
            <Box display="flex" alignItems="center" gap={2}>
              {rec.action === 'Buy' && <TrendingUp sx={{ fontSize: 40, color: 'success.main' }} />}
              {rec.action === 'Sell' && <TrendingDown sx={{ fontSize: 40, color: 'error.main' }} />}
              {rec.action === 'Hold' && <TrendingFlat sx={{ fontSize: 40, color: 'info.main' }} />}
              <Box>
                <Chip
                  label={`${rec.action.toUpperCase()} (${rec.strength})`}
                  color={actionColor as any}
                  size="large"
                  sx={{ fontSize: '1.1rem', fontWeight: 'bold', px: 2, py: 3 }}
                />
              </Box>
            </Box>
          </Grid>
          <Grid item xs={12} md={4}>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Typography variant="body2" color="text.secondary">
                Probability
              </Typography>
              <IconButton
                size="small"
                onClick={(e) => {
                  e.stopPropagation();
                  setSelectedMetric('trading_signal_probability');
                  setHelpOpen(true);
                }}
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
            <Typography variant="h5">{formatPercent(rec.probability)}</Typography>
          </Grid>
          <Grid item xs={12} md={4}>
            <Typography variant="body2" color="text.secondary">
              Rationale
            </Typography>
            <Typography variant="body1">{rec.rationale}</Typography>
          </Grid>
        </Grid>
      </Paper>
    );
  };

  const renderSignals = () => {
    if (!signalsQ.data?.signals) return null;

    return (
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Signal Breakdown
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={2}>
          Individual signals with probability scores and explanations
        </Typography>

        {signalsQ.data.signals.map((signal, idx) => {
          // Backend sends lowercase with underscores: 'momentum', 'mean_reversion', 'trend', 'combined'
          const signalMetricKey =
            signal.signal_type === 'combined' ? 'trading_signal_combined' :
            signal.signal_type === 'momentum' ? 'trading_signal_momentum' :
            signal.signal_type === 'mean_reversion' ? 'trading_signal_mean_reversion' :
            'trading_signal_trend';

          return (
          <Accordion key={idx} sx={{ mb: 1 }}>
            <AccordionSummary expandIcon={<ExpandMore />}>
              <Box display="flex" alignItems="center" gap={2} width="100%">
                {getDirectionIcon(signal.direction)}
                <Box flexGrow={1} display="flex" alignItems="center" gap={0.5}>
                  <Typography variant="subtitle1" fontWeight="bold">
                    {signal.signal_type === 'combined' ? '📊 ' :
                     signal.signal_type === 'momentum' ? '🔥 ' :
                     signal.signal_type === 'mean_reversion' ? '🔄 ' : '📈 '}
                    {signal.signal_type.replace('_', ' ').replace(/\b\w/g, (l) => l.toUpperCase())} Signal
                  </Typography>
                  <IconButton
                    size="small"
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedMetric(signalMetricKey);
                      setHelpOpen(true);
                    }}
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
                <Chip
                  label={signal.direction}
                  color={getDirectionColor(signal.direction)}
                  size="small"
                />
                <Box display="flex" alignItems="center" gap={0.5}>
                  <Chip
                    label={`${formatPercent(signal.probability)} Probability`}
                    color={getConfidenceColor(signal.confidence)}
                    size="small"
                  />
                  <IconButton
                    size="small"
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedMetric('trading_signal_probability');
                      setHelpOpen(true);
                    }}
                    sx={{
                      p: 0.25,
                      color: 'text.secondary',
                      '&:hover': {
                        color: 'primary.main',
                        backgroundColor: 'primary.50',
                      },
                    }}
                  >
                    <HelpOutline sx={{ fontSize: 14 }} />
                  </IconButton>
                </Box>
                <Box display="flex" alignItems="center" gap={0.5}>
                  <Chip
                    label={signal.confidence}
                    color={getConfidenceColor(signal.confidence)}
                    size="small"
                  />
                  <IconButton
                    size="small"
                    onClick={(e) => {
                      e.stopPropagation();
                      setSelectedMetric('trading_signal_confidence');
                      setHelpOpen(true);
                    }}
                    sx={{
                      p: 0.25,
                      color: 'text.secondary',
                      '&:hover': {
                        color: 'primary.main',
                        backgroundColor: 'primary.50',
                      },
                    }}
                  >
                    <HelpOutline sx={{ fontSize: 14 }} />
                  </IconButton>
                </Box>
              </Box>
            </AccordionSummary>
            <AccordionDetails>
              <Typography variant="body2" paragraph>
                <strong>Explanation:</strong> {signal.explanation}
              </Typography>

              {signal.contributing_factors && (
                <>
                  <Divider sx={{ my: 1 }} />
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    <strong>Contributing Factors:</strong>
                  </Typography>

                  {/* Display scalar metrics */}
                  <Grid container spacing={1} sx={{ mb: 2 }}>
                    {signal.contributing_factors.bullish_score !== undefined && (
                      <Grid item xs={12} sm={6} md={4}>
                        <Box sx={{ p: 1, bgcolor: 'success.50', borderRadius: 1 }}>
                          <Box display="flex" alignItems="center" gap={0.5}>
                            <Typography variant="caption" color="text.secondary">
                              BULLISH SCORE
                            </Typography>
                            <IconButton
                              size="small"
                              onClick={(e) => {
                                e.stopPropagation();
                                setSelectedMetric('trading_signal_bullish_score');
                                setHelpOpen(true);
                              }}
                              sx={{
                                p: 0.25,
                                color: 'text.secondary',
                                '&:hover': {
                                  color: 'primary.main',
                                  backgroundColor: 'primary.50',
                                },
                              }}
                            >
                              <HelpOutline sx={{ fontSize: 14 }} />
                            </IconButton>
                          </Box>
                          <Typography variant="body2" fontWeight="bold">
                            {signal.contributing_factors.bullish_score.toFixed(2)}
                          </Typography>
                        </Box>
                      </Grid>
                    )}
                    {signal.contributing_factors.bearish_score !== undefined && (
                      <Grid item xs={12} sm={6} md={4}>
                        <Box sx={{ p: 1, bgcolor: 'error.50', borderRadius: 1 }}>
                          <Box display="flex" alignItems="center" gap={0.5}>
                            <Typography variant="caption" color="text.secondary">
                              BEARISH SCORE
                            </Typography>
                            <IconButton
                              size="small"
                              onClick={(e) => {
                                e.stopPropagation();
                                setSelectedMetric('trading_signal_bearish_score');
                                setHelpOpen(true);
                              }}
                              sx={{
                                p: 0.25,
                                color: 'text.secondary',
                                '&:hover': {
                                  color: 'primary.main',
                                  backgroundColor: 'primary.50',
                                },
                              }}
                            >
                              <HelpOutline sx={{ fontSize: 14 }} />
                            </IconButton>
                          </Box>
                          <Typography variant="body2" fontWeight="bold">
                            {signal.contributing_factors.bearish_score.toFixed(2)}
                          </Typography>
                        </Box>
                      </Grid>
                    )}
                    {signal.contributing_factors.total_factors !== undefined && (
                      <Grid item xs={12} sm={6} md={4}>
                        <Box sx={{ p: 1, bgcolor: 'grey.100', borderRadius: 1 }}>
                          <Typography variant="caption" color="text.secondary">
                            TOTAL FACTORS
                          </Typography>
                          <Typography variant="body2" fontWeight="bold">
                            {signal.contributing_factors.total_factors}
                          </Typography>
                        </Box>
                      </Grid>
                    )}
                  </Grid>

                  {/* Display individual factors */}
                  {signal.contributing_factors.factors && Array.isArray(signal.contributing_factors.factors) && (
                    <Box sx={{ mt: 1 }}>
                      {signal.contributing_factors.factors.map((factor: SignalFactor, idx: number) => {
                        const helpKey = getIndicatorHelpKey(factor.indicator);
                        return (
                        <Box key={idx} sx={{ mb: 1, p: 1, bgcolor: 'grey.50', borderRadius: 1 }}>
                          <Box display="flex" justifyContent="space-between" alignItems="center">
                            <Box display="flex" alignItems="center" gap={0.5}>
                              <Typography variant="body2" fontWeight="bold">
                                {factor.indicator.replace(/_/g, ' ')}
                              </Typography>
                              {helpKey && (
                                <IconButton
                                  size="small"
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    setSelectedMetric(helpKey);
                                    setHelpOpen(true);
                                  }}
                                  sx={{
                                    p: 0.25,
                                    color: 'text.secondary',
                                    '&:hover': {
                                      color: 'primary.main',
                                      backgroundColor: 'primary.50',
                                    },
                                  }}
                                >
                                  <HelpOutline sx={{ fontSize: 14 }} />
                                </IconButton>
                              )}
                            </Box>
                            <Chip
                              label={factor.direction}
                              size="small"
                              color={
                                factor.direction === 'bullish' ? 'success' :
                                factor.direction === 'bearish' ? 'error' : 'default'
                              }
                            />
                          </Box>
                          <Typography variant="caption" color="text.secondary">
                            {factor.interpretation}
                          </Typography>
                        </Box>
                        );
                      })}
                    </Box>
                  )}
                </>
              )}
            </AccordionDetails>
          </Accordion>
        );
        })}
      </Paper>
    );
  };

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <TrendingUp sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Trading Signals
        </Typography>
      </Box>

      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Signal Parameters
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Generate probability-based trading signals using technical indicators (RSI, MACD, Bollinger Bands) and
          statistical models. Signals are classified by type and confidence level.
        </Typography>

        <Grid container spacing={2} alignItems="center">
          <Grid item xs={12} sm={3}>
            <TextField
              fullWidth
              label="Ticker Symbol"
              placeholder="e.g., AAPL"
              value={ticker}
              onChange={(e) => setTicker(e.target.value.toUpperCase())}
              onKeyPress={handleKeyPress}
            />
          </Grid>

          <Grid item xs={12} sm={3}>
            <FormControl fullWidth>
              <InputLabel>Horizon</InputLabel>
              <Select value={horizon} label="Horizon" onChange={(e) => setHorizon(Number(e.target.value))}>
                <MenuItem value={1}>1 Month</MenuItem>
                <MenuItem value={3}>3 Months</MenuItem>
                <MenuItem value={6}>6 Months</MenuItem>
                <MenuItem value={12}>12 Months</MenuItem>
              </Select>
            </FormControl>
          </Grid>

          <Grid item xs={12} sm={3}>
            <FormControl fullWidth>
              <InputLabel>Signal Type</InputLabel>
              <Select
                value={signalTypeFilter}
                label="Signal Type"
                onChange={(e) => setSignalTypeFilter(e.target.value)}
              >
                <MenuItem value="all">All Signals</MenuItem>
                <MenuItem value="Combined">Combined</MenuItem>
                <MenuItem value="Momentum">Momentum</MenuItem>
                <MenuItem value="MeanReversion">Mean Reversion</MenuItem>
                <MenuItem value="Trend">Trend</MenuItem>
              </Select>
            </FormControl>
          </Grid>

          <Grid item xs={12} sm={3}>
            <Button
              fullWidth
              variant="contained"
              size="large"
              startIcon={<Search />}
              onClick={handleSearch}
              disabled={!ticker.trim()}
            >
              Generate
            </Button>
          </Grid>
        </Grid>
      </Paper>

      {!searchTicker && (
        <Alert severity="info">
          Enter a ticker symbol to generate trading signals. Signals combine multiple factors:
          <ul style={{ marginTop: '8px', marginBottom: 0 }}>
            <li><strong>Momentum:</strong> RSI, MACD trends, volume</li>
            <li><strong>Mean Reversion:</strong> Bollinger Bands, price deviations</li>
            <li><strong>Trend:</strong> Moving averages, trend strength</li>
            <li><strong>Combined:</strong> Weighted ensemble of all signals</li>
          </ul>
        </Alert>
      )}

      {searchTicker && (
        <Box>
          <Box sx={{ mb: 3 }}>
            <Typography variant="h5" fontWeight="bold">
              {searchTicker} Trading Signals ({horizon}M Horizon)
            </Typography>
            {companyName && (
              <Typography variant="body1" color="text.secondary" sx={{ mt: 0.5 }}>
                {companyName}
              </Typography>
            )}
          </Box>

          {signalsQ.isLoading && (
            <Box display="flex" justifyContent="center" alignItems="center" minHeight={400}>
              <CircularProgress />
            </Box>
          )}

          {signalsQ.error && (
            <Alert severity="error">
              Failed to generate signals: {(signalsQ.error as Error).message}
            </Alert>
          )}

          {signalsQ.data && (
            <>
              {renderOverallRecommendation()}
              {renderSignals()}

              <Alert severity="info" icon={<InfoIcon />}>
                <Typography variant="body2">
                  <strong>Disclaimer:</strong> Trading signals are for informational purposes only and should not be
                  considered financial advice. Past performance does not guarantee future results. Always conduct your
                  own research and consider your risk tolerance before making investment decisions.
                </Typography>
              </Alert>
            </>
          )}
        </Box>
      )}

      {/* Help Dialog */}
      <MetricHelpDialog
        open={helpOpen}
        onClose={() => setHelpOpen(false)}
        metricKey={selectedMetric}
      />
    </Box>
  );
}
