import { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Paper,
  Button,
  Chip,
  TextField,
  Alert,
  Grid,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  CircularProgress,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Autocomplete,
  IconButton,
  Tooltip,
} from '@mui/material';
import {
  Compare,
  Add,
  Close,
  Download,
  HelpOutline,
} from '@mui/icons-material';
import { MetricHelpDialog } from './MetricHelpDialog';
import { TickerActionMenu } from './TickerActionMenu';
import { useQueries, useQuery } from '@tanstack/react-query';
import { getPositionRisk, listPortfolios } from '../lib/endpoints';
import { RiskLevel } from '../types';
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip as RechartsTooltip,
  ResponsiveContainer,
  Cell,
} from 'recharts';

interface RiskComparisonProps {
  onTickerNavigate?: (ticker: string, page?: string) => void;
}

export function RiskComparison({ onTickerNavigate }: RiskComparisonProps) {
  const [tickers, setTickers] = useState<string[]>([]);
  const [tickerInput, setTickerInput] = useState('');
  const [days] = useState(90);
  const [benchmark] = useState('SPY');
  const [selectedPortfolioId, setSelectedPortfolioId] = useState<string | null>(null);
  const [helpDialogOpen, setHelpDialogOpen] = useState<string | null>(null);

  // Fetch portfolios for dropdown
  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  // Auto-select first portfolio
  useEffect(() => {
    if (!selectedPortfolioId && portfoliosQ.data?.length) {
      setSelectedPortfolioId(portfoliosQ.data[0].id);
    }
  }, [portfoliosQ.data, selectedPortfolioId]);

  // Fetch holdings for selected portfolio to get ticker suggestions
  const holdingsQ = useQuery({
    queryKey: ['portfolio-latest-holdings', selectedPortfolioId],
    queryFn: async () => {
      if (!selectedPortfolioId) return [];
      const response = await fetch(
        `http://localhost:3000/api/portfolios/${selectedPortfolioId}/latest-holdings`
      );
      if (!response.ok) throw new Error('Failed to fetch holdings');
      return response.json();
    },
    enabled: !!selectedPortfolioId,
  });

  // Get unique tickers from holdings
  const availableTickers: string[] = holdingsQ.data
    ? Array.from(
        new Set(
          holdingsQ.data
            .map((h: any) => h.ticker as string)
            .filter((t: string) => t && t !== 'CASH')
        )
      ).sort()
    : [];

  // Fetch risk data for all tickers using useQueries
  const riskQueries = useQueries({
    queries: tickers.map((ticker) => ({
      queryKey: ['risk', ticker, days, benchmark],
      queryFn: () => getPositionRisk(ticker, days, benchmark),
      staleTime: 1000 * 60 * 60, // 1 hour
      retry: false,
    })),
  });

  const handleAddTicker = (ticker?: string) => {
    const tickerToAdd = ticker || tickerInput.trim().toUpperCase();
    if (tickerToAdd && !tickers.includes(tickerToAdd) && tickers.length < 4) {
      setTickers([...tickers, tickerToAdd]);
      setTickerInput('');
    }
  };

  const handleRemoveTicker = (ticker: string) => {
    setTickers(tickers.filter((t) => t !== ticker));
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleAddTicker();
    }
  };

  const getRiskColor = (level: RiskLevel): string => {
    switch (level) {
      case 'low':
        return '#4caf50';
      case 'moderate':
        return '#ff9800';
      case 'high':
        return '#f44336';
    }
  };

  const getBestWorstIndicator = (values: number[], currentValue: number, lowerIsBetter: boolean) => {
    const sortedValues = [...values].sort((a, b) => (lowerIsBetter ? a - b : b - a));
    if (currentValue === sortedValues[0]) {
      return '🏆'; // Best
    } else if (currentValue === sortedValues[sortedValues.length - 1]) {
      return '⚠️'; // Worst
    }
    return '';
  };

  // Prepare comparison data
  const comparisonData = riskQueries
    .map((query, index) => ({
      ticker: tickers[index],
      data: query.data,
      isLoading: query.isLoading,
      isError: query.isError,
    }))
    .filter((item) => item.data);

  // Prepare chart data
  const volatilityData = comparisonData.map((item) => ({
    ticker: item.ticker,
    value: item.data?.metrics.volatility || 0,
    color: getRiskColor(item.data?.risk_level || 'low'),
  }));

  const drawdownData = comparisonData.map((item) => ({
    ticker: item.ticker,
    value: Math.abs(item.data?.metrics.max_drawdown || 0),
    color: getRiskColor(item.data?.risk_level || 'low'),
  }));

  const betaData = comparisonData
    .filter((item) => item.data?.metrics.beta !== null)
    .map((item) => ({
      ticker: item.ticker,
      value: item.data?.metrics.beta || 0,
      color: getRiskColor(item.data?.risk_level || 'low'),
    }));

  const riskScoreData = comparisonData.map((item) => ({
    ticker: item.ticker,
    value: item.data?.risk_score || 0,
    color: getRiskColor(item.data?.risk_level || 'low'),
  }));

  const exportToCSV = () => {
    if (comparisonData.length === 0) return;

    const headers = ['Metric', ...comparisonData.map((d) => d.ticker)];
    const rows = [
      ['Risk Score', ...comparisonData.map((d) => d.data?.risk_score.toFixed(1))],
      ['Risk Level', ...comparisonData.map((d) => d.data?.risk_level.toUpperCase())],
      ['Volatility (%)', ...comparisonData.map((d) => d.data?.metrics.volatility.toFixed(2))],
      ['Max Drawdown (%)', ...comparisonData.map((d) => d.data?.metrics.max_drawdown.toFixed(2))],
      ['Beta', ...comparisonData.map((d) => d.data?.metrics.beta?.toFixed(2) || 'N/A')],
      ['Sharpe Ratio', ...comparisonData.map((d) => d.data?.metrics.sharpe?.toFixed(2) || 'N/A')],
      ['VaR (%)', ...comparisonData.map((d) => d.data?.metrics.value_at_risk?.toFixed(2) || 'N/A')],
    ];

    const csv = [headers, ...rows].map((row) => row.join(',')).join('\n');
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `risk-comparison-${tickers.join('-')}.csv`;
    a.click();
  };

  const isLoading = riskQueries.some((q) => q.isLoading);

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <Compare sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Risk Comparison
        </Typography>
      </Box>

      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Compare Risk Metrics Across Tickers
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Add 2-4 ticker symbols to compare their risk profiles side-by-side. Select from your portfolio holdings or enter any ticker manually.
        </Typography>

        {/* Portfolio Selector */}
        {portfoliosQ.data && portfoliosQ.data.length > 0 && (
          <FormControl fullWidth sx={{ mb: 3 }}>
            <InputLabel>Portfolio</InputLabel>
            <Select
              value={selectedPortfolioId || ''}
              label="Portfolio"
              onChange={(e) => setSelectedPortfolioId(e.target.value)}
            >
              {portfoliosQ.data.map((portfolio) => (
                <MenuItem key={portfolio.id} value={portfolio.id}>
                  {portfolio.name}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        )}

        {/* Ticker Input Section */}
        <Grid container spacing={2} mb={2}>
          {/* Autocomplete for portfolio tickers */}
          {availableTickers.length > 0 && (
            <Grid item xs={12} md={6}>
              <Autocomplete
                options={availableTickers.filter((t) => !tickers.includes(t))}
                renderInput={(params) => (
                  <TextField {...params} label="Select from Portfolio" placeholder="Choose a ticker" />
                )}
                onChange={(_, value) => {
                  if (value && typeof value === 'string') handleAddTicker(value);
                }}
                disabled={tickers.length >= 4}
                value={null}
              />
            </Grid>
          )}

          {/* Manual ticker input */}
          <Grid item xs={12} md={availableTickers.length > 0 ? 6 : 12}>
            <Box display="flex" gap={2}>
              <TextField
                label="Add Custom Ticker"
                placeholder="e.g., AAPL"
                value={tickerInput}
                onChange={(e) => setTickerInput(e.target.value.toUpperCase())}
                onKeyPress={handleKeyPress}
                disabled={tickers.length >= 4}
                fullWidth
              />
              <Button
                variant="contained"
                startIcon={<Add />}
                onClick={() => handleAddTicker()}
                disabled={!tickerInput.trim() || tickers.length >= 4}
              >
                Add
              </Button>
            </Box>
          </Grid>
        </Grid>

        {tickers.length > 0 && (
          <Box display="flex" gap={1} flexWrap="wrap">
            {tickers.map((ticker) => (
              <Chip
                key={ticker}
                label={ticker}
                onDelete={() => handleRemoveTicker(ticker)}
                deleteIcon={<Close />}
                color="primary"
                variant="outlined"
              />
            ))}
          </Box>
        )}

        {tickers.length < 2 && (
          <Alert severity="info" sx={{ mt: 2 }}>
            Add at least 2 tickers to start comparison (maximum 4).
          </Alert>
        )}
      </Paper>

      {isLoading && (
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', py: 4 }}>
          <CircularProgress />
          <Typography sx={{ ml: 2 }}>Loading risk data...</Typography>
        </Box>
      )}

      {tickers.length >= 2 && !isLoading && comparisonData.length > 0 && (
        <>
          {/* Export Button */}
          <Box display="flex" justifyContent="flex-end" mb={2}>
            <Button
              variant="outlined"
              startIcon={<Download />}
              onClick={exportToCSV}
            >
              Export to CSV
            </Button>
          </Box>

          {/* Comparison Table */}
          <Paper sx={{ mb: 3 }}>
            <Box sx={{ p: 2 }}>
              <Typography variant="h6">Metric Comparison</Typography>
            </Box>
            <TableContainer>
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell><strong>Metric</strong></TableCell>
                    {comparisonData.map((item) => (
                      <TableCell key={item.ticker} align="right">
                        {onTickerNavigate ? (
                          <TickerActionMenu
                            ticker={item.ticker}
                            variant="text"
                            onNavigate={onTickerNavigate}
                          />
                        ) : (
                          <strong>{item.ticker}</strong>
                        )}
                      </TableCell>
                    ))}
                  </TableRow>
                </TableHead>
                <TableBody>
                  <TableRow>
                    <TableCell>
                      <Box display="flex" alignItems="center" gap={0.5}>
                        Risk Score
                        <IconButton
                          size="small"
                          onClick={() => setHelpDialogOpen('risk_score')}
                          sx={{ p: 0.5 }}
                        >
                          <HelpOutline sx={{ fontSize: 14 }} />
                        </IconButton>
                      </Box>
                    </TableCell>
                    {comparisonData.map((item) => {
                      const scores = comparisonData.map((d) => d.data?.risk_score || 0);
                      const indicator = getBestWorstIndicator(scores, item.data?.risk_score || 0, true);
                      return (
                        <TableCell key={item.ticker} align="right">
                          {indicator} {item.data?.risk_score.toFixed(1)}
                        </TableCell>
                      );
                    })}
                  </TableRow>
                  <TableRow>
                    <TableCell>Risk Level</TableCell>
                    {comparisonData.map((item) => (
                      <TableCell key={item.ticker} align="right">
                        <Chip
                          label={item.data?.risk_level.toUpperCase()}
                          size="small"
                          sx={{
                            backgroundColor: getRiskColor(item.data?.risk_level || 'low'),
                            color: 'white',
                            fontWeight: 'bold',
                          }}
                        />
                      </TableCell>
                    ))}
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Box display="flex" alignItems="center" gap={0.5}>
                        Volatility (%)
                        <IconButton
                          size="small"
                          onClick={() => setHelpDialogOpen('volatility')}
                          sx={{ p: 0.5 }}
                        >
                          <HelpOutline sx={{ fontSize: 14 }} />
                        </IconButton>
                      </Box>
                    </TableCell>
                    {comparisonData.map((item) => {
                      const values = comparisonData.map((d) => d.data?.metrics.volatility || 0);
                      const indicator = getBestWorstIndicator(values, item.data?.metrics.volatility || 0, true);
                      return (
                        <TableCell key={item.ticker} align="right">
                          {indicator} {item.data?.metrics.volatility.toFixed(2)}%
                        </TableCell>
                      );
                    })}
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Box display="flex" alignItems="center" gap={0.5}>
                        Max Drawdown (%)
                        <IconButton
                          size="small"
                          onClick={() => setHelpDialogOpen('max_drawdown')}
                          sx={{ p: 0.5 }}
                        >
                          <HelpOutline sx={{ fontSize: 14 }} />
                        </IconButton>
                      </Box>
                    </TableCell>
                    {comparisonData.map((item) => {
                      const values = comparisonData.map((d) => d.data?.metrics.max_drawdown || 0);
                      const indicator = getBestWorstIndicator(values, item.data?.metrics.max_drawdown || 0, false);
                      return (
                        <TableCell key={item.ticker} align="right">
                          {indicator} {item.data?.metrics.max_drawdown.toFixed(2)}%
                        </TableCell>
                      );
                    })}
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Box display="flex" alignItems="center" gap={0.5}>
                        Beta
                        <IconButton
                          size="small"
                          onClick={() => setHelpDialogOpen('beta')}
                          sx={{ p: 0.5 }}
                        >
                          <HelpOutline sx={{ fontSize: 14 }} />
                        </IconButton>
                      </Box>
                    </TableCell>
                    {comparisonData.map((item) => (
                      <TableCell key={item.ticker} align="right">
                        {item.data?.metrics.beta !== null
                          ? item.data?.metrics.beta.toFixed(2)
                          : 'N/A'}
                      </TableCell>
                    ))}
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Box display="flex" alignItems="center" gap={0.5}>
                        Sharpe Ratio
                        <IconButton
                          size="small"
                          onClick={() => setHelpDialogOpen('sharpe_ratio')}
                          sx={{ p: 0.5 }}
                        >
                          <HelpOutline sx={{ fontSize: 14 }} />
                        </IconButton>
                      </Box>
                    </TableCell>
                    {comparisonData.map((item) => {
                      const values = comparisonData
                        .filter((d) => d.data?.metrics.sharpe !== null)
                        .map((d) => d.data?.metrics.sharpe || 0);
                      const currentValue = item.data?.metrics.sharpe;
                      const indicator =
                        currentValue !== null && currentValue !== undefined
                          ? getBestWorstIndicator(values, currentValue, false)
                          : '';
                      return (
                        <TableCell key={item.ticker} align="right">
                          {indicator}{' '}
                          {item.data?.metrics.sharpe !== null
                            ? item.data?.metrics.sharpe.toFixed(2)
                            : 'N/A'}
                        </TableCell>
                      );
                    })}
                  </TableRow>
                  <TableRow>
                    <TableCell>
                      <Tooltip title="5% Value at Risk - potential 1-day loss">
                        <span>VaR (%)</span>
                      </Tooltip>
                    </TableCell>
                    {comparisonData.map((item) => (
                      <TableCell key={item.ticker} align="right">
                        {item.data?.metrics.value_at_risk !== null
                          ? `${item.data?.metrics.value_at_risk.toFixed(2)}%`
                          : 'N/A'}
                      </TableCell>
                    ))}
                  </TableRow>
                </TableBody>
              </Table>
            </TableContainer>
          </Paper>

          {/* Visual Charts */}
          <Grid container spacing={3}>
            {/* Volatility Comparison */}
            <Grid item xs={12} md={6}>
              <Paper sx={{ p: 3 }}>
                <Typography variant="h6" gutterBottom>
                  Volatility Comparison
                </Typography>
                <Box sx={{ minHeight: 250 }}>
                  <ResponsiveContainer width="100%" height={250}>
                    <BarChart data={volatilityData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="ticker" />
                    <YAxis label={{ value: 'Volatility (%)', angle: -90, position: 'insideLeft' }} />
                    <RechartsTooltip formatter={(value: number | undefined) => value !== undefined ? `${value.toFixed(2)}%` : ''} />
                    <Bar dataKey="value" name="Volatility">
                      {volatilityData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} />
                      ))}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
                </Box>
                <Typography variant="caption" color="text.secondary" display="block" textAlign="center" mt={1}>
                  Lower volatility indicates more stable prices
                </Typography>
              </Paper>
            </Grid>

            {/* Drawdown Comparison */}
            <Grid item xs={12} md={6}>
              <Paper sx={{ p: 3 }}>
                <Typography variant="h6" gutterBottom>
                  Maximum Drawdown Comparison
                </Typography>
                <Box sx={{ minHeight: 250 }}>
                  <ResponsiveContainer width="100%" height={250}>
                    <BarChart data={drawdownData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="ticker" />
                    <YAxis label={{ value: 'Max Drawdown (%)', angle: -90, position: 'insideLeft' }} />
                    <RechartsTooltip formatter={(value: number | undefined) => value !== undefined ? `-${value.toFixed(2)}%` : ''} />
                    <Bar dataKey="value" name="Max Drawdown">
                      {drawdownData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} />
                      ))}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
                </Box>
                <Typography variant="caption" color="text.secondary" display="block" textAlign="center" mt={1}>
                  Lower drawdown indicates smaller historical losses
                </Typography>
              </Paper>
            </Grid>

            {/* Beta Comparison */}
            {betaData.length > 0 && (
              <Grid item xs={12} md={6}>
                <Paper sx={{ p: 3 }}>
                  <Typography variant="h6" gutterBottom>
                    Beta Comparison (vs {benchmark})
                  </Typography>
                  <Box sx={{ minHeight: 250 }}>
                    <ResponsiveContainer width="100%" height={250}>
                      <BarChart data={betaData}>
                      <CartesianGrid strokeDasharray="3 3" />
                      <XAxis dataKey="ticker" />
                      <YAxis label={{ value: 'Beta', angle: -90, position: 'insideLeft' }} />
                      <RechartsTooltip formatter={(value: number | undefined) => value !== undefined ? value.toFixed(2) : ''} />
                      <Bar dataKey="value" name="Beta">
                        {betaData.map((entry, index) => (
                          <Cell key={`cell-${index}`} fill={entry.color} />
                        ))}
                      </Bar>
                    </BarChart>
                  </ResponsiveContainer>
                  </Box>
                  <Typography variant="caption" color="text.secondary" display="block" textAlign="center" mt={1}>
                    Beta &gt; 1.0 is more volatile than market, &lt; 1.0 is less volatile
                  </Typography>
                </Paper>
              </Grid>
            )}

            {/* Risk Score Comparison */}
            <Grid item xs={12} md={6}>
              <Paper sx={{ p: 3 }}>
                <Typography variant="h6" gutterBottom>
                  Overall Risk Score Comparison
                </Typography>
                <Box sx={{ minHeight: 250 }}>
                  <ResponsiveContainer width="100%" height={250}>
                    <BarChart data={riskScoreData}>
                    <CartesianGrid strokeDasharray="3 3" />
                    <XAxis dataKey="ticker" />
                    <YAxis label={{ value: 'Risk Score', angle: -90, position: 'insideLeft' }} domain={[0, 100]} />
                    <RechartsTooltip formatter={(value: number | undefined) => value !== undefined ? value.toFixed(1) : ''} />
                    <Bar dataKey="value" name="Risk Score">
                      {riskScoreData.map((entry, index) => (
                        <Cell key={`cell-${index}`} fill={entry.color} />
                      ))}
                    </Bar>
                  </BarChart>
                </ResponsiveContainer>
                </Box>
                <Typography variant="caption" color="text.secondary" display="block" textAlign="center" mt={1}>
                  0-40: Low Risk • 40-60: Moderate • 60-100: High Risk
                </Typography>
              </Paper>
            </Grid>
          </Grid>

          <Alert severity="info" sx={{ mt: 3 }}>
            <Typography variant="caption">
              <strong>Legend:</strong> 🏆 = Best (lowest risk) • ⚠️ = Worst (highest risk) • Green = Low Risk • Orange = Moderate Risk • Red = High Risk
            </Typography>
          </Alert>
        </>
      )}

      {riskQueries.some((q) => q.isError) && (
        <Alert severity="warning" sx={{ mt: 2 }}>
          Some tickers failed to load. They may not have sufficient price history or could be invalid symbols.
        </Alert>
      )}

      {/* Help Dialog */}
      {helpDialogOpen && (
        <MetricHelpDialog
          open={true}
          onClose={() => setHelpDialogOpen(null)}
          metricKey={helpDialogOpen}
        />
      )}
    </Box>
  );
}
