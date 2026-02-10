import { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Paper,
  TextField,
  Button,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Grid,
  Alert,
  Tabs,
  Tab,
} from '@mui/material';
import { Assessment, Search } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { searchTickers } from '../lib/endpoints';
import { RiskMetricsPanel } from './RiskMetricsPanel';
import { PriceHistoryChart } from './PriceHistoryChart';
import { RiskChart } from './RiskChart';

interface RiskAnalysisProps {
  selectedTicker?: string | null;
}

export function RiskAnalysis({ selectedTicker }: RiskAnalysisProps) {
  const [ticker, setTicker] = useState('');
  const [searchTicker, setSearchTicker] = useState('');
  const [days, setDays] = useState(90);
  const [benchmark, setBenchmark] = useState('SPY');
  const [activeTab, setActiveTab] = useState(0);

  // Fetch company name when ticker is searched
  const companyInfoQ = useQuery({
    queryKey: ['companyInfo', searchTicker],
    queryFn: () => searchTickers(searchTicker),
    enabled: !!searchTicker,
    staleTime: 1000 * 60 * 60, // 1 hour
  });

  const companyName = companyInfoQ.data?.[0]?.name || null;

  // Auto-populate and search when selectedTicker changes
  useEffect(() => {
    if (selectedTicker) {
      setTicker(selectedTicker);
      setSearchTicker(selectedTicker);
      setActiveTab(0); // Reset to Risk Metrics tab
    }
  }, [selectedTicker]);

  const handleSearch = () => {
    if (ticker.trim()) {
      setSearchTicker(ticker.trim().toUpperCase());
      setActiveTab(0); // Reset to Risk Metrics tab
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <Assessment sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Risk Analysis
        </Typography>
      </Box>

      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Search Position Risk Metrics
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Enter a ticker symbol to analyze its risk profile including volatility, drawdown, beta, and other metrics.
        </Typography>

        <Grid container spacing={2} alignItems="center">
          <Grid item xs={12} sm={4}>
            <TextField
              fullWidth
              label="Ticker Symbol"
              placeholder="e.g., AAPL, MSFT"
              value={ticker}
              onChange={(e) => setTicker(e.target.value.toUpperCase())}
              onKeyPress={handleKeyPress}
              variant="outlined"
            />
          </Grid>

          <Grid item xs={12} sm={3}>
            <FormControl fullWidth>
              <InputLabel>Time Period</InputLabel>
              <Select
                value={days}
                label="Time Period"
                onChange={(e) => setDays(Number(e.target.value))}
              >
                <MenuItem value={30}>30 Days</MenuItem>
                <MenuItem value={60}>60 Days</MenuItem>
                <MenuItem value={90}>90 Days</MenuItem>
                <MenuItem value={180}>180 Days</MenuItem>
                <MenuItem value={365}>1 Year</MenuItem>
              </Select>
            </FormControl>
          </Grid>

          <Grid item xs={12} sm={3}>
            <FormControl fullWidth>
              <InputLabel>Benchmark</InputLabel>
              <Select
                value={benchmark}
                label="Benchmark"
                onChange={(e) => setBenchmark(e.target.value)}
              >
                <MenuItem value="SPY">S&P 500 (SPY)</MenuItem>
                <MenuItem value="QQQ">NASDAQ-100 (QQQ)</MenuItem>
                <MenuItem value="DIA">Dow Jones (DIA)</MenuItem>
                <MenuItem value="IWM">Russell 2000 (IWM)</MenuItem>
              </Select>
            </FormControl>
          </Grid>

          <Grid item xs={12} sm={2}>
            <Button
              fullWidth
              variant="contained"
              size="large"
              startIcon={<Search />}
              onClick={handleSearch}
              disabled={!ticker.trim()}
            >
              Analyze
            </Button>
          </Grid>
        </Grid>
      </Paper>

      {!searchTicker && (
        <Alert severity="info">
          Enter a ticker symbol above to view detailed risk analysis. Risk metrics include:
          <ul style={{ marginTop: '8px', marginBottom: 0 }}>
            <li><strong>Volatility:</strong> Measures price fluctuation intensity</li>
            <li><strong>Maximum Drawdown:</strong> Worst peak-to-trough decline</li>
            <li><strong>Beta:</strong> Correlation with market benchmark</li>
            <li><strong>Sharpe Ratio:</strong> Risk-adjusted return performance</li>
            <li><strong>Value at Risk (VaR):</strong> Potential loss at 5% confidence level</li>
          </ul>
        </Alert>
      )}

      {searchTicker && (
        <Box>
          <Box sx={{ mb: 3 }}>
            <Typography variant="h5" fontWeight="bold">
              {searchTicker} Risk Analysis
            </Typography>
            {companyName && (
              <Typography variant="body1" color="text.secondary" sx={{ mt: 0.5 }}>
                {companyName}
              </Typography>
            )}
          </Box>

          {/* Tabs */}
          <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 3 }}>
            <Tabs value={activeTab} onChange={(_, val) => setActiveTab(val)}>
              <Tab label="Risk Metrics" />
              <Tab label="Price History" />
              <Tab label="Risk Trends" />
            </Tabs>
          </Box>

          {/* Risk Metrics Tab */}
          {activeTab === 0 && (
            <RiskMetricsPanel ticker={searchTicker} days={days} benchmark={benchmark} />
          )}

          {/* Price History Tab */}
          {activeTab === 1 && (
            <PriceHistoryChart ticker={searchTicker} days={days} companyName={companyName} />
          )}

          {/* Risk Trends Tab */}
          {activeTab === 2 && (
            <RiskChart ticker={searchTicker} days={days} />
          )}
        </Box>
      )}
    </Box>
  );
}
