import { useState, useMemo, useEffect } from 'react';
import {
  Box,
  Typography,
  Paper,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Grid,
  Alert,
  FormHelperText,
  ToggleButtonGroup,
  ToggleButton,
} from '@mui/material';
import { Timeline, Science } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { listPortfolios, listAccounts, getLatestHoldings } from '../lib/endpoints';
import { RollingBetaChart } from './RollingBetaChart';
import { BetaForecastChart } from './BetaForecastChart';

interface RollingBetaPageProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
  initialTicker?: string;
}

const BENCHMARKS = [
  { value: 'SPY', label: 'SPY - S&P 500' },
  { value: 'QQQ', label: 'QQQ - Nasdaq 100' },
  { value: 'IWM', label: 'IWM - Russell 2000' },
  { value: 'DIA', label: 'DIA - Dow Jones' },
  { value: 'VTI', label: 'VTI - Total Stock Market' },
  { value: 'AGG', label: 'AGG - Total Bond Market' },
];

export function RollingBetaPage({
  selectedPortfolioId,
  onPortfolioChange,
  initialTicker
}: RollingBetaPageProps) {
  const [selectedTicker, setSelectedTicker] = useState<string>(initialTicker || '');
  const [selectedBenchmark, setSelectedBenchmark] = useState<string>('SPY');
  const [days] = useState<number>(180);
  const [view, setView] = useState<'historical' | 'forecast'>('historical');
  const [shouldAutoCalculate, setShouldAutoCalculate] = useState(!!initialTicker);

  // Handle initialTicker changes
  useEffect(() => {
    if (initialTicker) {
      setSelectedTicker(initialTicker);
      setShouldAutoCalculate(true);
    }
  }, [initialTicker]);

  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const accountsQ = useQuery({
    queryKey: ['accounts', selectedPortfolioId],
    queryFn: () => listAccounts(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  // Fetch holdings for all accounts to get list of tickers
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

  // Get unique tickers from holdings, plus initialTicker if provided
  const availableTickers = useMemo(() => {
    const tickerSet = new Set<string>();

    // Add initialTicker if provided (for navigation from ticker menu)
    if (initialTicker) {
      tickerSet.add(initialTicker);
    }

    // Add tickers from holdings
    if (holdingsQueries.data) {
      holdingsQueries.data.forEach((holding) => {
        if (holding.ticker) {
          tickerSet.add(holding.ticker);
        }
      });
    }

    return Array.from(tickerSet).sort();
  }, [holdingsQueries.data, initialTicker]);

  // Check if selected ticker might be unsupported
  const getTickerWarning = (ticker: string): string | null => {
    if (!holdingsQueries.data) return null;

    const holding = holdingsQueries.data.find(h => h.ticker === ticker);
    if (!holding) return null;

    const category = holding.asset_category?.toLowerCase() || '';
    const name = holding.holding_name?.toLowerCase() || '';

    // Check for mutual funds, money market, bonds, etc.
    if (category.includes('mutual') || category.includes('fund') ||
        name.includes('money market') || name.includes('mutual fund') ||
        category.includes('bond') || category.includes('fixed income')) {
      return 'This ticker appears to be a mutual fund, bond, or money market fund. Historical price data may not be available.';
    }

    return null;
  };

  // Auto-select first ticker if none selected
  useMemo(() => {
    if (!selectedTicker && availableTickers.length > 0) {
      setSelectedTicker(availableTickers[0]);
    }
  }, [availableTickers, selectedTicker]);

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Rolling Beta Analysis
      </Typography>

      <Paper sx={{ p: 3, mb: 3 }}>
        <Grid container spacing={3}>
          {/* Portfolio Selector */}
          <Grid item xs={12} sm={4}>
            <FormControl fullWidth>
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
          </Grid>

          {/* Ticker Selector */}
          <Grid item xs={12} sm={4}>
            <FormControl fullWidth disabled={availableTickers.length === 0}>
              <InputLabel>Ticker</InputLabel>
              <Select
                value={selectedTicker}
                onChange={(e) => {
                  setSelectedTicker(e.target.value);
                  setShouldAutoCalculate(false); // Don't auto-calculate when manually changing
                }}
                label="Ticker"
              >
                {availableTickers.map((ticker) => (
                  <MenuItem key={ticker} value={ticker}>
                    {ticker}
                  </MenuItem>
                ))}
              </Select>
              <FormHelperText>
                {initialTicker && !holdingsQueries.data?.some(h => h.ticker === initialTicker)
                  ? `Showing ${initialTicker} (not in portfolio)`
                  : 'Best results with stocks and ETFs'}
              </FormHelperText>
            </FormControl>
          </Grid>

          {/* Benchmark Selector */}
          <Grid item xs={12} sm={4}>
            <FormControl fullWidth>
              <InputLabel>Benchmark</InputLabel>
              <Select
                value={selectedBenchmark}
                onChange={(e) => setSelectedBenchmark(e.target.value)}
                label="Benchmark"
              >
                {BENCHMARKS.map((benchmark) => (
                  <MenuItem key={benchmark.value} value={benchmark.value}>
                    {benchmark.label}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Grid>
        </Grid>

        {/* View Toggle */}
        {selectedTicker && (
          <Box sx={{ mt: 2, mb: 2 }}>
            <ToggleButtonGroup
              value={view}
              exclusive
              onChange={(_, newView) => newView && setView(newView)}
              size="small"
              fullWidth
            >
              <ToggleButton value="historical">
                <Timeline sx={{ mr: 1 }} />
                Historical Beta
              </ToggleButton>
              <ToggleButton value="forecast">
                <Science sx={{ mr: 1 }} />
                Beta Forecast
              </ToggleButton>
            </ToggleButtonGroup>
          </Box>
        )}

        <Alert severity="info" sx={{ mt: 2 }}>
          {view === 'historical'
            ? 'Rolling beta shows how a stock\'s correlation with a benchmark changes over time. Select a ticker from your portfolio and a benchmark to analyze.'
            : 'Beta forecasting predicts future beta values using historical patterns, mean reversion, and trend analysis.'
          }
        </Alert>

        {selectedTicker && getTickerWarning(selectedTicker) && (
          <Alert severity="warning" sx={{ mt: 2 }}>
            {getTickerWarning(selectedTicker)}
          </Alert>
        )}
      </Paper>

      {/* Render Chart Based on View */}
      {selectedTicker ? (
        view === 'historical' ? (
          <RollingBetaChart
            ticker={selectedTicker}
            benchmark={selectedBenchmark}
            days={days}
            autoCalculate={shouldAutoCalculate}
          />
        ) : (
          <BetaForecastChart
            ticker={selectedTicker}
            benchmark={selectedBenchmark}
          />
        )
      ) : selectedPortfolioId ? (
        <Alert severity="info">
          Please select a ticker to view rolling beta analysis.
        </Alert>
      ) : (
        <Alert severity="info">
          Please select a portfolio to view rolling beta analysis.
        </Alert>
      )}
    </Box>
  );
}
