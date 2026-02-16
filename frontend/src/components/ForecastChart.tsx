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
  Tooltip as MuiTooltip,
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
  Area,
  ComposedChart,
  ReferenceLine,
} from 'recharts';
import { Info as InfoIcon } from '@mui/icons-material';
import { getPortfolioForecast } from '../lib/endpoints';
import type { PortfolioForecast, ForecastMethod } from '../types';

interface ForecastChartProps {
  portfolioId: string;
}

type TimeRange = 30 | 90 | 365 | 1825 | 3650 | 5475 | 7300; // 30d, 90d, 1y, 5y, 10y, 15y, 20y

export function ForecastChart({ portfolioId }: ForecastChartProps) {
  const [timeRange, setTimeRange] = useState<TimeRange>(365);
  const [method, setMethod] = useState<ForecastMethod>('ensemble');

  // Fetch forecast data
  const forecastQuery = useQuery({
    queryKey: ['portfolio-forecast', portfolioId, timeRange, method],
    queryFn: () => getPortfolioForecast(portfolioId, timeRange, method),
    staleTime: 1000 * 60 * 60, // 1 hour
    retry: 2,
  });

  const handleMethodChange = (
    _event: React.MouseEvent<HTMLElement>,
    newMethod: ForecastMethod | null
  ) => {
    if (newMethod !== null) {
      setMethod(newMethod);
    }
  };

  const getMethodDescription = (m: ForecastMethod): string => {
    const descriptions: Record<ForecastMethod, string> = {
      linear_regression: 'Linear trend extrapolation based on historical performance',
      exponential_smoothing: 'Exponential smoothing with trend (Holt\'s method)',
      moving_average: 'Simple moving average projection with adaptive window',
      ensemble: 'Weighted average of multiple forecasting methods',
    };
    return descriptions[m];
  };

  const formatCurrency = (value: number): string => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 0,
      maximumFractionDigits: 0,
    }).format(value);
  };

  if (forecastQuery.isLoading) {
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

  if (forecastQuery.error) {
    return (
      <Card>
        <CardContent>
          <Alert severity="error">
            Failed to generate forecast: {(forecastQuery.error as Error).message}
          </Alert>
        </CardContent>
      </Card>
    );
  }

  const forecast: PortfolioForecast | undefined = forecastQuery.data;

  if (!forecast) {
    return (
      <Card>
        <CardContent>
          <Alert severity="info">No forecast data available.</Alert>
        </CardContent>
      </Card>
    );
  }

  // Get the date of the last historical data point (current value date)
  // The first forecast point is for tomorrow, so we need to add today's current value
  const firstForecastDate = forecast.forecast_points[0]?.date;
  const currentDate = firstForecastDate ? (() => {
    const [year, month, day] = firstForecastDate.split('-').map(Number);
    const date = new Date(year, month - 1, day);
    date.setDate(date.getDate() - 1); // Go back one day to get "today"
    const y = date.getFullYear();
    const m = String(date.getMonth() + 1).padStart(2, '0');
    const d = String(date.getDate()).padStart(2, '0');
    return `${y}-${m}-${d}`;
  })() : null;

  // Add current value as the starting point, then add all forecast points
  const chartData = [
    // Add current value as the first point (today)
    ...(currentDate ? [{
      date: currentDate,
      predicted_value: forecast.current_value,
      lower_bound: forecast.current_value,
      upper_bound: forecast.current_value,
      confidence_range: [forecast.current_value, forecast.current_value],
      isCurrent: true, // Flag to identify this point
    }] : []),
    // Add all forecast points
    ...forecast.forecast_points.map((point) => ({
      date: point.date,
      predicted_value: point.predicted_value,
      lower_bound: point.lower_bound,
      upper_bound: point.upper_bound,
      confidence_range: [point.lower_bound, point.upper_bound],
      isCurrent: false,
    }))
  ];

  // Calculate forecast change
  const firstPoint = forecast.forecast_points[0];
  const lastPoint = forecast.forecast_points[forecast.forecast_points.length - 1];
  const forecastChange = lastPoint
    ? ((lastPoint.predicted_value - forecast.current_value) / forecast.current_value) * 100
    : 0;
  const forecastChangeAbsolute = lastPoint ? lastPoint.predicted_value - forecast.current_value : 0;

  return (
    <Card>
      <CardContent>
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
          <Box>
            <Box display="flex" alignItems="center" gap={1}>
              <Typography variant="h6">
                Portfolio Value Forecast
              </Typography>
              <MuiTooltip title="Statistical projections based on historical data. Not a guarantee of future performance.">
                <IconButton size="small">
                  <InfoIcon fontSize="small" />
                </IconButton>
              </MuiTooltip>
            </Box>
            <Box display="flex" gap={1} mt={1} alignItems="center">
              <Chip
                label={`Current: ${formatCurrency(forecast.current_value)}`}
                color="primary"
                size="small"
              />
              {lastPoint && (
                <Chip
                  label={`${timeRange >= 365 ? `${(timeRange / 365).toFixed(0)}y` : `${timeRange}d`} Forecast: ${formatCurrency(lastPoint.predicted_value)} (${forecastChange >= 0 ? '+' : ''}${forecastChange.toFixed(1)}%)`}
                  color={forecastChange >= 0 ? 'success' : 'error'}
                  size="small"
                />
              )}
            </Box>
          </Box>

          <Box display="flex" gap={2} alignItems="center">
            <FormControl size="small" sx={{ minWidth: 140 }}>
              <InputLabel>Time Range</InputLabel>
              <Select
                value={timeRange}
                label="Time Range"
                onChange={(e) => setTimeRange(e.target.value as TimeRange)}
              >
                <MenuItem value={30}>1 Month</MenuItem>
                <MenuItem value={90}>3 Months</MenuItem>
                <MenuItem value={365}>1 Year</MenuItem>
                <MenuItem value={1825}>5 Years</MenuItem>
                <MenuItem value={3650}>10 Years</MenuItem>
                <MenuItem value={5475}>15 Years</MenuItem>
                <MenuItem value={7300}>20 Years</MenuItem>
              </Select>
            </FormControl>
          </Box>
        </Box>

        {/* Method selection */}
        <Box mb={2}>
          <Typography variant="body2" color="text.secondary" gutterBottom>
            Forecasting Method:
          </Typography>
          <ToggleButtonGroup
            value={method}
            exclusive
            onChange={handleMethodChange}
            size="small"
            aria-label="forecast method"
          >
            <ToggleButton value="ensemble" aria-label="ensemble">
              Ensemble
            </ToggleButton>
            <ToggleButton value="linear_regression" aria-label="linear">
              Linear
            </ToggleButton>
            <ToggleButton value="exponential_smoothing" aria-label="exponential">
              Exponential
            </ToggleButton>
            <ToggleButton value="moving_average" aria-label="moving average">
              Moving Avg
            </ToggleButton>
          </ToggleButtonGroup>
          <Typography variant="caption" color="text.secondary" display="block" mt={0.5}>
            {getMethodDescription(method)}
          </Typography>
        </Box>

        {/* Warnings */}
        {forecast.warnings && forecast.warnings.length > 0 && (
          <Alert severity="warning" sx={{ mb: 2 }}>
            <Typography variant="body2" fontWeight="bold" gutterBottom>
              Important Considerations:
            </Typography>
            <Stack spacing={0.5}>
              {forecast.warnings.map((warning, idx) => (
                <Typography key={idx} variant="caption">
                  • {warning}
                </Typography>
              ))}
            </Stack>
          </Alert>
        )}

        {/* Chart */}
        <Box sx={{ minHeight: 400 }}>
          <ResponsiveContainer width="100%" height={400}>
            <ComposedChart data={chartData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis
                dataKey="date"
                tickFormatter={(date) => {
                  // Parse as local date to avoid timezone shifts
                  const [year, month, day] = date.split('-').map(Number);
                  return new Date(year, month - 1, day).toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
                }}
              />
              <YAxis
                tickFormatter={(value) => `$${(value / 1000).toFixed(0)}k`}
              />
              <Tooltip
                labelFormatter={(date) => {
                  // Parse as local date to avoid timezone shifts
                  const [year, month, day] = date.split('-').map(Number);
                  return new Date(year, month - 1, day).toLocaleDateString('en-US', {
                    year: 'numeric',
                    month: 'long',
                    day: 'numeric'
                  });
                }}
                formatter={(value: number, name: string) => {
                  if (name === 'Predicted Value') {
                    return [formatCurrency(value), name];
                  }
                  if (name === 'Confidence Range') {
                    const range = value as unknown as [number, number];
                    return [`${formatCurrency(range[0])} - ${formatCurrency(range[1])}`, name];
                  }
                  return [formatCurrency(value), name];
                }}
              />
              <Legend />

              {/* Confidence interval area */}
              <Area
                type="monotone"
                dataKey="lower_bound"
                stackId="1"
                stroke="none"
                fill="transparent"
              />
              <Area
                type="monotone"
                dataKey="upper_bound"
                stackId="1"
                stroke="none"
                fill="#2196f3"
                fillOpacity={0.2}
                name="95% Confidence Band"
              />

              {/* Predicted value line */}
              <Line
                type="monotone"
                dataKey="predicted_value"
                name="Predicted Value"
                stroke="#2196f3"
                strokeWidth={3}
                dot={false}
                activeDot={{ r: 6 }}
              />

              {/* Reference line at current value */}
              <ReferenceLine
                y={forecast.current_value}
                stroke="#4caf50"
                strokeDasharray="5 5"
                strokeWidth={2}
                label={{
                  value: 'Current',
                  position: 'insideTopLeft',
                  fill: '#4caf50',
                  fontSize: 12
                }}
              />
            </ComposedChart>
          </ResponsiveContainer>
        </Box>

        {/* Summary */}
        <Box mt={2}>
          <Typography variant="caption" color="text.secondary">
            Showing {forecast.forecast_points.length} forecast points over {timeRange >= 365 ? `${(timeRange / 365).toFixed(1)} years` : `${timeRange} days`}
            {' • '}
            {forecast.confidence_level * 100}% confidence interval
            {' • '}
            Methodology: {forecast.methodology.replace(/_/g, ' ')}
            {lastPoint && forecastChangeAbsolute !== 0 && (
              <>
                {' • '}
                Projected {forecastChange >= 0 ? 'gain' : 'loss'}: {formatCurrency(Math.abs(forecastChangeAbsolute))}
              </>
            )}
          </Typography>
        </Box>

        {/* Disclaimer */}
        <Alert severity={timeRange >= 1825 ? 'warning' : 'info'} sx={{ mt: 2 }}>
          <Typography variant="caption">
            <strong>Disclaimer:</strong> This forecast is a statistical projection based on historical data and is not a guarantee of future performance.
            Actual results may vary significantly due to market conditions, economic factors, and other variables not captured by the model.
            {timeRange >= 1825 && (
              <>
                {' '}
                <strong>Long-term forecasts (5+ years) are highly speculative and should be used for general planning purposes only.</strong>
              </>
            )}
          </Typography>
        </Alert>
      </CardContent>
    </Card>
  );
}
