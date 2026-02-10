import { useMemo } from 'react';
import {
  Box,
  Typography,
  Paper,
  Grid,
  Alert,
  CircularProgress,
  Card,
  CardContent,
} from '@mui/material';
import { TrendingDown, ShowChart } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getPriceHistory } from '../lib/endpoints';
import {
  LineChart,
  Line,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  Legend,
  ResponsiveContainer,
} from 'recharts';

interface RiskChartProps {
  ticker: string;
  days?: number;
}

interface RollingMetric {
  date: string;
  volatility: number;
  drawdown: number;
}

export function RiskChart({ ticker, days = 90 }: RiskChartProps) {
  const priceHistoryQ = useQuery({
    queryKey: ['priceHistory', ticker],
    queryFn: () => getPriceHistory(ticker),
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  // Calculate rolling metrics
  const rollingMetrics = useMemo(() => {
    if (!priceHistoryQ.data || priceHistoryQ.data.length < 30) return null;

    const prices = priceHistoryQ.data
      .slice(-days)
      .map((p) => ({
        date: p.date,
        close: parseFloat(p.close),
      }))
      .sort((a, b) => new Date(a.date).getTime() - new Date(b.date).getTime());

    const rollingWindow = 30; // 30-day rolling window
    const metrics: RollingMetric[] = [];

    // Track running peak for drawdown calculation
    let runningPeak = prices[0].close;

    for (let i = rollingWindow; i < prices.length; i++) {
      const windowPrices = prices.slice(i - rollingWindow, i);
      const date = prices[i].date;
      const currentPrice = prices[i].close;

      // Update running peak
      if (currentPrice > runningPeak) {
        runningPeak = currentPrice;
      }

      // Calculate returns for volatility
      const returns: number[] = [];
      for (let j = 1; j < windowPrices.length; j++) {
        const ret = (windowPrices[j].close - windowPrices[j - 1].close) / windowPrices[j - 1].close;
        returns.push(ret);
      }

      // Calculate volatility (standard deviation of returns, annualized)
      const mean = returns.reduce((sum, r) => sum + r, 0) / returns.length;
      const variance = returns.reduce((sum, r) => sum + Math.pow(r - mean, 2), 0) / returns.length;
      const dailyVol = Math.sqrt(variance);
      const annualizedVol = dailyVol * Math.sqrt(252) * 100; // Annualize to percentage

      // Calculate drawdown from running peak
      const drawdown = ((currentPrice - runningPeak) / runningPeak) * 100;

      metrics.push({
        date: new Date(date).toLocaleDateString('en-US', { month: 'short', day: 'numeric' }),
        volatility: parseFloat(annualizedVol.toFixed(2)),
        drawdown: parseFloat(drawdown.toFixed(2)),
      });
    }

    return metrics;
  }, [priceHistoryQ.data, days]);

  // Calculate statistics
  const stats = useMemo(() => {
    if (!rollingMetrics) return null;

    const volatilities = rollingMetrics.map((m) => m.volatility);
    const drawdowns = rollingMetrics.map((m) => m.drawdown);

    return {
      avgVolatility: (volatilities.reduce((a, b) => a + b, 0) / volatilities.length).toFixed(2),
      maxVolatility: Math.max(...volatilities).toFixed(2),
      minVolatility: Math.min(...volatilities).toFixed(2),
      maxDrawdown: Math.min(...drawdowns).toFixed(2),
      currentVolatility: volatilities[volatilities.length - 1].toFixed(2),
      currentDrawdown: drawdowns[drawdowns.length - 1].toFixed(2),
    };
  }, [rollingMetrics]);

  if (priceHistoryQ.isLoading) {
    return (
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', py: 8 }}>
        <CircularProgress />
        <Typography sx={{ ml: 2 }}>Loading risk trend data...</Typography>
      </Box>
    );
  }

  if (priceHistoryQ.isError) {
    return (
      <Alert severity="error">
        Failed to load price data for risk trend analysis. Please try again.
      </Alert>
    );
  }

  if (!priceHistoryQ.data || priceHistoryQ.data.length < 30) {
    return (
      <Alert severity="warning">
        Insufficient price data to calculate rolling risk metrics. At least 30 days of price history is required.
      </Alert>
    );
  }

  return (
    <Box>
      <Typography variant="h6" gutterBottom>
        Risk Trends Over Time
      </Typography>
      <Typography variant="body2" color="text.secondary" mb={3}>
        Rolling 30-day risk metrics showing how volatility and drawdown evolved over the selected period.
      </Typography>

      {/* Summary Cards */}
      {stats && (
        <Grid container spacing={2} sx={{ mb: 4 }}>
          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Box display="flex" alignItems="center" gap={1} mb={1}>
                  <ShowChart sx={{ fontSize: 20, color: 'primary.main' }} />
                  <Typography variant="caption" color="text.secondary" fontWeight={600}>
                    Current Volatility
                  </Typography>
                </Box>
                <Typography variant="h5" fontWeight="bold">
                  {stats.currentVolatility}%
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Box display="flex" alignItems="center" gap={1} mb={1}>
                  <ShowChart sx={{ fontSize: 20, color: 'info.main' }} />
                  <Typography variant="caption" color="text.secondary" fontWeight={600}>
                    Avg Volatility
                  </Typography>
                </Box>
                <Typography variant="h5" fontWeight="bold">
                  {stats.avgVolatility}%
                </Typography>
                <Typography variant="caption" color="text.secondary">
                  Range: {stats.minVolatility}% - {stats.maxVolatility}%
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Box display="flex" alignItems="center" gap={1} mb={1}>
                  <TrendingDown sx={{ fontSize: 20, color: 'error.main' }} />
                  <Typography variant="caption" color="text.secondary" fontWeight={600}>
                    Current Drawdown
                  </Typography>
                </Box>
                <Typography variant="h5" fontWeight="bold" color={parseFloat(stats.currentDrawdown) < 0 ? 'error.main' : 'success.main'}>
                  {stats.currentDrawdown}%
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Box display="flex" alignItems="center" gap={1} mb={1}>
                  <TrendingDown sx={{ fontSize: 20, color: 'warning.main' }} />
                  <Typography variant="caption" color="text.secondary" fontWeight={600}>
                    Max Drawdown
                  </Typography>
                </Box>
                <Typography variant="h5" fontWeight="bold" color="error.main">
                  {stats.maxDrawdown}%
                </Typography>
                <Typography variant="caption" color="text.secondary">
                  Worst decline from peak
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      )}

      {/* Volatility Trend Chart */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Rolling 30-Day Volatility
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Shows how price volatility changed over time. Higher values indicate more price fluctuation.
        </Typography>

        <ResponsiveContainer width="100%" height={300}>
          <LineChart data={rollingMetrics || []}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis
              dataKey="date"
              tick={{ fontSize: 12 }}
              interval="preserveStartEnd"
            />
            <YAxis
              label={{ value: 'Volatility (%)', angle: -90, position: 'insideLeft' }}
              tick={{ fontSize: 12 }}
            />
            <Tooltip
              formatter={(value: number) => [`${value.toFixed(2)}%`, 'Volatility']}
              contentStyle={{ backgroundColor: 'rgba(255, 255, 255, 0.95)', border: '1px solid #ccc' }}
            />
            <Legend />
            <Line
              type="monotone"
              dataKey="volatility"
              stroke="#2196f3"
              strokeWidth={2}
              name="Volatility"
              dot={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </Paper>

      {/* Underwater (Drawdown) Chart */}
      <Paper sx={{ p: 3 }}>
        <Typography variant="h6" gutterBottom>
          Drawdown from Peak (Underwater Chart)
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Shows how far the price is below its previous peak. Negative values indicate the position is underwater.
        </Typography>

        <ResponsiveContainer width="100%" height={300}>
          <AreaChart data={rollingMetrics || []}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis
              dataKey="date"
              tick={{ fontSize: 12 }}
              interval="preserveStartEnd"
            />
            <YAxis
              label={{ value: 'Drawdown (%)', angle: -90, position: 'insideLeft' }}
              tick={{ fontSize: 12 }}
              domain={['auto', 0]}
            />
            <Tooltip
              formatter={(value: number) => [`${value.toFixed(2)}%`, 'Drawdown']}
              contentStyle={{ backgroundColor: 'rgba(255, 255, 255, 0.95)', border: '1px solid #ccc' }}
            />
            <Legend />
            <Area
              type="monotone"
              dataKey="drawdown"
              stroke="#f44336"
              fill="#f4433620"
              strokeWidth={2}
              name="Drawdown"
            />
          </AreaChart>
        </ResponsiveContainer>

        <Alert severity="info" sx={{ mt: 2 }}>
          <Typography variant="caption">
            <strong>Interpretation:</strong> When the line is at 0%, the price is at a new peak. When negative, it shows the percentage decline from the previous peak. Longer periods underwater indicate extended recovery times.
          </Typography>
        </Alert>
      </Paper>
    </Box>
  );
}
