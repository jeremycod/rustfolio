import { useState } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  Box,
  Card,
  CardContent,
  Typography,
  ToggleButtonGroup,
  ToggleButton,
  Alert,
  CircularProgress,
  Chip,
  Stack,
  Tooltip as MuiTooltip,
  Button,
  LinearProgress,
  Paper,
  List,
  ListItem,
  ListItemIcon,
  ListItemText,
  IconButton,
} from '@mui/material';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  ReferenceLine,
} from 'recharts';
import {
  TrendingUp,
  ShowChart,
  Download,
  CheckCircle,
  Info,
  Timeline,
  HelpOutline,
  Refresh,
  AccessTime,
} from '@mui/icons-material';
import { getRollingBeta, updatePriceHistory } from '../lib/endpoints';
import type { RollingBetaAnalysis, BetaPoint } from '../types';
import { MetricHelpDialog } from './MetricHelpDialog';

interface RollingBetaChartProps {
  ticker: string;
  benchmark?: string;
  days?: number;
  autoCalculate?: boolean; // Auto-calculate if data not cached
}

type WindowSize = '30d' | '60d' | '90d';

export function RollingBetaChart({ ticker, benchmark = 'SPY', days = 180, autoCalculate = false }: RollingBetaChartProps) {
  const [selectedWindow, setSelectedWindow] = useState<WindowSize>('90d');
  const [helpOpen, setHelpOpen] = useState<string | null>(null);
  const [forceCalculate, setForceCalculate] = useState(autoCalculate);
  const queryClient = useQueryClient();

  // Fetch rolling beta data
  const rollingBetaQuery = useQuery({
    queryKey: ['rolling-beta', ticker, days, benchmark, forceCalculate],
    queryFn: () => getRollingBeta(ticker, days, benchmark, forceCalculate),
    staleTime: 1000 * 60 * 60, // 1 hour
    retry: forceCalculate ? 2 : 1, // Retry more when forcing calculation
  });

  // Mutation for fetching price history
  const fetchPriceMutation = useMutation({
    mutationFn: (ticker: string) => updatePriceHistory(ticker),
    onSuccess: () => {
      // Invalidate and refetch rolling beta after price update
      queryClient.invalidateQueries({ queryKey: ['rolling-beta', ticker, days, benchmark] });
    },
  });

  const handleWindowToggle = (
    _event: React.MouseEvent<HTMLElement>,
    newWindow: WindowSize | null
  ) => {
    if (newWindow) {
      setSelectedWindow(newWindow);
    }
  };

  const handleFetchData = async () => {
    // Fetch both ticker and benchmark data
    try {
      await fetchPriceMutation.mutateAsync(ticker);
      if (benchmark && benchmark !== ticker) {
        await fetchPriceMutation.mutateAsync(benchmark);
      }
    } catch (error) {
      console.error('Error fetching price data:', error);
    }
  };

  if (rollingBetaQuery.isLoading) {
    return (
      <Box display="flex" flexDirection="column" justifyContent="center" alignItems="center" minHeight="400px" gap={2}>
        <CircularProgress />
        <Typography variant="body2" color="text.secondary">
          Computing rolling beta... This may take up to a minute.
        </Typography>
      </Box>
    );
  }

  if (rollingBetaQuery.isError || rollingBetaQuery.error) {
    // Extract error message from various possible locations
    const error = rollingBetaQuery.error as any;
    const errorMessage =
      error?.response?.data?.error ||  // Standard error format
      error?.response?.data ||         // Plain text error (503 case)
      error?.message ||                // Error object message
      'Unknown error';

    console.log('Error details:', { error, errorMessage });

    const isInsufficientData = errorMessage.includes('Insufficient price data');

    if (isInsufficientData) {
      // Extract days information from error message
      const match = errorMessage.match(/got (\d+) for (\w+)/g);
      const tickerDays = errorMessage.match(/got (\d+) for (\w+)/)?.[1] || '?';

      return (
        <Card elevation={2}>
          <CardContent>
            <Box sx={{ textAlign: 'center', py: 4 }}>
              <Timeline sx={{ fontSize: 80, color: 'primary.main', mb: 2 }} />

              <Typography variant="h5" gutterBottom>
                Setup Required: Historical Price Data
              </Typography>

              <Typography variant="body1" color="text.secondary" sx={{ mb: 3, maxWidth: 600, mx: 'auto' }}>
                Rolling beta analysis requires at least 90 days of historical price data to calculate
                how {ticker}'s volatility correlates with {benchmark} over time.
              </Typography>

              <Alert severity="info" sx={{ mb: 3, maxWidth: 700, mx: 'auto', textAlign: 'left' }}>
                <Typography variant="body2" gutterBottom>
                  <strong>Current Status:</strong>
                </Typography>
                <Typography variant="body2">
                  • {ticker}: {tickerDays} days (need 90+)<br/>
                  • {benchmark}: Available ✓
                </Typography>
              </Alert>

              <Paper elevation={0} sx={{ p: 3, bgcolor: 'background.default', mb: 3, maxWidth: 600, mx: 'auto' }}>
                <Typography variant="subtitle2" gutterBottom sx={{ fontWeight: 600 }}>
                  What is Rolling Beta?
                </Typography>
                <List dense>
                  <ListItem>
                    <ListItemIcon><CheckCircle color="success" fontSize="small" /></ListItemIcon>
                    <ListItemText
                      primary="Measures how a stock moves relative to the market"
                      secondary="Beta = 1.0 means moves with the market, >1.0 more volatile, <1.0 less volatile"
                    />
                  </ListItem>
                  <ListItem>
                    <ListItemIcon><ShowChart color="primary" fontSize="small" /></ListItemIcon>
                    <ListItemText
                      primary="Shows how correlation changes over time"
                      secondary="Helps identify regime changes and market dynamics"
                    />
                  </ListItem>
                  <ListItem>
                    <ListItemIcon><TrendingUp color="secondary" fontSize="small" /></ListItemIcon>
                    <ListItemText
                      primary="Analyzes 30, 60, and 90-day windows"
                      secondary="Multiple timeframes reveal short and long-term trends"
                    />
                  </ListItem>
                </List>
              </Paper>

              <Box sx={{ mb: 2 }}>
                <Button
                  variant="contained"
                  size="large"
                  startIcon={fetchPriceMutation.isPending ? <CircularProgress size={20} color="inherit" /> : <Download />}
                  onClick={handleFetchData}
                  disabled={fetchPriceMutation.isPending}
                  sx={{ minWidth: 250 }}
                >
                  {fetchPriceMutation.isPending ? 'Fetching Data...' : `Fetch Price History for ${ticker}`}
                </Button>
              </Box>

              {fetchPriceMutation.isPending && (
                <Box sx={{ maxWidth: 400, mx: 'auto', mb: 2 }}>
                  <LinearProgress />
                  <Typography variant="caption" color="text.secondary" sx={{ mt: 1 }}>
                    Downloading historical price data from API...
                  </Typography>
                </Box>
              )}

              {fetchPriceMutation.isError && (
                <Alert severity="error" sx={{ maxWidth: 500, mx: 'auto' }}>
                  <Typography variant="body2" gutterBottom>
                    <strong>Failed to fetch price data for {ticker}</strong>
                  </Typography>
                  <Typography variant="body2">
                    {(() => {
                      const error = fetchPriceMutation.error as any;
                      const errorMessage = error?.response?.data || error?.message || 'Unknown error';

                      // Check if it's a ticker not found error
                      if (errorMessage.includes('not_found') || errorMessage.includes('not found')) {
                        return `Ticker "${ticker}" may not be supported by the price provider. This often happens with mutual funds, bonds, or invalid tickers.`;
                      }

                      // Check if it's a failure cache error
                      if (errorMessage.includes('failure cache')) {
                        return `Ticker "${ticker}" is temporarily blocked due to previous failures. Please try again later or contact support.`;
                      }

                      return errorMessage;
                    })()}
                  </Typography>
                </Alert>
              )}

              {fetchPriceMutation.isSuccess && !rollingBetaQuery.isLoading && (
                <Alert severity="success" sx={{ maxWidth: 500, mx: 'auto' }}>
                  ✓ Price data fetched successfully! Refreshing analysis...
                </Alert>
              )}

              <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 3 }}>
                <Info fontSize="small" sx={{ verticalAlign: 'middle', mr: 0.5 }} />
                This is a one-time setup. Data will be cached and updated automatically.
              </Typography>
            </Box>
          </CardContent>
        </Card>
      );
    }

    // Check if it's a "data not available" error that can be force-calculated
    const isDataNotAvailable = errorMessage.includes('not available') || errorMessage.includes('background job');

    if (isDataNotAvailable && !forceCalculate) {
      return (
        <Card elevation={2}>
          <CardContent>
            <Box sx={{ textAlign: 'center', py: 4 }}>
              <AccessTime sx={{ fontSize: 80, color: 'warning.main', mb: 2 }} />

              <Typography variant="h5" gutterBottom>
                Rolling Beta Data Not Cached
              </Typography>

              <Typography variant="body1" color="text.secondary" sx={{ mb: 3, maxWidth: 600, mx: 'auto' }}>
                Rolling beta analysis for {ticker} vs {benchmark} hasn't been calculated yet.
              </Typography>

              <Alert severity="info" sx={{ mb: 3, maxWidth: 600, mx: 'auto', textAlign: 'left' }}>
                <Typography variant="body2">
                  {errorMessage}
                </Typography>
              </Alert>

              <Button
                variant="contained"
                size="large"
                startIcon={<Refresh />}
                onClick={() => setForceCalculate(true)}
                sx={{ minWidth: 250 }}
              >
                Calculate Now
              </Button>

              <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 2 }}>
                This will calculate the rolling beta immediately (may take 30-60 seconds)
              </Typography>
            </Box>
          </CardContent>
        </Card>
      );
    }

    // Other errors
    return (
      <Card elevation={2}>
        <CardContent>
          <Alert severity="error">
            <Typography variant="body1" gutterBottom>
              <strong>Error loading rolling beta</strong>
            </Typography>
            <Typography variant="body2">
              {errorMessage}
            </Typography>
          </Alert>
        </CardContent>
      </Card>
    );
  }

  if (!rollingBetaQuery.data) {
    return (
      <Alert severity="info">
        No rolling beta data available. Need at least 90 days of price history for {ticker} and {benchmark}.
      </Alert>
    );
  }

  // Handle new cached response format
  const response = rollingBetaQuery.data as any;
  const analysis: RollingBetaAnalysis = response.data || response; // Support both old and new format
  const cacheStatus = response.cache_status;
  const actions = response.actions;

  // Validate data structure
  if (!analysis || typeof analysis !== 'object') {
    return (
      <Alert severity="error">
        Invalid data structure received. Please refresh the page or contact support.
      </Alert>
    );
  }

  // Select data based on window
  const selectedData: BetaPoint[] | undefined = {
    '30d': analysis.beta_30d,
    '60d': analysis.beta_60d,
    '90d': analysis.beta_90d,
  }[selectedWindow];

  // Check if data exists for the selected window
  if (!selectedData || !Array.isArray(selectedData) || selectedData.length === 0) {
    return (
      <Alert severity="warning" sx={{ mt: 2 }}>
        <Typography variant="body2" fontWeight="bold" gutterBottom>
          No {selectedWindow} rolling beta data available
        </Typography>
        <Typography variant="body2">
          This could mean:
        </Typography>
        <Typography variant="body2" component="div" sx={{ mt: 1 }}>
          • Insufficient price history (need at least {selectedWindow === '30d' ? '30' : selectedWindow === '60d' ? '60' : '90'} days)
          <br />
          • Data hasn't been calculated yet (background job runs every 6 hours)
          <br />
          • Try selecting a different time window or refresh the data
        </Typography>
      </Alert>
    );
  }

  // Transform data for recharts
  const chartData = selectedData.map((point) => ({
    date: new Date(point.date).toLocaleDateString(),
    beta: point.beta,
    r_squared: point.r_squared * 100, // Convert to percentage
  }));

  // Determine beta interpretation
  const getBetaInterpretation = (beta: number): string => {
    if (beta > 1.2) return 'Highly volatile relative to market';
    if (beta > 1.0) return 'More volatile than market';
    if (beta > 0.8) return 'Similar to market volatility';
    if (beta > 0.5) return 'Less volatile than market';
    if (beta > 0) return 'Low volatility relative to market';
    return 'Moves opposite to market';
  };

  const getBetaColor = (beta: number): string => {
    const absBeta = Math.abs(beta);
    if (absBeta > 1.2) return 'error';
    if (absBeta > 1.0) return 'warning';
    return 'success';
  };

  // Format cache age
  const formatCacheAge = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffMinutes = Math.floor((diffMs % (1000 * 60 * 60)) / (1000 * 60));

    if (diffHours > 0) {
      return `${diffHours}h ${diffMinutes}m ago`;
    }
    return `${diffMinutes}m ago`;
  };

  // Force refresh handler
  const handleForceRefresh = async () => {
    try {
      // Call the endpoint with force=true to bypass cache
      await getRollingBeta(ticker, days, benchmark, true);

      // Invalidate the query to trigger a refetch
      queryClient.invalidateQueries({ queryKey: ['rolling-beta', ticker, days, benchmark] });
    } catch (error) {
      console.error('Failed to refresh data:', error);
    }
  };

  return (
    <Card elevation={2}>
      <CardContent>
        {/* Cache Status Banner */}
        {cacheStatus && (
          <Alert
            severity={cacheStatus.is_stale ? "warning" : "info"}
            sx={{ mb: 2 }}
            icon={<AccessTime />}
            action={
              cacheStatus.is_stale && actions && actions.length > 0 && (
                <Button
                  color="inherit"
                  size="small"
                  startIcon={<Refresh />}
                  onClick={handleForceRefresh}
                >
                  Refresh Data
                </Button>
              )
            }
          >
            <Typography variant="body2">
              <strong>Cache Status:</strong> Last updated {formatCacheAge(cacheStatus.last_updated)}
              {cacheStatus.is_stale && ' - Data may be outdated'}
            </Typography>
          </Alert>
        )}

        <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
          <Box>
            <Typography variant="h6" gutterBottom>
              Rolling Beta Analysis: {ticker} vs {benchmark}
            </Typography>
            <Typography variant="caption" color="text.secondary">
              {getBetaInterpretation(analysis.current_beta)}
            </Typography>
          </Box>

          <Stack direction="row" spacing={2} alignItems="center">
            <Box display="flex" alignItems="center" gap={0.5}>
              <Chip
                icon={<TrendingUp />}
                label={`β: ${analysis.current_beta.toFixed(2)}`}
                color={getBetaColor(analysis.current_beta) as any}
                size="small"
              />
              <IconButton
                size="small"
                onClick={() => setHelpOpen('beta')}
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

            <Box display="flex" alignItems="center" gap={0.5}>
              <Chip
                icon={<ShowChart />}
                label={`σ: ${analysis.beta_volatility.toFixed(2)}`}
                color="default"
                size="small"
              />
              <IconButton
                size="small"
                onClick={() => setHelpOpen('beta_volatility')}
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
          </Stack>
        </Box>

        {/* Window Selection */}
        <Box mb={2}>
          <ToggleButtonGroup
            value={selectedWindow}
            exclusive
            onChange={handleWindowToggle}
            size="small"
          >
            <ToggleButton value="30d">30-Day Window</ToggleButton>
            <ToggleButton value="60d">60-Day Window</ToggleButton>
            <ToggleButton value="90d">90-Day Window</ToggleButton>
          </ToggleButtonGroup>
        </Box>

        <Alert severity="info" sx={{ mb: 2 }}>
          Rolling {selectedWindow} beta shows how {ticker}'s price movement correlation with {benchmark} changes over time.
          Beta = 1 means moves in line with the market. Higher beta indicates greater volatility.
        </Alert>

        {chartData.length > 0 ? (
          <ResponsiveContainer width="100%" height={400}>
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
                label={{ value: 'Beta', angle: -90, position: 'insideLeft' }}
                domain={[(dataMin: number) => Math.min(dataMin, 0.5), (dataMax: number) => Math.max(dataMax, 1.5)]}
              />
              <YAxis
                yAxisId="right"
                orientation="right"
                label={{ value: 'R² (%)', angle: 90, position: 'insideRight' }}
              />
              <Tooltip
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
                          <strong>Beta:</strong> {(payload[0].payload.beta as number).toFixed(3)}
                        </Typography>
                        <Typography variant="body2" color="secondary">
                          <strong>R²:</strong> {(payload[0].payload.r_squared as number).toFixed(1)}%
                        </Typography>
                      </Box>
                    );
                  }
                  return null;
                }}
              />
              <Legend />

              {/* Beta = 1.0 reference line */}
              <ReferenceLine
                yAxisId="left"
                y={1}
                stroke="#666"
                strokeDasharray="3 3"
                label={{ value: 'Market Beta (1.0)', position: 'right' }}
              />

              {/* Beta line */}
              <Line
                yAxisId="left"
                type="monotone"
                dataKey="beta"
                stroke="#1976d2"
                strokeWidth={2}
                dot={false}
                name="Beta"
              />

              {/* R-squared line */}
              <Line
                yAxisId="right"
                type="monotone"
                dataKey="r_squared"
                stroke="#ff9800"
                strokeWidth={2}
                dot={false}
                name="R² (%)"
              />
            </LineChart>
          </ResponsiveContainer>
        ) : (
          <Alert severity="warning">
            Insufficient data to calculate rolling beta for this window size.
          </Alert>
        )}

        {analysis.beta_volatility > 0.3 && (
          <Alert severity="warning" sx={{ mt: 2 }}>
            High beta volatility detected ({analysis.beta_volatility.toFixed(2)}).
            The stock's correlation with {benchmark} is unstable, which may indicate changing market dynamics or company-specific factors.
          </Alert>
        )}
      </CardContent>

      {/* Help Dialog */}
      {helpOpen && (
        <MetricHelpDialog
          open={true}
          onClose={() => setHelpOpen(null)}
          metricKey={helpOpen}
        />
      )}
    </Card>
  );
}
