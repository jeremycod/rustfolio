import {
  Box,
  Paper,
  Typography,
  Grid,
  Tooltip,
  Alert,
} from '@mui/material';
import {
  ShowChart,
  TrendingUp,
  Link as LinkIcon,
  CheckCircle,
} from '@mui/icons-material';
import type { CorrelationStatistics } from '../types';

interface CorrelationStatsCardProps {
  statistics: CorrelationStatistics;
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

export default function CorrelationStatsCard({ statistics }: CorrelationStatsCardProps) {
  // Determine color for average correlation
  const getCorrelationColor = (corr: number): string => {
    const corrPercent = corr * 100;
    if (corrPercent < 40) return '#4caf50'; // green
    if (corrPercent < 70) return '#ff9800'; // orange
    return '#f44336'; // red
  };

  // Determine color for diversification score
  const getDiversificationColor = (score: number): string => {
    if (score >= 7) return '#4caf50'; // green
    if (score >= 4) return '#ff9800'; // orange
    return '#f44336'; // red
  };

  const avgCorrPercent = (statistics.average_correlation * 100).toFixed(1);
  const maxCorrPercent = (statistics.max_correlation * 100).toFixed(1);
  const diversificationScore = statistics.adjusted_diversification_score.toFixed(1);

  return (
    <Box>
      <Typography variant="h6" gutterBottom sx={{ mb: 2 }}>
        Correlation Statistics
      </Typography>

      <Grid container spacing={2} sx={{ mb: 2 }}>
        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            icon={<ShowChart />}
            label="Average Correlation"
            value={`${avgCorrPercent}%`}
            color={getCorrelationColor(statistics.average_correlation)}
            tooltip="Average correlation across all position pairs. Lower values indicate better diversification."
          />
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            icon={<TrendingUp />}
            label="Max Correlation"
            value={`${maxCorrPercent}%`}
            subValue={`Min: ${(statistics.min_correlation * 100).toFixed(1)}%`}
            color="#1976d2"
            tooltip="Highest correlation between any two positions in the portfolio."
          />
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            icon={<LinkIcon />}
            label="High Correlation Pairs"
            value={statistics.high_correlation_pairs.toString()}
            subValue="Pairs > 70%"
            color={statistics.high_correlation_pairs > 3 ? '#ff9800' : '#4caf50'}
            tooltip="Number of position pairs with correlation above 70%. High values suggest concentration risk."
          />
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <MetricCard
            icon={<CheckCircle />}
            label="Diversification Score"
            value={`${diversificationScore}/10`}
            subValue="Correlation-adjusted"
            color={getDiversificationColor(statistics.adjusted_diversification_score)}
            tooltip="Combines position count and correlation to measure true diversification. Higher is better."
          />
        </Grid>
      </Grid>

      {statistics.high_correlation_pairs > 3 && (
        <Alert severity="warning" sx={{ mb: 2 }}>
          Portfolio has {statistics.high_correlation_pairs} highly correlated pairs (&gt;70%).
          Consider diversifying across less correlated assets to reduce concentration risk.
        </Alert>
      )}

      {statistics.average_correlation > 0.7 && (
        <Alert severity="warning" sx={{ mb: 2 }}>
          Average correlation is high ({avgCorrPercent}%).
          Most positions tend to move together, which may limit diversification benefits during market stress.
        </Alert>
      )}
    </Box>
  );
}
