import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  Box,
  Card,
  CardContent,
  Typography,
  ToggleButtonGroup,
  ToggleButton,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Alert,
  CircularProgress,
  Chip,
  Stack,
  IconButton,
  Tooltip,
} from '@mui/material';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip as RechartsTooltip,
  Legend,
  ResponsiveContainer,
  ReferenceDot,
  ReferenceLine,
} from 'recharts';
import { Warning as WarningIcon, HelpOutline } from '@mui/icons-material';
import { getRiskHistory, getRiskAlerts } from '../lib/endpoints';
import { MetricHelpDialog } from './MetricHelpDialog';
import type { RiskSnapshot, RiskAlert, RiskThresholdSettings } from '../types';

interface RiskHistoryChartProps {
  portfolioId: string;
  ticker?: string;
  thresholds?: RiskThresholdSettings;
}

type TimeRange = 30 | 90 | 180 | 365;
type MetricType = 'risk_score' | 'volatility' | 'max_drawdown' | 'sharpe' | 'sortino' | 'annualized_return' | 'beta';

const METRIC_HELP_KEYS: Record<MetricType, string> = {
  risk_score: 'risk_score',
  volatility: 'volatility',
  max_drawdown: 'max_drawdown',
  sharpe: 'sharpe_ratio',
  sortino: 'sortino_ratio',
  annualized_return: 'annualized_return',
  beta: 'beta',
};

