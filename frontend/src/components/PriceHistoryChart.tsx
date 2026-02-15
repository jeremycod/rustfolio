import { useMemo } from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  Grid,
  CircularProgress,
  Alert,
} from '@mui/material';
import { TrendingUp, TrendingDown } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Legend,
  ReferenceLine,
  Area,
  AreaChart,
  ComposedChart,
} from 'recharts';
import { getPriceHistory } from '../lib/endpoints';
import { formatCurrency, formatPercentage } from '../lib/formatters';

interface PriceHistoryChartProps {
  ticker: string;
  days: number;
  companyName?: string | null;
}

export function PriceHistoryChart({ ticker, days, companyName }: PriceHistoryChartProps) {
  const priceHistoryQ = useQuery({
    queryKey: ['priceHistory', ticker],
    queryFn: () => getPriceHistory(ticker),
    staleTime: 1000 * 60 * 30, // 30 minutes
  });

  const chartData = useMemo(() => {
    if (!priceHistoryQ.data) return null;

    const data = priceHistoryQ.data
      .slice(-days) // Get last N days
      .map((point) => ({
        date: point.date,
        price: parseFloat(point.close_price),
      }))
      .sort((a, b) => a.date.localeCompare(b.date));

    if (data.length === 0) return null;

    // Calculate statistics
    const prices = data.map((d) => d.price);
    const startPrice = prices[0];
    const endPrice = prices[prices.length - 1];
    const minPrice = Math.min(...prices);
    const maxPrice = Math.max(...prices);
    const change = endPrice - startPrice;
    const changePercent = (change / startPrice) * 100;

    // Calculate max drawdown and underwater drawdown for each point
    let peak = prices[0];
    let maxDrawdown = 0;
    let drawdownStartDate = data[0].date;
    let drawdownEndDate = data[0].date;
    const drawdowns: number[] = [];

    prices.forEach((price, i) => {
      if (price > peak) {
        peak = price;
      }
      const drawdown = ((price - peak) / peak) * 100;
      drawdowns.push(drawdown);

      if (drawdown < maxDrawdown) {
        maxDrawdown = drawdown;
        drawdownStartDate = data.slice(0, i + 1).find((d) => d.price === peak)?.date || data[0].date;
        drawdownEndDate = data[i].date;
      }
    });

    // Calculate 20-day moving average and add drawdown
    const dataWithMA = data.map((point, i) => {
      const start = Math.max(0, i - 19);
      const slice = prices.slice(start, i + 1);
      const ma20 = slice.reduce((sum, p) => sum + p, 0) / slice.length;
      return {
        ...point,
        ma20: i >= 19 ? ma20 : null,
        drawdown: drawdowns[i],
      };
    });

    return {
      data: dataWithMA,
      stats: {
        startPrice,
        endPrice,
        minPrice,
        maxPrice,
        change,
        changePercent,
        maxDrawdown,
        drawdownStartDate,
        drawdownEndDate,
      },
    };
  }, [priceHistoryQ.data, days]);

  if (priceHistoryQ.isLoading) {
    return (
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, p: 3 }}>
        <CircularProgress size={24} />
        <Typography>Loading price history...</Typography>
      </Box>
    );
  }

  if (priceHistoryQ.isError) {
    return (
      <Alert severity="error">
        Failed to load price history for {ticker}. The ticker may not have available price data.
      </Alert>
    );
  }

  if (!chartData || chartData.data.length === 0) {
    return (
      <Alert severity="info">
        No price history available for {ticker} in the selected time period.
      </Alert>
    );
  }

  const { data, stats } = chartData;

  return (
    <Box>
      {/* Statistics Cards */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" variant="body2" gutterBottom>
                Period Change
              </Typography>
              <Box display="flex" alignItems="center" gap={1}>
                <Typography variant="h5" fontWeight="bold" color={stats.change >= 0 ? 'success.main' : 'error.main'}>
                  {formatPercentage(stats.changePercent)}
                </Typography>
                {stats.change >= 0 ? (
                  <TrendingUp sx={{ color: 'success.main' }} />
                ) : (
                  <TrendingDown sx={{ color: 'error.main' }} />
                )}
              </Box>
              <Typography variant="caption" color="textSecondary">
                {formatCurrency(stats.change)}
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" variant="body2" gutterBottom>
                Current Price
              </Typography>
              <Typography variant="h5" fontWeight="bold">
                {formatCurrency(stats.endPrice)}
              </Typography>
              <Typography variant="caption" color="textSecondary">
                Started at {formatCurrency(stats.startPrice)}
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" variant="body2" gutterBottom>
                Period High
              </Typography>
              <Typography variant="h5" fontWeight="bold" color="success.main">
                {formatCurrency(stats.maxPrice)}
              </Typography>
              <Typography variant="caption" color="textSecondary">
                Peak value in period
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" variant="body2" gutterBottom>
                Period Low
              </Typography>
              <Typography variant="h5" fontWeight="bold" color="error.main">
                {formatCurrency(stats.minPrice)}
              </Typography>
              <Typography variant="caption" color="textSecondary">
                Lowest value in period
              </Typography>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Price Chart */}
      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Price History - Last {days} Days
          </Typography>
          <Box sx={{ height: 400, minHeight: 400, mt: 2 }}>
            <ResponsiveContainer width="100%" height="100%">
              <ComposedChart data={data}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis
                  dataKey="date"
                  tick={{ fontSize: 12 }}
                  tickFormatter={(value) => {
                    const date = new Date(value);
                    return `${date.getMonth() + 1}/${date.getDate()}`;
                  }}
                />
                <YAxis
                  domain={['auto', 'auto']}
                  tick={{ fontSize: 12 }}
                  tickFormatter={(value) => `$${value.toFixed(0)}`}
                />
                <Tooltip
                  formatter={(value: number, name: string) => {
                    if (name === 'Price') return [formatCurrency(value), name];
                    if (name === '20-Day MA') return [formatCurrency(value), name];
                    return [value, name];
                  }}
                  labelFormatter={(label) => {
                    const date = new Date(label);
                    return date.toLocaleDateString();
                  }}
                />
                <Legend />
                <Line
                  type="monotone"
                  dataKey="price"
                  stroke="#1976d2"
                  strokeWidth={2}
                  name="Price"
                  dot={false}
                />
                <Line
                  type="monotone"
                  dataKey="ma20"
                  stroke="#ff9800"
                  strokeWidth={2}
                  name="20-Day MA"
                  dot={false}
                  strokeDasharray="5 5"
                />
              </ComposedChart>
            </ResponsiveContainer>
          </Box>
        </CardContent>
      </Card>

      {/* Underwater (Drawdown) Chart */}
      <Card sx={{ mt: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Underwater Chart - Drawdown from Peak
          </Typography>
          <Typography variant="body2" color="text.secondary" mb={2}>
            Shows how far the price is below its previous peak. When the line is at 0%, the price is at a new all-time high. Negative values indicate the percentage decline from the peak.
          </Typography>
          <Box sx={{ height: 300, minHeight: 300, mt: 2 }}>
            <ResponsiveContainer width="100%" height="100%">
              <AreaChart data={data}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis
                  dataKey="date"
                  tick={{ fontSize: 12 }}
                  tickFormatter={(value) => {
                    const date = new Date(value);
                    return `${date.getMonth() + 1}/${date.getDate()}`;
                  }}
                />
                <YAxis
                  domain={['auto', 0]}
                  tick={{ fontSize: 12 }}
                  tickFormatter={(value) => `${value.toFixed(0)}%`}
                  label={{ value: 'Drawdown (%)', angle: -90, position: 'insideLeft' }}
                />
                <Tooltip
                  formatter={(value: number) => [`${value.toFixed(2)}%`, 'Drawdown from Peak']}
                  labelFormatter={(label) => {
                    const date = new Date(label);
                    return date.toLocaleDateString();
                  }}
                  contentStyle={{ backgroundColor: 'rgba(255, 255, 255, 0.95)', border: '1px solid #ccc' }}
                />
                <Legend />
                <ReferenceLine y={0} stroke="#666" strokeDasharray="3 3" label="Peak (0%)" />
                <Area
                  type="monotone"
                  dataKey="drawdown"
                  stroke="#f44336"
                  fill="#f4433630"
                  strokeWidth={2}
                  name="Drawdown from Peak"
                />
              </AreaChart>
            </ResponsiveContainer>
          </Box>

          {stats.maxDrawdown < -5 && (
            <Alert severity="info" sx={{ mt: 2 }}>
              <Typography variant="body2" gutterBottom>
                <strong>Maximum Drawdown:</strong> {formatPercentage(stats.maxDrawdown)}
              </Typography>
              <Typography variant="caption" color="textSecondary">
                The largest peak-to-trough decline occurred from {new Date(stats.drawdownStartDate).toLocaleDateString()} to {new Date(stats.drawdownEndDate).toLocaleDateString()}.
                {(() => {
                  const startIdx = data.findIndex(d => d.date === stats.drawdownStartDate);
                  const endIdx = data.findIndex(d => d.date === stats.drawdownEndDate);
                  const daysUnderwater = endIdx - startIdx;
                  return daysUnderwater > 0 ? ` The position was underwater for ${daysUnderwater} days.` : '';
                })()}
              </Typography>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Drawdown Information */}
      {stats.maxDrawdown < -5 && (
        <Alert severity="warning" sx={{ mt: 3 }}>
          <Typography variant="body2" gutterBottom>
            <strong>Risk Notice:</strong> This position experienced a drawdown of {formatPercentage(stats.maxDrawdown)} during the selected period.
          </Typography>
          <Typography variant="caption" color="textSecondary">
            Large drawdowns indicate high volatility and significant recovery challenges. Consider this when assessing position risk.
          </Typography>
        </Alert>
      )}
    </Box>
  );
}
