import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  CircularProgress,
  Alert,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Tooltip,
  Grid,
} from '@mui/material';
import { useQuery } from '@tanstack/react-query';
import { getPortfolioCorrelations, listPortfolios } from '../lib/endpoints';
import { CorrelationMatrix } from '../types';

type CorrelationHeatmapProps = {
  portfolioId: string;
};

export function CorrelationHeatmap({ portfolioId: initialPortfolioId }: CorrelationHeatmapProps) {
  const [days, setDays] = useState(30);
  const [selectedPortfolioId, setSelectedPortfolioId] = useState(initialPortfolioId);

  // Fetch portfolios for dropdown
  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const correlationQ = useQuery({
    queryKey: ['portfolio-correlations', selectedPortfolioId, days],
    queryFn: () => getPortfolioCorrelations(selectedPortfolioId, days),
    staleTime: 1000 * 60 * 60, // 1 hour
    gcTime: 1000 * 60 * 60, // Keep in cache for 1 hour
    retry: 1, // Only retry once
    enabled: !!selectedPortfolioId,
  });

  const getCorrelation = (matrix: CorrelationMatrix, ticker1: string, ticker2: string): number | null => {
    if (ticker1 === ticker2) return 1.0; // Diagonal is always 1.0

    // Check both directions since we store upper triangle only
    const pair = matrix.correlations.find(
      (c) =>
        (c.ticker1 === ticker1 && c.ticker2 === ticker2) ||
        (c.ticker1 === ticker2 && c.ticker2 === ticker1)
    );

    return pair ? pair.correlation : null;
  };

  const getCorrelationColor = (correlation: number | null): string => {
    if (correlation === null) return '#e0e0e0'; // Gray for missing data

    // Color scale: red (-1) → white (0) → green (+1)
    if (correlation >= 0) {
      // Positive correlation: white to green
      const intensity = Math.floor(correlation * 200);
      return `rgb(${200 - intensity}, ${220 + intensity * 0.15}, ${200 - intensity})`;
    } else {
      // Negative correlation: white to red
      const intensity = Math.floor(-correlation * 200);
      return `rgb(${220 + intensity * 0.15}, ${200 - intensity}, ${200 - intensity})`;
    }
  };

  const getCorrelationLabel = (correlation: number): string => {
    const abs = Math.abs(correlation);
    if (abs >= 0.8) return 'Strong';
    if (abs >= 0.5) return 'Moderate';
    if (abs >= 0.3) return 'Weak';
    return 'Very Weak';
  };

  if (correlationQ.isLoading) {
    return (
      <Box display="flex" flexDirection="column" justifyContent="center" alignItems="center" minHeight="300px" gap={2}>
        <CircularProgress />
        <Typography variant="body2" color="text.secondary">
          Computing correlations... This may take up to a minute for portfolios with many positions.
        </Typography>
      </Box>
    );
  }

  if (correlationQ.error) {
    const errorMessage = (correlationQ.error as any)?.response?.data?.error
      || (correlationQ.error as Error).message
      || 'Unknown error';

    return (
      <Box>
        <Typography variant="h5" gutterBottom mb={3}>
          Portfolio Correlation Heatmap
        </Typography>
        <Grid container spacing={2} mb={3}>
          <Grid item xs={12} md={6}>
            <FormControl fullWidth>
              <InputLabel>Portfolio</InputLabel>
              <Select
                value={selectedPortfolioId}
                label="Portfolio"
                onChange={(e) => setSelectedPortfolioId(e.target.value)}
              >
                {portfoliosQ.data?.map((portfolio) => (
                  <MenuItem key={portfolio.id} value={portfolio.id}>
                    {portfolio.name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Grid>
        </Grid>
        <Alert severity="error">
          Failed to load correlation data: {errorMessage}
        </Alert>
      </Box>
    );
  }

  if (!correlationQ.data) {
    return (
      <Alert severity="info">
        No correlation data available. Need at least 2 positions with sufficient price history.
      </Alert>
    );
  }

  const matrix = correlationQ.data;
  const tickers = matrix.tickers;

  return (
    <Box>
      <Typography variant="h5" gutterBottom mb={3}>
        Portfolio Correlation Heatmap
      </Typography>

      {/* Portfolio and Time Period Selectors */}
      <Grid container spacing={2} mb={3}>
        <Grid item xs={12} md={6}>
          <FormControl fullWidth>
            <InputLabel>Portfolio</InputLabel>
            <Select
              value={selectedPortfolioId}
              label="Portfolio"
              onChange={(e) => setSelectedPortfolioId(e.target.value)}
            >
              {portfoliosQ.data?.map((portfolio) => (
                <MenuItem key={portfolio.id} value={portfolio.id}>
                  {portfolio.name}
                </MenuItem>
              ))}
            </Select>
          </FormControl>
        </Grid>
        <Grid item xs={12} md={6}>
          <FormControl fullWidth>
            <InputLabel>Time Period</InputLabel>
            <Select value={days} label="Time Period" onChange={(e) => setDays(Number(e.target.value))}>
              <MenuItem value={30}>30 Days</MenuItem>
              <MenuItem value={60}>60 Days</MenuItem>
              <MenuItem value={90}>90 Days</MenuItem>
              <MenuItem value={180}>180 Days</MenuItem>
              <MenuItem value={365}>1 Year</MenuItem>
            </Select>
          </FormControl>
        </Grid>
      </Grid>

      <Alert severity="info" sx={{ mb: 3 }}>
        Correlation shows how positions move together. Values range from -1 (move opposite) to +1 (move
        together). High positive correlations indicate concentration risk. Analysis includes positions
        representing at least 1% of portfolio value (limited to top 10 positions for performance).
      </Alert>

      {tickers.length < 2 ? (
        <Alert severity="warning">
          Need at least 2 positions with sufficient price data for correlation analysis.
        </Alert>
      ) : (
        <Paper elevation={2} sx={{ p: 2, overflowX: 'auto' }}>
          <Box
            sx={{
              display: 'grid',
              gridTemplateColumns: `120px repeat(${tickers.length}, minmax(80px, 100px))`,
              gap: '2px',
              minWidth: 'fit-content',
            }}
          >
            {/* Top-left corner cell (empty) */}
            <Box />

            {/* Column headers (ticker symbols) */}
            {tickers.map((ticker) => (
              <Box
                key={`header-${ticker}`}
                sx={{
                  p: 1,
                  textAlign: 'center',
                  fontWeight: 'bold',
                  fontSize: '0.875rem',
                  backgroundColor: '#f5f5f5',
                  borderRadius: '4px',
                }}
              >
                {ticker}
              </Box>
            ))}

            {/* Data rows */}
            {tickers.map((rowTicker) => (
              <Box
                key={`row-${rowTicker}`}
                sx={{
                  display: 'contents', // Make the row ticker and its cells part of the grid
                }}
              >
                {/* Row header (ticker symbol) */}
                <Box
                  sx={{
                    p: 1,
                    display: 'flex',
                    alignItems: 'center',
                    fontWeight: 'bold',
                    fontSize: '0.875rem',
                    backgroundColor: '#f5f5f5',
                    borderRadius: '4px',
                  }}
                >
                  {rowTicker}
                </Box>

                {/* Correlation cells */}
                {tickers.map((colTicker) => {
                  const correlation = getCorrelation(matrix, rowTicker, colTicker);
                  const color = getCorrelationColor(correlation);
                  const label = correlation !== null ? getCorrelationLabel(correlation) : 'N/A';

                  return (
                    <Tooltip
                      key={`cell-${rowTicker}-${colTicker}`}
                      title={
                        correlation !== null
                          ? `${rowTicker} vs ${colTicker}: ${correlation.toFixed(3)} (${label})`
                          : 'No data'
                      }
                      arrow
                    >
                      <Box
                        sx={{
                          p: 1,
                          textAlign: 'center',
                          backgroundColor: color,
                          borderRadius: '4px',
                          cursor: 'pointer',
                          fontSize: '0.75rem',
                          fontWeight: rowTicker === colTicker ? 'bold' : 'normal',
                          transition: 'transform 0.2s',
                          '&:hover': {
                            transform: 'scale(1.05)',
                            boxShadow: 2,
                          },
                        }}
                      >
                        {correlation !== null ? correlation.toFixed(2) : 'N/A'}
                      </Box>
                    </Tooltip>
                  );
                })}
              </Box>
            ))}
          </Box>

          {/* Legend */}
          <Box mt={3} display="flex" justifyContent="center" alignItems="center" gap={1}>
            <Typography variant="body2" fontWeight="bold">
              Correlation:
            </Typography>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box
                sx={{
                  width: 20,
                  height: 20,
                  backgroundColor: getCorrelationColor(-1.0),
                  borderRadius: '4px',
                }}
              />
              <Typography variant="caption">-1.0</Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box
                sx={{
                  width: 20,
                  height: 20,
                  backgroundColor: getCorrelationColor(-0.5),
                  borderRadius: '4px',
                }}
              />
              <Typography variant="caption">-0.5</Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box
                sx={{
                  width: 20,
                  height: 20,
                  backgroundColor: getCorrelationColor(0.0),
                  borderRadius: '4px',
                }}
              />
              <Typography variant="caption">0.0</Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box
                sx={{
                  width: 20,
                  height: 20,
                  backgroundColor: getCorrelationColor(0.5),
                  borderRadius: '4px',
                }}
              />
              <Typography variant="caption">+0.5</Typography>
            </Box>
            <Box display="flex" alignItems="center" gap={0.5}>
              <Box
                sx={{
                  width: 20,
                  height: 20,
                  backgroundColor: getCorrelationColor(1.0),
                  borderRadius: '4px',
                }}
              />
              <Typography variant="caption">+1.0</Typography>
            </Box>
          </Box>

          <Box mt={2}>
            <Typography variant="body2" color="text.secondary" align="center">
              Hover over cells to see detailed correlation values and strength labels
            </Typography>
          </Box>
        </Paper>
      )}
    </Box>
  );
}
