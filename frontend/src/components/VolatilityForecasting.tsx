import { useState } from 'react';
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
  Card,
  CardContent,
  Alert,
  Chip,
  CircularProgress,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Divider,
} from '@mui/material';
import { ShowChart, Search, Warning as WarningIcon, Info as InfoIcon } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getVolatilityForecast, searchTickers } from '../lib/endpoints';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer, Area, ComposedChart } from 'recharts';

export function VolatilityForecasting() {
  const [ticker, setTicker] = useState('');
  const [searchTicker, setSearchTicker] = useState('');
  const [days, setDays] = useState(30);
  const [confidenceLevel, setConfidenceLevel] = useState(0.95);

  // Fetch company name when ticker is searched
  const companyInfoQ = useQuery({
    queryKey: ['companyInfo', searchTicker],
    queryFn: () => searchTickers(searchTicker),
    enabled: !!searchTicker,
    staleTime: 1000 * 60 * 60,
  });

  const companyName = companyInfoQ.data?.[0]?.name || null;

  // Fetch volatility forecast
  const forecastQ = useQuery({
    queryKey: ['volForecast', searchTicker, days, confidenceLevel],
    queryFn: () => getVolatilityForecast(searchTicker, days, confidenceLevel),
    enabled: !!searchTicker,
  });

  const handleSearch = () => {
    if (ticker.trim()) {
      setSearchTicker(ticker.trim().toUpperCase());
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  const formatPercent = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    return `${(value * 100).toFixed(2)}%`;
  };

  const formatNumber = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    return value.toFixed(4);
  };

  const getPersistenceLevel = (persistence: number): { level: string; color: string } => {
    if (persistence > 0.95) return { level: 'Very High', color: 'error' };
    if (persistence > 0.85) return { level: 'High', color: 'warning' };
    if (persistence > 0.70) return { level: 'Moderate', color: 'info' };
    return { level: 'Low', color: 'success' };
  };

  const renderGarchParameters = () => {
    if (!forecastQ.data) return null;

    const params = forecastQ.data.garch_parameters;
    const persistenceInfo = getPersistenceLevel(params.persistence);

    return (
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          GARCH(1,1) Model Parameters
        </Typography>
        <Grid container spacing={3}>
          <Grid item xs={12} md={3}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  ω (Omega)
                </Typography>
                <Typography variant="h5" fontFamily="monospace">
                  {formatNumber(params.omega)}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  Long-run variance constant
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={3}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  α (Alpha)
                </Typography>
                <Typography variant="h5" fontFamily="monospace">
                  {formatNumber(params.alpha)}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  ARCH effect (shock sensitivity)
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={3}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  β (Beta)
                </Typography>
                <Typography variant="h5" fontFamily="monospace">
                  {formatNumber(params.beta)}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  GARCH effect (persistence)
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={3}>
            <Card variant="outlined" sx={{ bgcolor: `${persistenceInfo.color}.50` }}>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  Persistence (α + β)
                </Typography>
                <Typography variant="h5" fontFamily="monospace">
                  {formatNumber(params.persistence)}
                </Typography>
                <Box sx={{ mt: 1 }}>
                  <Chip label={persistenceInfo.level} color={persistenceInfo.color as any} size="small" />
                </Box>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={6}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  Current Volatility
                </Typography>
                <Typography variant="h5" color="primary">
                  {formatPercent(forecastQ.data.current_volatility)}
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={6}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  Long-Run Volatility
                </Typography>
                <Typography variant="h5" color="secondary">
                  {formatPercent(params.long_run_volatility)}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </Paper>
    );
  };

  const renderForecastChart = () => {
    if (!forecastQ.data || !forecastQ.data.forecasts) return null;

    const chartData = forecastQ.data.forecasts.map((f) => ({
      day: f.day,
      volatility: f.predicted_volatility * 100,
      lowerBound: f.confidence_lower * 100,
      upperBound: f.confidence_upper * 100,
      current: forecastQ.data.current_volatility * 100,
      longRun: forecastQ.data.garch_parameters.long_run_volatility * 100,
    }));

    return (
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          {days}-Day Volatility Forecast
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={2}>
          {(confidenceLevel * 100).toFixed(0)}% Confidence Interval
        </Typography>
        <ResponsiveContainer width="100%" height={400}>
          <ComposedChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis
              dataKey="day"
              label={{ value: 'Days Ahead', position: 'insideBottom', offset: -5 }}
            />
            <YAxis
              label={{ value: 'Annualized Volatility (%)', angle: -90, position: 'insideLeft' }}
            />
            <Tooltip
              formatter={(value: number) => `${value.toFixed(2)}%`}
              labelFormatter={(label) => `Day ${label}`}
            />
            <Legend />

            {/* Confidence bands */}
            <Area
              type="monotone"
              dataKey="upperBound"
              stroke="none"
              fill="#ff9800"
              fillOpacity={0.2}
              name="Upper Bound"
            />
            <Area
              type="monotone"
              dataKey="lowerBound"
              stroke="none"
              fill="#ff9800"
              fillOpacity={0.2}
              name="Lower Bound"
            />

            {/* Forecast line */}
            <Line
              type="monotone"
              dataKey="volatility"
              stroke="#1976d2"
              strokeWidth={2}
              dot={false}
              name="Forecast"
            />

            {/* Reference lines */}
            <Line
              type="monotone"
              dataKey="current"
              stroke="#9e9e9e"
              strokeWidth={1}
              strokeDasharray="5 5"
              dot={false}
              name="Current"
            />
            <Line
              type="monotone"
              dataKey="longRun"
              stroke="#4caf50"
              strokeWidth={1}
              strokeDasharray="5 5"
              dot={false}
              name="Long-Run"
            />
          </ComposedChart>
        </ResponsiveContainer>
      </Paper>
    );
  };

  const renderWarnings = () => {
    if (!forecastQ.data || forecastQ.data.warnings.length === 0) return null;

    return (
      <Alert severity="warning" icon={<WarningIcon />} sx={{ mb: 3 }}>
        <Typography variant="subtitle2" gutterBottom>
          <strong>Model Warnings:</strong>
        </Typography>
        {forecastQ.data.warnings.map((warning, idx) => (
          <Typography key={idx} variant="body2">
            • {warning}
          </Typography>
        ))}
      </Alert>
    );
  };

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <ShowChart sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Volatility Forecasting (GARCH)
        </Typography>
      </Box>

      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Forecast Parameters
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Generate GARCH(1,1) volatility forecasts with confidence intervals. GARCH models capture volatility clustering
          and persistence in financial time series.
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
              <InputLabel>Forecast Horizon</InputLabel>
              <Select value={days} label="Forecast Horizon" onChange={(e) => setDays(Number(e.target.value))}>
                <MenuItem value={5}>5 Days</MenuItem>
                <MenuItem value={10}>10 Days</MenuItem>
                <MenuItem value={30}>30 Days</MenuItem>
                <MenuItem value={60}>60 Days</MenuItem>
                <MenuItem value={90}>90 Days</MenuItem>
              </Select>
            </FormControl>
          </Grid>

          <Grid item xs={12} sm={3}>
            <FormControl fullWidth>
              <InputLabel>Confidence Level</InputLabel>
              <Select
                value={confidenceLevel}
                label="Confidence Level"
                onChange={(e) => setConfidenceLevel(Number(e.target.value))}
              >
                <MenuItem value={0.80}>80%</MenuItem>
                <MenuItem value={0.90}>90%</MenuItem>
                <MenuItem value={0.95}>95%</MenuItem>
                <MenuItem value={0.99}>99%</MenuItem>
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
              Forecast
            </Button>
          </Grid>
        </Grid>
      </Paper>

      {!searchTicker && (
        <Alert severity="info">
          Enter a ticker symbol to generate volatility forecasts. GARCH models are particularly useful for:
          <ul style={{ marginTop: '8px', marginBottom: 0 }}>
            <li>Risk management and VaR calculations</li>
            <li>Options pricing and derivatives valuation</li>
            <li>Portfolio optimization under varying volatility</li>
          </ul>
        </Alert>
      )}

      {searchTicker && (
        <Box>
          <Box sx={{ mb: 3 }}>
            <Typography variant="h5" fontWeight="bold">
              {searchTicker} Volatility Forecast
            </Typography>
            {companyName && (
              <Typography variant="body1" color="text.secondary" sx={{ mt: 0.5 }}>
                {companyName}
              </Typography>
            )}
          </Box>

          {forecastQ.isLoading && (
            <Box display="flex" justifyContent="center" alignItems="center" minHeight={400}>
              <CircularProgress />
            </Box>
          )}

          {forecastQ.error && (
            <Alert severity="error">
              Failed to generate forecast: {(forecastQ.error as Error).message}
            </Alert>
          )}

          {forecastQ.data && (
            <>
              {renderGarchParameters()}
              {renderForecastChart()}
              {renderWarnings()}

              {/* Interpretation */}
              <Alert severity="info" icon={<InfoIcon />}>
                <Typography variant="subtitle2" gutterBottom>
                  <strong>Understanding the Forecast:</strong>
                </Typography>
                <Typography variant="body2">
                  • <strong>Persistence:</strong> High persistence ({forecastQ.data.garch_parameters.persistence.toFixed(2)})
                  means volatility shocks dissipate slowly
                </Typography>
                <Typography variant="body2">
                  • <strong>Mean Reversion:</strong> Volatility tends toward {formatPercent(forecastQ.data.garch_parameters.long_run_volatility)} over time
                </Typography>
                <Typography variant="body2">
                  • <strong>Confidence Bands:</strong> Wider bands indicate greater uncertainty in the forecast
                </Typography>
              </Alert>
            </>
          )}
        </Box>
      )}
    </Box>
  );
}