export function RiskHistoryChart({ portfolioId, ticker, thresholds }: RiskHistoryChartProps) {
  const [timeRange, setTimeRange] = useState<TimeRange>(90);
  const [selectedMetrics, setSelectedMetrics] = useState<MetricType[]>(['risk_score']);
  const [helpDialogOpen, setHelpDialogOpen] = useState<string | null>(null);

  // Fetch risk history
  const historyQuery = useQuery({
    queryKey: ['risk-history', portfolioId, ticker, timeRange],
    queryFn: () => getRiskHistory(portfolioId, ticker, timeRange),
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  // Fetch risk alerts
  const alertsQuery = useQuery({
    queryKey: ['risk-alerts', portfolioId, timeRange],
    queryFn: () => getRiskAlerts(portfolioId, timeRange, 20),
    enabled: !ticker, // Only fetch alerts for portfolio-level view
    staleTime: 1000 * 60 * 5,
  });

  const handleMetricToggle = (
    _event: React.MouseEvent<HTMLElement>,
    newMetrics: MetricType[]
  ) => {
    if (newMetrics.length > 0) {
      setSelectedMetrics(newMetrics);
    }
  };

  // Transform data for recharts
  // Note: Backend returns BigDecimal as strings, so we need to parse them
  const chartData = historyQuery.data?.map((snapshot: RiskSnapshot) => ({
    date: snapshot.snapshot_date,
    risk_score: typeof snapshot.risk_score === 'string' ? parseFloat(snapshot.risk_score) : snapshot.risk_score,
    volatility: typeof snapshot.volatility === 'string' ? parseFloat(snapshot.volatility) : snapshot.volatility,
    max_drawdown: Math.abs(typeof snapshot.max_drawdown === 'string' ? parseFloat(snapshot.max_drawdown) : snapshot.max_drawdown),
    sharpe: snapshot.sharpe ? (typeof snapshot.sharpe === 'string' ? parseFloat(snapshot.sharpe) : snapshot.sharpe) : 0,
    sortino: snapshot.sortino ? (typeof snapshot.sortino === 'string' ? parseFloat(snapshot.sortino) : snapshot.sortino) : 0,
    annualized_return: snapshot.annualized_return ? (typeof snapshot.annualized_return === 'string' ? parseFloat(snapshot.annualized_return) : snapshot.annualized_return) : 0,
    beta: snapshot.beta ? (typeof snapshot.beta === 'string' ? parseFloat(snapshot.beta) : snapshot.beta) : 0,
    risk_level: snapshot.risk_level,
  })) || [];

  // Create alert markers
  const alertMarkers = alertsQuery.data?.map((alert: RiskAlert) => ({
    date: alert.date,
    value: alert.current_value,
    alert,
  })) || [];

  const getMetricConfig = (metric: MetricType) => {
    const configs = {
      risk_score: {
        name: 'Risk Score',
        color: '#f44336',
        yAxisId: 'left',
      },
      volatility: {
        name: 'Volatility (%)',
        color: '#ff9800',
        yAxisId: 'left',
      },
      max_drawdown: {
        name: 'Max Drawdown (%)',
        color: '#9c27b0',
        yAxisId: 'left',
      },
      sharpe: {
        name: 'Sharpe Ratio',
        color: '#4caf50',
        yAxisId: 'right',
      },
      sortino: {
        name: 'Sortino Ratio',
        color: '#00bcd4',
        yAxisId: 'right',
      },
      annualized_return: {
        name: 'Annualized Return (%)',
        color: '#8bc34a',
        yAxisId: 'right',
      },
      beta: {
        name: 'Beta',
        color: '#2196f3',
        yAxisId: 'right',
      },
    };
    return configs[metric];
  };

  const getRiskLevelColor = (level: string) => {
    switch (level) {
      case 'low':
        return 'success';
      case 'moderate':
        return 'warning';
      case 'high':
        return 'error';
      default:
        return 'default';
    }
  };

  if (historyQuery.isLoading) {
    return (
      <Card>
        <CardContent>
          <Box display="flex" justifyContent="center" alignItems="center" minHeight={400}>
            <CircularProgress />
          </Box>
        </CardContent>
      </Card>
    );
  }

  if (historyQuery.error) {
    return (
      <Card>
        <CardContent>
          <Alert severity="error">
            Failed to load risk history: {(historyQuery.error as Error).message}
          </Alert>
        </CardContent>
      </Card>
    );
  }

  const latestSnapshot = historyQuery.data?.[historyQuery.data.length - 1];

  return (
    <Card>
      <CardContent>
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
          <Box>
            <Typography variant="h6" gutterBottom>
              Risk History {ticker ? `- ${ticker}` : '- Portfolio'}
            </Typography>
            {latestSnapshot && (
              <Box display="flex" gap={1} alignItems="center">
                <Chip
                  label={`Current Risk: ${latestSnapshot.risk_level.toUpperCase()}`}
                  color={getRiskLevelColor(latestSnapshot.risk_level)}
                  size="small"
                />
                <Typography variant="body2" color="text.secondary">
                  Score: {(typeof latestSnapshot.risk_score === 'string'
                    ? parseFloat(latestSnapshot.risk_score)
                    : latestSnapshot.risk_score).toFixed(1)}
                </Typography>
              </Box>
            )}
          </Box>

          <Box display="flex" gap={2} alignItems="center">
            <FormControl size="small" sx={{ minWidth: 120 }}>
              <InputLabel>Time Range</InputLabel>
              <Select
                value={timeRange}
                label="Time Range"
                onChange={(e) => setTimeRange(e.target.value as TimeRange)}
              >
                <MenuItem value={30}>30 Days</MenuItem>
                <MenuItem value={90}>90 Days</MenuItem>
                <MenuItem value={180}>180 Days</MenuItem>
                <MenuItem value={365}>1 Year</MenuItem>
              </Select>
            </FormControl>
          </Box>
        </Box>

        {/* Metric selection */}
        <Box mb={2}>
          <Box display="flex" alignItems="center" justifyContent="space-between" mb={1}>
            <Typography variant="body2" color="text.secondary">
              Select metrics to display (click ? for explanations):
            </Typography>
          </Box>
          <Box display="flex" flexDirection="column" gap={2}>
            {/* Row 1: Risk Score, Volatility, Max Drawdown, Beta */}
            <Box display="flex" gap={1} flexWrap="wrap" alignItems="center">
              <ToggleButtonGroup
                value={selectedMetrics}
                onChange={handleMetricToggle}
                size="small"
                aria-label="metric selection"
              >
                <ToggleButton value="risk_score" aria-label="risk score">
                  <Box display="flex" alignItems="center" gap={0.5}>
                    Risk Score
                    <IconButton
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        setHelpDialogOpen(METRIC_HELP_KEYS['risk_score']);
                      }}
                      sx={{
                        width: 18,
                        height: 18,
                        p: 0,
                        color: 'inherit',
                        opacity: 0.6,
                        '&:hover': {
                          opacity: 1,
                          backgroundColor: 'transparent',
                        },
                      }}
                    >
                      <HelpOutline sx={{ fontSize: 14 }} />
                    </IconButton>
                  </Box>
                </ToggleButton>
                <ToggleButton value="volatility" aria-label="volatility">
                  <Box display="flex" alignItems="center" gap={0.5}>
                    Volatility
                    <IconButton
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        setHelpDialogOpen(METRIC_HELP_KEYS['volatility']);
                      }}
                      sx={{
                        width: 18,
                        height: 18,
                        p: 0,
                        color: 'inherit',
                        opacity: 0.6,
                        '&:hover': {
                          opacity: 1,
                          backgroundColor: 'transparent',
                        },
                      }}
                    >
                      <HelpOutline sx={{ fontSize: 14 }} />
                    </IconButton>
                  </Box>
                </ToggleButton>
                <ToggleButton value="max_drawdown" aria-label="max drawdown">
                  <Box display="flex" alignItems="center" gap={0.5}>
                    Max Drawdown
                    <IconButton
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        setHelpDialogOpen(METRIC_HELP_KEYS['max_drawdown']);
                      }}
                      sx={{
                        width: 18,
                        height: 18,
                        p: 0,
                        color: 'inherit',
                        opacity: 0.6,
                        '&:hover': {
                          opacity: 1,
                          backgroundColor: 'transparent',
                        },
                      }}
                    >
                      <HelpOutline sx={{ fontSize: 14 }} />
                    </IconButton>
                  </Box>
                </ToggleButton>
                <ToggleButton value="beta" aria-label="beta">
                  <Box display="flex" alignItems="center" gap={0.5}>
                    Beta
                    <IconButton
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        setHelpDialogOpen(METRIC_HELP_KEYS['beta']);
                      }}
                      sx={{
                        width: 18,
                        height: 18,
                        p: 0,
                        color: 'inherit',
                        opacity: 0.6,
                        '&:hover': {
                          opacity: 1,
                          backgroundColor: 'transparent',
                        },
                      }}
                    >
                      <HelpOutline sx={{ fontSize: 14 }} />
                    </IconButton>
                  </Box>
                </ToggleButton>
              </ToggleButtonGroup>
            </Box>

            {/* Row 2: Sharpe, Sortino, Ann. Return */}
            <Box display="flex" gap={1} flexWrap="wrap" alignItems="center">
              <ToggleButtonGroup
                value={selectedMetrics}
                onChange={handleMetricToggle}
                size="small"
                aria-label="metric selection"
              >
                <ToggleButton value="sharpe" aria-label="sharpe ratio">
                  <Box display="flex" alignItems="center" gap={0.5}>
                    Sharpe Ratio
                    <IconButton
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        setHelpDialogOpen(METRIC_HELP_KEYS['sharpe']);
                      }}
                      sx={{
                        width: 18,
                        height: 18,
                        p: 0,
                        color: 'inherit',
                        opacity: 0.6,
                        '&:hover': {
                          opacity: 1,
                          backgroundColor: 'transparent',
                        },
                      }}
                    >
                      <HelpOutline sx={{ fontSize: 14 }} />
                    </IconButton>
                  </Box>
                </ToggleButton>
                <ToggleButton value="sortino" aria-label="sortino ratio">
                  <Box display="flex" alignItems="center" gap={0.5}>
                    Sortino Ratio
                    <IconButton
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        setHelpDialogOpen(METRIC_HELP_KEYS['sortino']);
                      }}
                      sx={{
                        width: 18,
                        height: 18,
                        p: 0,
                        color: 'inherit',
                        opacity: 0.6,
                        '&:hover': {
                          opacity: 1,
                          backgroundColor: 'transparent',
                        },
                      }}
                    >
                      <HelpOutline sx={{ fontSize: 14 }} />
                    </IconButton>
                  </Box>
                </ToggleButton>
                <ToggleButton value="annualized_return" aria-label="annualized return">
                  <Box display="flex" alignItems="center" gap={0.5}>
                    Ann. Return
                    <IconButton
                      size="small"
                      onClick={(e) => {
                        e.stopPropagation();
                        setHelpDialogOpen(METRIC_HELP_KEYS['annualized_return']);
                      }}
                      sx={{
                        width: 18,
                        height: 18,
                        p: 0,
                        color: 'inherit',
                        opacity: 0.6,
                        '&:hover': {
                          opacity: 1,
                          backgroundColor: 'transparent',
                        },
                      }}
                    >
                      <HelpOutline sx={{ fontSize: 14 }} />
                    </IconButton>
                  </Box>
                </ToggleButton>
              </ToggleButtonGroup>
            </Box>
          </Box>
        </Box>

        {/* Help Dialog */}
        {helpDialogOpen && (
          <MetricHelpDialog
            open={true}
            onClose={() => setHelpDialogOpen(null)}
            metricKey={helpDialogOpen}
          />
        )}

        {/* Alerts */}
        {!ticker && alertsQuery.data && alertsQuery.data.length > 0 && (
          <Alert severity="warning" icon={<WarningIcon />} sx={{ mb: 2 }}>
            <Typography variant="body2" fontWeight="bold">
              {alertsQuery.data.length} Risk Alert{alertsQuery.data.length > 1 ? 's' : ''} Detected
            </Typography>
            <Stack spacing={0.5} mt={1}>
              {alertsQuery.data.slice(0, 3).map((alert: RiskAlert) => (
                <Typography key={alert.date} variant="caption">
                  {alert.date}: {alert.metric_name} increased by {alert.change_percent.toFixed(1)}%
                  ({alert.previous_value.toFixed(1)} → {alert.current_value.toFixed(1)})
                </Typography>
              ))}
              {alertsQuery.data.length > 3 && (
                <Typography variant="caption" color="text.secondary">
                  ... and {alertsQuery.data.length - 3} more
                </Typography>
              )}
            </Stack>
          </Alert>
        )}

        {/* Chart */}
        {chartData.length === 0 ? (
          <Alert severity="info">
            No risk history data available. Create a snapshot to start tracking risk over time.
          </Alert>
        ) : (
          <Box sx={{ minHeight: 400 }}>
            <ResponsiveContainer width="100%" height={400}>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis
                dataKey="date"
                tickFormatter={(date) => new Date(date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
              />
              <YAxis yAxisId="left" />
              {(selectedMetrics.includes('sharpe') || selectedMetrics.includes('sortino') || selectedMetrics.includes('annualized_return') || selectedMetrics.includes('beta')) && (
                <YAxis yAxisId="right" orientation="right" />
              )}
              <Tooltip
                labelFormatter={(date) => new Date(date).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'long',
                  day: 'numeric'
                })}
                formatter={(value: number | undefined, name: string | undefined) => {
                  if (value === undefined) return '';
                  if (name === 'Sharpe Ratio' || name === 'Sortino Ratio' || name === 'Beta') {
                    return value.toFixed(2);
                  }
                  return value.toFixed(1);
                }}
              />
              <Legend />

              {selectedMetrics.map((metric) => {
                const config = getMetricConfig(metric);
                return (
                  <Line
                    key={metric}
                    type="monotone"
                    dataKey={metric}
                    name={config.name}
                    stroke={config.color}
                    yAxisId={config.yAxisId}
                    strokeWidth={2}
                    dot={false}
                    activeDot={{ r: 6 }}
                  />
                );
              })}

              {/* Alert markers */}
              {!ticker && selectedMetrics.includes('risk_score') && alertMarkers.map((marker, idx) => (
                <ReferenceDot
                  key={`alert-${idx}`}
                  x={marker.date}
                  y={marker.value}
                  r={8}
                  fill="#f44336"
                  stroke="#fff"
                  strokeWidth={2}
                  yAxisId="left"
                />
              ))}

              {/* Threshold reference lines */}
              {thresholds && (
                <>
                  {/* Risk Score thresholds */}
                  {selectedMetrics.includes('risk_score') && (
                    <>
                      <ReferenceLine
                        y={thresholds.risk_score_warning_threshold}
                        stroke="#ff9800"
                        strokeDasharray="5 5"
                        strokeWidth={2}
                        yAxisId="left"
                        label={{ value: 'Warning', position: 'insideTopRight', fill: '#ff9800', fontSize: 12 }}
                      />
                      <ReferenceLine
                        y={thresholds.risk_score_critical_threshold}
                        stroke="#f44336"
                        strokeDasharray="5 5"
                        strokeWidth={2}
                        yAxisId="left"
                        label={{ value: 'Critical', position: 'insideTopRight', fill: '#f44336', fontSize: 12 }}
                      />
                    </>
                  )}

                  {/* Volatility thresholds */}
                  {selectedMetrics.includes('volatility') && (
                    <>
                      <ReferenceLine
                        y={thresholds.volatility_warning_threshold}
                        stroke="#ff9800"
                        strokeDasharray="5 5"
                        strokeWidth={2}
                        yAxisId="left"
                      />
                      <ReferenceLine
                        y={thresholds.volatility_critical_threshold}
                        stroke="#f44336"
                        strokeDasharray="5 5"
                        strokeWidth={2}
                        yAxisId="left"
                      />
                    </>
                  )}

                  {/* Max Drawdown thresholds (negative values) */}
                  {selectedMetrics.includes('max_drawdown') && (
                    <>
                      <ReferenceLine
                        y={Math.abs(thresholds.drawdown_warning_threshold)}
                        stroke="#ff9800"
                        strokeDasharray="5 5"
                        strokeWidth={2}
                        yAxisId="left"
                      />
                      <ReferenceLine
                        y={Math.abs(thresholds.drawdown_critical_threshold)}
                        stroke="#f44336"
                        strokeDasharray="5 5"
                        strokeWidth={2}
                        yAxisId="left"
                      />
                    </>
                  )}

                  {/* Beta thresholds */}
                  {selectedMetrics.includes('beta') && (
                    <>
                      <ReferenceLine
                        y={thresholds.beta_warning_threshold}
                        stroke="#ff9800"
                        strokeDasharray="5 5"
                        strokeWidth={2}
                        yAxisId="right"
                      />
                      <ReferenceLine
                        y={thresholds.beta_critical_threshold}
                        stroke="#f44336"
                        strokeDasharray="5 5"
                        strokeWidth={2}
                        yAxisId="right"
                      />
                    </>
                  )}
                </>
              )}
            </LineChart>
          </ResponsiveContainer>
          </Box>
        )}

        {chartData.length > 0 && (
          <Box mt={2}>
            <Typography variant="caption" color="text.secondary">
              Showing {chartData.length} data points from the last {timeRange} days
              {!ticker && alertMarkers.length > 0 && (
                <span> • Red dots indicate risk alerts</span>
              )}
              {thresholds && (
                <span> • Dashed lines show warning (orange) and critical (red) thresholds</span>
              )}
            </Typography>
          </Box>
        )}
      </CardContent>
    </Card>
  );
}
