import { useState } from 'react';
import {
  Box,
  Paper,
  Typography,
  Grid,
  LinearProgress,
  Chip,
  Alert,
  CircularProgress,
  Tooltip,
  Divider,
  Accordion,
  AccordionSummary,
  AccordionDetails,
} from '@mui/material';
import {
  TrendingDown,
  ShowChart,
  Speed,
  AccountBalance,
  Warning as WarningIcon,
  ExpandMore,
  InfoOutlined,
} from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getPositionRisk } from '../lib/endpoints';
import type { RiskLevel } from '../types';

interface RiskMetricsPanelProps {
  ticker: string;
  holdingName?: string | null;
  days?: number;
  benchmark?: string;
  onTickerClick?: (ticker: string) => void;
}

interface MetricCardProps {
  icon: React.ReactNode;
  label: string;
  value: string;
  subValue?: string;
  color?: string;
  tooltip?: string;
}

function MetricCard({ icon, label, value, subValue, color = '#1976d2', tooltip }: MetricCardProps) {
  const content = (
    <Paper
      elevation={1}
      sx={{
        p: 2,
        height: '100%',
        borderLeft: `4px solid ${color}`,
        transition: 'box-shadow 0.2s',
        '&:hover': {
          boxShadow: 3,
        },
      }}
    >
      <Box display="flex" alignItems="center" gap={1} mb={1}>
        <Box sx={{ color }}>{icon}</Box>
        <Typography variant="caption" color="text.secondary" fontWeight={600}>
          {label}
        </Typography>
      </Box>
      <Typography variant="h5" fontWeight="bold" gutterBottom>
        {value}
      </Typography>
      {subValue && (
        <Typography variant="caption" color="text.secondary">
          {subValue}
        </Typography>
      )}
    </Paper>
  );

  return tooltip ? (
    <Tooltip title={tooltip} placement="top">
      {content}
    </Tooltip>
  ) : (
    content
  );
}

