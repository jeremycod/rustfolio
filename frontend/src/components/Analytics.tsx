import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  Grid,
  Chip,
  Alert,
  ToggleButton,
  ToggleButtonGroup,
  FormControlLabel,
  Switch,
  Card,
  CardContent,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Tabs,
  Tab,
} from '@mui/material';
import { useQuery } from '@tanstack/react-query';
import { getAnalytics, listPortfolios, getPortfolioHistory, getPortfolioRisk } from '../lib/endpoints';
import { PortfolioChart } from './PortfolioChart';
import { ForecastChart } from './ForecastChart';

interface AnalyticsProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
}

export function Analytics({ selectedPortfolioId, onPortfolioChange }: AnalyticsProps) {
  const [currentTab, setCurrentTab] = useState(0);
  const [dateRange, setDateRange] = useState('3m');
  const [selectedTicker, setSelectedTicker] = useState<string | null>(null);
  const [overlays, setOverlays] = useState({
    sma: true,
    ema: true,
    trend: true,
    bollinger: false,
  });

  const analyticsQ = useQuery({
    queryKey: ['analytics', selectedPortfolioId],
    queryFn: () => getAnalytics(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const historyQ = useQuery({
    queryKey: ['portfolio-history', selectedPortfolioId],
    queryFn: () => getPortfolioHistory(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  const riskQ = useQuery({
    queryKey: ['portfolio-risk', selectedPortfolioId],
    queryFn: () => getPortfolioRisk(selectedPortfolioId!, 90), // Use 90 days for faster calculation
    enabled: !!selectedPortfolioId,
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  const handleOverlayChange = (overlay: keyof typeof overlays) => {
    setOverlays(prev => ({ ...prev, [overlay]: !prev[overlay] }));
  };

  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD',
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value);
  };
  const formatPercent = (value: number) => `${value.toFixed(2)}%`;

  // Calculate current portfolio metrics from history
  const portfolioMetrics = historyQ.data ? (() => {
    // Group by date and sum values
    const byDate = historyQ.data.reduce((acc, record) => {
      const date = record.snapshot_date;
      if (!acc[date]) {
        acc[date] = { totalValue: 0, totalCost: 0 };
      }
      acc[date].totalValue += parseFloat(record.total_value);
      acc[date].totalCost += parseFloat(record.total_cost);
      return acc;
    }, {} as Record<string, { totalValue: number; totalCost: number }>);

    // Get latest date
    const dates = Object.keys(byDate).sort();
    const latestDate = dates[dates.length - 1];
    const latest = byDate[latestDate];

    const totalValue = latest.totalValue;
    const totalCost = latest.totalCost;
    const totalPL = totalValue - totalCost;
    const returnPct = totalCost > 0 ? (totalPL / totalCost) * 100 : 0;

    return { totalValue, totalCost, totalPL, returnPct };
  })() : null;

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Analytics
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

      {/* Tabs */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 3 }}>
        <Tabs value={currentTab} onChange={(_, newValue) => setCurrentTab(newValue)}>
          <Tab label="Portfolio Performance" />
          <Tab label="Portfolio Value Forecast" />
        </Tabs>
      </Box>

      {/* Tab Panel 1: Portfolio Performance */}
      {currentTab === 0 && (
        <Grid container spacing={3}>
        {/* Main Chart */}
        <Grid item xs={12} lg={8}>
          <Paper sx={{ p: 3 }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 2 }}>
              <Typography variant="h6">
                {selectedTicker ? `${selectedTicker} Analysis` : 'Portfolio Performance'}
              </Typography>
              
              {/* Date Range Controls */}
              <ToggleButtonGroup
                size="small"
                value={dateRange}
                exclusive
                onChange={(_, value) => value && setDateRange(value)}
              >
                <ToggleButton value="1m">1M</ToggleButton>
                <ToggleButton value="3m">3M</ToggleButton>
                <ToggleButton value="6m">6M</ToggleButton>
                <ToggleButton value="1y">1Y</ToggleButton>
              </ToggleButtonGroup>
            </Box>

            {/* Chart Overlays Toggle */}
            <Box sx={{ display: 'flex', gap: 2, mb: 2, flexWrap: 'wrap' }}>
              <FormControlLabel
                control={<Switch size="small" checked={overlays.sma} onChange={() => handleOverlayChange('sma')} />}
                label="SMA"
              />
              <FormControlLabel
                control={<Switch size="small" checked={overlays.ema} onChange={() => handleOverlayChange('ema')} />}
                label="EMA"
              />
              <FormControlLabel
                control={<Switch size="small" checked={overlays.trend} onChange={() => handleOverlayChange('trend')} />}
                label="Trendline"
              />
              <FormControlLabel
                control={<Switch size="small" checked={overlays.bollinger} onChange={() => handleOverlayChange('bollinger')} />}
                label="Bollinger Bands"
              />
            </Box>

            {/* Points and Range Summary */}
            {analyticsQ.data && (
              <Box sx={{ display: 'flex', gap: 1, mb: 2 }}>
                <Chip 
                  label={`Points: ${analyticsQ.data.meta.points}`} 
                  variant="outlined" 
                  size="small"
                />
                <Chip 
                  label={`Range: ${analyticsQ.data.meta.start ?? '—'} → ${analyticsQ.data.meta.end ?? '—'}`}
                  variant="outlined"
                  size="small"
                />
              </Box>
            )}

            {analyticsQ.isError && (
              <Alert severity="error">
                Failed to load analytics. Make sure you have price data for tickers in this portfolio.
              </Alert>
            )}

            {analyticsQ.data && analyticsQ.data.meta.points < 10 && (
              <Alert severity="warning" sx={{ mb: 2 }}>
                <strong>Limited Historical Data</strong><br />
                You only have {analyticsQ.data.meta.points} data point{analyticsQ.data.meta.points !== 1 ? 's' : ''}.
                Technical indicators (SMA, EMA) require at least 20 points to be meaningful.
                Please import more CSV snapshots from your brokerage to see trend analysis.
              </Alert>
            )}

            {analyticsQ.data ? (
              <PortfolioChart series={analyticsQ.data.series} />
            ) : (
              <Alert severity="info">
                No analytics data available. Add positions and generate price data to see charts.
              </Alert>
            )}
          </Paper>
        </Grid>

        {/* Side Panel */}
        <Grid item xs={12} lg={4}>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {/* Performance Metrics */}
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  {selectedTicker ? `${selectedTicker} Stats` : 'Portfolio Metrics'}
                </Typography>
                
                {selectedTicker ? (
                  // Ticker-specific stats
                  <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Current Price:</Typography>
                      <Typography variant="body2">{formatCurrency(157.50)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Cost Basis:</Typography>
                      <Typography variant="body2">{formatCurrency(150.00)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Unrealized P/L:</Typography>
                      <Typography variant="body2" color="success.main">{formatCurrency(75.00)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Return %:</Typography>
                      <Typography variant="body2" color="success.main">{formatPercent(5.00)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Max Drawdown:</Typography>
                      <Typography variant="body2" color="error.main">{formatPercent(-2.30)}</Typography>
                    </Box>
                  </Box>
                ) : portfolioMetrics ? (
                  // Portfolio metrics (from real data)
                  <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Total Value:</Typography>
                      <Typography variant="body2">{formatCurrency(portfolioMetrics.totalValue)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Total Cost:</Typography>
                      <Typography variant="body2">{formatCurrency(portfolioMetrics.totalCost)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Total P/L:</Typography>
                      <Typography variant="body2" color={portfolioMetrics.totalPL >= 0 ? "success.main" : "error.main"}>
                        {formatCurrency(portfolioMetrics.totalPL)}
                      </Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Return %:</Typography>
                      <Typography variant="body2" color={portfolioMetrics.returnPct >= 0 ? "success.main" : "error.main"}>
                        {formatPercent(portfolioMetrics.returnPct)}
                      </Typography>
                    </Box>
                  </Box>
                ) : (
                  <Typography variant="body2" color="text.secondary">
                    Loading portfolio metrics...
                  </Typography>
                )}
              </CardContent>
            </Card>

            {/* Risk Metrics */}
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Risk Analysis
                </Typography>
                {riskQ.isLoading ? (
                  <Typography variant="body2" color="text.secondary">
                    Loading risk metrics...
                  </Typography>
                ) : riskQ.isError ? (
                  <Alert severity="error" sx={{ mt: 1 }}>
                    Failed to load risk metrics. {(riskQ.error as Error)?.message || 'Unknown error'}
                  </Alert>
                ) : riskQ.data ? (
                  <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Volatility:</Typography>
                      <Typography variant="body2">{formatPercent(riskQ.data.portfolio_volatility)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Beta (SPY):</Typography>
                      <Typography variant="body2">
                        {riskQ.data.portfolio_beta?.toFixed(2) ?? 'N/A'}
                      </Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Sharpe Ratio:</Typography>
                      <Typography variant="body2">
                        {riskQ.data.portfolio_sharpe?.toFixed(2) ?? 'N/A'}
                      </Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Max Drawdown:</Typography>
                      <Typography variant="body2" color="error.main">
                        {formatPercent(riskQ.data.portfolio_max_drawdown)}
                      </Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Risk Score:</Typography>
                      <Typography variant="body2">
                        {riskQ.data.portfolio_risk_score.toFixed(1)}
                      </Typography>
                    </Box>
                  </Box>
                ) : null}
              </CardContent>
            </Card>

            {/* Allocation Summary */}
            {!selectedTicker && analyticsQ.data?.allocations && (
              <Card>
                <CardContent>
                  <Typography variant="h6" gutterBottom>
                    Allocation
                  </Typography>
                  {analyticsQ.data.allocations.map((alloc) => {
                    const weight = alloc.weight * 100;
                    return (
                      <Box key={alloc.ticker} sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                        <Typography variant="body2">{alloc.ticker}:</Typography>
                        <Typography variant="body2">{formatPercent(weight)}</Typography>
                      </Box>
                    );
                  })}
                </CardContent>
              </Card>
            )}
          </Box>
        </Grid>
      </Grid>
      )}

      {/* Tab Panel 2: Portfolio Value Forecast */}
      {currentTab === 1 && selectedPortfolioId && (
        <Box>
          <ForecastChart portfolioId={selectedPortfolioId} />
        </Box>
      )}

      {currentTab === 1 && !selectedPortfolioId && (
        <Alert severity="info">
          Please select a portfolio to view the forecast.
        </Alert>
      )}
    </Box>
  );
}