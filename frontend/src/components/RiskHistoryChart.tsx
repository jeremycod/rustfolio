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
  ReferenceDot,
} from 'recharts';
import { Warning as WarningIcon } from '@mui/icons-material';
import { getRiskHistory, getRiskAlerts } from '../lib/endpoints';
import type { RiskSnapshot, RiskAlert } from '../types';

interface RiskHistoryChartProps {
  portfolioId: string;
  ticker?: string;
}

type TimeRange = 30 | 90 | 180 | 365;
type MetricType = 'risk_score' | 'volatility' | 'max_drawdown' | 'sharpe' | 'beta';

export function RiskHistoryChart({ portfolioId, ticker }: RiskHistoryChartProps) {
  const [timeRange, setTimeRange] = useState<TimeRange>(90);
  const [selectedMetrics, setSelectedMetrics] = useState<MetricType[]>(['risk_score']);

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
  const chartData = historyQuery.data?.map((snapshot: RiskSnapshot) => ({
    date: snapshot.snapshot_date,
    risk_score: snapshot.risk_score,
    volatility: snapshot.volatility,
    max_drawdown: Math.abs(snapshot.max_drawdown), // Show as positive for readability
    sharpe: snapshot.sharpe || 0,
    beta: snapshot.beta || 0,
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
                  Score: {latestSnapshot.risk_score.toFixed(1)}
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
          <Typography variant="body2" color="text.secondary" gutterBottom>
            Select metrics to display:
          </Typography>
          <ToggleButtonGroup
            value={selectedMetrics}
            onChange={handleMetricToggle}
            size="small"
            aria-label="metric selection"
          >
            <ToggleButton value="risk_score" aria-label="risk score">
              Risk Score
            </ToggleButton>
            <ToggleButton value="volatility" aria-label="volatility">
              Volatility
            </ToggleButton>
            <ToggleButton value="max_drawdown" aria-label="max drawdown">
              Max Drawdown
            </ToggleButton>
            <ToggleButton value="sharpe" aria-label="sharpe ratio">
              Sharpe
            </ToggleButton>
            <ToggleButton value="beta" aria-label="beta">
              Beta
            </ToggleButton>
          </ToggleButtonGroup>
        </Box>

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
          <ResponsiveContainer width="100%" height={400}>
            <LineChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis
                dataKey="date"
                tickFormatter={(date) => new Date(date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
              />
              <YAxis yAxisId="left" />
              {(selectedMetrics.includes('sharpe') || selectedMetrics.includes('beta')) && (
                <YAxis yAxisId="right" orientation="right" />
              )}
              <Tooltip
                labelFormatter={(date) => new Date(date).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'long',
                  day: 'numeric'
                })}
                formatter={(value: number, name: string) => {
                  if (name === 'Sharpe Ratio' || name === 'Beta') {
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
            </LineChart>
          </ResponsiveContainer>
        )}

        {chartData.length > 0 && (
          <Box mt={2}>
            <Typography variant="caption" color="text.secondary">
              Showing {chartData.length} data points from the last {timeRange} days
              {!ticker && alertMarkers.length > 0 && (
                <span> • Red dots indicate risk alerts</span>
              )}
            </Typography>
          </Box>
        )}
      </CardContent>
    </Card>
  );
}
