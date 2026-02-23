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
  IconButton,
} from '@mui/material';
import { Assessment, Search, TrendingDown, Warning as WarningIcon, Info as InfoIcon, HelpOutline } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getPortfolioRisk, listPortfolios, searchTickers } from '../lib/endpoints';
import type { Portfolio } from '../types';
import { MetricHelpDialog } from './MetricHelpDialog';

interface CVaRAnalysisProps {
  selectedPortfolioId?: string | null;
}

export function CVaRAnalysis({ selectedPortfolioId }: CVaRAnalysisProps) {
  const [ticker, setTicker] = useState('');
  const [searchTicker, setSearchTicker] = useState('');
  const [days, setDays] = useState(90);
  const [portfolioId, setPortfolioId] = useState<string | null>(selectedPortfolioId || null);
  const [viewMode, setViewMode] = useState<'position' | 'portfolio'>('portfolio');
  const [helpOpen, setHelpOpen] = useState(false);
  const [selectedMetric, setSelectedMetric] = useState<string>('');

  // Fetch portfolios
  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  // Auto-select first portfolio if available
  useState(() => {
    if (portfoliosQ.data && portfoliosQ.data.length > 0 && !portfolioId) {
      setPortfolioId(portfoliosQ.data[0].id);
    }
  });

  // Fetch company name when ticker is searched
  const companyInfoQ = useQuery({
    queryKey: ['companyInfo', searchTicker],
    queryFn: () => searchTickers(searchTicker),
    enabled: !!searchTicker && viewMode === 'position',
    staleTime: 1000 * 60 * 60,
  });

  const companyName = companyInfoQ.data?.[0]?.name || null;

  // Fetch portfolio risk data
  const portfolioRiskQ = useQuery({
    queryKey: ['portfolioRisk', portfolioId, days],
    queryFn: () => portfolioId ? getPortfolioRisk(portfolioId, days) : Promise.reject('No portfolio'),
    enabled: !!portfolioId && viewMode === 'portfolio',
  });

  const handleSearch = () => {
    if (ticker.trim()) {
      setSearchTicker(ticker.trim().toUpperCase());
      setViewMode('position');
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter') {
      handleSearch();
    }
  };

  const handlePortfolioView = () => {
    setViewMode('portfolio');
    setSearchTicker('');
    setTicker('');
  };

  const formatPercent = (value: number | null | undefined): string => {
    if (value === null || value === undefined) return 'N/A';
    return `${value.toFixed(2)}%`;
  };

  const getRiskColor = (cvar: number | null | undefined): string => {
    if (cvar === null || cvar === undefined) return 'text.secondary';
    const absValue = Math.abs(cvar);
    if (absValue < 5) return 'success.main';
    if (absValue < 10) return 'warning.main';
    return 'error.main';
  };

  const getSeverityChip = (cvar: number | null | undefined) => {
    if (cvar === null || cvar === undefined) return null;
    const absValue = Math.abs(cvar);
    if (absValue < 5) return <Chip label="Low Risk" color="success" size="small" />;
    if (absValue < 10) return <Chip label="Moderate Risk" color="warning" size="small" />;
    return <Chip label="High Risk" color="error" size="small" />;
  };

  const renderPortfolioView = () => {
    if (!portfolioRiskQ.data) return null;

    const portfolioData = portfolioRiskQ.data;
    const portfolioVar95 = portfolioData.portfolio_var_95;
    const portfolioVar99 = portfolioData.portfolio_var_99;
    const portfolioCVaR95 = portfolioData.portfolio_expected_shortfall_95;
    const portfolioCVaR99 = portfolioData.portfolio_expected_shortfall_99;

    return (
      <Box>
        {/* Portfolio-level CVaR */}
        <Paper sx={{ p: 3, mb: 3 }}>
          <Typography variant="h6" gutterBottom>
            Portfolio Tail-Risk Summary
          </Typography>
          <Grid container spacing={3}>
            <Grid item xs={12} md={6}>
              <Card variant="outlined">
                <CardContent>
                  <Box display="flex" alignItems="center" justifyContent="space-between" mb={1}>
                    <Typography variant="subtitle2" color="text.secondary">
                      Value at Risk (95% Confidence)
                    </Typography>
                    <IconButton
                      size="small"
                      onClick={() => {
                        setSelectedMetric('var_95');
                        setHelpOpen(true);
                      }}
                      sx={{
                        p: 0.5,
                        color: 'text.secondary',
                        '&:hover': {
                          color: 'primary.main',
                          backgroundColor: 'primary.50',
                        },
                      }}
                    >
                      <HelpOutline fontSize="small" />
                    </IconButton>
                  </Box>
                  <Typography variant="h4" sx={{ color: getRiskColor(portfolioVar95) }}>
                    {formatPercent(portfolioVar95)}
                  </Typography>
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                    5% chance of exceeding this loss
                  </Typography>
                </CardContent>
              </Card>
            </Grid>
            <Grid item xs={12} md={6}>
              <Card variant="outlined" sx={{ bgcolor: 'error.50' }}>
                <CardContent>
                  <Box display="flex" alignItems="center" justifyContent="space-between" mb={1}>
                    <Typography variant="subtitle2" color="text.secondary">
                      Expected Shortfall / CVaR (95% Confidence)
                    </Typography>
                    <IconButton
                      size="small"
                      onClick={() => {
                        setSelectedMetric('portfolio_cvar_95');
                        setHelpOpen(true);
                      }}
                      sx={{
                        p: 0.5,
                        color: 'text.secondary',
                        '&:hover': {
                          color: 'primary.main',
                          backgroundColor: 'primary.50',
                        },
                      }}
                    >
                      <HelpOutline fontSize="small" />
                    </IconButton>
                  </Box>
                  <Typography variant="h4" sx={{ color: getRiskColor(portfolioCVaR95) }}>
                    {formatPercent(portfolioCVaR95)}
                  </Typography>
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                    Average loss in worst 5% scenarios
                  </Typography>
                  <Box sx={{ mt: 1 }}>{getSeverityChip(portfolioCVaR95)}</Box>
                </CardContent>
              </Card>
            </Grid>
            <Grid item xs={12} md={6}>
              <Card variant="outlined">
                <CardContent>
                  <Box display="flex" alignItems="center" justifyContent="space-between" mb={1}>
                    <Typography variant="subtitle2" color="text.secondary">
                      Value at Risk (99% Confidence)
                    </Typography>
                    <IconButton
                      size="small"
                      onClick={() => {
                        setSelectedMetric('var_99');
                        setHelpOpen(true);
                      }}
                      sx={{
                        p: 0.5,
                        color: 'text.secondary',
                        '&:hover': {
                          color: 'primary.main',
                          backgroundColor: 'primary.50',
                        },
                      }}
                    >
                      <HelpOutline fontSize="small" />
                    </IconButton>
                  </Box>
                  <Typography variant="h4" sx={{ color: getRiskColor(portfolioVar99) }}>
                    {formatPercent(portfolioVar99)}
                  </Typography>
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                    1% chance of exceeding this loss
                  </Typography>
                </CardContent>
              </Card>
            </Grid>
            <Grid item xs={12} md={6}>
              <Card variant="outlined" sx={{ bgcolor: 'error.100' }}>
                <CardContent>
                  <Box display="flex" alignItems="center" justifyContent="space-between" mb={1}>
                    <Typography variant="subtitle2" color="text.secondary">
                      Expected Shortfall / CVaR (99% Confidence)
                    </Typography>
                    <IconButton
                      size="small"
                      onClick={() => {
                        setSelectedMetric('portfolio_cvar_99');
                        setHelpOpen(true);
                      }}
                      sx={{
                        p: 0.5,
                        color: 'text.secondary',
                        '&:hover': {
                          color: 'primary.main',
                          backgroundColor: 'primary.50',
                        },
                      }}
                    >
                      <HelpOutline fontSize="small" />
                    </IconButton>
                  </Box>
                  <Typography variant="h4" sx={{ color: getRiskColor(portfolioCVaR99) }}>
                    {formatPercent(portfolioCVaR99)}
                  </Typography>
                  <Typography variant="body2" color="text.secondary" sx={{ mt: 1 }}>
                    Average loss in worst 1% scenarios
                  </Typography>
                  <Box sx={{ mt: 1 }}>{getSeverityChip(portfolioCVaR99)}</Box>
                </CardContent>
              </Card>
            </Grid>
          </Grid>
        </Paper>

        {/* Interpretation Alert */}
        <Alert severity="info" icon={<InfoIcon />} sx={{ mb: 3 }}>
          <Typography variant="subtitle2" gutterBottom>
            <strong>Understanding CVaR vs VaR:</strong>
          </Typography>
          <Typography variant="body2">
            • <strong>VaR</strong> tells you the threshold: "There's a 5% chance losses will exceed{' '}
            {formatPercent(portfolioVar95)}"
          </Typography>
          <Typography variant="body2">
            • <strong>CVaR</strong> tells you the average: "In the worst 5% of scenarios, average loss is{' '}
            {formatPercent(portfolioCVaR95)}"
          </Typography>
          <Typography variant="body2" sx={{ mt: 1 }}>
            CVaR is always worse than VaR because it accounts for the full distribution of tail losses, not just
            the threshold. This makes CVaR a more conservative and informative risk measure.
          </Typography>
        </Alert>

        {/* Position Breakdown */}
        <Paper sx={{ p: 3 }}>
          <Typography variant="h6" gutterBottom>
            Position-Level CVaR Breakdown
          </Typography>
          <TableContainer>
            <Table size="small">
              <TableHead>
                <TableRow>
                  <TableCell>Ticker</TableCell>
                  <TableCell align="right">Weight</TableCell>
                  <TableCell align="right">VaR-95</TableCell>
                  <TableCell align="right">CVaR-95</TableCell>
                  <TableCell align="right">VaR-99</TableCell>
                  <TableCell align="right">CVaR-99</TableCell>
                  <TableCell align="right">Risk Level</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {portfolioData.position_risks?.map((pos) => (
                  <TableRow key={pos.ticker}>
                    <TableCell>
                      <strong>{pos.ticker}</strong>
                    </TableCell>
                    <TableCell align="right">{(pos.weight * 100).toFixed(1)}%</TableCell>
                    <TableCell align="right" sx={{ color: getRiskColor(pos.risk_assessment.metrics.var_95) }}>
                      {formatPercent(pos.risk_assessment.metrics.var_95)}
                    </TableCell>
                    <TableCell align="right" sx={{ color: getRiskColor(pos.risk_assessment.metrics.expected_shortfall_95) }}>
                      <strong>{formatPercent(pos.risk_assessment.metrics.expected_shortfall_95)}</strong>
                    </TableCell>
                    <TableCell align="right" sx={{ color: getRiskColor(pos.risk_assessment.metrics.var_99) }}>
                      {formatPercent(pos.risk_assessment.metrics.var_99)}
                    </TableCell>
                    <TableCell align="right" sx={{ color: getRiskColor(pos.risk_assessment.metrics.expected_shortfall_99) }}>
                      <strong>{formatPercent(pos.risk_assessment.metrics.expected_shortfall_99)}</strong>
                    </TableCell>
                    <TableCell align="right">
                      <Chip
                        label={pos.risk_assessment.risk_level.toUpperCase()}
                        size="small"
                        color={
                          pos.risk_assessment.risk_level === 'low' ? 'success' :
                          pos.risk_assessment.risk_level === 'moderate' ? 'warning' : 'error'
                        }
                      />
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </TableContainer>
        </Paper>
      </Box>
    );
  };

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <TrendingDown sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          CVaR (Tail-Risk) Analysis
        </Typography>
      </Box>

      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Search & Filter
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Analyze Conditional Value at Risk (CVaR / Expected Shortfall) for your portfolio or individual positions.
          CVaR measures the average loss in worst-case scenarios beyond VaR.
        </Typography>

        <Grid container spacing={2} alignItems="center">
          <Grid item xs={12} sm={4}>
            <FormControl fullWidth>
              <InputLabel>Portfolio</InputLabel>
              <Select
                value={portfolioId || ''}
                label="Portfolio"
                onChange={(e) => setPortfolioId(e.target.value)}
              >
                {portfoliosQ.data?.map((p: Portfolio) => (
                  <MenuItem key={p.id} value={p.id}>
                    {p.name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          </Grid>

          <Grid item xs={12} sm={3}>
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

          <Grid item xs={12} sm={3}>
            <Button
              fullWidth
              variant={viewMode === 'portfolio' ? 'contained' : 'outlined'}
              size="large"
              onClick={handlePortfolioView}
              disabled={!portfolioId}
            >
              Portfolio View
            </Button>
          </Grid>

          <Grid item xs={12} sm={2}>
            <Button
              fullWidth
              variant="outlined"
              size="large"
              startIcon={<Search />}
              onClick={handleSearch}
              disabled={!ticker.trim()}
            >
              Search
            </Button>
          </Grid>
        </Grid>
      </Paper>

      {!portfolioId && (
        <Alert severity="warning" icon={<WarningIcon />}>
          Please select a portfolio to view CVaR analysis.
        </Alert>
      )}

      {portfolioRiskQ.isLoading && (
        <Box display="flex" justifyContent="center" alignItems="center" minHeight={400}>
          <CircularProgress />
        </Box>
      )}

      {portfolioRiskQ.error && (
        <Alert severity="error">
          Failed to load portfolio risk data: {(portfolioRiskQ.error as Error).message}
        </Alert>
      )}

      {portfolioRiskQ.data && viewMode === 'portfolio' && renderPortfolioView()}

      {!portfolioRiskQ.data && !portfolioRiskQ.isLoading && viewMode === 'portfolio' && portfolioId && (
        <Alert severity="info">
          Select a portfolio and time period to view CVaR analysis.
        </Alert>
      )}

      <MetricHelpDialog
        open={helpOpen}
        onClose={() => setHelpOpen(false)}
        metricKey={selectedMetric}
      />
    </Box>
  );
}
