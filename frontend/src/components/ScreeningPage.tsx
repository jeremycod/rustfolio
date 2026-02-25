import { useState, useCallback } from 'react';
import {
  Box,
  Typography,
  Paper,
  Button,
  Grid,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  TextField,
  Chip,
  CircularProgress,
  Alert,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  TableSortLabel,
  Slider,
  Switch,
  FormControlLabel,
  Collapse,
  IconButton,
  Tooltip,
  LinearProgress,
  Pagination,
} from '@mui/material';
import {
  FilterList,
  Search,
  Download,
  ExpandMore,
  ExpandLess,
  TrendingUp,
  TrendingDown,
  Star,
  StarBorder,
} from '@mui/icons-material';
import { useQuery, useMutation } from '@tanstack/react-query';
import { screenStocks, exportScreeningCSV } from '../lib/endpoints';
import type {
  ScreeningRequest,
  ScreeningResponse,
  ScreeningResult,
  ScreeningFactorCategory,
  ScreeningFilter,
  RiskAppetite,
} from '../types';

const SECTOR_OPTIONS = [
  'All Sectors',
  'Technology',
  'Healthcare',
  'Financials',
  'Consumer Discretionary',
  'Consumer Staples',
  'Energy',
  'Industrials',
  'Materials',
  'Real Estate',
  'Utilities',
  'Communication Services',
];

const MARKET_CAP_OPTIONS = [
  { label: 'All Sizes', value: '' },
  { label: 'Mega Cap (>$200B)', value: 'mega' },
  { label: 'Large Cap ($10B-$200B)', value: 'large' },
  { label: 'Mid Cap ($2B-$10B)', value: 'mid' },
  { label: 'Small Cap (<$2B)', value: 'small' },
];

const DEFAULT_FILTERS: ScreeningFilter[] = [
  { metric: 'pe_ratio', min: 0, max: 50, enabled: false },
  { metric: 'pb_ratio', min: 0, max: 10, enabled: false },
  { metric: 'debt_to_equity', min: 0, max: 2, enabled: false },
  { metric: 'rsi', min: 30, max: 70, enabled: false },
  { metric: 'earnings_growth', min: 0, max: 100, enabled: false },
  { metric: 'dividend_yield', min: 0, max: 10, enabled: false },
];

type SortField = 'composite_score' | 'symbol' | 'price' | 'market_cap' | 'risk_level';
type SortDir = 'asc' | 'desc';

