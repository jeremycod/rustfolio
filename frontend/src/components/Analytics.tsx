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
} from '@mui/material';
import { useQuery } from '@tanstack/react-query';
import { getAnalytics, listPortfolios } from '../lib/endpoints';
import { PortfolioChart } from './PortfolioChart';

interface AnalyticsProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
}

export function Analytics({ selectedPortfolioId, onPortfolioChange }: AnalyticsProps) {
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

  const handleOverlayChange = (overlay: keyof typeof overlays) => {
    setOverlays(prev => ({ ...prev, [overlay]: !prev[overlay] }));
  };

  const formatCurrency = (value: number) => `$${value.toFixed(2)}`;
  const formatPercent = (value: number) => `${value.toFixed(2)}%`;

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
                ) : (
                  // Portfolio metrics
                  <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Total Value:</Typography>
                      <Typography variant="body2">{formatCurrency(15750.00)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Total Cost:</Typography>
                      <Typography variant="body2">{formatCurrency(15000.00)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Total P/L:</Typography>
                      <Typography variant="body2" color="success.main">{formatCurrency(750.00)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Return %:</Typography>
                      <Typography variant="body2" color="success.main">{formatPercent(5.00)}</Typography>
                    </Box>
                    <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                      <Typography variant="body2" color="text.secondary">Sharpe Ratio:</Typography>
                      <Typography variant="body2">1.25</Typography>
                    </Box>
                  </Box>
                )}
              </CardContent>
            </Card>

            {/* Risk Metrics */}
            <Card>
              <CardContent>
                <Typography variant="h6" gutterBottom>
                  Risk Analysis
                </Typography>
                <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" color="text.secondary">Volatility:</Typography>
                    <Typography variant="body2">{formatPercent(18.5)}</Typography>
                  </Box>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" color="text.secondary">Beta:</Typography>
                    <Typography variant="body2">1.15</Typography>
                  </Box>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
                    <Typography variant="body2" color="text.secondary">VaR (95%):</Typography>
                    <Typography variant="body2" color="error.main">{formatCurrency(-450.00)}</Typography>
                  </Box>
                </Box>
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
    </Box>
  );
}