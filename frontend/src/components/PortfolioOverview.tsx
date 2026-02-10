import { useMemo } from 'react';
import {
  Box,
  Typography,
  Paper,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Alert,
  CircularProgress,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Grid,
  Card,
  CardContent,
  Chip,
} from '@mui/material';
import { TrendingUp, TrendingDown } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import {
  listPortfolios,
  listAccounts,
  getPortfolioTruePerformance,
  getLatestHoldings,
} from '../lib/endpoints';
import { formatCurrency, formatPercentage } from '../lib/formatters';
import { TickerChip } from './TickerChip';

interface PortfolioOverviewProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
  onTickerNavigate: (ticker: string) => void;
}

export function PortfolioOverview({ selectedPortfolioId, onPortfolioChange, onTickerNavigate }: PortfolioOverviewProps) {
  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const accountsQ = useQuery({
    queryKey: ['accounts', selectedPortfolioId],
    queryFn: () => listAccounts(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  const performanceQ = useQuery({
    queryKey: ['portfolioPerformance', selectedPortfolioId],
    queryFn: () => getPortfolioTruePerformance(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  // Fetch holdings for all accounts
  const holdingsQueries = useQuery({
    queryKey: ['allHoldings', selectedPortfolioId, accountsQ.data],
    queryFn: async () => {
      if (!accountsQ.data || accountsQ.data.length === 0) return [];

      const holdingsPromises = accountsQ.data.map(async (account) => {
        try {
          const holdings = await getLatestHoldings(account.id);
          return holdings;
        } catch (error) {
          console.error(`Failed to fetch holdings for account ${account.id}:`, error);
          return [];
        }
      });

      const allHoldings = await Promise.all(holdingsPromises);
      return allHoldings.flat();
    },
    enabled: !!selectedPortfolioId && !!accountsQ.data && accountsQ.data.length > 0,
  });

  // Aggregate holdings by ticker
  const aggregatedHoldings = useMemo(() => {
    if (!holdingsQueries.data) return [];

    const tickerMap = new Map<string, {
      ticker: string;
      holding_name: string | null;
      asset_category: string | null;
      total_quantity: number;
      total_market_value: number;
      total_gain_loss: number;
    }>();

    holdingsQueries.data.forEach((holding) => {
      // Skip cash holdings
      if (!holding.ticker) return;

      const existing = tickerMap.get(holding.ticker);
      const quantity = parseFloat(holding.quantity);
      const marketValue = parseFloat(holding.market_value);
      const gainLoss = parseFloat(holding.gain_loss || '0');

      if (existing) {
        existing.total_quantity += quantity;
        existing.total_market_value += marketValue;
        existing.total_gain_loss += gainLoss;
      } else {
        tickerMap.set(holding.ticker, {
          ticker: holding.ticker,
          holding_name: holding.holding_name,
          asset_category: holding.asset_category,
          total_quantity: quantity,
          total_market_value: marketValue,
          total_gain_loss: gainLoss,
        });
      }
    });

    return Array.from(tickerMap.values()).sort((a, b) =>
      b.total_market_value - a.total_market_value
    );
  }, [holdingsQueries.data]);

  const portfolioTotals = useMemo(() => {
    if (!performanceQ.data) return {
      currentValue: 0,
      totalDeposits: 0,
      totalWithdrawals: 0,
      trueGainLoss: 0,
    };

    return performanceQ.data.reduce((acc, perf) => ({
      currentValue: acc.currentValue + parseFloat(perf.current_value),
      totalDeposits: acc.totalDeposits + parseFloat(perf.total_deposits),
      totalWithdrawals: acc.totalWithdrawals + parseFloat(perf.total_withdrawals),
      trueGainLoss: acc.trueGainLoss + parseFloat(perf.true_gain_loss),
    }), {
      currentValue: 0,
      totalDeposits: 0,
      totalWithdrawals: 0,
      trueGainLoss: 0,
    });
  }, [performanceQ.data]);

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Portfolio Overview
      </Typography>

      {/* Portfolio Selector */}
      <Box sx={{ mb: 3 }}>
        <FormControl sx={{ minWidth: 200 }}>
          <InputLabel>Portfolio</InputLabel>
          <Select
            value={selectedPortfolioId ?? ''}
            onChange={(e) => onPortfolioChange(e.target.value)}
            label="Portfolio"
          >
            {(portfoliosQ.data ?? []).map((p) => (
              <MenuItem key={p.id} value={p.id}>
                {p.name}
              </MenuItem>
            ))}
          </Select>
        </FormControl>
      </Box>

      {/* Summary Cards */}
      {selectedPortfolioId && performanceQ.data && (
        <Grid container spacing={3} sx={{ mb: 3 }}>
          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Typography color="textSecondary" variant="body2">
                  Current Value
                </Typography>
                <Typography variant="h5">
                  {formatCurrency(portfolioTotals.currentValue)}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Typography color="textSecondary" variant="body2">
                  Total Deposits
                </Typography>
                <Typography variant="h5">
                  {formatCurrency(portfolioTotals.totalDeposits)}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={12} sm={6} md={3}>
            <Card>
              <CardContent>
                <Typography color="textSecondary" variant="body2">
                  Total Withdrawals
                </Typography>
                <Typography variant="h5">
                  {formatCurrency(portfolioTotals.totalWithdrawals)}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
          <Grid item xs={12} sm={6} md={3}>
            <Card sx={{ bgcolor: portfolioTotals.trueGainLoss >= 0 ? 'success.light' : 'error.light' }}>
              <CardContent>
                <Typography color="white" variant="body2">
                  True Gain/Loss
                </Typography>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Typography variant="h5" color="white">
                    {formatCurrency(portfolioTotals.trueGainLoss)}
                  </Typography>
                  {portfolioTotals.trueGainLoss >= 0 ? (
                    <TrendingUp sx={{ color: 'white' }} />
                  ) : (
                    <TrendingDown sx={{ color: 'white' }} />
                  )}
                </Box>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      )}

      {/* Holdings Table */}
      <Paper>
        <Box sx={{ p: 2 }}>
          <Typography variant="h6">Aggregated Holdings</Typography>
          <Typography variant="body2" color="textSecondary">
            Combined holdings across all accounts in this portfolio
          </Typography>
        </Box>

        {holdingsQueries.isLoading && (
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, p: 3 }}>
            <CircularProgress size={20} />
            <Typography>Loading holdings...</Typography>
          </Box>
        )}

        {holdingsQueries.isError && (
          <Alert severity="error" sx={{ m: 2 }}>Failed to load holdings</Alert>
        )}

        {aggregatedHoldings.length > 0 && (
          <TableContainer>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Ticker</TableCell>
                  <TableCell>Name</TableCell>
                  <TableCell>Asset Type</TableCell>
                  <TableCell align="right">Total Quantity</TableCell>
                  <TableCell align="right">Market Value</TableCell>
                  <TableCell align="right">Gain/Loss</TableCell>
                  <TableCell align="right">G/L %</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {aggregatedHoldings.map((holding) => {
                  const avgPrice = holding.total_market_value / holding.total_quantity;
                  const gainLossPct = holding.total_market_value > 0
                    ? (holding.total_gain_loss / (holding.total_market_value - holding.total_gain_loss)) * 100
                    : 0;

                  const getAssetTypeColor = (assetType: string | null): 'default' | 'primary' | 'secondary' | 'info' => {
                    if (!assetType) return 'default';
                    const type = assetType.toLowerCase();
                    if (type.includes('stock') || type.includes('equity')) return 'primary';
                    if (type.includes('mutual fund') || type.includes('fund')) return 'secondary';
                    if (type.includes('bond') || type.includes('fixed')) return 'info';
                    return 'default';
                  };

                  return (
                    <TableRow key={holding.ticker}>
                      <TableCell>
                        <TickerChip ticker={holding.ticker} onNavigate={onTickerNavigate} />
                      </TableCell>
                      <TableCell>
                        <Typography variant="body2">
                          {holding.holding_name || '—'}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        {holding.asset_category ? (
                          <Chip
                            label={holding.asset_category}
                            size="small"
                            color={getAssetTypeColor(holding.asset_category)}
                            variant="outlined"
                          />
                        ) : (
                          <Typography variant="body2" color="textSecondary">
                            —
                          </Typography>
                        )}
                      </TableCell>
                      <TableCell align="right">
                        {holding.total_quantity.toFixed(2)}
                      </TableCell>
                      <TableCell align="right">
                        {formatCurrency(holding.total_market_value)}
                      </TableCell>
                      <TableCell
                        align="right"
                        sx={{ color: holding.total_gain_loss >= 0 ? 'success.main' : 'error.main' }}
                      >
                        {formatCurrency(holding.total_gain_loss)}
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

        {aggregatedHoldings.length === 0 && !holdingsQueries.isLoading && (
          <Alert severity="info" sx={{ m: 2 }}>
            No holdings found. Import CSV data in the Accounts tab to get started.
          </Alert>
        )}
      </Paper>
    </Box>
  );
}
