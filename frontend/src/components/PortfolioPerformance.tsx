import { useMemo } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Grid,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Alert,
  CircularProgress,
  Chip,
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
} from 'recharts';
import { getPortfolioTruePerformance, getPortfolioHistory } from '../lib/endpoints';
import { formatCurrency, formatPercentage } from '../lib/formatters';

interface PortfolioPerformanceProps {
  portfolioId: string;
}

export function PortfolioPerformance({ portfolioId }: PortfolioPerformanceProps) {
  // Fetch portfolio true performance (aggregated across all accounts)
  const performanceQ = useQuery({
    queryKey: ['portfolioPerformance', portfolioId],
    queryFn: () => getPortfolioTruePerformance(portfolioId),
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  // Fetch portfolio history
  const historyQ = useQuery({
    queryKey: ['portfolioHistory', portfolioId],
    queryFn: () => getPortfolioHistory(portfolioId),
    staleTime: 1000 * 60 * 5,
  });

  // Calculate portfolio totals
  const portfolioTotals = useMemo(() => {
    if (!performanceQ.data || performanceQ.data.length === 0) return null;

    const totalDeposits = performanceQ.data.reduce(
      (sum, account) => sum + parseFloat(account.total_deposits),
      0
    );
    const totalWithdrawals = performanceQ.data.reduce(
      (sum, account) => sum + parseFloat(account.total_withdrawals),
      0
    );
    const totalCurrentValue = performanceQ.data.reduce(
      (sum, account) => sum + parseFloat(account.current_value),
      0
    );
    const totalBookValue = performanceQ.data.reduce(
      (sum, account) => sum + parseFloat(account.book_value),
      0
    );
    const totalGainLoss = performanceQ.data.reduce(
      (sum, account) => sum + parseFloat(account.true_gain_loss),
      0
    );

    const gainLossPct = totalDeposits - totalWithdrawals !== 0
      ? (totalGainLoss / (totalDeposits - totalWithdrawals)) * 100
      : 0;

    return {
      totalDeposits,
      totalWithdrawals,
      totalCurrentValue,
      totalBookValue,
      totalGainLoss,
      gainLossPct,
    };
  }, [performanceQ.data]);

  // Transform history data for chart
  const chartData = useMemo(() => {
    if (!historyQ.data || historyQ.data.length === 0) return [];

    // Group by snapshot_date and sum values
    const grouped = historyQ.data.reduce((acc, point) => {
      const date = point.snapshot_date;
      if (!acc[date]) {
        acc[date] = {
          date,
          totalValue: 0,
          totalCost: 0,
          totalGainLoss: 0,
        };
      }
      acc[date].totalValue += parseFloat(point.total_value);
      acc[date].totalCost += parseFloat(point.total_cost);
      acc[date].totalGainLoss += point.total_gain_loss ? parseFloat(point.total_gain_loss) : 0;
      return acc;
    }, {} as Record<string, { date: string; totalValue: number; totalCost: number; totalGainLoss: number }>);

    return Object.values(grouped).sort((a, b) => a.date.localeCompare(b.date));
  }, [historyQ.data]);

  if (performanceQ.isLoading || historyQ.isLoading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', py: 8 }}>
        <CircularProgress />
        <Typography sx={{ ml: 2 }}>Loading portfolio performance...</Typography>
      </Box>
    );
  }

  if (performanceQ.isError || historyQ.isError) {
    return (
      <Alert severity="error">
        Failed to load portfolio performance data.
      </Alert>
    );
  }

  if (!portfolioTotals) {
    return (
      <Alert severity="info">
        No performance data available. Import account data to see portfolio performance.
      </Alert>
    );
  }

  return (
    <Box>
      {/* Summary Cards */}
      <Card sx={{ mb: 3, bgcolor: 'primary.main', color: 'primary.contrastText' }}>
        <CardContent>
          <Grid container spacing={3}>
            <Grid item xs={12} sm={6} md={2.4}>
              <Typography variant="body2" sx={{ opacity: 0.9 }}>
                Current Value
              </Typography>
              <Typography variant="h4" fontWeight="bold">
                {formatCurrency(portfolioTotals.totalCurrentValue)}
              </Typography>
            </Grid>

            <Grid item xs={12} sm={6} md={2.4}>
              <Typography variant="body2" sx={{ opacity: 0.9 }}>
                Total Deposits
              </Typography>
              <Typography variant="h4" fontWeight="bold">
                {formatCurrency(portfolioTotals.totalDeposits)}
              </Typography>
            </Grid>

            <Grid item xs={12} sm={6} md={2.4}>
              <Typography variant="body2" sx={{ opacity: 0.9 }}>
                Total Withdrawals
              </Typography>
              <Typography variant="h4" fontWeight="bold">
                {formatCurrency(portfolioTotals.totalWithdrawals)}
              </Typography>
            </Grid>

            <Grid item xs={12} sm={6} md={2.4}>
              <Typography variant="body2" sx={{ opacity: 0.9 }}>
                True Gain/Loss
              </Typography>
              <Box display="flex" alignItems="center" gap={1}>
                <Typography variant="h4" fontWeight="bold">
                  {formatCurrency(portfolioTotals.totalGainLoss)}
                </Typography>
                {portfolioTotals.totalGainLoss >= 0 ? (
                  <TrendingUp fontSize="large" />
                ) : (
                  <TrendingDown fontSize="large" />
                )}
              </Box>
            </Grid>

            <Grid item xs={12} sm={6} md={2.4}>
              <Typography variant="body2" sx={{ opacity: 0.9 }}>
                True G/L (%)
              </Typography>
              <Typography
                variant="h4"
                fontWeight="bold"
                sx={{
                  color: portfolioTotals.gainLossPct >= 0
                    ? 'success.light'
                    : 'error.light',
                }}
              >
                {formatPercentage(portfolioTotals.gainLossPct)}
              </Typography>
            </Grid>
          </Grid>
        </CardContent>
      </Card>

      {/* Portfolio Value History Chart */}
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Portfolio Value History
          </Typography>
          {chartData.length > 0 ? (
            <Box sx={{ height: 400, minHeight: 400, mt: 2 }}>
              <ResponsiveContainer width="100%" height="100%">
                <LineChart data={chartData}>
                  <CartesianGrid strokeDasharray="3 3" />
                  <XAxis dataKey="date" />
                  <YAxis tickFormatter={(value) => `$${(value / 1000).toFixed(0)}k`} />
                  <Tooltip
                    formatter={(value: number | undefined) => value !== undefined ? formatCurrency(value) : ''}
                  />
                  <Legend />
                  <Line
                    type="monotone"
                    dataKey="totalValue"
                    stroke="#8884d8"
                    name="Market Value"
                    strokeWidth={2}
                    dot={{ r: 4 }}
                  />
                  <Line
                    type="monotone"
                    dataKey="totalCost"
                    stroke="#82ca9d"
                    name="Book Value"
                    strokeWidth={2}
                    dot={{ r: 4 }}
                  />
                  <Line
                    type="monotone"
                    dataKey="totalGainLoss"
                    stroke="#ff7300"
                    name="Gain/Loss"
                    strokeWidth={2}
                    dot={{ r: 4 }}
                  />
                </LineChart>
              </ResponsiveContainer>
            </Box>
          ) : (
            <Typography color="textSecondary" sx={{ textAlign: 'center', py: 4 }}>
              No historical data available. Import more CSV snapshots to see trends.
            </Typography>
          )}

          {/* History Table */}
          {chartData.length > 0 && (
            <TableContainer component={Paper} sx={{ mt: 3 }}>
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>Date</TableCell>
                    <TableCell align="right">Market Value</TableCell>
                    <TableCell align="right">Book Value</TableCell>
                    <TableCell align="right">Gain/Loss</TableCell>
                    <TableCell align="right">G/L %</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {chartData.slice().reverse().map((point) => {
                    const gainLossPct = point.totalCost !== 0
                      ? (point.totalGainLoss / point.totalCost) * 100
                      : 0;

                    return (
                      <TableRow key={point.date}>
                        <TableCell>{point.date}</TableCell>
                        <TableCell align="right">
                          {formatCurrency(point.totalValue)}
                        </TableCell>
                        <TableCell align="right">
                          {formatCurrency(point.totalCost)}
                        </TableCell>
                        <TableCell
                          align="right"
                          sx={{
                            color: point.totalGainLoss >= 0 ? 'success.main' : 'error.main',
                          }}
                        >
                          {formatCurrency(point.totalGainLoss)}
                        </TableCell>
                        <TableCell
                          align="right"
                          sx={{
                            color: gainLossPct >= 0 ? 'success.main' : 'error.main',
                          }}
                        >
                          {formatPercentage(gainLossPct)}
                        </TableCell>
                      </TableRow>
                    );
                  })}
                </TableBody>
              </Table>
            </TableContainer>
          )}
        </CardContent>
      </Card>

      {/* Account Breakdown */}
      <Card>
        <CardContent>
          <Typography variant="h6" gutterBottom>
            Account Performance Breakdown
          </Typography>
          <TableContainer component={Paper} variant="outlined">
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Account</TableCell>
                  <TableCell align="right">Current Value</TableCell>
                  <TableCell align="right">Deposits</TableCell>
                  <TableCell align="right">Withdrawals</TableCell>
                  <TableCell align="right">Gain/Loss</TableCell>
                  <TableCell align="right">Return %</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {performanceQ.data?.map((account) => {
                  const gainLoss = parseFloat(account.true_gain_loss);
                  const gainLossPct = parseFloat(account.true_gain_loss_pct);

                  return (
                    <TableRow key={account.account_id}>
                      <TableCell>
                        <Typography variant="body2" fontWeight={600}>
                          {account.account_nickname}
                        </Typography>
                        <Typography variant="caption" color="textSecondary">
                          {account.account_number}
                        </Typography>
                      </TableCell>
                      <TableCell align="right">
                        {formatCurrency(account.current_value)}
                      </TableCell>
                      <TableCell align="right">
                        {formatCurrency(account.total_deposits)}
                      </TableCell>
                      <TableCell align="right">
                        {formatCurrency(account.total_withdrawals)}
                      </TableCell>
                      <TableCell
                        align="right"
                        sx={{
                          color: gainLoss >= 0 ? 'success.main' : 'error.main',
                        }}
                      >
                        {formatCurrency(gainLoss)}
                      </TableCell>
                      <TableCell
                        align="right"
                        sx={{
                          color: gainLossPct >= 0 ? 'success.main' : 'error.main',
                        }}
                      >
                        <Chip
                          label={formatPercentage(gainLossPct)}
                          size="small"
                          color={gainLossPct >= 0 ? 'success' : 'error'}
                        />
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </TableContainer>
        </CardContent>
      </Card>
    </Box>
  );
}
