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
} from '@mui/material';
import {
  TrendingDown,
  ShowChart,
  Speed,
  AccountBalance,
  Warning as WarningIcon,
} from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getPositionRisk } from '../lib/endpoints';
import type { RiskLevel } from '../types';

interface RiskMetricsPanelProps {
  ticker: string;
  days?: number;
  benchmark?: string;
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

export function RiskMetricsPanel({ ticker, days = 90, benchmark = 'SPY' }: RiskMetricsPanelProps) {
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
    return (
      <Alert severity="error">
        Failed to load risk metrics. The ticker may not have sufficient price history.
      </Alert>
    );
  }

  if (!risk) {
    return <Alert severity="info">No risk data available for {ticker}</Alert>;
  }

  const riskColor = getRiskColor(risk.risk_level);

  return (
    <Box>
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

        {risk.metrics.value_at_risk !== null && (
          <Grid item xs={12} sm={6} md={4}>
            <MetricCard
              icon={<WarningIcon />}
              label="Value at Risk (5%)"
              value={`${risk.metrics.value_at_risk.toFixed(2)}%`}
              subValue="Potential 1-day loss"
              color="#f44336"
              tooltip="5% chance of losing more than this amount in a single day."
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
