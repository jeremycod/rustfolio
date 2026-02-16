import { useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  Alert,
  AlertTitle,
  Chip,
  Grid,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  CircularProgress,
  Divider,
  Stack,
  Tooltip,
} from '@mui/material';
import {
  ExpandMore,
  TrendingUp,
  TrendingDown,
  Warning,
  Info,
  Error,
  CheckCircle,
  LocalFireDepartment,
  Psychology,
  BarChart,
} from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getPortfolioOptimization } from '../lib/endpoints';
import { formatCurrency, formatPercentage } from '../lib/formatters';
import type {
  OptimizationRecommendation,
  Severity,
  PortfolioHealth,
  OptimizationAnalysis,
} from '../types';

interface OptimizationRecommendationsProps {
  portfolioId: string;
}

export function OptimizationRecommendations({ portfolioId }: OptimizationRecommendationsProps) {
  const [expandedRec, setExpandedRec] = useState<string | null>(null);

  const { data: analysis, isLoading, error } = useQuery({
    queryKey: ['portfolioOptimization', portfolioId],
    queryFn: () => getPortfolioOptimization(portfolioId),
    staleTime: 1000 * 60 * 5, // 5 minutes
  });

  const handleAccordionChange = (recId: string) => {
    setExpandedRec(expandedRec === recId ? null : recId);
  };

  if (isLoading) {
    return (
      <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', py: 8 }}>
        <CircularProgress />
        <Typography sx={{ ml: 2 }}>Analyzing portfolio...</Typography>
      </Box>
    );
  }

  if (error) {
    return (
      <Alert severity="error" sx={{ my: 2 }}>
        Failed to analyze portfolio: {error instanceof Error ? error.message : 'Unknown error'}
      </Alert>
    );
  }

  if (!analysis) {
    return (
      <Alert severity="info" sx={{ my: 2 }}>
        No optimization analysis available.
      </Alert>
    );
  }

  return (
    <Box>
      {/* Portfolio Health Summary */}
      <Card sx={{ mb: 3, bgcolor: getHealthColor(analysis.summary.overall_health) }}>
        <CardContent>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
            {getHealthIcon(analysis.summary.overall_health)}
            <Box>
              <Typography variant="h5" color="white">
                Portfolio Health: {analysis.summary.overall_health.toUpperCase()}
              </Typography>
              <Typography variant="body2" sx={{ color: 'rgba(255,255,255,0.9)' }}>
                {analysis.summary.total_recommendations} recommendation
                {analysis.summary.total_recommendations !== 1 ? 's' : ''} found
              </Typography>
            </Box>
          </Box>

          <Grid container spacing={2}>
            <Grid item xs={12} sm={3}>
              <Typography variant="body2" sx={{ color: 'rgba(255,255,255,0.8)' }}>
                Critical Issues
              </Typography>
              <Typography variant="h6" color="white">
                {analysis.summary.critical_issues}
              </Typography>
            </Grid>
            <Grid item xs={12} sm={3}>
              <Typography variant="body2" sx={{ color: 'rgba(255,255,255,0.8)' }}>
                High Priority
              </Typography>
              <Typography variant="h6" color="white">
                {analysis.summary.high_priority}
              </Typography>
            </Grid>
            <Grid item xs={12} sm={3}>
              <Typography variant="body2" sx={{ color: 'rgba(255,255,255,0.8)' }}>
                Warnings
              </Typography>
              <Typography variant="h6" color="white">
                {analysis.summary.warnings}
              </Typography>
            </Grid>
            <Grid item xs={12} sm={3}>
              <Tooltip
                title={
                  analysis.current_metrics.correlation_adjusted_diversification_score
                    ? `Base Score: ${analysis.current_metrics.diversification_score.toFixed(1)} | Correlation-Adjusted: ${analysis.current_metrics.correlation_adjusted_diversification_score.toFixed(1)} (Avg Correlation: ${((analysis.current_metrics.average_correlation || 0) * 100).toFixed(0)}%)`
                    : 'Position count + concentration (Herfindahl index)'
                }
                arrow
              >
                <Box>
                  <Typography variant="body2" sx={{ color: 'rgba(255,255,255,0.8)' }}>
                    Diversification
                  </Typography>
                  <Typography variant="h6" color="white">
                    {analysis.current_metrics.correlation_adjusted_diversification_score?.toFixed(1) || analysis.current_metrics.diversification_score.toFixed(1)}/10
                  </Typography>
                  {analysis.current_metrics.correlation_adjusted_diversification_score && (
                    <Typography variant="caption" sx={{ color: 'rgba(255,255,255,0.7)' }}>
                      (correlation-adjusted)
                    </Typography>
                  )}
                </Box>
              </Tooltip>
            </Grid>
          </Grid>

          {analysis.summary.key_findings.length > 0 && (
            <Box sx={{ mt: 2, pt: 2, borderTop: '1px solid rgba(255,255,255,0.2)' }}>
              <Typography variant="body2" fontWeight={600} sx={{ color: 'rgba(255,255,255,0.9)', mb: 1 }}>
                Key Findings:
              </Typography>
              <Stack spacing={0.5}>
                {analysis.summary.key_findings.map((finding, idx) => (
                  <Typography key={idx} variant="body2" sx={{ color: 'rgba(255,255,255,0.8)' }}>
                    • {finding}
                  </Typography>
                ))}
              </Stack>
            </Box>
          )}
        </CardContent>
      </Card>

      {/* Current Portfolio Metrics */}
      <Card sx={{ mb: 3 }}>
        <CardContent>
          <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <BarChart /> Current Portfolio Metrics
          </Typography>
          <Divider sx={{ mb: 2 }} />

          <Grid container spacing={2}>
            <Grid item xs={12} sm={6} md={3}>
              <Typography color="textSecondary" variant="body2">
                Risk Score
              </Typography>
              <Typography variant="h6">
                {analysis.current_metrics.risk_score.toFixed(1)}/100
              </Typography>
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <Typography color="textSecondary" variant="body2">
                Volatility
              </Typography>
              <Typography variant="h6">
                {analysis.current_metrics.volatility.toFixed(2)}%
              </Typography>
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <Typography color="textSecondary" variant="body2">
                Largest Position
              </Typography>
              <Typography variant="h6">
                {analysis.current_metrics.largest_position_weight.toFixed(1)}%
              </Typography>
            </Grid>
            <Grid item xs={12} sm={6} md={3}>
              <Typography color="textSecondary" variant="body2">
                Positions
              </Typography>
              <Typography variant="h6">
                {analysis.current_metrics.position_count}
              </Typography>
            </Grid>
          </Grid>
        </CardContent>
      </Card>

      {/* Diversification Breakdown (if correlation-adjusted score is available) */}
      {analysis.current_metrics.correlation_adjusted_diversification_score && (
        <Card sx={{ mb: 3 }}>
          <CardContent>
            <Typography variant="h6" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
              <Info /> Diversification Breakdown
            </Typography>
            <Divider sx={{ mb: 2 }} />

            <Alert severity="info" sx={{ mb: 2 }}>
              Your portfolio diversification is scored using two methods:
            </Alert>

            <Grid container spacing={2}>
              <Grid item xs={12} md={6}>
                <Paper variant="outlined" sx={{ p: 2 }}>
                  <Typography variant="subtitle2" gutterBottom fontWeight={600}>
                    Basic Diversification Score
                  </Typography>
                  <Typography variant="h4" color="primary" gutterBottom>
                    {analysis.current_metrics.diversification_score.toFixed(1)}/10
                  </Typography>
                  <Typography variant="body2" color="textSecondary">
                    Based on number of positions and concentration (Herfindahl index).
                    Does not account for how correlated your positions are.
                  </Typography>
                </Paper>
              </Grid>
              <Grid item xs={12} md={6}>
                <Paper
                  variant="outlined"
                  sx={{
                    p: 2,
                    bgcolor: 'success.light',
                    borderColor: 'success.main',
                    borderWidth: 2,
                  }}
                >
                  <Typography variant="subtitle2" gutterBottom fontWeight={600}>
                    Correlation-Adjusted Score
                  </Typography>
                  <Typography variant="h4" color="success.dark" gutterBottom>
                    {analysis.current_metrics.correlation_adjusted_diversification_score.toFixed(1)}/10
                  </Typography>
                  <Typography variant="body2" sx={{ mb: 1 }}>
                    Accounts for how your positions move together (avg correlation:{' '}
                    <strong>{((analysis.current_metrics.average_correlation || 0) * 100).toFixed(0)}%</strong>).
                  </Typography>
                  <Typography variant="caption" color="textSecondary">
                    Lower correlation = better diversification = higher bonus (up to +4 points)
                  </Typography>
                </Paper>
              </Grid>
            </Grid>
          </CardContent>
        </Card>
      )}

      {/* Recommendations */}
      <Typography variant="h6" gutterBottom sx={{ mb: 2 }}>
        Optimization Recommendations
      </Typography>

      {analysis.recommendations.length === 0 ? (
        <Alert severity="success" icon={<CheckCircle />}>
          <AlertTitle>Great Job!</AlertTitle>
          Your portfolio is well-optimized. No recommendations at this time.
        </Alert>
      ) : (
        <Stack spacing={2}>
          {analysis.recommendations.map((rec) => (
            <RecommendationCard
              key={rec.id}
              recommendation={rec}
              expanded={expandedRec === rec.id}
              onToggle={() => handleAccordionChange(rec.id)}
            />
          ))}
        </Stack>
      )}
    </Box>
  );
}

