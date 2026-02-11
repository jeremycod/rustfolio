import { useState, useMemo } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Tabs,
  Tab,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  Chip,
  Button,
  Grid,
  Alert,
} from '@mui/material';
import { ArrowBack, TrendingUp, TrendingDown } from '@mui/icons-material';
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
import {
  getAccount,
  getLatestHoldings,
  getAccountHistory,
  getAccountTransactions,
  getAccountTruePerformance,
  getAccountActivity,
} from '../lib/endpoints';
import { formatCurrency, formatNumber, formatPercentage } from '../lib/formatters';
import { TickerChip } from './TickerChip';
import { RiskMetricsPanel } from './RiskMetricsPanel';
import { AssetTypeChip } from './AssetTypeChip';
import { AssetTypeLegend } from './AssetTypeLegend';

interface AccountDetailProps {
  accountId: string;
  onBack: () => void;
  onTickerNavigate: (ticker: string) => void;
}

export function AccountDetail({ accountId, onBack, onTickerNavigate }: AccountDetailProps) {
  const [activeTab, setActiveTab] = useState(0);

  const accountQ = useQuery({
    queryKey: ['account', accountId],
    queryFn: () => getAccount(accountId),
  });

  const holdingsQ = useQuery({
    queryKey: ['holdings', accountId],
    queryFn: () => getLatestHoldings(accountId),
  });

  const historyQ = useQuery({
    queryKey: ['accountHistory', accountId],
    queryFn: () => getAccountHistory(accountId),
  });

  const transactionsQ = useQuery({
    queryKey: ['transactions', accountId],
    queryFn: () => getAccountTransactions(accountId),
  });

  const activityQ = useQuery({
    queryKey: ['activity', accountId],
    queryFn: () => getAccountActivity(accountId),
  });

  const truePerformanceQ = useQuery({
    queryKey: ['truePerformance', accountId],
    queryFn: () => getAccountTruePerformance(accountId),
  });

  // Calculate totals from holdings
  const totals = useMemo(() => {
    if (!holdingsQ.data) return { bookValue: 0, marketValue: 0, gainLoss: 0, gainLossPct: 0 };

    const bookValue = holdingsQ.data.reduce(
      (sum, h) => sum + (parseFloat(h.quantity) * parseFloat(h.price)),
      0
    );
    const marketValue = holdingsQ.data.reduce(
      (sum, h) => sum + parseFloat(h.market_value),
      0
    );
    const gainLoss = marketValue - bookValue;
    const gainLossPct = bookValue > 0 ? (gainLoss / bookValue) * 100 : 0;

    return { bookValue, marketValue, gainLoss, gainLossPct };
  }, [holdingsQ.data]);

  // Format history data for chart
  const chartData = useMemo(() => {
    if (!historyQ.data) return [];
    return historyQ.data.map((point) => ({
      date: point.snapshot_date,
      value: parseFloat(point.total_value),
      cost: parseFloat(point.total_cost),
      gainLoss: parseFloat(point.total_gain_loss),
    }));
  }, [historyQ.data]);

  if (accountQ.isLoading) {
    return <Typography>Loading account...</Typography>;
  }

  if (!accountQ.data) {
    return <Typography>Account not found</Typography>;
  }

  const account = accountQ.data;

  return (
    <Box>
      <Button startIcon={<ArrowBack />} onClick={onBack} sx={{ mb: 2 }}>
        Back to Accounts
      </Button>

      {/* Account Header */}
      <Card sx={{ mb: 3, bgcolor: 'primary.dark', color: 'white' }}>
        <CardContent>
          <Typography variant="h5" gutterBottom>
            {account.account_nickname}
          </Typography>
          <Box sx={{ display: 'flex', gap: 2, mb: 2 }}>
            <Chip label={account.account_number} size="small" sx={{ bgcolor: 'rgba(255,255,255,0.2)', color: 'white' }} />
            {account.client_name && (
              <Typography variant="body2" sx={{ opacity: 0.8 }}>
                {account.client_name}
              </Typography>
            )}
          </Box>

          <Grid container spacing={3}>
            <Grid item xs={12} sm={2.4}>
              <Typography color="rgba(255,255,255,0.7)" variant="body2">
                Current Value
              </Typography>
              <Typography variant="h6">
                {truePerformanceQ.data ? formatCurrency(truePerformanceQ.data.current_value) : formatCurrency(totals.marketValue)}
              </Typography>
            </Grid>
            <Grid item xs={12} sm={2.4}>
              <Typography color="rgba(255,255,255,0.7)" variant="body2">
                Total Deposits
              </Typography>
              <Typography variant="h6">
                {truePerformanceQ.data ? formatCurrency(truePerformanceQ.data.total_deposits) : formatCurrency(0)}
              </Typography>
            </Grid>
            <Grid item xs={12} sm={2.4}>
              <Typography color="rgba(255,255,255,0.7)" variant="body2">
                Total Withdrawals
              </Typography>
              <Typography variant="h6">
                {truePerformanceQ.data ? formatCurrency(truePerformanceQ.data.total_withdrawals) : formatCurrency(0)}
              </Typography>
            </Grid>
            <Grid item xs={12} sm={2.4}>
              <Typography color="rgba(255,255,255,0.7)" variant="body2">
                True Gain/Loss
              </Typography>
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Typography variant="h6">
                  {truePerformanceQ.data ? formatCurrency(truePerformanceQ.data.true_gain_loss) : formatCurrency(totals.gainLoss)}
                </Typography>
                {(truePerformanceQ.data ? parseFloat(truePerformanceQ.data.true_gain_loss) : totals.gainLoss) >= 0 ? (
                  <TrendingUp sx={{ color: 'success.light' }} />
                ) : (
                  <TrendingDown sx={{ color: 'error.light' }} />
                )}
              </Box>
            </Grid>
            <Grid item xs={12} sm={2.4}>
              <Typography color="rgba(255,255,255,0.7)" variant="body2">
                True G/L (%)
              </Typography>
              <Typography
                variant="h6"
                sx={{
                  color: (truePerformanceQ.data ? parseFloat(truePerformanceQ.data.true_gain_loss_pct) : totals.gainLossPct) >= 0
                    ? 'success.light'
                    : 'error.light'
                }}
              >
                {truePerformanceQ.data ? formatPercentage(truePerformanceQ.data.true_gain_loss_pct) : formatPercentage(totals.gainLossPct)}
              </Typography>
            </Grid>
          </Grid>
        </CardContent>
      </Card>

      {/* Tabs */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 3 }}>
        <Tabs value={activeTab} onChange={(_, val) => setActiveTab(val)}>
          <Tab label="Holdings" />
          <Tab label="History" />
          <Tab label="Transactions" />
          <Tab label="Risk Analysis" />
        </Tabs>
      </Box>

      {/* Holdings Tab */}
      {activeTab === 0 && (
        <>
          {holdingsQ.data && holdingsQ.data.length > 0 && <AssetTypeLegend />}
          <TableContainer component={Paper}>
          <Table>
            <TableHead>
              <TableRow>
                <TableCell>Symbol</TableCell>
                <TableCell>Holding Name</TableCell>
                <TableCell align="right">Quantity</TableCell>
                <TableCell align="right">Price</TableCell>
                <TableCell align="right">Market Value</TableCell>
                <TableCell align="right">G/L ($)</TableCell>
                <TableCell align="right">G/L (%)</TableCell>
                <TableCell>Category</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {holdingsQ.data?.map((holding) => {
                const gainLoss = holding.gain_loss ? parseFloat(holding.gain_loss) : 0;
                const gainLossPct = holding.gain_loss_pct ? parseFloat(holding.gain_loss_pct) : 0;

                return (
                  <TableRow key={holding.id}>
                    <TableCell>
                      <TickerChip ticker={holding.ticker} onNavigate={onTickerNavigate} />
                    </TableCell>
                    <TableCell>
                      <Typography variant="body2">{holding.holding_name || '—'}</Typography>
                    </TableCell>
                    <TableCell align="right">{formatNumber(holding.quantity)}</TableCell>
                    <TableCell align="right">{formatCurrency(holding.price)}</TableCell>
                    <TableCell align="right">{formatCurrency(holding.market_value)}</TableCell>
                    <TableCell
                      align="right"
                      sx={{ color: gainLoss >= 0 ? 'success.main' : 'error.main' }}
                    >
                      {formatCurrency(gainLoss)}
                    </TableCell>
                    <TableCell
                      align="right"
                      sx={{ color: gainLossPct >= 0 ? 'success.main' : 'error.main' }}
                    >
                      {formatPercentage(gainLossPct)}
                    </TableCell>
                    <TableCell>
                      <AssetTypeChip
                        ticker={holding.ticker}
                        holdingName={holding.holding_name}
                        assetCategory={holding.asset_category}
                      />
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </TableContainer>
        </>
      )}

      {/* History Tab */}
      {activeTab === 1 && (
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Account Value History
            </Typography>
            {chartData.length > 0 ? (
              <Box sx={{ height: 400, mt: 2 }}>
                <ResponsiveContainer width="100%" height="100%">
                  <LineChart data={chartData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="date" />
                    <YAxis />
                    <Tooltip
                      formatter={(value: number) => formatCurrency(value)}
                    />
                    <Legend />
                    <Line
                      type="monotone"
                      dataKey="value"
                      stroke="#8884d8"
                      name="Market Value"
                      strokeWidth={2}
                    />
                    <Line
                      type="monotone"
                      dataKey="cost"
                      stroke="#82ca9d"
                      name="Book Value"
                      strokeWidth={2}
                    />
                    <Line
                      type="monotone"
                      dataKey="gainLoss"
                      stroke="#ff7300"
                      name="Gain/Loss"
                      strokeWidth={2}
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
                    {historyQ.data?.slice().reverse().map((point) => {
                      const gainLoss = parseFloat(point.total_gain_loss);
                      const gainLossPct = parseFloat(point.total_gain_loss_pct);

                      return (
                        <TableRow key={point.snapshot_date}>
                          <TableCell>{point.snapshot_date}</TableCell>
                          <TableCell align="right">
                            {formatCurrency(point.total_value)}
                          </TableCell>
                          <TableCell align="right">
                            {formatCurrency(point.total_cost)}
                          </TableCell>
                          <TableCell
                            align="right"
                            sx={{ color: gainLoss >= 0 ? 'success.main' : 'error.main' }}
                          >
                            {formatCurrency(gainLoss)}
                          </TableCell>
                          <TableCell
                            align="right"
                            sx={{ color: gainLossPct >= 0 ? 'success.main' : 'error.main' }}
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
      )}

      {/* Transactions Tab */}
      {activeTab === 2 && (
        <Card>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Activity History (Transactions & Cash Flows)
            </Typography>
            {activityQ.data && activityQ.data.length > 0 ? (
              <TableContainer component={Paper} sx={{ mt: 2 }}>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Date</TableCell>
                      <TableCell>Activity</TableCell>
                      <TableCell>Type</TableCell>
                      <TableCell>Ticker</TableCell>
                      <TableCell align="right">Quantity</TableCell>
                      <TableCell align="right">Amount</TableCell>
                      <TableCell>Description</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {activityQ.data.map((activity, idx) => {
                      const isCashFlow = activity.activity_type === 'CASH_FLOW';
                      const isDeposit = activity.type_detail === 'DEPOSIT';
                      const isWithdrawal = activity.type_detail === 'WITHDRAWAL';
                      const isBuy = activity.type_detail === 'BUY';
                      const isSell = activity.type_detail === 'SELL';

                      return (
                        <TableRow key={`${activity.activity_type}-${activity.activity_date}-${idx}`}>
                          <TableCell>{activity.activity_date}</TableCell>
                          <TableCell>
                            <Chip
                              label={activity.activity_type.replace('_', ' ')}
                              size="small"
                              variant="outlined"
                              color={isCashFlow ? 'primary' : 'default'}
                            />
                          </TableCell>
                          <TableCell>
                            <Chip
                              label={activity.type_detail}
                              size="small"
                              color={
                                isBuy || isDeposit ? 'success' :
                                isSell || isWithdrawal ? 'error' :
                                'default'
                              }
                            />
                          </TableCell>
                          <TableCell>
                            {activity.ticker ? (
                              <Typography variant="body2" fontWeight="bold">
                                {activity.ticker}
                              </Typography>
                            ) : (
                              <Typography variant="body2" color="textSecondary">
                                —
                              </Typography>
                            )}
                          </TableCell>
                          <TableCell align="right">
                            {activity.quantity ? formatNumber(activity.quantity) : '—'}
                          </TableCell>
                          <TableCell align="right">
                            {activity.amount ? formatCurrency(activity.amount) : '—'}
                          </TableCell>
                          <TableCell>
                            <Typography variant="body2" color="textSecondary">
                              {activity.description || '—'}
                            </Typography>
                          </TableCell>
                        </TableRow>
                      );
                    })}
                  </TableBody>
                </Table>
              </TableContainer>
            ) : (
              <Typography color="textSecondary" sx={{ textAlign: 'center', py: 4 }}>
                No activity detected yet. Import multiple CSV snapshots to see deposits, withdrawals, and transactions.
              </Typography>
            )}
          </CardContent>
        </Card>
      )}

      {/* Risk Analysis Tab */}
      {activeTab === 3 && (
        <Box>
          <Typography variant="h6" gutterBottom>
            Risk Analysis by Position
          </Typography>
          <Typography variant="body2" color="textSecondary" sx={{ mb: 3 }}>
            View risk metrics for each equity holding in this account. Click on a ticker to see detailed risk analysis.
          </Typography>
          {holdingsQ.data && holdingsQ.data.length > 0 ? (
            (() => {
              // Filter holdings that can have risk analysis
              // Exclude: cash (no ticker), mutual funds with proprietary codes, bonds/fixed income
              const analyzableHoldings = holdingsQ.data.filter((holding) => {
                // Must have a ticker
                if (!holding.ticker) return false;

                // Exclude cash equivalents
                if (holding.asset_category === 'Cash and Cash Equivalents') return false;

                // Exclude fixed income (bonds)
                if (holding.asset_category === 'FIXED INCOME') return false;

                // Exclude alternatives
                if (holding.asset_category === 'ALTERNATIVES AND OTHER') return false;

                // Exclude mutual funds with proprietary codes (contain numbers but not ETF-style)
                // Common patterns: EDG5001, FID5494, AGF9110, RBF1684, etc.
                if (/^[A-Z]{3,}[0-9]{3,}/.test(holding.ticker)) return false;

                return true;
              });

              const excludedCount = holdingsQ.data.length - analyzableHoldings.length;

              return (
                <>
                  {excludedCount > 0 && (
                    <Alert severity="info" sx={{ mb: 3 }}>
                      {excludedCount} holding{excludedCount !== 1 ? 's' : ''} excluded from risk analysis
                      (mutual funds, bonds, cash, or securities without public price data).
                    </Alert>
                  )}
                  {analyzableHoldings.length > 0 ? (
                    <Grid container spacing={3}>
                      {analyzableHoldings.map((holding) => (
                        <Grid item xs={12} md={6} key={holding.id}>
                          <RiskMetricsPanel
                            ticker={holding.ticker}
                            holdingName={holding.holding_name}
                            days={90}
                            benchmark="SPY"
                            onTickerClick={onTickerNavigate}
                          />
                        </Grid>
                      ))}
                    </Grid>
                  ) : (
                    <Card>
                      <CardContent>
                        <Typography color="textSecondary" sx={{ textAlign: 'center', py: 4 }}>
                          No equity holdings available for risk analysis.
                          This account contains only mutual funds, bonds, or other securities without public price data.
                        </Typography>
                      </CardContent>
                    </Card>
                  )}
                </>
              );
            })()
          ) : (
            <Card>
              <CardContent>
                <Typography color="textSecondary" sx={{ textAlign: 'center', py: 4 }}>
                  No holdings available for risk analysis.
                </Typography>
              </CardContent>
            </Card>
          )}
        </Box>
      )}
    </Box>
  );
}
