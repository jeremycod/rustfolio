import { useState, useMemo } from 'react';
import {
  Box,
  Typography,
  Paper,
  CircularProgress,
  Alert,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Tooltip,
  Grid,
  Button,
  Snackbar,
  Chip,
} from '@mui/material';
import { Refresh, Cached, Schedule } from '@mui/icons-material';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { getPortfolioCorrelations, listPortfolios } from '../lib/endpoints';
import { CorrelationMatrix, CorrelationMatrixWithStats } from '../types';
import CorrelationStatsCard from './CorrelationStatsCard';
import { TickerActionMenu } from './TickerActionMenu';

type CorrelationHeatmapProps = {
  portfolioId: string;
  onTickerNavigate?: (ticker: string, page?: string) => void;
};

export function CorrelationHeatmap({ portfolioId: initialPortfolioId, onTickerNavigate }: CorrelationHeatmapProps) {
  const [days, setDays] = useState(90);
  const [selectedPortfolioId, setSelectedPortfolioId] = useState(initialPortfolioId);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [snackbarOpen, setSnackbarOpen] = useState(false);
  const [snackbarMessage, setSnackbarMessage] = useState('');
  const [snackbarSeverity, setSnackbarSeverity] = useState<'success' | 'error'>('success');
  const [loadingStep, setLoadingStep] = useState<string>('');

  const queryClient = useQueryClient();

  // Fetch portfolios for dropdown
  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const correlationQ = useQuery({
    queryKey: ['portfolio-correlations', selectedPortfolioId, days],
    queryFn: async () => {
      setLoadingStep('Fetching portfolio holdings...');
      const result = await getPortfolioCorrelations(selectedPortfolioId, days);
      setLoadingStep('');
      return result;
    },
    staleTime: 1000 * 60 * 60, // 1 hour
    gcTime: 1000 * 60 * 60, // Keep in cache for 1 hour
    retry: (failureCount, error: any) => {
      if (error?.response?.status === 503) {
        return failureCount < 3; // Retry 503 up to 3 times
      }
      return failureCount < 1; // Only retry once for other errors
    },
    retryDelay: (attemptIndex, error: any) => {
      if (error?.response?.status === 503) {
        // For 503, wait progressively longer: 15s, 30s, 45s
        return 15000 * (attemptIndex + 1);
      }
      return 1000; // Normal retry delay
    },
    enabled: !!selectedPortfolioId,
  });

  // Calculate next scheduled calculation time (every 2 hours at :45)
  const getNextCalculationTime = useMemo(() => {
    const now = new Date();
    const currentHour = now.getHours();
    const currentMinute = now.getMinutes();

    // Find the next hour that's a multiple of 2
    let nextHour = currentHour;
    if (currentMinute >= 45) {
      nextHour = currentHour + 2;
    } else if (currentHour % 2 === 1) {
      nextHour = currentHour + 1;
    } else {
      nextHour = currentHour;
    }
    nextHour = Math.ceil(nextHour / 2) * 2;

    const nextTime = new Date(now);
    nextTime.setHours(nextHour, 45, 0, 0);
    if (nextTime <= now) {
      nextTime.setHours(nextTime.getHours() + 2);
    }

    return nextTime;
  }, []);

  // Calculate cache freshness message
  const cacheStatusMessage = useMemo(() => {
    if (!correlationQ.data || !correlationQ.dataUpdatedAt) return null;

    const updatedAt = new Date(correlationQ.dataUpdatedAt);
    const now = new Date();
    const diffMs = now.getTime() - updatedAt.getTime();
    const diffMinutes = Math.floor(diffMs / (1000 * 60));

    if (diffMinutes < 1) return 'Just updated';
    if (diffMinutes < 60) return `Updated ${diffMinutes} minute${diffMinutes > 1 ? 's' : ''} ago`;

    const diffHours = Math.floor(diffMinutes / 60);
    return `Updated ${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
  }, [correlationQ.data, correlationQ.dataUpdatedAt]);

  const handleRefresh = async () => {
    if (!selectedPortfolioId) return;

    setIsRefreshing(true);
    setLoadingStep('Fetching fresh portfolio data...');

    try {
      // Force fetch new correlation data
      const freshData = await getPortfolioCorrelations(selectedPortfolioId, days, true);

      // Update the cache with fresh data
      queryClient.setQueryData(['portfolio-correlations', selectedPortfolioId, days], freshData);

      setSnackbarMessage('Correlation data refreshed successfully!');
      setSnackbarSeverity('success');
      setSnackbarOpen(true);
    } catch (error: any) {
      const status = error?.response?.status;

      if (status === 503) {
        const nextCalc = getNextCalculationTime.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
        setSnackbarMessage(
          `Correlation data is being calculated in the background. Please wait 30-60 seconds and try again. Next scheduled calculation: ${nextCalc}`
        );
      } else {
        const errorMessage = error?.response?.data?.error || error.message || 'Unknown error';
        setSnackbarMessage(`Failed to refresh: ${errorMessage}`);
      }

      setSnackbarSeverity('error');
      setSnackbarOpen(true);
    } finally {
      setIsRefreshing(false);
      setLoadingStep('');
    }
  };

  const getCorrelation = (matrix: CorrelationMatrix, ticker1: string, ticker2: string): number | null => {
    if (ticker1 === ticker2) return 1.0; // Diagonal is always 1.0

    // Check both directions since we store upper triangle only
    const pair = matrix.correlations.find(
      (c) =>
        (c.ticker1 === ticker1 && c.ticker2 === ticker2) ||
        (c.ticker1 === ticker2 && c.ticker2 === ticker1)
    );

    return pair ? pair.correlation : null;
  };

  const getCorrelationColor = (correlation: number | null): string => {
    if (correlation === null) return '#e0e0e0'; // Gray for missing data

    // Color scale: red (-1) → white (0) → green (+1)
    if (correlation >= 0) {
      // Positive correlation: white to green
      const intensity = Math.floor(correlation * 200);
      return `rgb(${200 - intensity}, ${220 + intensity * 0.15}, ${200 - intensity})`;
    } else {
      // Negative correlation: white to red
      const intensity = Math.floor(-correlation * 200);
      return `rgb(${220 + intensity * 0.15}, ${200 - intensity}, ${200 - intensity})`;
    }
  };

  const getCorrelationLabel = (correlation: number): string => {
    const abs = Math.abs(correlation);
    if (abs >= 0.8) return 'Strong';
    if (abs >= 0.5) return 'Moderate';
    if (abs >= 0.3) return 'Weak';
    return 'Very Weak';
  };

  if (correlationQ.isLoading || isRefreshing) {
    const steps = [
      'Fetching portfolio holdings...',
      'Retrieving price history...',
      'Computing correlations...',
      'Analyzing patterns...',
    ];

    const currentStep = loadingStep || steps[0];

    return (
      <Box>
        <Typography variant="h5" gutterBottom mb={3}>
          Portfolio Correlation Heatmap
        </Typography>
        <Box display="flex" flexDirection="column" justifyContent="center" alignItems="center" minHeight="300px" gap={2}>
          <CircularProgress size={60} />
          <Typography variant="h6" color="text.primary">
            {currentStep}
          </Typography>
          <Typography variant="body2" color="text.secondary">
            This may take 30-60 seconds for portfolios with many positions.
          </Typography>
          <Typography variant="caption" color="text.secondary" sx={{ mt: 2 }}>
            Estimated time remaining: ~30-45 seconds
          </Typography>
        </Box>
      </Box>
    );
  }

  if (correlationQ.error) {
    const error = correlationQ.error as any;
    const status = error?.response?.status;
    const errorMessage = error?.response?.data?.error || error.message || 'Unknown error';

    let alertSeverity: 'error' | 'warning' = 'error';
    let displayMessage = '';

    if (status === 503) {
      alertSeverity = 'warning';
      const nextCalc = getNextCalculationTime.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
      displayMessage = `Correlation data is being calculated in the background. Please wait 30-60 seconds and try again. Next scheduled calculation: ${nextCalc}`;
    } else {
      displayMessage = `Failed to load correlation data: ${errorMessage}`;
    }

    return (
      <Box>
        <Typography variant="h5" gutterBottom mb={3}>
          Portfolio Correlation Heatmap
        </Typography>
        <Grid container spacing={2} mb={3}>
          <Grid item xs={12} md={6}>
            <FormControl fullWidth>
              <InputLabel>Portfolio</InputLabel>
              <Select
                value={selectedPortfolioId}
                label="Portfolio"
                onChange={(e) => setSelectedPortfolioId(e.target.value)}
              >
                {portfoliosQ.data?.map((portfolio) => (
                  <MenuItem key={portfolio.id} value={portfolio.id}>
                    {portfolio.name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Grid>
        </Grid>
        <Alert severity={alertSeverity} sx={{ mb: 2 }}>
          {displayMessage}
        </Alert>
        {status === 503 && (
          <Box display="flex" justifyContent="center" mt={2}>
            <Button
              variant="contained"
              startIcon={<Refresh />}
              onClick={() => correlationQ.refetch()}
            >
              Try Again
            </Button>
          </Box>
        )}
      </Box>
    );
  }

  if (!correlationQ.data) {
    return (
      <Alert severity="info">
        No correlation data available. Need at least 2 positions with sufficient price history.
      </Alert>
    );
  }

  const matrix = correlationQ.data;
  const tickers = matrix.tickers;

  return (
    <Box>
      <Typography variant="h5" gutterBottom mb={3}>
        Portfolio Correlation Heatmap
      </Typography>

      {/* Portfolio and Time Period Selectors */}
      <Grid container spacing={2} mb={3}>
        <Grid item xs={12} md={4}>
          <FormControl fullWidth>
            <InputLabel>Portfolio</InputLabel>
            <Select
              value={selectedPortfolioId}
              label="Portfolio"
              onChange={(e) => setSelectedPortfolioId(e.target.value)}
            >
              {portfoliosQ.data?.map((portfolio) => (
                <MenuItem key={portfolio.id} value={portfolio.id}>
                  {portfolio.name}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        </Grid>
        <Grid item xs={12} md={4}>
          <FormControl fullWidth>
            <InputLabel>Time Period</InputLabel>
            <Select value={days} label="Time Period" onChange={(e) => setDays(Number(e.target.value))}>
              <MenuItem value={30}>30 Days</MenuItem>
              <MenuItem value={60}>60 Days</MenuItem>
              <MenuItem value={90}>90 Days</MenuItem>
              <MenuItem value={180}>180 Days</MenuItem>
              <MenuItem value={365}>1 Year</MenuItem>
            </Select>
          </FormControl>
        </Grid>
        <Grid item xs={12} md={4}>
          <Box display="flex" gap={1} height="100%">
            <Button
              variant="outlined"
              startIcon={
                <Refresh
                  sx={{
                    animation: isRefreshing ? 'spin 1s linear infinite' : 'none',
                    '@keyframes spin': {
                      '0%': { transform: 'rotate(0deg)' },
                      '100%': { transform: 'rotate(360deg)' },
                    },
                  }}
                />
              }
              onClick={handleRefresh}
              disabled={isRefreshing || correlationQ.isLoading}
              fullWidth
            >
              {isRefreshing ? 'Refreshing...' : 'Refresh'}
            </Button>
          </Box>
        </Grid>
      </Grid>

      {/* Cache Status Indicator */}
      {cacheStatusMessage && (
        <Box display="flex" alignItems="center" gap={1} mb={2}>
          <Chip
            icon={<Cached />}
            label="Cached"
            color="success"
            variant="outlined"
            size="small"
          />
          <Chip
            icon={<Schedule />}
            label={cacheStatusMessage}
            variant="outlined"
            size="small"
          />
          <Typography variant="caption" color="text.secondary">
            Next auto-update: {getNextCalculationTime.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
          </Typography>
        </Box>
      )}

      <Alert severity="info" sx={{ mb: 3 }}>
        Correlation shows how positions move together. Values range from -1 (move opposite) to +1 (move
        together). High positive correlations indicate concentration risk. Analysis includes positions
        representing at least 1% of portfolio value (limited to top 10 positions for performance).
      </Alert>

      {/* Correlation Statistics Card */}
      {matrix.statistics && (
        <Box sx={{ mb: 3 }}>
          <CorrelationStatsCard statistics={matrix.statistics} />
        </Box>
      )}

      {tickers.length < 2 ? (
        <Alert severity="warning">
          Need at least 2 positions with sufficient price data for correlation analysis.
        </Alert>
      ) : (
        <Paper elevation={2} sx={{ p: 2, overflowX: 'auto' }}>
          <Box
            sx={{
              display: 'grid',
              gridTemplateColumns: `120px repeat(${tickers.length}, minmax(80px, 100px))`,
              gap: '2px',
              minWidth: 'fit-content',
            }}
          >
            {/* Top-left corner cell (empty) */}
            <Box />

            {/* Column headers (ticker symbols) */}
            {tickers.map((ticker) => (
              <Box
                key={`header-${ticker}`}
                sx={{
                  p: 1,
                  textAlign: 'center',
                  fontWeight: 'bold',
                  fontSize: '0.875rem',
                  backgroundColor: '#f5f5f5',
                  borderRadius: '4px',
                }}
              >
                {onTickerNavigate ? (
                  <TickerActionMenu
                    ticker={ticker}
                    variant="text"
                    size="small"
                    onNavigate={onTickerNavigate}
                  />
                ) : (
                  ticker
                )}
              </Box>
            ))}

            {/* Data rows */}
            {tickers.map((rowTicker) => (
              <Box
                key={`row-${rowTicker}`}
                sx={{
                  display: 'contents', // Make the row ticker and its cells part of the grid
                }}
              >
                {/* Row header (ticker symbol) */}
                <Box
                  sx={{
                    p: 1,
                    display: 'flex',
                    alignItems: 'center',
                    fontWeight: 'bold',
                    fontSize: '0.875rem',
                    backgroundColor: '#f5f5f5',
                    borderRadius: '4px',
                  }}
                >
                  {onTickerNavigate ? (
                    <TickerActionMenu
                      ticker={rowTicker}
                      variant="text"
                      size="small"
                      onNavigate={onTickerNavigate}
                    />
                  ) : (
                    rowTicker
                  )}
                </Box>

                {/* Correlation cells */}
                {tickers.map((colTicker) => {
                  const correlation = getCorrelation(matrix, rowTicker, colTicker);
                  const color = getCorrelationColor(correlation);
                  const label = correlation !== null ? getCorrelationLabel(correlation) : 'N/A';

                  return (
                    <Tooltip
                      key={`cell-${rowTicker}-${colTicker}`}
                      title={
                        correlation !== null
                          ? `${rowTicker} vs ${colTicker}: ${correlation.toFixed(3)} (${label})`
                          : 'No data'
                      }
                      arrow
                    >
                      <Box
                        sx={{
                          p: 1,
                          textAlign: 'center',
                          backgroundColor: color,
                          borderRadius: '4px',
                          cursor: 'pointer',
                          fontSize: '0.75rem',
                          fontWeight: rowTicker === colTicker ? 'bold' : 'normal',
                          transition: 'transform 0.2s',
                          '&:hover': {
                            transform: 'scale(1.05)',
                            boxShadow: 2,
                          },
                        }}
                      >
                        {correlation !== null ? correlation.toFixed(2) : 'N/A'}
                      </Box>
                    </Tooltip>
                  );
                })}
              </Box>
            ))}
          </Box>

          {/* Legend */}
          <Box mt={3} display="flex" justifyContent="center" alignItems="center" gap={1}>
            <Typography variant="body2" fontWeight="bold">
              Correlation:
            </Typography>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box
                sx={{
                  width: 20,
                  height: 20,
                  backgroundColor: getCorrelationColor(-1.0),
                  borderRadius: '4px',
                }}
              />
              <Typography variant="caption">-1.0</Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box
                sx={{
                  width: 20,
                  height: 20,
                  backgroundColor: getCorrelationColor(-0.5),
                  borderRadius: '4px',
                }}
              />
              <Typography variant="caption">-0.5</Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box
                sx={{
                  width: 20,
                  height: 20,
                  backgroundColor: getCorrelationColor(0.0),
                  borderRadius: '4px',
                }}
              />
              <Typography variant="caption">0.0</Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box
                sx={{
                  width: 20,
                  height: 20,
                  backgroundColor: getCorrelationColor(0.5),
                  borderRadius: '4px',
                }}
              />
              <Typography variant="caption">+0.5</Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box
                sx={{
                  width: 20,
                  height: 20,
                  backgroundColor: getCorrelationColor(1.0),
                  borderRadius: '4px',
                }}
              />
              <Typography variant="caption">+1.0</Typography>
            </Box>
          </Box>

          <Box mt={2}>
            <Typography variant="body2" color="text.secondary" align="center">
              Hover over cells to see detailed correlation values and strength labels
            </Typography>
          </Box>
        </Paper>
      )}

      {/* Snackbar for feedback messages */}
      <Snackbar
        open={snackbarOpen}
        autoHideDuration={6000}
        onClose={() => setSnackbarOpen(false)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert
          onClose={() => setSnackbarOpen(false)}
          severity={snackbarSeverity}
          variant="filled"
          sx={{ width: '100%' }}
        >
          {snackbarMessage}
        </Alert>
      </Snackbar>
    </Box>
  );
}