// Individual recommendation card
interface RecommendationCardProps {
  recommendation: OptimizationRecommendation;
  expanded: boolean;
  onToggle: () => void;
}

function RecommendationCard({ recommendation, expanded, onToggle }: RecommendationCardProps) {
  return (
    <Accordion expanded={expanded} onChange={onToggle}>
      <AccordionSummary expandIcon={<ExpandMore />}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, width: '100%' }}>
          <Box>{getSeverityIcon(recommendation.severity)}</Box>
          <Box sx={{ flex: 1 }}>
            <Typography variant="subtitle1" fontWeight={600}>
              {recommendation.title}
            </Typography>
            <Box sx={{ display: 'flex', gap: 1, mt: 0.5 }}>
              <Chip
                label={recommendation.severity.toUpperCase()}
                size="small"
                color={getSeverityColor(recommendation.severity)}
              />
              <Chip
                label={getRecommendationTypeLabel(recommendation.recommendation_type)}
                size="small"
                variant="outlined"
              />
            </Box>
          </Box>
        </Box>
      </AccordionSummary>

      <AccordionDetails>
        {/* Rationale */}
        <Alert severity="info" icon={<Psychology />} sx={{ mb: 2 }}>
          <AlertTitle>Why This Matters</AlertTitle>
          {recommendation.rationale}
        </Alert>

        {/* Affected Positions */}
        {recommendation.affected_positions.length > 0 && (
          <Box sx={{ mb: 2 }}>
            <Typography variant="subtitle2" gutterBottom>
              Affected Positions
            </Typography>
            <TableContainer component={Paper} variant="outlined">
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>Ticker</TableCell>
                    <TableCell>Action</TableCell>
                    <TableCell align="right">Current</TableCell>
                    <TableCell align="right">Recommended</TableCell>
                    <TableCell align="right">Change</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {recommendation.affected_positions.map((pos, idx) => (
                    <TableRow key={idx}>
                      <TableCell>
                        <Typography variant="body2" fontWeight={600}>
                          {pos.ticker}
                        </Typography>
                        {pos.holding_name && (
                          <Typography variant="caption" color="textSecondary">
                            {pos.holding_name}
                          </Typography>
                        )}
                      </TableCell>
                      <TableCell>
                        <Chip
                          label={pos.action}
                          size="small"
                          color={pos.action === 'SELL' ? 'error' : pos.action === 'BUY' ? 'success' : 'default'}
                        />
                      </TableCell>
                      <TableCell align="right">
                        {formatCurrency(pos.current_value)}
                        <Typography variant="caption" display="block" color="textSecondary">
                          {pos.current_weight.toFixed(1)}%
                        </Typography>
                      </TableCell>
                      <TableCell align="right">
                        {formatCurrency(pos.recommended_value)}
                        <Typography variant="caption" display="block" color="textSecondary">
                          {pos.recommended_weight.toFixed(1)}%
                        </Typography>
                      </TableCell>
                      <TableCell align="right">
                        <Typography
                          variant="body2"
                          sx={{ color: pos.amount_change < 0 ? 'error.main' : 'success.main' }}
                        >
                          {formatCurrency(pos.amount_change)}
                        </Typography>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          </Box>
        )}

        {/* Expected Impact */}
        <Box sx={{ mb: 2 }}>
          <Typography variant="subtitle2" gutterBottom>
            Expected Impact
          </Typography>
          <Grid container spacing={2}>
            <Grid item xs={12} sm={6} md={4}>
              <ImpactMetric
                label="Risk Score"
                before={recommendation.expected_impact.risk_score_before}
                after={recommendation.expected_impact.risk_score_after}
                change={recommendation.expected_impact.risk_score_change}
                lowerIsBetter
              />
            </Grid>
            <Grid item xs={12} sm={6} md={4}>
              <ImpactMetric
                label="Volatility"
                before={recommendation.expected_impact.volatility_before}
                after={recommendation.expected_impact.volatility_after}
                change={recommendation.expected_impact.volatility_change}
                suffix="%"
                lowerIsBetter
              />
            </Grid>
            <Grid item xs={12} sm={6} md={4}>
              <ImpactMetric
                label="Diversification"
                before={recommendation.expected_impact.diversification_before}
                after={recommendation.expected_impact.diversification_after}
                change={recommendation.expected_impact.diversification_change}
                suffix="/10"
              />
            </Grid>
          </Grid>
        </Box>

        {/* Suggested Actions */}
        <Box>
          <Typography variant="subtitle2" gutterBottom>
            Suggested Actions
          </Typography>
          <Stack spacing={1}>
            {recommendation.suggested_actions.map((action, idx) => (
              <Box key={idx} sx={{ display: 'flex', gap: 1 }}>
                <Typography variant="body2" color="primary" fontWeight={600}>
                  {idx + 1}.
                </Typography>
                <Typography variant="body2">{action}</Typography>
              </Box>
            ))}
          </Stack>
        </Box>
      </AccordionDetails>
    </Accordion>
  );
}

// Impact metric display
interface ImpactMetricProps {
  label: string;
  before: number;
  after: number;
  change: number;
  suffix?: string;
  lowerIsBetter?: boolean;
}

function ImpactMetric({ label, before, after, change, suffix = '', lowerIsBetter = false }: ImpactMetricProps) {
  const isImprovement = lowerIsBetter ? change < 0 : change > 0;

  return (
    <Paper variant="outlined" sx={{ p: 1.5 }}>
      <Typography variant="caption" color="textSecondary" gutterBottom display="block">
        {label}
      </Typography>
      <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
        <Typography variant="body2">
          {before.toFixed(1)}
          {suffix} → {after.toFixed(1)}
          {suffix}
        </Typography>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 0.5 }}>
          {isImprovement ? (
            <TrendingUp fontSize="small" color="success" />
          ) : (
            <TrendingDown fontSize="small" color="error" />
          )}
          <Typography
            variant="caption"
            sx={{ color: isImprovement ? 'success.main' : 'error.main' }}
            fontWeight={600}
          >
            {change >= 0 ? '+' : ''}
            {change.toFixed(1)}
            {suffix}
          </Typography>
        </Box>
      </Box>
    </Paper>
  );
}

