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
  Divider,
} from '@mui/material';
import { Psychology, Search, Warning as WarningIcon, Info as InfoIcon, TrendingUp, TrendingDown } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getSentimentForecast, searchTickers } from '../lib/endpoints';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from 'recharts';

export function SentimentForecasting() {
  const [ticker, setTicker] = useState('');
  const [searchTicker, setSearchTicker] = useState('');
  const [days, setDays] = useState(30);

  // Fetch company name
  const companyInfoQ = useQuery({
    queryKey: ['companyInfo', searchTicker],
    queryFn: () => searchTickers(searchTicker),
    enabled: !!searchTicker,
    staleTime: 1000 * 60 * 60,
  });

  const companyName = companyInfoQ.data?.[0]?.name || null;

  // Fetch sentiment forecast
  const forecastQ = useQuery({
    queryKey: ['sentimentForecast', searchTicker, days],
    queryFn: () => getSentimentForecast(searchTicker, days),
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

  const formatSentiment = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    if (value > 0.3) return 'Positive';
    if (value < -0.3) return 'Negative';
    return 'Neutral';
  };

  const getSentimentColor = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'text.secondary';
    if (value > 0.3) return 'success.main';
    if (value < -0.3) return 'error.main';
    return 'text.secondary';
  };

  const formatSentimentValue = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    return value.toFixed(2);
  };

  const formatMomentumValue = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    return value.toFixed(3);
  };

  const renderSentimentFactors = () => {
    if (!forecastQ.data) return null;

    const factors = forecastQ.data.sentiment_factors;

    // Check if factors data exists
    if (!factors) return null;

    return (
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Sentiment Factors
        </Typography>
        <Grid container spacing={3}>
          <Grid item xs={12} md={3}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  Combined Sentiment
                </Typography>
                <Typography variant="h4" sx={{ color: getSentimentColor(factors.combined_sentiment) }}>
                  {formatSentimentValue(factors.combined_sentiment)}
                </Typography>
                <Chip
                  label={formatSentiment(factors.combined_sentiment)}
                  color={
                    factors.combined_sentiment && factors.combined_sentiment > 0.3 ? 'success' :
                    factors.combined_sentiment && factors.combined_sentiment < -0.3 ? 'error' : 'default'
                  }
                  size="small"
                  sx={{ mt: 1 }}
                />
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={3}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  News Sentiment (40%)
                </Typography>
                <Typography variant="h5" sx={{ color: getSentimentColor(factors.news_sentiment) }}>
                  {formatSentimentValue(factors.news_sentiment)}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  From news articles
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={3}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  SEC Filings (30%)
                </Typography>
                <Typography variant="h5" sx={{ color: getSentimentColor(factors.sec_sentiment) }}>
                  {formatSentimentValue(factors.sec_sentiment)}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  From SEC documents
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={3}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  Insider Activity (30%)
                </Typography>
                <Typography variant="h5" sx={{ color: getSentimentColor(factors.insider_sentiment) }}>
                  {formatSentimentValue(factors.insider_sentiment)}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  From insider trades
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </Paper>
    );
  };

  const renderMomentum = () => {
    if (!forecastQ.data) return null;

    const momentum = forecastQ.data.sentiment_momentum;

    // Check if momentum data exists
    if (!momentum) return null;

    return (
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Sentiment Momentum
        </Typography>
        <Grid container spacing={3}>
          <Grid item xs={12} md={4}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  7-Day Change
                </Typography>
                <Box display="flex" alignItems="center" gap={1}>
                  {momentum.change_7d && momentum.change_7d > 0 ? <TrendingUp color="success" /> : <TrendingDown color="error" />}
                  <Typography variant="h5" color={momentum.change_7d && momentum.change_7d > 0 ? 'success.main' : 'error.main'}>
                    {momentum.change_7d && momentum.change_7d > 0 ? '+' : ''}{formatMomentumValue(momentum.change_7d)}
                  </Typography>
                </Box>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={4}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  30-Day Change
                </Typography>
                <Box display="flex" alignItems="center" gap={1}>
                  {momentum.change_30d && momentum.change_30d > 0 ? <TrendingUp color="success" /> : <TrendingDown color="error" />}
                  <Typography variant="h5" color={momentum.change_30d && momentum.change_30d > 0 ? 'success.main' : 'error.main'}>
                    {momentum.change_30d && momentum.change_30d > 0 ? '+' : ''}{formatMomentumValue(momentum.change_30d)}
                  </Typography>
                </Box>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={4}>
            <Card variant="outlined">
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  Acceleration
                </Typography>
                <Box display="flex" alignItems="center" gap={1}>
                  {momentum.acceleration && momentum.acceleration > 0 ? <TrendingUp color="success" /> : <TrendingDown color="error" />}
                  <Typography variant="h5" color={momentum.acceleration && momentum.acceleration > 0 ? 'success.main' : 'error.main'}>
                    {momentum.acceleration && momentum.acceleration > 0 ? '+' : ''}{formatMomentumValue(momentum.acceleration)}
                  </Typography>
                </Box>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  {momentum.acceleration && momentum.acceleration > 0 ? 'Accelerating positive' : 'Decelerating'}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>
      </Paper>
    );
  };

  const renderDivergence = () => {
    if (!forecastQ.data?.divergence?.detected) return null;

    const div = forecastQ.data.divergence;

    return (
      <Alert severity="warning" icon={<WarningIcon />} sx={{ mb: 3 }}>
        <Typography variant="subtitle2" gutterBottom>
          <strong>{div.type} Divergence Detected</strong>
        </Typography>
        <Typography variant="body2" gutterBottom>
          {div.explanation}
        </Typography>
        <Divider sx={{ my: 1 }} />
        <Grid container spacing={2}>
          <Grid item xs={12} sm={6}>
            <Typography variant="body2">
              <strong>Divergence Score:</strong> {formatSentimentValue(div.score)}
            </Typography>
          </Grid>
          <Grid item xs={12} sm={6}>
            <Typography variant="body2">
              <strong>Reversal Probability:</strong> {div.reversal_probability !== undefined && div.reversal_probability !== null ? `${(div.reversal_probability * 100).toFixed(0)}%` : 'N/A'}
            </Typography>
          </Grid>
        </Grid>
      </Alert>
    );
  };

  const renderForecastChart = () => {
    if (!forecastQ.data) return null;

    // Combine both forecasts for chart
    const chartData = forecastQ.data.base_forecast.map((point, idx) => ({
      date: new Date(point.date).toLocaleDateString(),
      baseForecast: point.predicted_value,
      adjustedForecast: forecastQ.data.adjusted_forecast[idx]?.predicted_value,
      baseLower: point.lower_bound,
      baseUpper: point.upper_bound,
      adjustedLower: forecastQ.data.adjusted_forecast[idx]?.lower_bound,
      adjustedUpper: forecastQ.data.adjusted_forecast[idx]?.upper_bound,
    }));

    return (
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Price Forecast: Base vs Sentiment-Adjusted
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={2}>
          Comparing standard forecast with sentiment-adjusted projections
        </Typography>
        <ResponsiveContainer width="100%" height={400}>
          <LineChart data={chartData}>
            <CartesianGrid strokeDasharray="3 3" />
            <XAxis dataKey="date" />
            <YAxis label={{ value: 'Price ($)', angle: -90, position: 'insideLeft' }} />
            <Tooltip />
            <Legend />
            <Line
              type="monotone"
              dataKey="baseForecast"
              stroke="#9e9e9e"
              strokeWidth={2}
              strokeDasharray="5 5"
              dot={false}
              name="Base Forecast"
            />
            <Line
              type="monotone"
              dataKey="adjustedForecast"
              stroke="#1976d2"
              strokeWidth={2}
              dot={false}
              name="Sentiment-Adjusted"
            />
          </LineChart>
        </ResponsiveContainer>
      </Paper>
    );
  };

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <Psychology sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Sentiment-Aware Forecasting
        </Typography>
      </Box>

      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Forecast Parameters
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Generate price forecasts adjusted for sentiment from news, SEC filings, and insider activity.
          Detects sentiment-price divergences that may signal reversals.
        </Typography>

        <Grid container spacing={2} alignItems="center">
          <Grid item xs={12} sm={6}>
            <TextField
              fullWidth
              label="Ticker Symbol"
              placeholder="e.g., AAPL"
              value={ticker}
              onChange={(e) => setTicker(e.target.value.toUpperCase())}
              onKeyPress={handleKeyPress}
            />
          </Grid>

          <Grid item xs={12} sm={3}>
            <FormControl fullWidth>
              <InputLabel>Forecast Horizon</InputLabel>
              <Select value={days} label="Forecast Horizon" onChange={(e) => setDays(Number(e.target.value))}>
                <MenuItem value={7}>7 Days</MenuItem>
                <MenuItem value={14}>14 Days</MenuItem>
                <MenuItem value={30}>30 Days</MenuItem>
                <MenuItem value={60}>60 Days</MenuItem>
              </Select>
            </FormControl>
          </Grid>

          <Grid item xs={12} sm={3}>
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
          Enter a ticker symbol to generate sentiment-aware forecasts. This combines:
          <ul style={{ marginTop: '8px', marginBottom: 0 }}>
            <li>Traditional price forecasting models</li>
            <li>Multi-source sentiment analysis (news, SEC, insider)</li>
            <li>Divergence detection for potential reversals</li>
          </ul>
        </Alert>
      )}

      {searchTicker && (
        <Box>
          <Box sx={{ mb: 3 }}>
            <Typography variant="h5" fontWeight="bold">
              {searchTicker} Sentiment Forecast
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
              {renderDivergence()}
              {renderSentimentFactors()}
              {renderMomentum()}
              {renderForecastChart()}

              {forecastQ.data.sentiment_spike.detected && (
                <Alert severity="info" icon={<InfoIcon />} sx={{ mb: 3 }}>
                  <Typography variant="body2">
                    <strong>Sentiment Spike Detected:</strong> Z-score of {forecastQ.data.sentiment_spike.z_score.toFixed(2)}.
                    This may indicate a significant event or news affecting the stock.
                  </Typography>
                </Alert>
              )}

              <Alert severity="info" icon={<InfoIcon />}>
                <Typography variant="body2">
                  {forecastQ.data.interpretation}
                </Typography>
              </Alert>
            </>
          )}
        </Box>
      )}
    </Box>
  );
}
