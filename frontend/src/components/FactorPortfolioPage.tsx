import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  Button,
  Grid,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  Alert,
  Slider,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Switch,
  FormControlLabel,
  Divider,
  LinearProgress,
  Tooltip,
  Tabs,
  Tab,
} from '@mui/material';
import {
  Category,
  TrendingUp,
  TrendingDown,
  ShowChart,
  Speed,
  Shield,
  Star,
  Info,
} from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getFactorRecommendations, listPortfolios } from '../lib/endpoints';
import type { FactorType, FactorAnalysisResponse, TickerFactorScores, FactorEtfSuggestion, FactorBacktestResult } from '../types';

const FACTOR_INFO: Record<FactorType, {
  label: string;
  description: string;
  icon: React.ReactNode;
  color: string;
}> = {
  value: {
    label: 'Value',
    description: 'Stocks trading below intrinsic value based on P/E, P/B, and dividend yield metrics.',
    icon: <Star />,
    color: '#1976d2',
  },
  growth: {
    label: 'Growth',
    description: 'Companies with above-average revenue and earnings growth rates.',
    icon: <TrendingUp />,
    color: '#2e7d32',
  },
  momentum: {
    label: 'Momentum',
    description: 'Stocks with strong recent price performance and positive trend continuation.',
    icon: <Speed />,
    color: '#ed6c02',
  },
  quality: {
    label: 'Quality',
    description: 'Companies with high profitability, low debt, and consistent earnings.',
    icon: <Shield />,
    color: '#9c27b0',
  },
  low_volatility: {
    label: 'Low Volatility',
    description: 'Stocks with lower price fluctuations and more stable returns.',
    icon: <ShowChart />,
    color: '#0288d1',
  },
};