// Helper functions
function getHealthColor(health: PortfolioHealth): string {
  switch (health) {
    case 'excellent':
      return '#2e7d32'; // green
    case 'good':
      return '#388e3c'; // light green
    case 'fair':
      return '#f57c00'; // orange
    case 'poor':
      return '#d32f2f'; // red
    case 'critical':
      return '#b71c1c'; // dark red
  }
}

function getHealthIcon(health: PortfolioHealth) {
  switch (health) {
    case 'excellent':
      return <CheckCircle sx={{ fontSize: 40, color: 'white' }} />;
    case 'good':
      return <TrendingUp sx={{ fontSize: 40, color: 'white' }} />;
    case 'fair':
      return <Info sx={{ fontSize: 40, color: 'white' }} />;
    case 'poor':
      return <Warning sx={{ fontSize: 40, color: 'white' }} />;
    case 'critical':
      return <LocalFireDepartment sx={{ fontSize: 40, color: 'white' }} />;
  }
}

function getSeverityIcon(severity: Severity) {
  switch (severity) {
    case 'info':
      return <Info color="info" />;
    case 'warning':
      return <Warning color="warning" />;
    case 'high':
      return <Error color="error" />;
    case 'critical':
      return <LocalFireDepartment color="error" />;
  }
}

function getSeverityColor(severity: Severity): 'info' | 'warning' | 'error' {
  switch (severity) {
    case 'info':
      return 'info';
    case 'warning':
      return 'warning';
    case 'high':
    case 'critical':
      return 'error';
  }
}

function getRecommendationTypeLabel(type: string): string {
  return type
    .split('_')
    .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
    .join(' ');
}
