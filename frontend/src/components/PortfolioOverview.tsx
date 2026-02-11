import { useMemo, useState } from 'react';
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
  Badge,
  Button,
  ButtonGroup,
  Tabs,
  Tab,
} from '@mui/material';
import { TrendingUp, TrendingDown, Warning, Download, Description } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import {
  listPortfolios,
  listAccounts,
  getPortfolioTruePerformance,
  getLatestHoldings,
  getRiskThresholds,
  getPositionRisk,
} from '../lib/endpoints';
import { formatCurrency, formatPercentage } from '../lib/formatters';
import { arrayToCSV, downloadCSV, generateFilename, generatePortfolioPDF, type PortfolioPDFData } from '../lib/exportUtils';
import { TickerChip } from './TickerChip';
import { RiskBadge } from './RiskBadge';
import { AssetTypeChip } from './AssetTypeChip';
import { AssetTypeLegend } from './AssetTypeLegend';
import { OptimizationRecommendations } from './OptimizationRecommendations';
import { PortfolioPerformance } from './PortfolioPerformance';

interface PortfolioOverviewProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
  onTickerNavigate: (ticker: string) => void;
}

export function PortfolioOverview({ selectedPortfolioId, onPortfolioChange, onTickerNavigate }: PortfolioOverviewProps) {
  const [isExporting, setIsExporting] = useState(false);
  const [activeTab, setActiveTab] = useState<'holdings' | 'performance'>('holdings');

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

  // Fetch risk thresholds
  const thresholdsQ = useQuery({
    queryKey: ['riskThresholds'],
    queryFn: getRiskThresholds,
    staleTime: 1000 * 60 * 60, // 1 hour
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
      industry: string | null;
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
          industry: holding.industry,
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

  // Get unique tickers for risk analysis
  const tickers = useMemo(() => {
    return aggregatedHoldings.map(h => h.ticker);
  }, [aggregatedHoldings]);

  // Export portfolio risk data to CSV
  const handleExportCSV = async () => {
    if (aggregatedHoldings.length === 0) {
      alert('No holdings to export');
      return;
    }

    setIsExporting(true);
    try {
      // Fetch risk data for all tickers in parallel
      const riskDataPromises = aggregatedHoldings.map(async (holding) => {
        try {
          const risk = await getPositionRisk(holding.ticker, 90, 'SPY');
          return {
            ticker: holding.ticker,
            risk,
          };
        } catch (error) {
          // Return null for tickers without risk data (mutual funds, bonds, etc.)
          return {
            ticker: holding.ticker,
            risk: null,
          };
        }
      });

      const riskDataResults = await Promise.all(riskDataPromises);
      const riskDataMap = new Map(
        riskDataResults.map(r => [r.ticker, r.risk])
      );

      // Combine holdings with risk data
      const exportData = aggregatedHoldings.map((holding) => {
        const gainLossPct = holding.total_market_value > 0
          ? (holding.total_gain_loss / (holding.total_market_value - holding.total_gain_loss)) * 100
          : 0;

        const risk = riskDataMap.get(holding.ticker);

        return {
          Ticker: holding.ticker,
          Name: holding.holding_name || '',
          'Asset Type': holding.asset_category || '',
          Quantity: holding.total_quantity.toFixed(2),
          'Market Value': holding.total_market_value.toFixed(2),
          'Gain/Loss': holding.total_gain_loss.toFixed(2),
          'G/L %': gainLossPct.toFixed(2),
          'Risk Score': risk?.risk_score.toFixed(1) || 'N/A',
          'Risk Level': risk?.risk_level.toUpperCase() || 'N/A',
          'Volatility %': risk?.metrics.volatility.toFixed(2) || 'N/A',
          'Max Drawdown %': risk?.metrics.max_drawdown.toFixed(2) || 'N/A',
          'Beta': risk?.metrics.beta?.toFixed(2) || 'N/A',
          'VaR 95%': risk?.metrics.var_95?.toFixed(2) || 'N/A',
        };
      });

      // Generate CSV
      const csvContent = arrayToCSV(exportData);
      const portfolioName = portfoliosQ.data?.find(p => p.id === selectedPortfolioId)?.name || 'portfolio';
      const filename = generateFilename(`${portfolioName.replace(/\s+/g, '_')}_risk_report`, 'csv');

      downloadCSV(csvContent, filename);
    } catch (error) {
      console.error('Export failed:', error);
      alert('Failed to export portfolio data. Please try again.');
    } finally {
      setIsExporting(false);
    }
  };

  // Export portfolio risk data to PDF
  const handleExportPDF = async () => {
    if (aggregatedHoldings.length === 0) {
      alert('No holdings to export');
      return;
    }

    setIsExporting(true);
    try {
      // Fetch risk data for all tickers in parallel
      const riskDataPromises = aggregatedHoldings.map(async (holding) => {
        try {
          const risk = await getPositionRisk(holding.ticker, 90, 'SPY');
          return {
            ticker: holding.ticker,
            risk,
          };
        } catch (error) {
          return {
            ticker: holding.ticker,
            risk: null,
          };
        }
      });

      const riskDataResults = await Promise.all(riskDataPromises);
      const riskDataMap = new Map(
        riskDataResults.map(r => [r.ticker, r.risk])
      );

      const portfolioName = portfoliosQ.data?.find(p => p.id === selectedPortfolioId)?.name || 'Portfolio';
      const reportDate = new Date().toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'long',
        day: 'numeric',
      });

      // Prepare PDF data
      const pdfData: PortfolioPDFData = {
        portfolioName,
        reportDate,
        summary: {
          currentValue: formatCurrency(portfolioTotals.currentValue),
          totalDeposits: formatCurrency(portfolioTotals.totalDeposits),
          totalWithdrawals: formatCurrency(portfolioTotals.totalWithdrawals),
          trueGainLoss: formatCurrency(portfolioTotals.trueGainLoss),
        },
        holdings: aggregatedHoldings.map((holding) => {
          const gainLossPct = holding.total_market_value > 0
            ? (holding.total_gain_loss / (holding.total_market_value - holding.total_gain_loss)) * 100
            : 0;

          const risk = riskDataMap.get(holding.ticker);

          return {
            ticker: holding.ticker,
            name: holding.holding_name || '',
            assetType: holding.asset_category || '',
            quantity: holding.total_quantity.toFixed(2),
            marketValue: formatCurrency(holding.total_market_value),
            gainLoss: formatCurrency(holding.total_gain_loss),
            gainLossPct: formatPercentage(gainLossPct),
            riskScore: risk?.risk_score.toFixed(1) || 'N/A',
            riskLevel: risk?.risk_level.toUpperCase() || 'N/A',
            volatility: risk?.metrics.volatility.toFixed(2) || 'N/A',
            maxDrawdown: risk?.metrics.max_drawdown.toFixed(2) || 'N/A',
            beta: risk?.metrics.beta?.toFixed(2) || 'N/A',
          };
        }),
      };

      generatePortfolioPDF(pdfData);
    } catch (error) {
      console.error('PDF export failed:', error);
      alert('Failed to generate PDF report. Please try again.');
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Portfolio Overview
      </Typography>

      {/* Portfolio Selector and Export */}
      <Box sx={{ mb: 3, display: 'flex', alignItems: 'center', gap: 2 }}>
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

        {selectedPortfolioId && aggregatedHoldings.length > 0 && activeTab === 'holdings' && (
          <ButtonGroup variant="outlined" disabled={isExporting}>
            <Button
              startIcon={<Download />}
              onClick={handleExportCSV}
            >
              Export CSV
            </Button>
            <Button
              startIcon={<Description />}
              onClick={handleExportPDF}
            >
              Export PDF
            </Button>
          </ButtonGroup>
        )}
        {isExporting && (
          <Typography variant="caption" color="textSecondary">
            Generating report...
          </Typography>
        )}
      </Box>

      {/* Tab Navigation */}
      {selectedPortfolioId && (
        <Paper sx={{ mb: 3 }}>
          <Tabs
            value={activeTab}
            onChange={(_, newValue) => setActiveTab(newValue)}
            indicatorColor="primary"
            textColor="primary"
          >
            <Tab label="Holdings" value="holdings" />
            <Tab label="Performance" value="performance" />
          </Tabs>
        </Paper>
      )}

      {/* Holdings Tab Content */}
      {activeTab === 'holdings' && (
        <>
          {/* Risk Warning Banner - Shown only if there are high-risk positions */}
          {thresholdsQ.data && aggregatedHoldings.length > 0 && (
            <Alert
              severity="warning"
              icon={<Warning />}
              sx={{
                mb: 3,
                display: 'none', // Will be shown by RiskBadge logic
              }}
              id="risk-warning-banner"
            >
              <Typography variant="body2">
                Some positions exceed your risk thresholds. Review the Risk column below for details.
              </Typography>
            </Alert>
          )}

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

          {/* Optimization Recommendations */}
          {selectedPortfolioId && (
            <Box sx={{ mb: 3 }}>
              <OptimizationRecommendations portfolioId={selectedPortfolioId} />
            </Box>
          )}

          {/* Asset Type Legend */}
          {aggregatedHoldings.length > 0 && <AssetTypeLegend />}

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
                      <TableCell align="center">Risk</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {aggregatedHoldings.map((holding) => {
                      const avgPrice = holding.total_market_value / holding.total_quantity;
                      const gainLossPct = holding.total_market_value > 0
                        ? (holding.total_gain_loss / (holding.total_market_value - holding.total_gain_loss)) * 100
                        : 0;

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
                            <AssetTypeChip
                              ticker={holding.ticker}
                              holdingName={holding.holding_name}
                              assetCategory={holding.asset_category}
                            />
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
                          <TableCell align="center">
                            <RiskBadge
                              ticker={holding.ticker}
                              days={90}
                              showLabel={false}
                              onNavigate={onTickerNavigate}
                              assetCategory={holding.asset_category}
                              industry={holding.industry}
                            />
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
        </>
      )}

      {/* Performance Tab Content */}
      {activeTab === 'performance' && selectedPortfolioId && (
        <PortfolioPerformance portfolioId={selectedPortfolioId} />
      )}
    </Box>
  );
}
