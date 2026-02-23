import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  Grid,
  Card,
  CardContent,
  Alert,
  Chip,
  CircularProgress,
  LinearProgress,
  Divider,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
} from '@mui/material';
import {
  TrendingUp,
  TrendingDown,
  ShowChart,
  CalendarToday,
  Info as InfoIcon,
  Warning as WarningIcon,
} from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getMarketRegime, getRegimeForecast } from '../lib/endpoints';
import type { RegimeType } from '../types';

export function MarketRegimePage() {
  const [forecastDays, setForecastDays] = useState(30);

  // Fetch current regime
  const regimeQ = useQuery({
    queryKey: ['marketRegime'],
    queryFn: getMarketRegime,
    refetchInterval: 60000, // Refresh every minute
  });

  // Fetch forecast
  const forecastQ = useQuery({
    queryKey: ['regimeForecast', forecastDays],
    queryFn: () => getRegimeForecast(forecastDays),
  });

  const getRegimeIcon = (regimeType: RegimeType) => {
    switch (regimeType) {
      case 'Bull':
        return <TrendingUp sx={{ fontSize: 40, color: 'success.main' }} />;
      case 'Bear':
        return <TrendingDown sx={{ fontSize: 40, color: 'error.main' }} />;
      case 'HighVolatility':
        return <ShowChart sx={{ fontSize: 40, color: 'warning.main' }} />;
      case 'Normal':
        return <ShowChart sx={{ fontSize: 40, color: 'info.main' }} />;
      default:
        return <ShowChart sx={{ fontSize: 40 }} />;
    }
  };

  const getRegimeColor = (regimeType: string): "success" | "error" | "warning" | "info" | "default" => {
    switch (regimeType) {
      case 'Bull':
        return 'success';
      case 'Bear':
        return 'error';
      case 'HighVolatility':
        return 'warning';
      case 'Normal':
        return 'info';
      default:
        return 'default';
    }
  };

  const formatPercent = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    // Backend returns percentages (0-100), so just format them
    return `${value.toFixed(1)}%`;
  };

  const formatMultiplier = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    return `${value.toFixed(2)}x`;
  };

  const formatProbability = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    // HMM probabilities are decimals (0-1), so multiply by 100
    return `${(value * 100).toFixed(1)}%`;
  };

  const renderCurrentRegime = () => {
    if (!regimeQ.data) return null;

    const regime = regimeQ.data;

    return (
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Current Market Regime
        </Typography>

        <Grid container spacing={3}>
          <Grid item xs={12} md={4}>
            <Card variant="outlined" sx={{ bgcolor: `${getRegimeColor(regime.regime_type)}.50`, height: '100%' }}>
              <CardContent>
                <Box display="flex" alignItems="center" gap={2} mb={2}>
                  {getRegimeIcon(regime.regime_type)}
                  <Box>
                    <Typography variant="h4" fontWeight="bold">
                      {regime.regime_type}
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      Market
                    </Typography>
                  </Box>
                </Box>
                <Chip
                  label={`${regime.confidence.toFixed(0)}% Confidence`}
                  color={regime.confidence > 80 ? 'success' : regime.confidence > 60 ? 'info' : 'warning'}
                  size="small"
                />
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={4}>
            <Card variant="outlined" sx={{ height: '100%' }}>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  Volatility Level
                </Typography>
                <Typography variant="h4" color="primary">
                  {formatPercent(regime.volatility_level)}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  Annualized market volatility
                </Typography>
              </CardContent>
            </Card>
          </Grid>

          <Grid item xs={12} md={4}>
            <Card variant="outlined" sx={{ height: '100%' }}>
              <CardContent>
                <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                  Risk Threshold Multiplier
                </Typography>
                <Typography variant="h4" color="primary">
                  {formatMultiplier(regime.threshold_multiplier)}
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                  {regime.threshold_multiplier < 1 ? 'Tightened thresholds' :
                   regime.threshold_multiplier > 1 ? 'Relaxed thresholds' :
                   'Standard thresholds'}
                </Typography>
              </CardContent>
            </Card>
          </Grid>
        </Grid>

        {regime.hmm_probabilities && (
          <Box sx={{ mt: 3 }}>
            <Typography variant="subtitle1" gutterBottom>
              HMM State Probabilities
            </Typography>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={6} md={3}>
                <Box>
                  <Box display="flex" justifyContent="space-between" mb={0.5}>
                    <Typography variant="body2">Bull</Typography>
                    <Typography variant="body2" fontWeight="bold">
                      {formatProbability(regime.hmm_probabilities.bull)}
                    </Typography>
                  </Box>
                  <LinearProgress
                    variant="determinate"
                    value={regime.hmm_probabilities.bull * 100}
                    color="success"
                    sx={{ height: 8, borderRadius: 1 }}
                  />
                </Box>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <Box>
                  <Box display="flex" justifyContent="space-between" mb={0.5}>
                    <Typography variant="body2">Normal</Typography>
                    <Typography variant="body2" fontWeight="bold">
                      {formatProbability(regime.hmm_probabilities.normal)}
                    </Typography>
                  </Box>
                  <LinearProgress
                    variant="determinate"
                    value={regime.hmm_probabilities.normal * 100}
                    color="info"
                    sx={{ height: 8, borderRadius: 1 }}
                  />
                </Box>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <Box>
                  <Box display="flex" justifyContent="space-between" mb={0.5}>
                    <Typography variant="body2">Bear</Typography>
                    <Typography variant="body2" fontWeight="bold">
                      {formatProbability(regime.hmm_probabilities.bear)}
                    </Typography>
                  </Box>
                  <LinearProgress
                    variant="determinate"
                    value={regime.hmm_probabilities.bear * 100}
                    color="error"
                    sx={{ height: 8, borderRadius: 1 }}
                  />
                </Box>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <Box>
                  <Box display="flex" justifyContent="space-between" mb={0.5}>
                    <Typography variant="body2">High Volatility</Typography>
                    <Typography variant="body2" fontWeight="bold">
                      {formatProbability(regime.hmm_probabilities.high_volatility)}
                    </Typography>
                  </Box>
                  <LinearProgress
                    variant="determinate"
                    value={regime.hmm_probabilities.high_volatility * 100}
                    color="warning"
                    sx={{ height: 8, borderRadius: 1 }}
                  />
                </Box>
              </Grid>
            </Grid>
          </Box>
        )}
      </Paper>
    );
  };

  const renderThresholdExplanation = () => {
    if (!regimeQ.data) return null;

    const regime = regimeQ.data;
    const baseThreshold = 30; // Example base volatility threshold
    const adjustedThreshold = baseThreshold * regime.threshold_multiplier;

    return (
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Risk Threshold Adjustments
        </Typography>
        <Grid container spacing={2}>
          <Grid item xs={12} md={4}>
            <Typography variant="body2" color="text.secondary">
              Base Volatility Threshold
            </Typography>
            <Typography variant="h5">{baseThreshold}%</Typography>
          </Grid>
          <Grid item xs={12} md={4}>
            <Typography variant="body2" color="text.secondary">
              Adjusted for {regime.regime_type} Market
            </Typography>
            <Typography variant="h5" color="primary">
              {adjustedThreshold.toFixed(1)}% ({formatMultiplier(regime.threshold_multiplier)})
            </Typography>
          </Grid>
          <Grid item xs={12} md={4}>
            <Typography variant="body2" color="text.secondary">
              Effect
            </Typography>
            <Typography variant="body1">
              {regime.threshold_multiplier < 1
                ? 'Earlier risk detection during calm markets'
                : regime.threshold_multiplier > 1
                ? 'More tolerance during volatile periods'
                : 'Standard risk detection'}
            </Typography>
          </Grid>
        </Grid>
      </Paper>
    );
  };

  const renderForecast = () => {
    if (!forecastQ.data?.forecasts || forecastQ.data.forecasts.length === 0) return null;

    return (
      <Paper sx={{ p: 3 }}>
        <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
          <Typography variant="h6">
            Regime Forecast
          </Typography>
          <FormControl size="small" sx={{ minWidth: 150 }}>
            <InputLabel>Forecast Period</InputLabel>
            <Select
              value={forecastDays}
              label="Forecast Period"
              onChange={(e) => setForecastDays(Number(e.target.value))}
            >
              <MenuItem value={5}>5 Days</MenuItem>
              <MenuItem value={10}>10 Days</MenuItem>
              <MenuItem value={30}>30 Days</MenuItem>
            </Select>
          </FormControl>
        </Box>

        <TableContainer>
          <Table size="small">
            <TableHead>
              <TableRow>
                <TableCell>Horizon</TableCell>
                <TableCell>Predicted Regime</TableCell>
                <TableCell align="right">Confidence</TableCell>
                <TableCell align="right">Bull Prob</TableCell>
                <TableCell align="right">Normal Prob</TableCell>
                <TableCell align="right">Bear Prob</TableCell>
                <TableCell align="right">High Vol Prob</TableCell>
                <TableCell align="right">Transition Prob</TableCell>
              </TableRow>
            </TableHead>
            <TableBody>
              {forecastQ.data.forecasts.map((forecast) => (
                <TableRow key={forecast.days_ahead}>
                  <TableCell>
                    <Box display="flex" alignItems="center" gap={1}>
                      <CalendarToday sx={{ fontSize: 16 }} />
                      {forecast.days_ahead} days
                    </Box>
                  </TableCell>
                  <TableCell>
                    <Chip
                      label={forecast.predicted_regime}
                      color={getRegimeColor(forecast.predicted_regime)}
                      size="small"
                    />
                  </TableCell>
                  <TableCell align="right">
                    <Typography
                      variant="body2"
                      color={forecast.confidence > 0.7 ? 'success.main' : forecast.confidence > 0.5 ? 'warning.main' : 'error.main'}
                    >
                      {formatProbability(forecast.confidence)}
                    </Typography>
                  </TableCell>
                  <TableCell align="right">{formatProbability(forecast.state_probabilities.bull)}</TableCell>
                  <TableCell align="right">{formatProbability(forecast.state_probabilities.normal)}</TableCell>
                  <TableCell align="right">{formatProbability(forecast.state_probabilities.bear)}</TableCell>
                  <TableCell align="right">{formatProbability(forecast.state_probabilities.high_volatility)}</TableCell>
                  <TableCell align="right">
                    {formatProbability(forecast.transition_probability)}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </TableContainer>

        <Alert severity="info" icon={<InfoIcon />} sx={{ mt: 2 }}>
          <Typography variant="body2">
            Forecast confidence decreases with longer horizons. Transition probability indicates the likelihood
            of changing from the current regime.
          </Typography>
        </Alert>
      </Paper>
    );
  };

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <ShowChart sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Market Regime Detection
        </Typography>
      </Box>

      <Alert severity="info" icon={<InfoIcon />} sx={{ mb: 3 }}>
        <Typography variant="body2">
          Market regimes are detected using volatility patterns and Hidden Markov Models (HMM). Risk thresholds
          are dynamically adjusted based on the current regime to provide context-aware risk assessment.
        </Typography>
      </Alert>

      {regimeQ.isLoading && (
        <Box display="flex" justifyContent="center" alignItems="center" minHeight={400}>
          <CircularProgress />
        </Box>
      )}

      {regimeQ.error && (
        <Alert severity="error" icon={<WarningIcon />}>
          Failed to load market regime data: {(regimeQ.error as Error).message}
        </Alert>
      )}

      {regimeQ.data && (
        <>
          {renderCurrentRegime()}
          {renderThresholdExplanation()}
        </>
      )}

      {forecastQ.isLoading && (
        <Box display="flex" justifyContent="center" alignItems="center" minHeight={200}>
          <CircularProgress />
        </Box>
      )}

      {forecastQ.error && (
        <Alert severity="warning" icon={<WarningIcon />}>
          Failed to load forecast data: {(forecastQ.error as Error).message}
        </Alert>
      )}

      {forecastQ.data && renderForecast()}
    </Box>
  );
}
