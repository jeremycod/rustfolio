import { useMemo, useState } from 'react';
import {
  Box,
  Grid,
  Card,
  CardContent,
  Typography,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Button,
  Alert,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Paper,
  CircularProgress,
  LinearProgress,
  Divider,
} from '@mui/material';
import {
  Add,
  Refresh,
  TrendingUp,
  TrendingDown,
  Remove as TrendingFlat,
  Warning,
  ArrowForward,
  Info,
} from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  createPortfolio,
  listPortfolios,
  getAnalytics,
  listAccounts,
  getPortfolioTruePerformance,
  getPortfolioRisk,
  getRiskAlerts,
  getPortfolioNews,
  getPortfolioSentiment,
  getPortfolioOptimization,
  getPortfolioNarrative,
} from '../lib/endpoints';
interface DashboardProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
  onNavigate?: (page: string) => void;
}

export function Dashboard({ selectedPortfolioId, onPortfolioChange, onNavigate }: DashboardProps) {
  const qc = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [portfolioName, setPortfolioName] = useState('');

  // Fetch all necessary data
  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const accountsQ = useQuery({
    queryKey: ['accounts', selectedPortfolioId],
    queryFn: () => listAccounts(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  const performanceQ = useQuery({
    queryKey: ['portfolioPerformance', selectedPortfolioId],
    queryFn: () => getPortfolioTruePerformance(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  const analyticsQ = useQuery({
    queryKey: ['analytics', selectedPortfolioId],
    queryFn: () => getAnalytics(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  const riskQ = useQuery({
    queryKey: ['portfolioRisk', selectedPortfolioId],
    queryFn: () => getPortfolioRisk(selectedPortfolioId!, 90, 'SPY'),
    enabled: !!selectedPortfolioId,
  });

  const alertsQ = useQuery({
    queryKey: ['riskAlerts', selectedPortfolioId],
    queryFn: () => getRiskAlerts(selectedPortfolioId!, 20),
    enabled: !!selectedPortfolioId,
  });

  const newsQ = useQuery({
    queryKey: ['portfolioNews', selectedPortfolioId],
    queryFn: () => getPortfolioNews(selectedPortfolioId!, 7),
    enabled: !!selectedPortfolioId,
    retry: 0, // Don't retry - first load is slow
    staleTime: 30 * 60 * 1000, // 30 minutes - news doesn't change that fast
    gcTime: 30 * 60 * 1000,
  });

  const sentimentQ = useQuery({
    queryKey: ['portfolioSentiment', selectedPortfolioId],
    queryFn: () => getPortfolioSentiment(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
    retry: 0, // Don't retry on failure
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 5 * 60 * 1000,
  });

  const optimizationQ = useQuery({
    queryKey: ['portfolioOptimization', selectedPortfolioId],
    queryFn: () => getPortfolioOptimization(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
    retry: 0, // Don't retry on failure
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 5 * 60 * 1000,
  });

  const narrativeQ = useQuery({
    queryKey: ['portfolioNarrative', selectedPortfolioId, '30d'],
    queryFn: () => getPortfolioNarrative(selectedPortfolioId!, '30d'),
    enabled: !!selectedPortfolioId,
    retry: 0, // Don't retry - expensive LLM call
    staleTime: 60 * 60 * 1000, // 1 hour - narrative is expensive to generate
    gcTime: 60 * 60 * 1000,
  });

  const createPortfolioM = useMutation({
    mutationFn: (name: string) => createPortfolio(name),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ['portfolios'] });
      setIsModalOpen(false);
      setPortfolioName('');
    },
  });

  const handleCreatePortfolio = () => {
    if (portfolioName.trim()) {
      createPortfolioM.mutate(portfolioName.trim());
    }
  };

  // Calculated values
  const portfolioValue = useMemo(() => {
    if (!performanceQ.data) return 0;
    return performanceQ.data.reduce((sum, acc) => sum + parseFloat(acc.current_value), 0);
  }, [performanceQ.data]);

  const totalDeposits = useMemo(() => {
    if (!performanceQ.data) return 0;
    return performanceQ.data.reduce((sum, acc) => sum + parseFloat(acc.total_deposits), 0);
  }, [performanceQ.data]);

  const totalGainLoss = useMemo(() => {
    if (!performanceQ.data) return 0;
    return performanceQ.data.reduce((sum, acc) => sum + parseFloat(acc.true_gain_loss), 0);
  }, [performanceQ.data]);

  const riskScore = riskQ.data?.portfolio_risk_score ?? null;
  const riskLevel = riskQ.data?.risk_level ?? null;
  const volatility = riskQ.data?.portfolio_volatility ?? null;
  const sharpeRatio = riskQ.data?.portfolio_sharpe ?? null;

  // Sentiment aggregation
  const sentimentSummary = useMemo(() => {
    if (!sentimentQ.data?.signals) return { positive: 0, neutral: 0, negative: 0 };

    let positive = 0, neutral = 0, negative = 0;
    sentimentQ.data.signals.forEach((signal: any) => {
      if (signal.current_sentiment > 0.2) positive++;
      else if (signal.current_sentiment < -0.2) negative++;
      else neutral++;
    });

    return { positive, neutral, negative };
  }, [sentimentQ.data]);

  // Top risk contributors
  const topRiskPositions = useMemo(() => {
    if (!riskQ.data?.position_risks) return [];
    return [...riskQ.data.position_risks]
      .sort((a, b) => (b.risk_assessment?.risk_score ?? 0) - (a.risk_assessment?.risk_score ?? 0))
      .slice(0, 5);
  }, [riskQ.data]);

  // Top holdings for table
  const topHoldings = useMemo(() => {
    if (!riskQ.data?.position_risks || !analyticsQ.data?.allocations) return [];

    const allocMap = new Map(
      analyticsQ.data.allocations.map(a => [a.ticker, a])
    );

    return [...riskQ.data.position_risks]
      .map(risk => {
        const alloc = allocMap.get(risk.ticker);
        return {
          ticker: risk.ticker,
          value: alloc?.value ?? 0,
          weight: risk.weight ?? alloc?.weight ?? 0,
          riskScore: risk.risk_assessment?.risk_score ?? 0,
          volatility: risk.risk_assessment?.metrics?.volatility ?? 0,
        };
      })
      .sort((a, b) => b.weight - a.weight)
      .slice(0, 10);
  }, [riskQ.data, analyticsQ.data]);

  const getRiskColor = (score: number) => {
    if (score >= 70) return 'error';
    if (score >= 50) return 'warning';
    return 'success';
  };

  const getRiskIcon = (score: number) => {
    if (score >= 70) return '🔴';
    if (score >= 50) return '🟠';
    return '🟢';
  };

  const getRiskLevelColor = (level: string) => {
    if (level === 'High') return 'error';
    if (level === 'Moderate') return 'warning';
    return 'success';
  };

  const getSharpeColor = (sharpe: number) => {
    if (sharpe >= 1.0) return 'success.main';
    if (sharpe >= 0.5) return 'warning.main';
    return 'error.main';
  };

  const getSentimentIcon = () => {
    const { positive, negative } = sentimentSummary;
    if (positive > negative) return '😊';
    if (negative > positive) return '😟';
    return '😐';
  };

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Dashboard
      </Typography>

      {/* Portfolio Selector */}
      <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', mb: 3 }}>
        <FormControl sx={{ minWidth: 200 }}>
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

        <Button
          variant="contained"
          startIcon={<Add />}
          onClick={() => setIsModalOpen(true)}
        >
          New Portfolio
        </Button>

        <Button
          variant="outlined"
          startIcon={<Refresh />}
          onClick={() => qc.invalidateQueries()}
        >
          Refresh
        </Button>
      </Box>

      {/* Critical Alerts Banner */}
      {selectedPortfolioId && alertsQ.data && alertsQ.data.length > 0 && (() => {
        // Filter out invalid alerts
        const validAlerts = alertsQ.data.filter(
          alert => alert.ticker && alert.metric_name && alert.percentage_change != null
        );

        return validAlerts.length > 0 ? (
          <Alert severity="warning" icon={<Warning />} sx={{ mb: 3 }}>
            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, flexWrap: 'wrap' }}>
              <Typography variant="body2" fontWeight="bold">
                ⚠️ {validAlerts.length} Risk Alert{validAlerts.length > 1 ? 's' : ''}
              </Typography>
              {validAlerts.slice(0, 3).map((alert, idx) => (
                <Chip
                  key={idx}
                  label={`${alert.ticker}: ${alert.metric_name} increased ${alert.percentage_change.toFixed(1)}%`}
                  size="small"
                  color="warning"
                  variant="outlined"
                />
              ))}
              {validAlerts.length > 3 && (
                <Chip label={`+${validAlerts.length - 3} more`} size="small" variant="outlined" />
              )}
              <Button
                size="small"
                endIcon={<ArrowForward />}
                onClick={() => onNavigate?.('portfolio-risk')}
              >
                View Details
              </Button>
            </Box>
          </Alert>
        ) : null;
      })()}

      {/* Enhanced KPI Cards - 8 cards in 2 rows */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        {/* Row 1: Financial Metrics */}
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom variant="body2">
                Total Portfolio Value
              </Typography>
              <Typography variant="h5">
                ${portfolioValue.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom variant="body2">
                Total Deposits
              </Typography>
              <Typography variant="h5">
                ${totalDeposits.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom variant="body2">
                True Gain/Loss
              </Typography>
              <Typography variant="h5" color={totalGainLoss >= 0 ? 'success.main' : 'error.main'}>
                ${totalGainLoss >= 0 ? '+' : ''}{totalGainLoss.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom variant="body2">
                Accounts
              </Typography>
              <Typography variant="h5">
                {accountsQ.data?.length ?? 0}
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        {/* Row 2: Risk Metrics */}
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom variant="body2">
                Portfolio Risk Score
              </Typography>
              {riskScore !== null ? (
                <Box>
                  <Box sx={{ display: 'flex', alignItems: 'baseline', gap: 1 }}>
                    <Typography variant="h5">
                      {riskScore.toFixed(0)}
                    </Typography>
                    <Typography variant="body2" color="textSecondary">
                      / 100
                    </Typography>
                  </Box>
                  <Chip
                    label={riskLevel ?? 'Unknown'}
                    size="small"
                    color={getRiskLevelColor(riskLevel ?? '')}
                    sx={{ mt: 1 }}
                  />
                </Box>
              ) : (
                <Typography variant="body2" color="textSecondary">
                  {riskQ.isLoading ? <CircularProgress size={20} /> : 'N/A'}
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom variant="body2">
                Portfolio Volatility
              </Typography>
              {volatility !== null ? (
                <Box>
                  <Typography variant="h5">
                    {(volatility * 100).toFixed(2)}%
                  </Typography>
                  <Typography variant="caption" color="textSecondary">
                    Annualized
                  </Typography>
                </Box>
              ) : (
                <Typography variant="body2" color="textSecondary">
                  {riskQ.isLoading ? <CircularProgress size={20} /> : 'N/A'}
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom variant="body2">
                Sharpe Ratio
              </Typography>
              {sharpeRatio !== null ? (
                <Box>
                  <Typography variant="h5" color={getSharpeColor(sharpeRatio)}>
                    {sharpeRatio.toFixed(2)}
                  </Typography>
                  <Typography variant="caption" color="textSecondary">
                    Risk-adjusted return
                  </Typography>
                </Box>
              ) : (
                <Typography variant="body2" color="textSecondary">
                  {riskQ.isLoading ? <CircularProgress size={20} /> : 'N/A'}
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom variant="body2">
                News Sentiment
              </Typography>
              {sentimentQ.isLoading ? (
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <CircularProgress size={20} />
                  <Typography variant="caption" color="textSecondary">
                    Loading...
                  </Typography>
                </Box>
              ) : sentimentQ.isError ? (
                <Typography variant="caption" color="error">
                  Failed to load sentiment
                  {sentimentQ.error && (
                    <Typography variant="caption" display="block" sx={{ fontSize: '0.7rem' }}>
                      {String(sentimentQ.error)}
                    </Typography>
                  )}
                </Typography>
              ) : sentimentQ.data ? (
                <Box>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                    <Typography variant="h5">
                      {getSentimentIcon()}
                    </Typography>
                    <Typography variant="h6">
                      {sentimentSummary.positive > sentimentSummary.negative ? 'Positive' :
                       sentimentSummary.negative > sentimentSummary.positive ? 'Negative' : 'Neutral'}
                    </Typography>
                  </Box>
                  <Box sx={{ display: 'flex', gap: 1 }}>
                    <Chip label={`+${sentimentSummary.positive}`} size="small" color="success" />
                    <Chip label={`~${sentimentSummary.neutral}`} size="small" />
                    <Chip label={`-${sentimentSummary.negative}`} size="small" color="error" />
                  </Box>
                </Box>
              ) : (
                <Typography variant="body2" color="textSecondary">
                  N/A
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Quick Insights Section */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        {/* Top Risk Contributors */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Top Risk Contributors
              </Typography>
              {riskQ.isLoading ? (
                <CircularProgress />
              ) : topRiskPositions.length > 0 ? (
                <Box>
                  {topRiskPositions.map((pos, idx) => (
                    <Box
                      key={idx}
                      sx={{
                        display: 'flex',
                        justifyContent: 'space-between',
                        alignItems: 'center',
                        py: 1,
                        borderBottom: idx < topRiskPositions.length - 1 ? '1px solid #eee' : 'none',
                      }}
                    >
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                        <Typography variant="body2">{getRiskIcon(pos.risk_assessment?.risk_score ?? 0)}</Typography>
                        <Typography variant="body2" fontWeight="bold">
                          {pos.ticker}
                        </Typography>
                      </Box>
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                        <Chip
                          label={`Risk: ${pos.risk_assessment?.risk_score?.toFixed(0) ?? 'N/A'}`}
                          size="small"
                          color={getRiskColor(pos.risk_assessment?.risk_score ?? 0)}
                        />
                        <Typography variant="caption" color="textSecondary">
                          Vol: {pos.risk_assessment?.metrics?.volatility ? (pos.risk_assessment.metrics.volatility).toFixed(1) : 'N/A'}%
                        </Typography>
                      </Box>
                    </Box>
                  ))}
                  <Button
                    size="small"
                    endIcon={<ArrowForward />}
                    onClick={() => onNavigate?.('portfolio-risk')}
                    sx={{ mt: 2 }}
                  >
                    View Full Risk Analysis
                  </Button>
                </Box>
              ) : (
                <Typography color="textSecondary">No risk data available</Typography>
              )}
            </CardContent>
          </Card>
        </Grid>

        {/* Optimization Recommendations */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Optimization Recommendations
              </Typography>
              {optimizationQ.isLoading ? (
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <CircularProgress size={24} />
                  <Typography variant="body2" color="textSecondary">
                    Analyzing portfolio...
                  </Typography>
                </Box>
              ) : optimizationQ.isError ? (
                <Box>
                  <Typography variant="body2" color="error">
                    Failed to load optimization recommendations
                  </Typography>
                  {optimizationQ.error && (
                    <Typography variant="caption" color="error" display="block" sx={{ mt: 1, fontSize: '0.7rem' }}>
                      {String(optimizationQ.error)}
                    </Typography>
                  )}
                </Box>
              ) : optimizationQ.data?.recommendations && optimizationQ.data.recommendations.length > 0 ? (
                <Box>
                  {optimizationQ.data.recommendations.slice(0, 3).map((rec, idx) => (
                    <Box
                      key={idx}
                      sx={{
                        py: 1,
                        borderBottom: idx < Math.min(2, optimizationQ.data.recommendations.length - 1) ? '1px solid #eee' : 'none',
                      }}
                    >
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0.5 }}>
                        <Chip label={rec.action_type} size="small" color="primary" variant="outlined" />
                        <Typography variant="body2" fontWeight="bold">
                          {rec.ticker}
                        </Typography>
                      </Box>
                      <Typography variant="caption" color="textSecondary">
                        {rec.rationale}
                      </Typography>
                    </Box>
                  ))}
                  <Button
                    size="small"
                    endIcon={<ArrowForward />}
                    onClick={() => onNavigate?.('portfolio-risk')}
                    sx={{ mt: 2 }}
                  >
                    View All Recommendations
                  </Button>
                </Box>
              ) : (
                <Typography color="textSecondary">No recommendations available</Typography>
              )}
            </CardContent>
          </Card>
        </Grid>

        {/* Recent News Themes */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Recent News Themes
              </Typography>
              {newsQ.isLoading ? (
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <CircularProgress size={24} />
                  <Typography variant="body2" color="textSecondary">
                    Fetching news (first load may take 30-60 seconds)...
                  </Typography>
                </Box>
              ) : newsQ.isError ? (
                <Typography variant="body2" color="error">
                  Failed to load news. Try refreshing the page.
                </Typography>
              ) : newsQ.data?.themes && newsQ.data.themes.length > 0 ? (
                <Box>
                  {newsQ.data.themes.slice(0, 3).map((theme, idx) => (
                    <Box
                      key={idx}
                      sx={{
                        py: 1,
                        borderBottom: idx < Math.min(2, newsQ.data.themes.length - 1) ? '1px solid #eee' : 'none',
                      }}
                    >
                      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 0.5 }}>
                        <Typography variant="body2" fontWeight="bold">
                          {theme.theme}
                        </Typography>
                        <Chip
                          label={theme.sentiment}
                          size="small"
                          color={
                            theme.sentiment === 'Positive' ? 'success' :
                            theme.sentiment === 'Negative' ? 'error' : 'default'
                          }
                        />
                      </Box>
                      <Typography variant="caption" color="textSecondary">
                        Relevance: {(theme.relevance_score * 100).toFixed(0)}% • {theme.article_count} articles
                      </Typography>
                    </Box>
                  ))}
                  <Button
                    size="small"
                    endIcon={<ArrowForward />}
                    onClick={() => onNavigate?.('portfolio-risk')}
                    sx={{ mt: 2 }}
                  >
                    View Full News Analysis
                  </Button>
                </Box>
              ) : (
                <Typography color="textSecondary">No news themes available</Typography>
              )}
            </CardContent>
          </Card>
        </Grid>

        {/* AI Portfolio Narrative */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                AI Portfolio Narrative
              </Typography>
              {narrativeQ.isLoading ? (
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <CircularProgress size={24} />
                  <Typography variant="body2" color="textSecondary">
                    Generating narrative (may take 20-30 seconds)...
                  </Typography>
                </Box>
              ) : narrativeQ.isError ? (
                <Typography variant="body2" color="error">
                  Failed to generate narrative. Try refreshing the page.
                </Typography>
              ) : narrativeQ.data?.summary ? (
                <Box>
                  <Typography variant="body2" paragraph>
                    {narrativeQ.data.summary}
                  </Typography>
                  {narrativeQ.data.top_contributors && narrativeQ.data.top_contributors.length > 0 && (
                    <Box sx={{ mt: 2 }}>
                      <Typography variant="caption" color="textSecondary" fontWeight="bold" display="block" sx={{ mb: 1 }}>
                        Top Contributors:
                      </Typography>
                      {narrativeQ.data.top_contributors.slice(0, 2).map((contributor: string, idx: number) => (
                        <Typography key={idx} variant="caption" color="textSecondary" display="block" sx={{ mb: 0.5 }}>
                          • {contributor.split(':')[0]}
                        </Typography>
                      ))}
                    </Box>
                  )}
                  <Button
                    size="small"
                    endIcon={<ArrowForward />}
                    onClick={() => onNavigate?.('portfolio-risk')}
                    sx={{ mt: 2 }}
                  >
                    View Full Narrative
                  </Button>
                </Box>
              ) : (
                <Typography color="textSecondary">No narrative available</Typography>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Visualization Row */}
      <Grid container spacing={3}>
        {/* Top Holdings Table */}
        <Grid item xs={12} md={8}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Top Holdings
              </Typography>
              {riskQ.isLoading || analyticsQ.isLoading ? (
                <CircularProgress />
              ) : topHoldings.length > 0 ? (
                <TableContainer>
                  <Table size="small">
                    <TableHead>
                      <TableRow>
                        <TableCell>Ticker</TableCell>
                        <TableCell align="right">Value</TableCell>
                        <TableCell align="right">Weight</TableCell>
                        <TableCell align="right">Risk</TableCell>
                        <TableCell align="right">Volatility</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {topHoldings.map((holding, idx) => (
                        <TableRow key={idx} hover sx={{ cursor: 'pointer' }}>
                          <TableCell>
                            <Typography variant="body2" fontWeight="bold">
                              {holding.ticker}
                            </Typography>
                          </TableCell>
                          <TableCell align="right">
                            ${holding.value.toLocaleString('en-US', { maximumFractionDigits: 0 })}
                          </TableCell>
                          <TableCell align="right">
                            {holding.weight ? (holding.weight * 100).toFixed(1) : '0.0'}%
                          </TableCell>
                          <TableCell align="right">
                            <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'flex-end', gap: 0.5 }}>
                              <Typography variant="caption">{getRiskIcon(holding.riskScore ?? 0)}</Typography>
                              <Chip
                                label={holding.riskScore?.toFixed(0) ?? 'N/A'}
                                size="small"
                                color={getRiskColor(holding.riskScore ?? 0)}
                              />
                            </Box>
                          </TableCell>
                          <TableCell align="right">
                            {holding.volatility ? (holding.volatility * 100).toFixed(1) : 'N/A'}%
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </TableContainer>
              ) : (
                <Typography color="textSecondary" sx={{ textAlign: 'center', py: 4 }}>
                  No holdings to display
                </Typography>
              )}
              {topHoldings.length > 0 && riskQ.data && riskQ.data.position_risks.length > 10 && (
                <Box sx={{ mt: 2, display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
                  <Typography variant="caption" color="textSecondary">
                    + {riskQ.data.position_risks.length - 10} more positions
                  </Typography>
                  <Button
                    size="small"
                    endIcon={<ArrowForward />}
                    onClick={() => onNavigate?.('holdings')}
                  >
                    View All Holdings
                  </Button>
                </Box>
              )}
            </CardContent>
          </Card>
        </Grid>

        {/* Risk Overview */}
        <Grid item xs={12} md={4}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Risk Overview
              </Typography>
              {riskQ.isLoading ? (
                <CircularProgress />
              ) : riskQ.data?.portfolio_risk_score ? (
                <Box>
                  {/* Risk Score Gauge */}
                  <Box sx={{ textAlign: 'center', mb: 3 }}>
                    <Typography variant="h2" color={getRiskLevelColor(riskLevel ?? '')}>
                      {riskScore?.toFixed(0)}
                    </Typography>
                    <Typography variant="body2" color="textSecondary">
                      Risk Score (0-100)
                    </Typography>
                    <LinearProgress
                      variant="determinate"
                      value={riskScore ?? 0}
                      color={getRiskLevelColor(riskLevel ?? '')}
                      sx={{ mt: 2, height: 10, borderRadius: 5 }}
                    />
                  </Box>

                  <Divider sx={{ my: 2 }} />

                  {/* Key Metrics */}
                  <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                    <Box>
                      <Typography variant="caption" color="textSecondary">
                        Max Drawdown
                      </Typography>
                      <Typography variant="body1" fontWeight="bold" color="error.main">
                        {riskQ.data.portfolio_max_drawdown?.toFixed(2) ?? 'N/A'}%
                      </Typography>
                    </Box>

                    <Box>
                      <Typography variant="caption" color="textSecondary">
                        Portfolio Beta
                      </Typography>
                      <Typography variant="body1" fontWeight="bold">
                        {riskQ.data.portfolio_beta?.toFixed(2) ?? 'N/A'}
                      </Typography>
                    </Box>

                    <Box>
                      <Typography variant="caption" color="textSecondary">
                        Positions Analyzed
                      </Typography>
                      <Typography variant="body1" fontWeight="bold">
                        {riskQ.data.position_risks.length}
                      </Typography>
                    </Box>
                  </Box>

                  <Button
                    size="small"
                    endIcon={<ArrowForward />}
                    onClick={() => onNavigate?.('portfolio-risk')}
                    sx={{ mt: 3 }}
                  >
                    View Full Risk Analysis
                  </Button>
                </Box>
              ) : (
                <Typography color="textSecondary">No risk data available</Typography>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Alerts */}
      {(!accountsQ.data || accountsQ.data.length === 0) && (
        <Alert severity="info" sx={{ mt: 3 }} icon={<Info />}>
          No accounts found. Go to the Accounts tab to import CSV data and get started.
        </Alert>
      )}

      {/* Create Portfolio Modal */}
      <Dialog open={isModalOpen} onClose={() => setIsModalOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create New Portfolio</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Portfolio Name"
            fullWidth
            variant="outlined"
            value={portfolioName}
            onChange={(e) => setPortfolioName(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleCreatePortfolio()}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIsModalOpen(false)}>Cancel</Button>
          <Button
            onClick={handleCreatePortfolio}
            variant="contained"
            disabled={!portfolioName.trim() || createPortfolioM.isPending}
          >
            Create
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
