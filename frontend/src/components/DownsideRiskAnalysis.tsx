import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
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
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Divider,
  IconButton,
} from '@mui/material';
import { TrendingDown, CheckCircle, Warning as WarningIcon, Error as ErrorIcon, Info as InfoIcon, HelpOutline, AccessTime, Refresh } from '@mui/icons-material';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { getPortfolioDownsideRisk, listPortfolios } from '../lib/endpoints';
import type { Portfolio, PositionDownsideContribution } from '../types';
import { MetricHelpDialog } from './MetricHelpDialog';
import { Button } from '@mui/material';

interface DownsideRiskAnalysisProps {
  portfolioId?: string | null;
}

export function DownsideRiskAnalysis({ portfolioId: initialPortfolioId }: DownsideRiskAnalysisProps) {
  const [portfolioId, setPortfolioId] = useState<string | null>(initialPortfolioId || null);
  const [days, setDays] = useState(90);
  const [benchmark, setBenchmark] = useState('SPY');
  const [helpOpen, setHelpOpen] = useState(false);
  const [selectedMetric, setSelectedMetric] = useState<string>('');
  const queryClient = useQueryClient();

  // Fetch portfolios
  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  // Auto-select first portfolio if available
  useState(() => {
    if (portfoliosQ.data && portfoliosQ.data.length > 0 && !portfolioId) {
      setPortfolioId(portfoliosQ.data[0].id);
    }
  });

  // Fetch downside risk data
  const downsideRiskQ = useQuery({
    queryKey: ['downsideRisk', portfolioId, days, benchmark],
    queryFn: () => portfolioId ? getPortfolioDownsideRisk(portfolioId, days, benchmark) : Promise.reject('No portfolio'),
    enabled: !!portfolioId,
    retry: 1,
  });

  // Extract data from cached response format
  const response = downsideRiskQ.data as any;
  const analysisData = response?.data || response; // Support both old and new format
  const cacheStatus = response?.cache_status;
  const actions = response?.actions;

  const formatPercent = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    return `${value.toFixed(2)}%`;
  };

  const formatNumber = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    return value.toFixed(2);
  };

  const getRatingIcon = (rating: string) => {
    switch (rating.toLowerCase()) {
      case 'excellent':
        return <CheckCircle sx={{ color: 'success.main', mr: 1 }} />;
      case 'good':
        return <CheckCircle sx={{ color: 'info.main', mr: 1 }} />;
      case 'fair':
        return <WarningIcon sx={{ color: 'warning.main', mr: 1 }} />;
      case 'poor':
        return <ErrorIcon sx={{ color: 'error.main', mr: 1 }} />;
      default:
        return null;
    }
  };

  const getRatingColor = (rating: string): string => {
    switch (rating.toLowerCase()) {
      case 'excellent':
        return 'success';
      case 'good':
        return 'info';
      case 'fair':
        return 'warning';
      case 'poor':
        return 'error';
      default:
        return 'default';
    }
  };

  const getRiskLevelColor = (level: string): "success" | "warning" | "error" | "default" => {
    const lower = level.toLowerCase();
    if (lower === 'low') return 'success';
    if (lower === 'moderate') return 'warning';
    if (lower === 'high' || lower === 'very high') return 'error';
    return 'default';
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
    if (!portfolioId) return;

    try {
      // Call the endpoint with force=true to bypass cache
      await getPortfolioDownsideRisk(portfolioId, days, benchmark, true);

      // Invalidate the query to trigger a refetch
      queryClient.invalidateQueries({ queryKey: ['downsideRisk', portfolioId, days, benchmark] });
    } catch (error) {
      console.error('Failed to refresh data:', error);
    }
  };

  const renderPortfolioMetrics = () => {
    if (!analysisData) return null;

    const metrics = analysisData.portfolio_metrics;
    const interpretation = metrics.interpretation;

    return (
      <Box>
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

        {/* Portfolio-level Metrics */}
        <Paper sx={{ p: 3, mb: 3 }}>
          <Typography variant="h6" gutterBottom>
            Portfolio Downside Metrics
          </Typography>
          <Grid container spacing={3}>
            <Grid item xs={12} md={6}>
              <Card variant="outlined">
                <CardContent>
                  <Box display="flex" alignItems="center" justifyContent="space-between" mb={1}>
                    <Typography variant="subtitle2" color="text.secondary">
                      Downside Deviation (Semi-Deviation)
                    </Typography>
                    <IconButton
                      size="small"
                      onClick={() => {
                        setSelectedMetric('downside_deviation');
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
                  <Typography variant="h4" color="primary">
                    {formatPercent(metrics.downside_deviation)}
                  </Typography>
                  <Box sx={{ mt: 1 }}>
                    <Chip
                      label={interpretation.downside_risk_level}
                      color={getRiskLevelColor(interpretation.downside_risk_level)}
                      size="small"
                    />
                  </Box>
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
                    Volatility of returns below the target (MAR: {formatPercent(metrics.mar)})
                  </Typography>
                </CardContent>
              </Card>
            </Grid>

            <Grid item xs={12} md={6}>
              <Card variant="outlined" sx={{ bgcolor: 'info.50' }}>
                <CardContent>
                  <Box display="flex" alignItems="center" justifyContent="space-between" mb={1}>
                    <Typography variant="subtitle2" color="text.secondary">
                      Sortino Ratio
                    </Typography>
                    <IconButton
                      size="small"
                      onClick={() => {
                        setSelectedMetric('sortino_ratio');
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
                  <Box display="flex" alignItems="center">
                    {getRatingIcon(interpretation.sortino_rating)}
                    <Typography variant="h4" color="primary">
                      {formatNumber(metrics.sortino_ratio)}
                    </Typography>
                  </Box>
                  <Box sx={{ mt: 1 }}>
                    <Chip
                      label={interpretation.sortino_rating}
                      color={getRatingColor(interpretation.sortino_rating) as any}
                      size="small"
                    />
                  </Box>
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
                    Risk-adjusted return (downside focus)
                  </Typography>
                </CardContent>
              </Card>
            </Grid>

            <Grid item xs={12} md={6}>
              <Card variant="outlined">
                <CardContent>
                  <Box display="flex" alignItems="center" justifyContent="space-between" mb={1}>
                    <Typography variant="subtitle2" color="text.secondary">
                      Sharpe Ratio (For Comparison)
                    </Typography>
                    <IconButton
                      size="small"
                      onClick={() => {
                        setSelectedMetric('sharpe_ratio');
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
                  <Typography variant="h4" color="text.secondary">
                    {formatNumber(metrics.sharpe_ratio)}
                  </Typography>
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
                    Traditional risk-adjusted return measure
                  </Typography>
                </CardContent>
              </Card>
            </Grid>

            <Grid item xs={12} md={6}>
              <Card variant="outlined">
                <CardContent>
                  <Box display="flex" alignItems="center" justifyContent="space-between" mb={1}>
                    <Typography variant="subtitle2" color="text.secondary">
                      Minimum Acceptable Return (MAR)
                    </Typography>
                    <IconButton
                      size="small"
                      onClick={() => {
                        setSelectedMetric('mar');
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
                  <Typography variant="h4" color="text.secondary">
                    {formatPercent(metrics.mar)}
                  </Typography>
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
                    Threshold for downside calculation
                  </Typography>
                </CardContent>
              </Card>
            </Grid>
          </Grid>
        </Paper>

        {/* Interpretation */}
        <Alert severity="info" icon={<InfoIcon />} sx={{ mb: 3 }}>
          <Typography variant="subtitle2" gutterBottom>
            <strong>Interpretation:</strong>
          </Typography>
          <Typography variant="body2" gutterBottom>
            {interpretation.summary}
          </Typography>
          <Divider sx={{ my: 1 }} />
          <Typography variant="body2" gutterBottom>
            <strong>Sortino vs Sharpe:</strong> {interpretation.sortino_vs_sharpe}
          </Typography>
          <Typography variant="body2" sx={{ mt: 1 }}>
            • <strong>Sortino Ratio:</strong> Uses only downside volatility (more conservative)
          </Typography>
          <Typography variant="body2">
            • <strong>Sharpe Ratio:</strong> Uses total volatility (penalizes upside)
          </Typography>
          <Typography variant="body2" sx={{ mt: 1 }}>
            A higher Sortino than Sharpe suggests good downside protection with acceptable upside volatility.
          </Typography>
        </Alert>

        {/* Position Breakdown */}
        <Paper sx={{ p: 3 }}>
          <Typography variant="h6" gutterBottom>
            Position Breakdown (Sorted by Downside Risk)
          </Typography>
          <Typography variant="body2" color="text.secondary" mb={2}>
            Positions sorted by downside deviation from highest to lowest risk.
          </Typography>
          <TableContainer>
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>Ticker</TableCell>
                  <TableCell align="right">Weight</TableCell>
                  <TableCell align="right">Downside Deviation</TableCell>
                  <TableCell align="right">Sortino Ratio</TableCell>
                  <TableCell align="right">Risk Level</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {analysisData?.position_downside_risks && analysisData.position_downside_risks.length > 0 ? (
                  analysisData.position_downside_risks
                    .sort((a: PositionDownsideContribution, b: PositionDownsideContribution) => b.downside_metrics.downside_deviation - a.downside_metrics.downside_deviation)
                    .map((pos: PositionDownsideContribution) => (
                      <TableRow key={pos.ticker}>
                        <TableCell>
                          <strong>{pos.ticker}</strong>
                        </TableCell>
                        <TableCell align="right">{(pos.weight * 100).toFixed(1)}%</TableCell>
                        <TableCell align="right">
                          <Typography
                            variant="body2"
                            color={
                              pos.downside_metrics.downside_deviation > 20 ? 'error.main' :
                              pos.downside_metrics.downside_deviation > 15 ? 'warning.main' : 'success.main'
                            }
                          >
                            {formatPercent(pos.downside_metrics.downside_deviation)}
                          </Typography>
                        </TableCell>
                        <TableCell align="right">
                          <Typography
                            variant="body2"
                            color={
                              pos.downside_metrics.sortino_ratio && pos.downside_metrics.sortino_ratio > 2 ? 'success.main' :
                              pos.downside_metrics.sortino_ratio && pos.downside_metrics.sortino_ratio > 1 ? 'info.main' : 'warning.main'
                            }
                          >
                            {formatNumber(pos.downside_metrics.sortino_ratio || 0)}
                          </Typography>
                        </TableCell>
                        <TableCell align="right">
                          <Chip
                            label={
                              pos.downside_metrics.downside_deviation < 15 ? 'Low' :
                              pos.downside_metrics.downside_deviation < 25 ? 'Moderate' :
                              pos.downside_metrics.downside_deviation < 35 ? 'High' : 'Very High'
                            }
                            size="small"
                            color={
                              pos.downside_metrics.downside_deviation < 15 ? 'success' :
                              pos.downside_metrics.downside_deviation < 25 ? 'warning' : 'error'
                            }
                          />
                        </TableCell>
                      </TableRow>
                    ))
                ) : (
                  <TableRow>
                    <TableCell colSpan={5} align="center">
                      <Typography variant="body2" color="text.secondary" sx={{ py: 3 }}>
                        No position data available
                      </Typography>
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </TableContainer>
        </Paper>
      </Box>
    );
  };

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <TrendingDown sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Downside Risk Analysis
        </Typography>
      </Box>

      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Portfolio Selection
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Analyze downside-focused risk metrics including Sortino ratio and downside deviation (semi-deviation).
          These metrics focus only on negative returns below a target threshold.
        </Typography>

        <Grid container spacing={2} alignItems="center">
          <Grid item xs={12} sm={4}>
            <FormControl fullWidth>
              <InputLabel>Portfolio</InputLabel>
              <Select
                value={portfolioId || ''}
                label="Portfolio"
                onChange={(e) => setPortfolioId(e.target.value)}
              >
                {portfoliosQ.data?.map((p: Portfolio) => (
                  <MenuItem key={p.id} value={p.id}>
                    {p.name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Grid>

          <Grid item xs={12} sm={4}>
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

          <Grid item xs={12} sm={4}>
            <FormControl fullWidth>
              <InputLabel>Benchmark</InputLabel>
              <Select value={benchmark} label="Benchmark" onChange={(e) => setBenchmark(e.target.value)}>
                <MenuItem value="SPY">S&P 500 (SPY)</MenuItem>
                <MenuItem value="QQQ">NASDAQ-100 (QQQ)</MenuItem>
                <MenuItem value="DIA">Dow Jones (DIA)</MenuItem>
                <MenuItem value="IWM">Russell 2000 (IWM)</MenuItem>
              </Select>
            </FormControl>
          </Grid>
        </Grid>
      </Paper>

      {!portfolioId && (
        <Alert severity="warning" icon={<WarningIcon />}>
          Please select a portfolio to view downside risk analysis.
        </Alert>
      )}

      {downsideRiskQ.isLoading && (
        <Box display="flex" justifyContent="center" alignItems="center" minHeight={400}>
          <CircularProgress />
        </Box>
      )}

      {downsideRiskQ.error && (
        <Alert severity="error">
          <Typography variant="body1" gutterBottom>
            <strong>Failed to load downside risk data</strong>
          </Typography>
          <Typography variant="body2">
            {(() => {
              const error = downsideRiskQ.error as any;
              const errorMessage = error?.response?.data || error?.message || 'Unknown error';

              // Check if it's a cache miss (404)
              if (error?.response?.status === 404 || errorMessage.includes('not available')) {
                return (
                  <Box>
                    <Typography variant="body2" paragraph>
                      {errorMessage}
                    </Typography>
                    <Typography variant="body2">
                      <strong>To fix this:</strong>
                    </Typography>
                    <Typography variant="body2" component="ul" sx={{ pl: 2 }}>
                      <li>Go to Admin → Jobs</li>
                      <li>Find "populate_downside_risk_cache"</li>
                      <li>Click "Trigger Now"</li>
                      <li>Wait 2-5 minutes, then refresh this page</li>
                    </Typography>
                  </Box>
                );
              }

              return errorMessage;
            })()}
          </Typography>
        </Alert>
      )}

      {downsideRiskQ.data && renderPortfolioMetrics()}

      {!downsideRiskQ.data && !downsideRiskQ.isLoading && portfolioId && (
        <Alert severity="info">
          Select a portfolio and time period to view downside risk analysis.
        </Alert>
      )}

      <MetricHelpDialog
        open={helpOpen}
        onClose={() => setHelpOpen(false)}
        metricKey={selectedMetric}
      />
    </Box>
  );
}