export function ScreeningPage() {
  const [factors, setFactors] = useState<ScreeningFactorCategory[]>(['fundamental', 'technical']);
  const [filters, setFilters] = useState<ScreeningFilter[]>(DEFAULT_FILTERS);
  const [sector, setSector] = useState('All Sectors');
  const [marketCap, setMarketCap] = useState('');
  const [riskAppetite, setRiskAppetite] = useState<RiskAppetite>('moderate');
  const [horizon, setHorizon] = useState(6);
  const [priceMin, setPriceMin] = useState<string>('');
  const [priceMax, setPriceMax] = useState<string>('');
  const [showFilters, setShowFilters] = useState(true);
  const [sortField, setSortField] = useState<SortField>('composite_score');
  const [sortDir, setSortDir] = useState<SortDir>('desc');
  const [page, setPage] = useState(1);
  const [searchTriggered, setSearchTriggered] = useState(false);

  const pageSize = 20;

  const buildRequest = useCallback((): ScreeningRequest => {
    // Calculate weights from selected factors (equal weight for each selected factor)
    const selectedFactors = factors.length > 0 ? factors : ['fundamental', 'technical'];
    const weight = 1.0 / selectedFactors.length;

    return {
      symbols: [], // Empty means screen all stocks in database
      weights: {
        fundamental: selectedFactors.includes('fundamental') ? weight : 0,
        technical: selectedFactors.includes('technical') ? weight : 0,
        sentiment: selectedFactors.includes('sentiment') ? weight : 0,
        momentum: selectedFactors.includes('momentum') ? weight : 0,
      },
      filters: {
        sectors: sector !== 'All Sectors' ? [sector] : undefined,
        market_cap: marketCap ? marketCap as any : undefined,
        min_price: priceMin ? Number(priceMin) : undefined,
        max_price: priceMax ? Number(priceMax) : undefined,
        min_avg_volume: undefined,
        geographies: undefined,
      },
      limit: pageSize,
      offset: (page - 1) * pageSize,
      risk_appetite: riskAppetite,
      horizon_months: horizon,
      refresh: false,
    };
  }, [factors, sector, marketCap, priceMin, priceMax, page, riskAppetite, horizon]);

  const screeningQ = useQuery({
    queryKey: ['screening', factors, filters, sector, marketCap, priceMin, priceMax, sortField, sortDir, page, riskAppetite, horizon],
    queryFn: () => screenStocks(buildRequest()),
    enabled: searchTriggered,
  });

  const exportMutation = useMutation({
    mutationFn: () => exportScreeningCSV(buildRequest()),
  });

  const handleSearch = () => {
    setPage(1);
    setSearchTriggered(true);
  };

  const toggleFactor = (factor: ScreeningFactorCategory) => {
    setFactors(prev =>
      prev.includes(factor)
        ? prev.filter(f => f !== factor)
        : [...prev, factor]
    );
  };

  const toggleFilter = (index: number) => {
    setFilters(prev => prev.map((f, i) =>
      i === index ? { ...f, enabled: !f.enabled } : f
    ));
  };

  const updateFilterRange = (index: number, field: 'min' | 'max', value: number) => {
    setFilters(prev => prev.map((f, i) =>
      i === index ? { ...f, [field]: value } : f
    ));
  };

  const handleSort = (field: SortField) => {
    if (sortField === field) {
      setSortDir(prev => prev === 'asc' ? 'desc' : 'asc');
    } else {
      setSortField(field);
      setSortDir('desc');
    }
  };

  const getRiskColor = (level: string) => {
    switch (level) {
      case 'low': return 'success';
      case 'moderate': return 'warning';
      case 'high': return 'error';
      default: return 'default';
    }
  };

  const getStrengthColor = (strength: string): 'success' | 'warning' | 'error' => {
    switch (strength) {
      case 'Strong': return 'success';
      case 'Moderate': return 'warning';
      case 'Weak': return 'error';
      default: return 'warning';
    }
  };

  const formatMetricName = (metric: string) => {
    return metric.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase());
  };

  const formatMarketCap = (value: number): string => {
    if (value >= 1e12) return `$${(value / 1e12).toFixed(1)}T`;
    if (value >= 1e9) return `$${(value / 1e9).toFixed(1)}B`;
    if (value >= 1e6) return `$${(value / 1e6).toFixed(1)}M`;
    return `$${value.toLocaleString()}`;
  };

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <FilterList sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Stock Screener
        </Typography>
      </Box>

      {/* Factor Selection */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Factor Selection
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={2}>
          Select the factor categories to include in your screening analysis. Multiple factors are combined into a composite score.
        </Typography>
        <Box display="flex" gap={1} flexWrap="wrap" mb={3}>
          {(['fundamental', 'technical', 'sentiment', 'momentum'] as ScreeningFactorCategory[]).map(factor => (
            <Chip
              key={factor}
              label={factor.charAt(0).toUpperCase() + factor.slice(1)}
              color={factors.includes(factor) ? 'primary' : 'default'}
              onClick={() => toggleFactor(factor)}
              variant={factors.includes(factor) ? 'filled' : 'outlined'}
              sx={{ px: 1 }}
            />
          ))}
        </Box>

        <Grid container spacing={2}>
          <Grid item xs={12} sm={4}>
            <FormControl fullWidth size="small">
              <InputLabel>Risk Appetite</InputLabel>
              <Select
                value={riskAppetite}
                label="Risk Appetite"
                onChange={(e) => setRiskAppetite(e.target.value as RiskAppetite)}
              >
                <MenuItem value="conservative">Conservative</MenuItem>
                <MenuItem value="moderate">Moderate</MenuItem>
                <MenuItem value="aggressive">Aggressive</MenuItem>
              </Select>
            </FormControl>
          </Grid>
          <Grid item xs={12} sm={4}>
            <FormControl fullWidth size="small">
              <InputLabel>Investment Horizon</InputLabel>
              <Select
                value={horizon}
                label="Investment Horizon"
                onChange={(e) => setHorizon(Number(e.target.value))}
              >
                <MenuItem value={1}>1 Month</MenuItem>
                <MenuItem value={3}>3 Months</MenuItem>
                <MenuItem value={6}>6 Months</MenuItem>
                <MenuItem value={12}>12 Months</MenuItem>
                <MenuItem value={24}>24 Months</MenuItem>
              </Select>
            </FormControl>
          </Grid>
          <Grid item xs={12} sm={4}>
            <FormControl fullWidth size="small">
              <InputLabel>Sector</InputLabel>
              <Select value={sector} label="Sector" onChange={(e) => setSector(e.target.value)}>
                {SECTOR_OPTIONS.map(s => (
                  <MenuItem key={s} value={s}>{s}</MenuItem>
                ))}
              </Select>
            </FormControl>
          </Grid>
        </Grid>

        {/* Advanced Filters Toggle */}
        <Box mt={2}>
          <Button
            size="small"
            startIcon={showFilters ? <ExpandLess /> : <ExpandMore />}
            onClick={() => setShowFilters(!showFilters)}
          >
            {showFilters ? 'Hide' : 'Show'} Advanced Filters
          </Button>
        </Box>

        <Collapse in={showFilters}>
          <Box mt={2}>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={4}>
                <FormControl fullWidth size="small">
                  <InputLabel>Market Cap</InputLabel>
                  <Select value={marketCap} label="Market Cap" onChange={(e) => setMarketCap(e.target.value)}>
                    {MARKET_CAP_OPTIONS.map(opt => (
                      <MenuItem key={opt.value} value={opt.value}>{opt.label}</MenuItem>
                    ))}
                  </Select>
                </FormControl>
              </Grid>
              <Grid item xs={12} sm={4}>
                <TextField
                  fullWidth
                  size="small"
                  label="Min Price"
                  type="number"
                  value={priceMin}
                  onChange={(e) => setPriceMin(e.target.value)}
                />
              </Grid>
              <Grid item xs={12} sm={4}>
                <TextField
                  fullWidth
                  size="small"
                  label="Max Price"
                  type="number"
                  value={priceMax}
                  onChange={(e) => setPriceMax(e.target.value)}
                />
              </Grid>
            </Grid>

            <Typography variant="subtitle2" sx={{ mt: 2, mb: 1 }}>
              Metric Filters
            </Typography>
            <Grid container spacing={2}>
              {filters.map((filter, idx) => (
                <Grid item xs={12} sm={6} md={4} key={filter.metric}>
                  <Paper variant="outlined" sx={{ p: 1.5 }}>
                    <FormControlLabel
                      control={
                        <Switch
                          size="small"
                          checked={filter.enabled}
                          onChange={() => toggleFilter(idx)}
                        />
                      }
                      label={
                        <Typography variant="body2" fontWeight="bold">
                          {formatMetricName(filter.metric)}
                        </Typography>
                      }
                    />
                    {filter.enabled && (
                      <Box display="flex" gap={1} mt={1}>
                        <TextField
                          size="small"
                          label="Min"
                          type="number"
                          value={filter.min ?? ''}
                          onChange={(e) => updateFilterRange(idx, 'min', Number(e.target.value))}
                          sx={{ flex: 1 }}
                        />
                        <TextField
                          size="small"
                          label="Max"
                          type="number"
                          value={filter.max ?? ''}
                          onChange={(e) => updateFilterRange(idx, 'max', Number(e.target.value))}
                          sx={{ flex: 1 }}
                        />
                      </Box>
                    )}
                  </Paper>
                </Grid>
              ))}
            </Grid>
          </Box>
        </Collapse>

        <Box display="flex" gap={2} mt={3}>
          <Button
            variant="contained"
            size="large"
            startIcon={<Search />}
            onClick={handleSearch}
            disabled={factors.length === 0}
          >
            Screen Stocks
          </Button>
          {screeningQ.data && (
            <Button
              variant="outlined"
              startIcon={<Download />}
              onClick={() => exportMutation.mutate()}
              disabled={exportMutation.isPending}
            >
              {exportMutation.isPending ? 'Exporting...' : 'Export CSV'}
            </Button>
          )}
        </Box>
      </Paper>

      {/* Results */}
      {!searchTriggered && (
        <Alert severity="info">
          Configure your screening criteria above and click "Screen Stocks" to find investment opportunities
          matching your preferences. The screener analyzes stocks across multiple factor dimensions
          and ranks them by composite score.
        </Alert>
      )}

      {screeningQ.isLoading && (
        <Box display="flex" flexDirection="column" alignItems="center" py={6}>
          <CircularProgress size={48} />
          <Typography variant="body1" color="text.secondary" mt={2}>
            Screening stocks across {factors.length} factor categories...
          </Typography>
        </Box>
      )}

      {screeningQ.error && (
        <Alert severity="error" sx={{ mb: 2 }}>
          Screening failed: {(screeningQ.error as Error).message}
        </Alert>
      )}

      {screeningQ.data && (
        <Paper sx={{ p: 0, overflow: 'hidden' }}>
          <Box sx={{ p: 2, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <Box>
              <Typography variant="h6">
                Screening Results
              </Typography>
              <Typography variant="body2" color="text.secondary" component="div">
                {screeningQ.data.total_passed_filters} stocks matched ({screeningQ.data.total_screened} screened)
                {screeningQ.data.cache_hit && (
                  <Chip label="Cached" size="small" sx={{ ml: 1 }} />
                )}
              </Typography>
            </Box>
          </Box>

          <TableContainer>
            <Table size="small">
              <TableHead>
                <TableRow sx={{ bgcolor: 'grey.50' }}>
                  <TableCell width="5%"></TableCell>
                  <TableCell width="10%">
                    <TableSortLabel
                      active={sortField === 'symbol'}
                      direction={sortField === 'symbol' ? sortDir : 'asc'}
                      onClick={() => handleSort('symbol')}
                    >
                      Symbol
                    </TableSortLabel>
                  </TableCell>
                  <TableCell width="8%" align="center">Rank</TableCell>
                  <TableCell width="10%" align="center">
                    <TableSortLabel
                      active={sortField === 'composite_score'}
                      direction={sortField === 'composite_score' ? sortDir : 'asc'}
                      onClick={() => handleSort('composite_score')}
                    >
                      Score
                    </TableSortLabel>
                  </TableCell>
                  <TableCell width="12%" align="center">Fundamental</TableCell>
                  <TableCell width="12%" align="center">Technical</TableCell>
                  <TableCell width="12%" align="center">Sentiment</TableCell>
                  <TableCell width="12%" align="center">Momentum</TableCell>
                  <TableCell width="19%">Explanation</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {screeningQ.data.results.map((result) => (
                  <ScreeningResultRow key={result.symbol} result={result} />
                ))}
                {screeningQ.data.results.length === 0 && (
                  <TableRow>
                    <TableCell colSpan={9} align="center" sx={{ py: 4 }}>
                      <Typography color="text.secondary">
                        No stocks match your screening criteria. Try adjusting weights or refresh data.
                      </Typography>
                    </TableCell>
                  </TableRow>
                )}
              </TableBody>
            </Table>
          </TableContainer>

          {screeningQ.data.total_passed_filters > pageSize && (
            <Box display="flex" justifyContent="center" py={2}>
              <Pagination
                count={Math.ceil(screeningQ.data.total_passed_filters / pageSize)}
                page={page}
                onChange={(_, p) => setPage(p)}
                color="primary"
              />
            </Box>
          )}
        </Paper>
      )}
    </Box>
  );
}

function ScreeningResultRow({ result }: { result: ScreeningResult }) {
  const [expanded, setExpanded] = useState(false);

  const scoreColor = result.composite_score >= 70 ? 'success.main' :
    result.composite_score >= 40 ? 'warning.main' : 'error.main';

  const getScoreChip = (score: number) => {
    const color = score >= 70 ? 'success' : score >= 40 ? 'warning' : 'error';
    return (
      <Chip
        label={score.toFixed(0)}
        size="small"
        color={color}
        sx={{ minWidth: 50 }}
      />
    );
  };

  return (
    <>
      <TableRow
        hover
        sx={{ cursor: 'pointer', '&:hover': { bgcolor: 'action.hover' } }}
        onClick={() => setExpanded(!expanded)}
      >
        <TableCell>
          <IconButton size="small" sx={{ p: 0 }}>
            {expanded ? <ExpandLess fontSize="small" /> : <ExpandMore fontSize="small" />}
          </IconButton>
        </TableCell>
        <TableCell>
          <Typography fontWeight="bold">{result.symbol}</Typography>
        </TableCell>
        <TableCell align="center">
          <Chip label={`#${result.rank}`} size="small" variant="outlined" />
        </TableCell>
        <TableCell align="center">
          <Box sx={{
            width: 50,
            height: 50,
            borderRadius: '50%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            bgcolor: scoreColor,
            color: 'white',
            fontWeight: 'bold',
            fontSize: '1rem',
            margin: '0 auto'
          }}>
            {result.composite_score.toFixed(0)}
          </Box>
        </TableCell>
        <TableCell align="center">
          {getScoreChip(result.fundamental.composite)}
        </TableCell>
        <TableCell align="center">
          {getScoreChip(result.technical.composite)}
        </TableCell>
        <TableCell align="center">
          {getScoreChip(result.sentiment.composite)}
        </TableCell>
        <TableCell align="center">
          {getScoreChip(result.momentum.composite)}
        </TableCell>
        <TableCell>
          <Typography variant="body2" noWrap sx={{ maxWidth: 250 }}>
            {result.explanation}
          </Typography>
        </TableCell>
      </TableRow>
      {expanded && (
        <TableRow>
          <TableCell colSpan={9} sx={{ py: 2, bgcolor: 'grey.50' }}>
            <Typography variant="subtitle2" gutterBottom>Score Details</Typography>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={6} md={3}>
                <Paper variant="outlined" sx={{ p: 1.5 }}>
                  <Typography variant="caption" color="text.secondary" fontWeight="bold">
                    FUNDAMENTAL (weight: {(result.weights_used.fundamental * 100).toFixed(0)}%)
                  </Typography>
                  <Box display="flex" alignItems="center" gap={1} mt={0.5}>
                    <LinearProgress
                      variant="determinate"
                      value={Math.min(result.fundamental.composite, 100)}
                      sx={{ flex: 1, height: 8, borderRadius: 4 }}
                    />
                    <Typography variant="body2" fontWeight="bold">
                      {result.fundamental.composite.toFixed(0)}
                    </Typography>
                  </Box>
                </Paper>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <Paper variant="outlined" sx={{ p: 1.5 }}>
                  <Typography variant="caption" color="text.secondary" fontWeight="bold">
                    TECHNICAL (weight: {(result.weights_used.technical * 100).toFixed(0)}%)
                  </Typography>
                  <Box display="flex" alignItems="center" gap={1} mt={0.5}>
                    <LinearProgress
                      variant="determinate"
                      value={Math.min(result.technical.composite, 100)}
                      sx={{ flex: 1, height: 8, borderRadius: 4 }}
                    />
                    <Typography variant="body2" fontWeight="bold">
                      {result.technical.composite.toFixed(0)}
                    </Typography>
                  </Box>
                </Paper>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <Paper variant="outlined" sx={{ p: 1.5 }}>
                  <Typography variant="caption" color="text.secondary" fontWeight="bold">
                    SENTIMENT (weight: {(result.weights_used.sentiment * 100).toFixed(0)}%)
                  </Typography>
                  <Box display="flex" alignItems="center" gap={1} mt={0.5}>
                    <LinearProgress
                      variant="determinate"
                      value={Math.min(result.sentiment.composite, 100)}
                      sx={{ flex: 1, height: 8, borderRadius: 4 }}
                    />
                    <Typography variant="body2" fontWeight="bold">
                      {result.sentiment.composite.toFixed(0)}
                    </Typography>
                  </Box>
                </Paper>
              </Grid>
              <Grid item xs={12} sm={6} md={3}>
                <Paper variant="outlined" sx={{ p: 1.5 }}>
                  <Typography variant="caption" color="text.secondary" fontWeight="bold">
                    MOMENTUM (weight: {(result.weights_used.momentum * 100).toFixed(0)}%)
                  </Typography>
                  <Box display="flex" alignItems="center" gap={1} mt={0.5}>
                    <LinearProgress
                      variant="determinate"
                      value={Math.min(result.momentum.composite, 100)}
                      sx={{ flex: 1, height: 8, borderRadius: 4 }}
                    />
                    <Typography variant="body2" fontWeight="bold">
                      {result.momentum.composite.toFixed(0)}
                    </Typography>
                  </Box>
                </Paper>
              </Grid>
            </Grid>
            <Box mt={2}>
              <Typography variant="subtitle2" gutterBottom>Explanation</Typography>
              <Typography variant="body2" color="text.secondary">
                {result.explanation}
              </Typography>
            </Box>
          </TableCell>
        </TableRow>
      )}
    </>
  );
}

function formatMarketCapShort(value: number): string {
  if (value >= 1e12) return `$${(value / 1e12).toFixed(1)}T`;
  if (value >= 1e9) return `$${(value / 1e9).toFixed(1)}B`;
  if (value >= 1e6) return `$${(value / 1e6).toFixed(0)}M`;
  return `$${value.toLocaleString()}`;
}
