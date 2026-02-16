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
import { TrendingUp, ShowChart } from '@mui/icons-material';
import { getRollingBeta } from '../lib/endpoints';
import type { RollingBetaAnalysis, BetaPoint } from '../types';

interface RollingBetaChartProps {
  ticker: string;
  benchmark?: string;
  days?: number;
}

type WindowSize = '30d' | '60d' | '90d';

export function RollingBetaChart({ ticker, benchmark = 'SPY', days = 180 }: RollingBetaChartProps) {
  const [selectedWindow, setSelectedWindow] = useState<WindowSize>('90d');

  // Fetch rolling beta data
  const rollingBetaQuery = useQuery({
    queryKey: ['rolling-beta', ticker, days, benchmark],
    queryFn: () => getRollingBeta(ticker, days, benchmark),
    staleTime: 1000 * 60 * 60, // 1 hour
    retry: 1,
  });

  const handleWindowToggle = (
    _event: React.MouseEvent<HTMLElement>,
    newWindow: WindowSize | null
  ) => {
    if (newWindow) {
      setSelectedWindow(newWindow);
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

  if (rollingBetaQuery.error) {
    const errorMessage = (rollingBetaQuery.error as any)?.response?.data?.error
      || (rollingBetaQuery.error as Error).message
      || 'Unknown error';

    return (
      <Alert severity="error">
        Failed to load rolling beta analysis: {errorMessage}
      </Alert>
    );
  }

  if (!rollingBetaQuery.data) {
    return (
      <Alert severity="info">
        No rolling beta data available. Need at least 90 days of price history for {ticker} and {benchmark}.
      </Alert>
    );
  }

  const analysis: RollingBetaAnalysis = rollingBetaQuery.data;

  // Select data based on window
  const selectedData: BetaPoint[] = {
    '30d': analysis.beta_30d,
    '60d': analysis.beta_60d,
    '90d': analysis.beta_90d,
  }[selectedWindow];

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

  return (
    <Card elevation={2}>
      <CardContent>
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
            <MuiTooltip title="Current beta coefficient">
              <Chip
                icon={<TrendingUp />}
                label={`β: ${analysis.current_beta.toFixed(2)}`}
                color={getBetaColor(analysis.current_beta) as any}
                size="small"
              />
            </MuiTooltip>

            <MuiTooltip title="Beta volatility (standard deviation)">
              <Chip
                icon={<ShowChart />}
                label={`σ: ${analysis.beta_volatility.toFixed(2)}`}
                color="default"
                size="small"
              />
            </MuiTooltip>
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
    </Card>
  );
}