export function FactorPortfolioPage() {
  const [selectedFactors, setSelectedFactors] = useState<FactorType[]>(['value', 'quality']);
  const [weights, setWeights] = useState<Record<string, number>>({
    value: 50,
    growth: 50,
    momentum: 50,
    quality: 50,
    low_volatility: 50,
  });
  const [limit, setLimit] = useState(10);
  const [submitted, setSubmitted] = useState(false);
  const [activeTab, setActiveTab] = useState(0);

  // Fetch portfolios to get first portfolio ID
  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  // Auto-select first portfolio
  const portfolioId = portfoliosQ.data?.[0]?.id;

  const normalizedWeights = () => {
    const active = selectedFactors.reduce((acc, f) => {
      acc[f] = weights[f];
      return acc;
    }, {} as Record<string, number>);
    const total = Object.values(active).reduce((a, b) => a + b, 0);
    if (total === 0) return active;
    return Object.fromEntries(
      Object.entries(active).map(([k, v]) => [k, v / total])
    );
  };

  const portfolioQ = useQuery({
    queryKey: ['factor-portfolio', portfolioId, selectedFactors],
    queryFn: () => {
      if (!portfolioId) throw new Error('No portfolio available. Please create a portfolio first.');
      return getFactorRecommendations(portfolioId, 252, true, true);
    },
    enabled: submitted && selectedFactors.length > 0 && !!portfolioId,
  });

  const toggleFactor = (factor: FactorType) => {
    setSelectedFactors(prev =>
      prev.includes(factor)
        ? prev.filter(f => f !== factor)
        : [...prev, factor]
    );
    setSubmitted(false);
  };

  const handleGenerate = () => {
    setSubmitted(true);
  };

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <Category sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Factor Portfolio Builder
        </Typography>
      </Box>

      {/* Factor Selection */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Select Investment Factors
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Choose the factors you want to include in your portfolio. Factor investing targets specific
          drivers of returns that have historically delivered premiums.
        </Typography>

        <Grid container spacing={2} mb={3}>
          {(Object.keys(FACTOR_INFO) as FactorType[]).map(factor => {
            const info = FACTOR_INFO[factor];
            const isSelected = selectedFactors.includes(factor);
            return (
              <Grid item xs={12} sm={6} md={4} key={factor}>
                <Card
                  sx={{
                    cursor: 'pointer',
                    border: 2,
                    borderColor: isSelected ? info.color : 'transparent',
                    transition: 'all 0.2s',
                    opacity: isSelected ? 1 : 0.7,
                    '&:hover': {
                      opacity: 1,
                      borderColor: isSelected ? info.color : 'divider',
                    },
                  }}
                  onClick={() => toggleFactor(factor)}
                >
                  <CardContent>
                    <Box display="flex" alignItems="center" gap={1.5}>
                      <Box sx={{ color: isSelected ? info.color : 'text.secondary' }}>
                        {info.icon}
                      </Box>
                      <Box flex={1}>
                        <Typography variant="subtitle1" fontWeight="bold">
                          {info.label}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                          {info.description}
                        </Typography>
                      </Box>
                      <Switch
                        checked={isSelected}
                        onChange={() => toggleFactor(factor)}
                        onClick={(e) => e.stopPropagation()}
                        sx={{ '& .MuiSwitch-switchBase.Mui-checked': { color: info.color } }}
                      />
                    </Box>
                  </CardContent>
                </Card>
              </Grid>
            );
          })}
        </Grid>

        {/* Weight Adjustment */}
        {selectedFactors.length > 1 && (
          <Box mb={3}>
            <Typography variant="subtitle2" gutterBottom>
              Factor Weights
            </Typography>
            <Typography variant="caption" color="text.secondary" display="block" mb={2}>
              Adjust the relative importance of each selected factor. Weights are normalized automatically.
            </Typography>
            <Grid container spacing={2}>
              {selectedFactors.map(factor => {
                const info = FACTOR_INFO[factor];
                return (
                  <Grid item xs={12} sm={6} key={factor}>
                    <Box display="flex" alignItems="center" gap={2}>
                      <Typography variant="body2" sx={{ minWidth: 100 }}>
                        {info.label}
                      </Typography>
                      <Slider
                        value={weights[factor]}
                        onChange={(_, v) => {
                          setWeights(prev => ({ ...prev, [factor]: v as number }));
                          setSubmitted(false);
                        }}
                        min={0}
                        max={100}
                        valueLabelDisplay="auto"
                        sx={{
                          flex: 1,
                          '& .MuiSlider-thumb': { bgcolor: info.color },
                          '& .MuiSlider-track': { bgcolor: info.color },
                        }}
                      />
                      <Typography variant="body2" fontWeight="bold" sx={{ minWidth: 40 }}>
                        {weights[factor]}%
                      </Typography>
                    </Box>
                  </Grid>
                );
              })}
            </Grid>
          </Box>
        )}

        <Box display="flex" gap={2} alignItems="center">
          <Button
            variant="contained"
            size="large"
            onClick={handleGenerate}
            disabled={selectedFactors.length === 0}
          >
            Build Factor Portfolio
          </Button>
          <Typography variant="body2" color="text.secondary">
            Top {limit} stocks per factor combination
          </Typography>
        </Box>
      </Paper>

      {/* Results */}
      {!submitted && (
        <Alert severity="info">
          Select one or more investment factors above and click "Build Factor Portfolio" to discover
          stocks that align with your factor preferences. You can also adjust factor weights to
          customize the blend.
        </Alert>
      )}

      {portfolioQ.isLoading && (
        <Box display="flex" flexDirection="column" alignItems="center" py={6}>
          <CircularProgress size={48} />
          <Typography variant="body1" color="text.secondary" mt={2}>
            Building factor portfolio...
          </Typography>
        </Box>
      )}

      {portfolioQ.error && (
        <Alert severity="error">
          Failed to build portfolio: {(portfolioQ.error as Error).message}
        </Alert>
      )}

      {portfolioQ.data && (
        <FactorPortfolioResults analysis={portfolioQ.data} />
      )}
    </Box>
  );
}