export function RiskMetricsPanel({ ticker, holdingName, days = 90, benchmark = 'SPY', onTickerClick }: RiskMetricsPanelProps) {
  const { data: risk, isLoading, error } = useQuery({
    queryKey: ['risk', ticker, days, benchmark],
    queryFn: () => getPositionRisk(ticker, days, benchmark),
    staleTime: 1000 * 60 * 60, // 1 hour
    retry: 1,
  });

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

  if (isLoading) {
    return (
      <Paper sx={{ p: 3, textAlign: 'center' }}>
        <CircularProgress />
        <Typography variant="body2" color="text.secondary" mt={2}>
          Calculating risk metrics for {ticker}...
        </Typography>
      </Paper>
    );
  }

  if (error) {
    // Parse error message to provide better feedback
    const errorMsg = error instanceof Error ? error.message : String(error);

    // Log error details for debugging
    console.error(`[RiskMetricsPanel] Failed to load risk data for ${ticker}:`, {
      error: errorMsg,
      ticker,
      days,
      benchmark,
    });

    let severity: 'error' | 'warning' | 'info' = 'error';
    let message = `Failed to load risk metrics for ${ticker}`;
    let details = '';

    if (errorMsg.includes('failure cache') || errorMsg.includes('not_found')) {
      severity = 'info';
      message = `${ticker} is not available for risk analysis`;
      details = 'This security may be a mutual fund, bond, or other instrument without publicly available price data.';
    } else if (errorMsg.includes('rate limit')) {
      severity = 'warning';
      message = `Rate limit reached for ${ticker}`;
      details = 'Too many API requests. Risk data will be available after cooldown period.';
    } else if (errorMsg.includes('Grow plan') || errorMsg.includes('upgrade')) {
      severity = 'info';
      message = `${ticker} requires paid API tier`;
      details = 'Canadian securities require a paid subscription to the price data provider.';
    } else if (errorMsg.includes('symbol') && errorMsg.includes('invalid')) {
      severity = 'info';
      message = `${ticker} is not a publicly traded security`;
      details = 'This appears to be a mutual fund or proprietary security code.';
    } else {
      // Generic error - include ticker and actual error message
      details = `The ticker may not have sufficient price history, or the data provider is unavailable. Error: ${errorMsg}`;
    }

    return (
      <Alert severity={severity}>
        <strong>{message}</strong>
        {details && (
          <Typography variant="body2" sx={{ mt: 0.5 }}>
            {details}
          </Typography>
        )}
      </Alert>
    );
  }

  if (!risk) {
    return <Alert severity="info">No risk data available for {ticker}</Alert>;
  }

  const riskColor = getRiskColor(risk.risk_level);

  return (
    <Box>
      {/* Ticker Header */}
      <Box
        sx={{
          mb: 2,
          p: 2,
          backgroundColor: 'primary.main',
          color: 'primary.contrastText',
          borderRadius: 1,
          cursor: onTickerClick ? 'pointer' : 'default',
          '&:hover': onTickerClick ? {
            backgroundColor: 'primary.dark',
          } : {},
        }}
        onClick={() => onTickerClick?.(ticker)}
      >
        <Typography variant="h5" fontWeight="bold">
          {ticker}
        </Typography>
        {holdingName && (
          <Typography variant="body2" sx={{ opacity: 0.9, mt: 0.5 }}>
            {holdingName}
          </Typography>
        )}
      </Box>

      {/* Header with overall risk score */}
      <Paper sx={{ p: 3, mb: 3, background: `linear-gradient(135deg, ${riskColor}15 0%, ${riskColor}05 100%)` }}>
        <Grid container spacing={2} alignItems="center">
          <Grid item xs={12} sm={6}>
            <Typography variant="overline" color="text.secondary">
              Overall Risk Assessment
            </Typography>
            <Box display="flex" alignItems="center" gap={2} mt={1}>
              <Typography variant="h3" fontWeight="bold">
                {risk.risk_score.toFixed(1)}
              </Typography>
              <Box>
                <Chip
                  label={risk.risk_level.toUpperCase()}
                  sx={{
                    backgroundColor: riskColor,
                    color: 'white',
                    fontWeight: 'bold',
                  }}
                />
                <Typography variant="caption" display="block" color="text.secondary" mt={0.5}>
                  Risk Score (0-100)
                </Typography>
              </Box>
            </Box>
          </Grid>
          <Grid item xs={12} sm={6}>
            <LinearProgress
              variant="determinate"
              value={risk.risk_score}
              sx={{
                height: 10,
                borderRadius: 5,
                backgroundColor: `${riskColor}30`,
                '& .MuiLinearProgress-bar': {
                  backgroundColor: riskColor,
                },
              }}
            />
            <Typography variant="caption" color="text.secondary" mt={1} display="block">
              Calculated over {days} trading days vs {benchmark}
            </Typography>
          </Grid>
        </Grid>
      </Paper>

      {/* Risk Score Explanation */}
      <Accordion sx={{ mb: 3 }}>
        <AccordionSummary expandIcon={<ExpandMore />}>
          <Box display="flex" alignItems="center" gap={1}>
            <InfoOutlined color="primary" fontSize="small" />
            <Typography variant="body2" fontWeight={600}>
              How is this risk score calculated?
            </Typography>
          </Box>
        </AccordionSummary>
        <AccordionDetails>
          <Typography variant="body2" color="text.secondary" paragraph>
            The risk score (0-100) is a weighted combination of multiple risk metrics. Higher scores indicate higher risk.
          </Typography>

          <Grid container spacing={2}>
            {/* Volatility Contribution */}
            <Grid item xs={12}>
              <Box>
                <Box display="flex" justifyContent="space-between" alignItems="center" mb={0.5}>
                  <Typography variant="body2" fontWeight={600}>
                    Volatility ({risk.metrics.volatility.toFixed(2)}%)
                  </Typography>
                  <Typography variant="body2" color="primary" fontWeight={600}>
                    {((risk.metrics.volatility / 50.0) * 40.0).toFixed(1)} / 40 points
                  </Typography>
                </Box>
                <LinearProgress
                  variant="determinate"
                  value={Math.min((risk.metrics.volatility / 50.0) * 100, 100)}
                  sx={{
                    height: 8,
                    borderRadius: 4,
                    backgroundColor: '#e0e0e0',
                    '& .MuiLinearProgress-bar': {
                      backgroundColor: '#2196f3',
                    },
                  }}
                />
                <Typography variant="caption" color="text.secondary" display="block" mt={0.5}>
                  40% weight • Higher volatility = higher risk
                </Typography>
              </Box>
            </Grid>

            {/* Max Drawdown Contribution */}
            <Grid item xs={12}>
              <Box>
                <Box display="flex" justifyContent="space-between" alignItems="center" mb={0.5}>
                  <Typography variant="body2" fontWeight={600}>
                    Max Drawdown ({risk.metrics.max_drawdown.toFixed(2)}%)
                  </Typography>
                  <Typography variant="body2" color="warning.main" fontWeight={600}>
                    {((Math.abs(risk.metrics.max_drawdown) / 50.0) * 30.0).toFixed(1)} / 30 points
                  </Typography>
                </Box>
                <LinearProgress
                  variant="determinate"
                  value={Math.min((Math.abs(risk.metrics.max_drawdown) / 50.0) * 100, 100)}
                  sx={{
                    height: 8,
                    borderRadius: 4,
                    backgroundColor: '#e0e0e0',
                    '& .MuiLinearProgress-bar': {
                      backgroundColor: '#ff9800',
                    },
                  }}
                />
                <Typography variant="caption" color="text.secondary" display="block" mt={0.5}>
                  30% weight • Larger drawdowns = higher risk
                </Typography>
              </Box>
            </Grid>

            {/* Beta Contribution */}
            <Grid item xs={12}>
              <Box>
                <Box display="flex" justifyContent="space-between" alignItems="center" mb={0.5}>
                  <Typography variant="body2" fontWeight={600}>
                    Beta {risk.metrics.beta !== null ? `(${risk.metrics.beta.toFixed(2)})` : '(N/A)'}
                  </Typography>
                  <Typography variant="body2" color="success.main" fontWeight={600}>
                    {risk.metrics.beta !== null
                      ? ((Math.min(risk.metrics.beta / 2.0, 1.0)) * 20.0).toFixed(1)
                      : '10.0'} / 20 points
                  </Typography>
                </Box>
                <LinearProgress
                  variant="determinate"
                  value={risk.metrics.beta !== null
                    ? Math.min((risk.metrics.beta / 2.0) * 100, 100)
                    : 50}
                  sx={{
                    height: 8,
                    borderRadius: 4,
                    backgroundColor: '#e0e0e0',
                    '& .MuiLinearProgress-bar': {
                      backgroundColor: '#4caf50',
                    },
                  }}
                />
                <Typography variant="caption" color="text.secondary" display="block" mt={0.5}>
                  20% weight • Higher beta vs market = higher risk
                </Typography>
              </Box>
            </Grid>

            {/* VaR Contribution */}
            <Grid item xs={12}>
              <Box>
                <Box display="flex" justifyContent="space-between" alignItems="center" mb={0.5}>
                  <Typography variant="body2" fontWeight={600}>
                    Value at Risk {risk.metrics.value_at_risk !== null ? `(${risk.metrics.value_at_risk.toFixed(2)}%)` : '(N/A)'}
                  </Typography>
                  <Typography variant="body2" color="error.main" fontWeight={600}>
                    {risk.metrics.value_at_risk !== null
                      ? ((Math.abs(risk.metrics.value_at_risk) / 20.0) * 10.0).toFixed(1)
                      : '5.0'} / 10 points
                  </Typography>
                </Box>
                <LinearProgress
                  variant="determinate"
                  value={risk.metrics.value_at_risk !== null
                    ? Math.min((Math.abs(risk.metrics.value_at_risk) / 20.0) * 100, 100)
                    : 50}
                  sx={{
                    height: 8,
                    borderRadius: 4,
                    backgroundColor: '#e0e0e0',
                    '& .MuiLinearProgress-bar': {
                      backgroundColor: '#f44336',
                    },
                  }}
                />
                <Typography variant="caption" color="text.secondary" display="block" mt={0.5}>
                  10% weight • Larger potential loss = higher risk
                </Typography>
              </Box>
            </Grid>
          </Grid>

          <Alert severity="info" sx={{ mt: 2 }}>
            <Typography variant="caption">
              <strong>Formula:</strong> Risk Score = (Volatility × 0.4) + (|Drawdown| × 0.3) + (Beta × 0.2) + (|VaR| × 0.1)
              <br />
              Scores: 0-40 = Low Risk • 40-60 = Moderate Risk • 60-100 = High Risk
            </Typography>
          </Alert>
        </AccordionDetails>
      </Accordion>

      <Divider sx={{ mb: 3 }} />

      {/* Individual metrics */}
      <Grid container spacing={2}>
        <Grid item xs={12} sm={6} md={4}>
          <MetricCard
            icon={<ShowChart />}
            label="Volatility (Annualized)"
            value={`${risk.metrics.volatility.toFixed(2)}%`}
            subValue="Standard deviation of returns"
            color="#2196f3"
            tooltip="Higher volatility means larger price swings. Typical stocks range from 15-40%."
          />
        </Grid>

        <Grid item xs={12} sm={6} md={4}>
          <MetricCard
            icon={<TrendingDown />}
            label="Maximum Drawdown"
            value={`${risk.metrics.max_drawdown.toFixed(2)}%`}
            subValue="Worst peak-to-trough decline"
            color={risk.metrics.max_drawdown < -15 ? '#f44336' : '#ff9800'}
            tooltip="The largest percentage drop from a peak. Lower (more negative) is riskier."
          />
        </Grid>

        {risk.metrics.beta !== null && (
          <Grid item xs={12} sm={6} md={4}>
            <MetricCard
              icon={<Speed />}
              label="Beta"
              value={risk.metrics.beta.toFixed(2)}
              subValue={`vs ${benchmark}`}
              color={Math.abs(risk.metrics.beta) > 1.2 ? '#ff9800' : '#4caf50'}
              tooltip={`Beta measures volatility vs market. ${risk.metrics.beta > 1 ? 'More' : 'Less'} volatile than ${benchmark}.`}
            />
          </Grid>
        )}

        {risk.metrics.sharpe !== null && (
          <Grid item xs={12} sm={6} md={4}>
            <MetricCard
              icon={<AccountBalance />}
              label="Sharpe Ratio"
              value={risk.metrics.sharpe.toFixed(2)}
              subValue="Risk-adjusted return"
              color={risk.metrics.sharpe > 1 ? '#4caf50' : risk.metrics.sharpe > 0 ? '#ff9800' : '#f44336'}
              tooltip="Measures return per unit of risk. Higher is better. >1 is good, >2 is excellent."
            />
          </Grid>
        )}

        {risk.metrics.sortino !== null && (
          <Grid item xs={12} sm={6} md={4}>
            <MetricCard
              icon={<AccountBalance />}
              label="Sortino Ratio"
              value={risk.metrics.sortino.toFixed(2)}
              subValue="Downside risk-adjusted return"
              color={risk.metrics.sortino > 1 ? '#4caf50' : risk.metrics.sortino > 0 ? '#ff9800' : '#f44336'}
              tooltip="Like Sharpe but focuses only on downside risk. Higher is better. >1 is good, >2 is excellent."
            />
          </Grid>
        )}

        {risk.metrics.annualized_return !== null && (
          <Grid item xs={12} sm={6} md={4}>
            <MetricCard
              icon={<TrendingDown />}
              label="Annualized Return"
              value={`${risk.metrics.annualized_return.toFixed(2)}%`}
              subValue="Expected yearly return"
              color={risk.metrics.annualized_return > 10 ? '#4caf50' : risk.metrics.annualized_return > 0 ? '#ff9800' : '#f44336'}
              tooltip="Expected yearly return based on historical data. Calculated from average daily returns."
            />
          </Grid>
        )}

        {risk.metrics.var_95 !== null && (
          <Grid item xs={12} sm={6} md={4}>
            <MetricCard
              icon={<WarningIcon />}
              label="Value at Risk (95%)"
              value={`${risk.metrics.var_95.toFixed(2)}%`}
              subValue="1-in-20 days worst case"
              color="#ff9800"
              tooltip="95% confidence VaR: 5% chance of losing more than this in a single day."
            />
          </Grid>
        )}

        {risk.metrics.var_99 !== null && (
          <Grid item xs={12} sm={6} md={4}>
            <MetricCard
              icon={<WarningIcon />}
              label="Value at Risk (99%)"
              value={`${risk.metrics.var_99.toFixed(2)}%`}
              subValue="1-in-100 days worst case"
              color="#f44336"
              tooltip="99% confidence VaR: 1% chance of losing more than this in a single day."
            />
          </Grid>
        )}

        {risk.metrics.expected_shortfall_95 !== null && (
          <Grid item xs={12} sm={6} md={4}>
            <MetricCard
              icon={<WarningIcon />}
              label="Expected Shortfall (95%)"
              value={`${risk.metrics.expected_shortfall_95.toFixed(2)}%`}
              subValue="Average loss beyond VaR"
              color="#d32f2f"
              tooltip="Average loss when the 95% VaR threshold is exceeded. More conservative than VaR."
            />
          </Grid>
        )}

        {risk.metrics.expected_shortfall_99 !== null && (
          <Grid item xs={12} sm={6} md={4}>
            <MetricCard
              icon={<WarningIcon />}
              label="Expected Shortfall (99%)"
              value={`${risk.metrics.expected_shortfall_99.toFixed(2)}%`}
              subValue="Extreme loss scenario"
              color="#b71c1c"
              tooltip="Average loss when the 99% VaR threshold is exceeded. Captures tail risk."
            />
          </Grid>
        )}
      </Grid>

      {/* Information note */}
      <Alert severity="info" sx={{ mt: 3 }}>
        <Typography variant="caption">
          Risk metrics are calculated using historical price data and should not be used as the sole basis for investment decisions. Past performance does not guarantee future results.
        </Typography>
      </Alert>
    </Box>
  );
}
