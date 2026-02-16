import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
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
  Paper,
} from '@mui/material';
import {
  ComposedChart,
  Line,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
  ReferenceLine,
  Scatter,
} from 'recharts';
import {
  TrendingUp,
  ShowChart,
  Warning,
  Science,
} from '@mui/icons-material';
import { getBetaForecast } from '../lib/endpoints';
import type { BetaForecast, ForecastMethod } from '../types';

interface BetaForecastChartProps {
  ticker: string;
  benchmark?: string;
  currentBeta?: number;
}

type ForecastHorizon = 30 | 60 | 90;

export function BetaForecastChart({
  ticker,
  benchmark = 'SPY',
  currentBeta = 1.0
}: BetaForecastChartProps) {
  const [forecastDays, setForecastDays] = useState<ForecastHorizon>(30);
  const [method, setMethod] = useState<ForecastMethod>('ensemble');

  // Fetch beta forecast data
  const forecastQuery = useQuery({
    queryKey: ['beta-forecast', ticker, benchmark, forecastDays, method],
    queryFn: () => getBetaForecast(ticker, forecastDays, benchmark, method),
    staleTime: 1000 * 60 * 60, // 1 hour
    retry: 1,
  });

  const handleHorizonChange = (
    _event: React.MouseEvent<HTMLElement>,
    newHorizon: ForecastHorizon | null
  ) => {
    if (newHorizon) {
      setForecastDays(newHorizon);
    }
  };

  const handleMethodChange = (
    _event: React.MouseEvent<HTMLElement>,
    newMethod: ForecastMethod | null
  ) => {
    if (newMethod) {
      setMethod(newMethod);
    }
  };

  if (forecastQuery.isLoading) {
    return (
      <Box display="flex" flexDirection="column" justifyContent="center" alignItems="center" minHeight="400px" gap={2}>
        <CircularProgress />
        <Typography variant="body2" color="text.secondary">
          Generating beta forecast...
        </Typography>
      </Box>
    );
  }

  if (forecastQuery.isError) {
    const error = forecastQuery.error as any;
    const errorMessage =
      error?.response?.data?.error ||
      error?.response?.data ||
      error?.message ||
      'Unknown error';

    return (
      <Card elevation={2}>
        <CardContent>
          <Alert severity="error">
            <Typography variant="body1" gutterBottom>
              <strong>Error generating beta forecast</strong>
            </Typography>
            <Typography variant="body2">
              {errorMessage}
            </Typography>
            {errorMessage.includes('Insufficient') && (
              <Typography variant="body2" sx={{ mt: 1 }}>
                Beta forecasting requires at least 60 days of historical rolling beta data.
                Please ensure the ticker has sufficient price history.
              </Typography>
            )}
          </Alert>
        </CardContent>
      </Card>
    );
  }

  if (!forecastQuery.data) {
    return (
      <Alert severity="info">
        No forecast data available.
      </Alert>
    );
  }

  const forecast: BetaForecast = forecastQuery.data;

  // Transform data for recharts
  const chartData = forecast.forecast_points.map((point) => ({
    date: new Date(point.date).toLocaleDateString(),
    predicted_beta: point.predicted_beta,
    lower_bound: point.lower_bound,
    upper_bound: point.upper_bound,
    confidence_band: [point.lower_bound, point.upper_bound],
  }));

  // Add regime change markers
  const regimeMarkers = forecast.regime_changes
    .filter(rc => {
      const rcDate = new Date(rc.date);
      const lastForecastDate = new Date(forecast.forecast_points[forecast.forecast_points.length - 1]?.date || '');
      return rcDate <= lastForecastDate;
    })
    .map(rc => ({
      date: new Date(rc.date).toLocaleDateString(),
      predicted_beta: rc.beta_after,
      regime_type: rc.regime_type,
      z_score: rc.z_score,
    }));

  const getMethodLabel = (m: ForecastMethod): string => {
    switch (m) {
      case 'mean_reversion': return 'Mean Reversion';
      case 'exponential_smoothing': return 'Exp Smoothing';
      case 'linear_regression': return 'Linear Trend';
      case 'ensemble': return 'Ensemble';
      default: return m;
    }
  };

  const getBetaInterpretation = (beta: number): string => {
    if (beta > 1.2) return 'Highly volatile forecast';
    if (beta > 1.0) return 'More volatile than market';
    if (beta > 0.8) return 'Near market volatility';
    if (beta > 0.5) return 'Lower volatility';
    return 'Low market correlation';
  };

  const getBetaColor = (beta: number): string => {
    const absBeta = Math.abs(beta);
    if (absBeta > 1.2) return 'error';
    if (absBeta > 1.0) return 'warning';
    return 'success';
  };

  const finalBeta = forecast.forecast_points[forecast.forecast_points.length - 1]?.predicted_beta || forecast.current_beta;

  return (
    <Card elevation={2}>
      <CardContent>
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={2} flexWrap="wrap" gap={2}>
          <Box>
            <Typography variant="h6" gutterBottom>
              Beta Forecast: {ticker} vs {benchmark}
            </Typography>
            <Typography variant="caption" color="text.secondary">
              {getBetaInterpretation(finalBeta)}
            </Typography>
          </Box>

          <Stack direction="row" spacing={1} alignItems="center" flexWrap="wrap">
            <MuiTooltip title="Current beta">
              <Chip
                icon={<TrendingUp />}
                label={`Current: ${forecast.current_beta.toFixed(2)}`}
                color={getBetaColor(forecast.current_beta) as any}
                size="small"
              />
            </MuiTooltip>

            <MuiTooltip title={`Predicted beta at end of ${forecastDays}-day period`}>
              <Chip
                icon={<Science />}
                label={`Forecast: ${finalBeta.toFixed(2)}`}
                color="primary"
                size="small"
              />
            </MuiTooltip>

            <MuiTooltip title="Beta volatility">
              <Chip
                icon={<ShowChart />}
                label={`σ: ${forecast.beta_volatility.toFixed(2)}`}
                color="default"
                size="small"
              />
            </MuiTooltip>
          </Stack>
        </Box>

        {/* Controls */}
        <Box mb={2} display="flex" gap={2} flexWrap="wrap">
          <Box>
            <Typography variant="caption" color="text.secondary" display="block" mb={0.5}>
              Forecast Horizon
            </Typography>
            <ToggleButtonGroup
              value={forecastDays}
              exclusive
              onChange={handleHorizonChange}
              size="small"
            >
              <ToggleButton value={30}>30 Days</ToggleButton>
              <ToggleButton value={60}>60 Days</ToggleButton>
              <ToggleButton value={90}>90 Days</ToggleButton>
            </ToggleButtonGroup>
          </Box>

          <Box>
            <Typography variant="caption" color="text.secondary" display="block" mb={0.5}>
              Method
            </Typography>
            <ToggleButtonGroup
              value={method}
              exclusive
              onChange={handleMethodChange}
              size="small"
            >
              <ToggleButton value="ensemble">Ensemble</ToggleButton>
              <ToggleButton value="mean_reversion">Mean Reversion</ToggleButton>
              <ToggleButton value="exponential_smoothing">Exp Smoothing</ToggleButton>
            </ToggleButtonGroup>
          </Box>
        </Box>

        <Alert severity="info" sx={{ mb: 2 }}>
          <Typography variant="body2">
            <strong>{getMethodLabel(forecast.methodology)}</strong> forecast predicts beta will move from{' '}
            <strong>{forecast.current_beta.toFixed(2)}</strong> to approximately{' '}
            <strong>{finalBeta.toFixed(2)}</strong> over the next {forecastDays} days.
            {forecast.methodology === 'ensemble' && ' (Combined 60% mean reversion + 30% exponential smoothing + 10% linear)'}
          </Typography>
        </Alert>

        {/* Warnings */}
        {forecast.warnings && forecast.warnings.length > 0 && (
          <Alert severity="warning" sx={{ mb: 2 }} icon={<Warning />}>
            <Typography variant="body2" gutterBottom>
              <strong>Forecast Caveats:</strong>
            </Typography>
            {forecast.warnings.map((warning, idx) => (
              <Typography key={idx} variant="body2" component="div">
                • {warning}
              </Typography>
            ))}
          </Alert>
        )}

        {/* Chart */}
        {chartData.length > 0 ? (
          <ResponsiveContainer width="100%" height={400}>
            <ComposedChart data={chartData} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis
                dataKey="date"
                angle={-45}
                textAnchor="end"
                height={80}
                tick={{ fontSize: 12 }}
              />
              <YAxis
                label={{ value: 'Beta', angle: -90, position: 'insideLeft' }}
                domain={['auto', 'auto']}
              />
              <Tooltip
                content={({ active, payload }) => {
                  if (active && payload && payload.length) {
                    const data = payload[0].payload;
                    return (
                      <Paper
                        sx={{
                          p: 2,
                          border: '1px solid #ccc',
                        }}
                      >
                        <Typography variant="body2">
                          <strong>Date:</strong> {data.date}
                        </Typography>
                        <Typography variant="body2" color="primary">
                          <strong>Predicted Beta:</strong> {data.predicted_beta?.toFixed(3)}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          <strong>95% CI:</strong> [{data.lower_bound?.toFixed(3)}, {data.upper_bound?.toFixed(3)}]
                        </Typography>
                      </Paper>
                    );
                  }
                  return null;
                }}
              />
              <Legend />

              {/* Beta = 1.0 reference line */}
              <ReferenceLine
                y={1}
                stroke="#666"
                strokeDasharray="3 3"
                label={{ value: 'Market Beta (1.0)', position: 'right', fontSize: 12 }}
              />

              {/* Confidence interval */}
              <Area
                type="monotone"
                dataKey="confidence_band"
                fill="#1976d2"
                fillOpacity={0.15}
                stroke="none"
                name="95% Confidence Interval"
              />

              {/* Predicted beta line */}
              <Line
                type="monotone"
                dataKey="predicted_beta"
                stroke="#1976d2"
                strokeWidth={2}
                dot={false}
                name="Predicted Beta"
              />

              {/* Regime change markers */}
              {regimeMarkers.length > 0 && (
                <Scatter
                  data={regimeMarkers}
                  fill="#f44336"
                  shape="circle"
                  name="Regime Changes"
                />
              )}
            </ComposedChart>
          </ResponsiveContainer>
        ) : (
          <Alert severity="warning">
            Insufficient data to generate forecast.
          </Alert>
        )}

        {/* Regime Changes Info */}
        {forecast.regime_changes && forecast.regime_changes.length > 0 && (
          <Paper elevation={0} sx={{ p: 2, bgcolor: 'background.default', mt: 2 }}>
            <Typography variant="subtitle2" gutterBottom>
              Detected Regime Changes ({forecast.regime_changes.length})
            </Typography>
            <Stack spacing={1}>
              {forecast.regime_changes.slice(0, 3).map((change, idx) => (
                <Typography key={idx} variant="body2" color="text.secondary">
                  • <strong>{new Date(change.date).toLocaleDateString()}</strong>: Beta changed from{' '}
                  {change.beta_before.toFixed(2)} to {change.beta_after.toFixed(2)}{' '}
                  (z-score: {change.z_score.toFixed(2)}, type: {change.regime_type})
                </Typography>
              ))}
              {forecast.regime_changes.length > 3 && (
                <Typography variant="caption" color="text.secondary">
                  ... and {forecast.regime_changes.length - 3} more
                </Typography>
              )}
            </Stack>
          </Paper>
        )}

        {/* Methodology Info */}
        <Paper elevation={0} sx={{ p: 2, bgcolor: 'background.default', mt: 2 }}>
          <Typography variant="caption" color="text.secondary">
            <strong>Methodology:</strong> {getMethodLabel(forecast.methodology)} |{' '}
            <strong>Confidence:</strong> {(forecast.confidence_level * 100).toFixed(0)}% |{' '}
            <strong>Generated:</strong> {new Date(forecast.generated_at).toLocaleString()}
          </Typography>
        </Paper>
      </CardContent>
    </Card>
  );
}