function FactorPortfolioResults({ analysis }: { analysis: FactorAnalysisResponse }) {
  const [activeTab, setActiveTab] = useState(0);

  return (
    <Box>
      {/* Summary */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Factor Analysis Summary
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={2}>
          Portfolio: {analysis.portfolio_name} • Analysis Date: {new Date(analysis.analysis_date).toLocaleDateString()}
        </Typography>

        <Grid container spacing={3}>
          <Grid item xs={12} sm={3}>
            <Typography variant="caption" color="text.secondary">HOLDINGS ANALYZED</Typography>
            <Typography variant="h5" fontWeight="bold">{analysis.holdings_scores.length}</Typography>
          </Grid>
          <Grid item xs={12} sm={3}>
            <Typography variant="caption" color="text.secondary">COMPOSITE SCORE</Typography>
            <Typography variant="h5" fontWeight="bold" color="primary.main">
              {analysis.summary.overall_composite_score.toFixed(1)}
            </Typography>
          </Grid>
          <Grid item xs={12} sm={3}>
            <Typography variant="caption" color="text.secondary">DOMINANT FACTOR</Typography>
            <Typography variant="h5" fontWeight="bold" color="success.main">
              {analysis.summary.dominant_factor}
            </Typography>
          </Grid>
          <Grid item xs={12} sm={3}>
            <Typography variant="caption" color="text.secondary">WEAKEST FACTOR</Typography>
            <Typography variant="h5" fontWeight="bold" color="warning.main">
              {analysis.summary.weakest_factor}
            </Typography>
          </Grid>
        </Grid>

        {/* Key Findings */}
        {analysis.summary.key_findings.length > 0 && (
          <Box mt={2}>
            <Typography variant="subtitle2" gutterBottom>Key Findings</Typography>
            <Box component="ul" sx={{ pl: 2, m: 0 }}>
              {analysis.summary.key_findings.map((finding, idx) => (
                <Typography key={idx} component="li" variant="body2" color="text.secondary">
                  {finding}
                </Typography>
              ))}
            </Box>
          </Box>
        )}

        {/* Factor Weights Applied */}
        <Box mt={2}>
          <Typography variant="subtitle2" gutterBottom>Factor Weights</Typography>
          <Box display="flex" gap={1} flexWrap="wrap">
            <Chip
              label={`Value: ${(analysis.factor_weights.value * 100).toFixed(0)}%`}
              size="small"
              variant="outlined"
            />
            <Chip
              label={`Growth: ${(analysis.factor_weights.growth * 100).toFixed(0)}%`}
              size="small"
              variant="outlined"
            />
            <Chip
              label={`Momentum: ${(analysis.factor_weights.momentum * 100).toFixed(0)}%`}
              size="small"
              variant="outlined"
            />
            <Chip
              label={`Quality: ${(analysis.factor_weights.quality * 100).toFixed(0)}%`}
              size="small"
              variant="outlined"
            />
            <Chip
              label={`Low Volatility: ${(analysis.factor_weights.low_volatility * 100).toFixed(0)}%`}
              size="small"
              variant="outlined"
            />
          </Box>
        </Box>
      </Paper>

      {/* Tabs */}
      <Paper sx={{ mb: 2 }}>
        <Tabs value={activeTab} onChange={(_, v) => setActiveTab(v)}>
          <Tab label={`Holdings (${analysis.holdings_scores.length})`} />
          <Tab label={`Factor Exposures (${analysis.factor_exposures.length})`} />
          <Tab label={`ETF Suggestions (${analysis.etf_suggestions.length})`} />
          {analysis.backtest_results.length > 0 && (
            <Tab label="Backtest Results" />
          )}
        </Tabs>
      </Paper>

      {activeTab === 0 && (
        <HoldingsScoresTable holdings={analysis.holdings_scores} />
      )}

      {activeTab === 1 && (
        <FactorExposuresTable exposures={analysis.factor_exposures} />
      )}

      {activeTab === 2 && (
        <ETFSuggestionsTable etfs={analysis.etf_suggestions} />
      )}

      {activeTab === 3 && (
        <BacktestResultsTable results={analysis.backtest_results} />
      )}

      {/* Disclaimer */}
      <Alert severity="info" icon={<Info />} sx={{ mt: 3 }}>
        <Typography variant="body2">
          <strong>Disclaimer:</strong> Factor-based recommendations are for informational purposes only.
          Factor premiums are not guaranteed and can underperform for extended periods.
          Past performance is not indicative of future results.
        </Typography>
      </Alert>
    </Box>
  );
}

function HoldingsScoresTable({ holdings }: { holdings: TickerFactorScores[] }) {
  return (
    <TableContainer component={Paper}>
      <Table size="small">
        <TableHead>
          <TableRow sx={{ bgcolor: 'grey.50' }}>
            <TableCell>Ticker</TableCell>
            <TableCell>Name</TableCell>
            <TableCell align="right">Weight</TableCell>
            <TableCell align="center">Composite</TableCell>
            <TableCell align="center">
              <Tooltip title={FACTOR_INFO.value.description}>
                <span>Value</span>
              </Tooltip>
            </TableCell>
            <TableCell align="center">
              <Tooltip title={FACTOR_INFO.growth.description}>
                <span>Growth</span>
              </Tooltip>
            </TableCell>
            <TableCell align="center">
              <Tooltip title={FACTOR_INFO.momentum.description}>
                <span>Momentum</span>
              </Tooltip>
            </TableCell>
            <TableCell align="center">
              <Tooltip title={FACTOR_INFO.quality.description}>
                <span>Quality</span>
              </Tooltip>
            </TableCell>
            <TableCell align="center">
              <Tooltip title={FACTOR_INFO.low_volatility.description}>
                <span>Low Vol</span>
              </Tooltip>
            </TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {holdings.map(holding => (
            <TableRow key={holding.ticker} hover>
              <TableCell>
                <Typography fontWeight="bold">{holding.ticker}</Typography>
              </TableCell>
              <TableCell>
                <Typography variant="body2" noWrap sx={{ maxWidth: 180 }}>
                  {holding.holding_name || holding.ticker}
                </Typography>
              </TableCell>
              <TableCell align="right">{(holding.weight * 100).toFixed(1)}%</TableCell>
              <TableCell align="center">
                <Box
                  sx={{
                    display: 'inline-flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    width: 36,
                    height: 36,
                    borderRadius: '50%',
                    bgcolor: holding.composite_score >= 70 ? 'success.light' :
                      holding.composite_score >= 40 ? 'warning.light' : 'error.light',
                    color: holding.composite_score >= 70 ? 'success.dark' :
                      holding.composite_score >= 40 ? 'warning.dark' : 'error.dark',
                    fontWeight: 'bold',
                    fontSize: '0.8rem',
                  }}
                >
                  {holding.composite_score.toFixed(0)}
                </Box>
              </TableCell>
              <TableCell align="center">
                <Typography
                  variant="body2"
                  fontWeight="bold"
                  color={
                    holding.value_score >= 70 ? 'success.main' :
                    holding.value_score >= 40 ? 'warning.main' : 'error.main'
                  }
                >
                  {holding.value_score.toFixed(1)}
                </Typography>
              </TableCell>
              <TableCell align="center">
                <Typography
                  variant="body2"
                  fontWeight="bold"
                  color={
                    holding.growth_score >= 70 ? 'success.main' :
                    holding.growth_score >= 40 ? 'warning.main' : 'error.main'
                  }
                >
                  {holding.growth_score.toFixed(1)}
                </Typography>
              </TableCell>
              <TableCell align="center">
                <Typography
                  variant="body2"
                  fontWeight="bold"
                  color={
                    holding.momentum_score >= 70 ? 'success.main' :
                    holding.momentum_score >= 40 ? 'warning.main' : 'error.main'
                  }
                >
                  {holding.momentum_score.toFixed(1)}
                </Typography>
              </TableCell>
              <TableCell align="center">
                <Typography
                  variant="body2"
                  fontWeight="bold"
                  color={
                    holding.quality_score >= 70 ? 'success.main' :
                    holding.quality_score >= 40 ? 'warning.main' : 'error.main'
                  }
                >
                  {holding.quality_score.toFixed(1)}
                </Typography>
              </TableCell>
              <TableCell align="center">
                <Typography
                  variant="body2"
                  fontWeight="bold"
                  color={
                    holding.low_volatility_score >= 70 ? 'success.main' :
                    holding.low_volatility_score >= 40 ? 'warning.main' : 'error.main'
                  }
                >
                  {holding.low_volatility_score.toFixed(1)}
                </Typography>
              </TableCell>
            </TableRow>
          ))}
          {holdings.length === 0 && (
            <TableRow>
              <TableCell colSpan={9} align="center" sx={{ py: 4 }}>
                <Typography color="text.secondary">
                  No holdings found in this portfolio.
                </Typography>
              </TableCell>
            </TableRow>
          )}
        </TableBody>
      </Table>
    </TableContainer>
  );
}

function FactorExposuresTable({ exposures }: { exposures: any[] }) {
  if (exposures.length === 0) {
    return (
      <Paper sx={{ p: 4, textAlign: 'center' }}>
        <Typography color="text.secondary">
          No factor exposure data available.
        </Typography>
      </Paper>
    );
  }

  return (
    <TableContainer component={Paper}>
      <Table size="small">
        <TableHead>
          <TableRow sx={{ bgcolor: 'grey.50' }}>
            <TableCell>Factor</TableCell>
            <TableCell>Description</TableCell>
            <TableCell align="center">Score</TableCell>
            <TableCell align="center">Exposure Level</TableCell>
            <TableCell align="right">Risk Premium</TableCell>
            <TableCell>Recommendation</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {exposures.map(exp => (
            <TableRow key={exp.factor} hover>
              <TableCell>
                <Chip
                  label={exp.label}
                  size="small"
                  sx={{
                    bgcolor: FACTOR_INFO[exp.factor as FactorType]?.color || 'grey.500',
                    color: 'white',
                    fontWeight: 'bold',
                  }}
                />
              </TableCell>
              <TableCell>
                <Typography variant="body2" color="text.secondary">
                  {exp.description}
                </Typography>
              </TableCell>
              <TableCell align="center">
                <Typography variant="h6" fontWeight="bold">
                  {exp.score.toFixed(1)}
                </Typography>
              </TableCell>
              <TableCell align="center">
                <Chip
                  label={exp.exposure_level.toUpperCase()}
                  size="small"
                  color={
                    exp.exposure_level === 'overweight' ? 'success' :
                    exp.exposure_level === 'underweight' ? 'error' : 'default'
                  }
                />
              </TableCell>
              <TableCell align="right">
                <Typography fontWeight="bold" color="success.main">
                  +{exp.expected_risk_premium.toFixed(2)}%
                </Typography>
              </TableCell>
              <TableCell>
                <Typography variant="body2">
                  {exp.recommendation}
                </Typography>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
}

function ETFSuggestionsTable({ etfs }: { etfs: FactorEtfSuggestion[] }) {
  if (etfs.length === 0) {
    return (
      <Paper sx={{ p: 4, textAlign: 'center' }}>
        <Typography color="text.secondary">
          No ETF suggestions available.
        </Typography>
      </Paper>
    );
  }

  return (
    <TableContainer component={Paper}>
      <Table size="small">
        <TableHead>
          <TableRow sx={{ bgcolor: 'grey.50' }}>
            <TableCell>Symbol</TableCell>
            <TableCell>Name</TableCell>
            <TableCell>Factor</TableCell>
            <TableCell align="right">Expense Ratio</TableCell>
            <TableCell align="right">AUM</TableCell>
            <TableCell>Description</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {etfs.map(etf => (
            <TableRow key={etf.ticker} hover>
              <TableCell>
                <Typography fontWeight="bold">{etf.ticker}</Typography>
              </TableCell>
              <TableCell>
                <Typography variant="body2" noWrap sx={{ maxWidth: 200 }}>
                  {etf.name}
                </Typography>
              </TableCell>
              <TableCell>
                <Chip
                  label={FACTOR_INFO[etf.factor]?.label || etf.factor}
                  size="small"
                  sx={{
                    bgcolor: FACTOR_INFO[etf.factor]?.color || 'grey.500',
                    color: 'white',
                  }}
                />
              </TableCell>
              <TableCell align="right">{(etf.expense_ratio * 100).toFixed(2)}%</TableCell>
              <TableCell align="right">${etf.aum_billions.toFixed(1)}B</TableCell>
              <TableCell>
                <Typography variant="body2" color="text.secondary">
                  {etf.description}
                </Typography>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
}

function BacktestResultsTable({ results }: { results: FactorBacktestResult[] }) {
  if (results.length === 0) {
    return (
      <Paper sx={{ p: 4, textAlign: 'center' }}>
        <Typography color="text.secondary">
          No backtest results available.
        </Typography>
      </Paper>
    );
  }

  return (
    <TableContainer component={Paper}>
      <Table size="small">
        <TableHead>
          <TableRow sx={{ bgcolor: 'grey.50' }}>
            <TableCell>Factor</TableCell>
            <TableCell align="right">Annual Return</TableCell>
            <TableCell align="right">Volatility</TableCell>
            <TableCell align="right">Sharpe Ratio</TableCell>
            <TableCell align="right">Max Drawdown</TableCell>
            <TableCell align="right">Cumulative Return</TableCell>
            <TableCell align="right">Observation Days</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {results.map(result => (
            <TableRow key={result.factor} hover>
              <TableCell>
                <Chip
                  label={FACTOR_INFO[result.factor]?.label || result.factor}
                  size="small"
                  sx={{
                    bgcolor: FACTOR_INFO[result.factor]?.color || 'grey.500',
                    color: 'white',
                  }}
                />
              </TableCell>
              <TableCell align="right">
                <Typography
                  fontWeight="bold"
                  color={result.annualized_return >= 0 ? 'success.main' : 'error.main'}
                >
                  {result.annualized_return >= 0 ? '+' : ''}{result.annualized_return.toFixed(2)}%
                </Typography>
              </TableCell>
              <TableCell align="right">{result.annualized_volatility.toFixed(2)}%</TableCell>
              <TableCell align="right">
                <Typography fontWeight="bold" color={result.sharpe_ratio >= 1 ? 'success.main' : 'text.primary'}>
                  {result.sharpe_ratio.toFixed(2)}
                </Typography>
              </TableCell>
              <TableCell align="right">
                <Typography color="error.main">
                  {result.max_drawdown.toFixed(2)}%
                </Typography>
              </TableCell>
              <TableCell align="right">
                <Typography
                  fontWeight="bold"
                  color={result.cumulative_return >= 0 ? 'success.main' : 'error.main'}
                >
                  {result.cumulative_return >= 0 ? '+' : ''}{result.cumulative_return.toFixed(2)}%
                </Typography>
              </TableCell>
              <TableCell align="right">{result.observation_days.toLocaleString()}</TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
}

function formatMarketCap(value: number): string {
  if (value >= 1e12) return `$${(value / 1e12).toFixed(1)}T`;
  if (value >= 1e9) return `$${(value / 1e9).toFixed(1)}B`;
  if (value >= 1e6) return `$${(value / 1e6).toFixed(0)}M`;
  return `$${value.toLocaleString()}`;
}
