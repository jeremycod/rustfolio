import { useMemo, useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  Card,
  CardContent,
  Grid,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Chip,
  CircularProgress,
  Alert,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  LinearProgress,
  Button,
  Snackbar,
  Tabs,
  Tab,
} from '@mui/material';
import { TrendingUp, TrendingDown, ShowChart, Assessment, Camera, Settings, Download, Warning, ErrorOutline, Psychology, Timeline, AutoAwesome, TipsAndUpdates, Refresh, Newspaper, QuestionAnswer } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getPortfolioRisk, listPortfolios, createRiskSnapshot, exportPortfolioRiskCSV } from '../lib/endpoints';
import { formatCurrency, formatPercentage } from '../lib/formatters';
import { RiskLevel } from '../types';
import { TickerChip } from './TickerChip';
import { RiskHistoryChart } from './RiskHistoryChart';
import { RiskThresholdSettings } from './RiskThresholdSettings';
import { OptimizationRecommendations } from './OptimizationRecommendations';
import PortfolioNarrative from './PortfolioNarrative';
import PortfolioNews from './PortfolioNews';
import PortfolioQA from './PortfolioQA';

interface PortfolioRiskOverviewProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
  onTickerNavigate: (ticker: string) => void;
}

interface TabPanelProps {
  children?: React.ReactNode;
  value: number;
  index: number;
}

function TabPanel({ children, value, index }: TabPanelProps) {
  return (
    <div role="tabpanel" hidden={value !== index}>
      {value === index && <Box sx={{ py: 3 }}>{children}</Box>}
    </div>
  );
}

export function PortfolioRiskOverview({
  selectedPortfolioId,
  onPortfolioChange,
  onTickerNavigate,
}: PortfolioRiskOverviewProps) {
  const [snackbarOpen, setSnackbarOpen] = useState(false);
  const [snackbarMessage, setSnackbarMessage] = useState('');
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [exporting, setExporting] = useState(false);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [activeTab, setActiveTab] = useState(0);
  const [aiInsightsSubTab, setAiInsightsSubTab] = useState(0);
  const queryClient = useQueryClient();

  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const portfolioRiskQ = useQuery({
    queryKey: ['portfolioRisk', selectedPortfolioId],
    queryFn: () => getPortfolioRisk(selectedPortfolioId!, 90, 'SPY', false),
    enabled: !!selectedPortfolioId,
    staleTime: 4 * 60 * 60 * 1000, // 4 hours (matches backend cache)
  });

  // Extract data from response
  // Note: PortfolioRisk fields are flattened into the response (serde flatten)
  const riskData = portfolioRiskQ.data;
  const violations = portfolioRiskQ.data?.violations ?? [];
  const criticalViolations = violations.filter(v => v.threshold_type === 'critical');
  const warningViolations = violations.filter(v => v.threshold_type === 'warning');

  const createSnapshotMutation = useMutation({
    mutationFn: (portfolioId: string) => createRiskSnapshot(portfolioId),
    onSuccess: (data) => {
      setSnackbarMessage(`Successfully created ${data.length} risk snapshot${data.length > 1 ? 's' : ''}!`);
      setSnackbarOpen(true);
      // Invalidate risk history cache to refresh the chart
      queryClient.invalidateQueries({ queryKey: ['risk-history', selectedPortfolioId] });
      queryClient.invalidateQueries({ queryKey: ['risk-alerts', selectedPortfolioId] });
    },
    onError: (error: Error) => {
      setSnackbarMessage(`Failed to create snapshot: ${error.message}`);
      setSnackbarOpen(true);
    },
  });

  const handleExportCSV = async () => {
    if (!selectedPortfolioId) return;

    setExporting(true);
    try {
      await exportPortfolioRiskCSV(selectedPortfolioId, 90, 'SPY');
      setSnackbarMessage('Risk report exported successfully!');
      setSnackbarOpen(true);
    } catch (error) {
      setSnackbarMessage(`Export failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      setSnackbarOpen(true);
    } finally {
      setExporting(false);
    }
  };

  const handleRefreshRisk = async () => {
    if (!selectedPortfolioId) return;

    setIsRefreshing(true);
    try {
      // Force fetch new risk data
      const freshRisk = await getPortfolioRisk(selectedPortfolioId, 90, 'SPY', true);
      // Update the cache with fresh data
      queryClient.setQueryData(['portfolioRisk', selectedPortfolioId], freshRisk);
      setSnackbarMessage('Risk metrics refreshed successfully!');
      setSnackbarOpen(true);
    } catch (error) {
      setSnackbarMessage(`Refresh failed: ${error instanceof Error ? error.message : 'Unknown error'}`);
      setSnackbarOpen(true);
    } finally {
      setIsRefreshing(false);
    }
  };

  const getRiskColor = (level: RiskLevel): string => {
    switch (level) {
      case 'low':
        return '#4caf50'; // green
      case 'moderate':
        return '#ff9800'; // orange
      case 'high':
        return '#f44336'; // red
    }
  };

  const getRiskIcon = (level: RiskLevel) => {
    switch (level) {
      case 'low':
        return <TrendingUp />;
      case 'moderate':
        return <ShowChart />;
      case 'high':
        return <TrendingDown />;
    }
  };

  const riskColorForScore = useMemo(() => {
    if (!riskData) return '#999';
    return getRiskColor(riskData.risk_level);
  }, [riskData]);

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <Assessment sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Portfolio Risk Overview
        </Typography>
      </Box>

      {/* Portfolio Selector */}
      <Box sx={{ mb: 3, display: 'flex', gap: 2, alignItems: 'center' }}>
        <FormControl sx={{ minWidth: 250 }}>
          <InputLabel>Select Portfolio</InputLabel>
          <Select
            value={selectedPortfolioId ?? ''}
            onChange={(e) => onPortfolioChange(e.target.value)}
            label="Select Portfolio"
          >
            {(portfoliosQ.data ?? []).map((p) => (
              <MenuItem key={p.id} value={p.id}>
                {p.name}
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        <Button
          variant="outlined"
          startIcon={<Settings />}
          onClick={() => setSettingsOpen(true)}
          disabled={!selectedPortfolioId}
          sx={{ height: 'fit-content' }}
        >
          Settings
        </Button>

        <Button
          variant="outlined"
          startIcon={<Download />}
          onClick={handleExportCSV}
          disabled={!selectedPortfolioId || exporting}
          sx={{ height: 'fit-content' }}
        >
          {exporting ? 'Exporting...' : 'Export CSV'}
        </Button>

        <Button
          variant="outlined"
          startIcon={<Refresh
            sx={{
              animation: isRefreshing ? 'spin 1s linear infinite' : 'none',
              '@keyframes spin': {
                '0%': { transform: 'rotate(0deg)' },
                '100%': { transform: 'rotate(360deg)' },
              },
            }}
          />}
          onClick={handleRefreshRisk}
          disabled={!selectedPortfolioId || isRefreshing || portfolioRiskQ.isLoading}
          sx={{ height: 'fit-content' }}
        >
          {isRefreshing ? 'Refreshing...' : 'Refresh Risk'}
        </Button>

        <Button
          variant="contained"
          startIcon={<Camera />}
          onClick={() => selectedPortfolioId && createSnapshotMutation.mutate(selectedPortfolioId)}
          disabled={!selectedPortfolioId || createSnapshotMutation.isPending}
          sx={{ height: 'fit-content' }}
        >
          {createSnapshotMutation.isPending ? 'Creating...' : 'Create Snapshot'}
        </Button>
      </Box>

      {portfolioRiskQ.isLoading && (
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, p: 3 }}>
          <CircularProgress size={24} />
          <Typography>Calculating portfolio risk metrics...</Typography>
        </Box>
      )}

      {portfolioRiskQ.isError && (
        <Alert severity="error" sx={{ mb: 3 }}>
          Failed to load portfolio risk data. {portfolioRiskQ.error instanceof Error ? portfolioRiskQ.error.message : 'Please try again.'}
        </Alert>
      )}

      {/* Threshold Violations Alert */}
      {violations.length > 0 && (
        <Alert
          severity={criticalViolations.length > 0 ? "error" : "warning"}
          icon={criticalViolations.length > 0 ? <ErrorOutline /> : <Warning />}
          sx={{ mb: 3 }}
        >
          <Typography variant="body1" fontWeight="bold" gutterBottom>
            {violations.length} Threshold Violation{violations.length !== 1 ? 's' : ''} Detected
          </Typography>
          <Typography variant="body2">
            {criticalViolations.length > 0 && (
              <>{criticalViolations.length} critical, </>
            )}
            {warningViolations.length} warning{warningViolations.length !== 1 ? 's' : ''}
          </Typography>
          <Box sx={{ mt: 2 }}>
            {violations.slice(0, 5).map((violation, idx) => (
              <Chip
                key={idx}
                label={`${violation.ticker}: ${violation.metric_name} ${violation.metric_value.toFixed(2)}`}
                size="small"
                color={violation.threshold_type === 'critical' ? 'error' : 'warning'}
                sx={{ mr: 1, mb: 1 }}
              />
            ))}
            {violations.length > 5 && (
              <Typography variant="caption" display="block" sx={{ mt: 1 }}>
                ...and {violations.length - 5} more
              </Typography>
            )}
          </Box>
        </Alert>
      )}

      {riskData && (
        <>
          {/* Tabs */}
          <Paper sx={{ mb: 3 }}>
            <Tabs
              value={activeTab}
              onChange={(_, newValue) => setActiveTab(newValue)}
              variant="fullWidth"
              sx={{
                borderBottom: 1,
                borderColor: 'divider',
              }}
            >
              <Tab icon={<Assessment />} label="Overview" iconPosition="start" />
              <Tab icon={<Timeline />} label="Analysis" iconPosition="start" />
              <Tab icon={<AutoAwesome />} label="AI Insights" iconPosition="start" />
              <Tab icon={<TipsAndUpdates />} label="Optimization" iconPosition="start" />
            </Tabs>

            {/* Tab 1: Overview */}
            <TabPanel value={activeTab} index={0}>
              {/* Overall Portfolio Risk Score */}
              <Card sx={{ mb: 3, bgcolor: riskColorForScore, color: 'white' }}>
                <CardContent>
                  <Box display="flex" justifyContent="space-between" alignItems="center">
                    <Box>
                      <Typography variant="h6" gutterBottom>
                        Overall Portfolio Risk
                      </Typography>
                      <Box display="flex" alignItems="center" gap={2}>
                        <Typography variant="h3" fontWeight="bold">
                          {riskData.risk_level.toUpperCase()}
                        </Typography>
                        {getRiskIcon(riskData.risk_level)}
                      </Box>
                      <Typography variant="body2" sx={{ opacity: 0.9, mt: 1 }}>
                        Risk Score: {riskData.portfolio_risk_score.toFixed(1)}/100
                      </Typography>
                    </Box>
                    <Box textAlign="right">
                      <Typography variant="body2" sx={{ opacity: 0.9 }}>
                        Total Portfolio Value
                      </Typography>
                      <Typography variant="h4" fontWeight="bold">
                        {formatCurrency(riskData.total_value)}
                      </Typography>
                      <Typography variant="body2" sx={{ opacity: 0.9, mt: 1 }}>
                        {riskData.position_risks.length} positions analyzed
                      </Typography>
                    </Box>
                  </Box>

                  {/* Risk Score Progress Bar */}
                  <Box sx={{ mt: 2 }}>
                    <LinearProgress
                      variant="determinate"
                      value={riskData.portfolio_risk_score}
                      sx={{
                        height: 10,
                        borderRadius: 5,
                        bgcolor: 'rgba(255,255,255,0.3)',
                        '& .MuiLinearProgress-bar': {
                          bgcolor: 'white',
                        },
                      }}
                    />
                  </Box>
                </CardContent>
              </Card>

          {/* Portfolio-Wide Metrics */}
          <Grid container spacing={3} sx={{ mb: 3 }}>
            <Grid item xs={12} sm={6} md={3}>
              <Card>
                <CardContent>
                  <Typography color="textSecondary" variant="body2" gutterBottom>
                    Portfolio Volatility
                  </Typography>
                  <Typography variant="h5" fontWeight="bold">
                    {riskData.portfolio_volatility.toFixed(2)}%
                  </Typography>
                  <Typography variant="caption" color="textSecondary">
                    Annualized standard deviation
                  </Typography>
                </CardContent>
              </Card>
            </Grid>

            <Grid item xs={12} sm={6} md={3}>
              <Card>
                <CardContent>
                  <Typography color="textSecondary" variant="body2" gutterBottom>
                    Maximum Drawdown
                  </Typography>
                  <Typography variant="h5" fontWeight="bold" color="error">
                    {riskData.portfolio_max_drawdown.toFixed(2)}%
                  </Typography>
                  <Typography variant="caption" color="textSecondary">
                    Worst peak-to-trough decline
                  </Typography>
                </CardContent>
              </Card>
            </Grid>

            <Grid item xs={12} sm={6} md={3}>
              <Card>
                <CardContent>
                  <Typography color="textSecondary" variant="body2" gutterBottom>
                    Portfolio Beta
                  </Typography>
                  <Typography variant="h5" fontWeight="bold">
                    {riskData.portfolio_beta?.toFixed(2) ?? 'N/A'}
                  </Typography>
                  <Typography variant="caption" color="textSecondary">
                    vs SPY benchmark
                  </Typography>
                </CardContent>
              </Card>
            </Grid>

            <Grid item xs={12} sm={6} md={3}>
              <Card>
                <CardContent>
                  <Typography color="textSecondary" variant="body2" gutterBottom>
                    Sharpe Ratio
                  </Typography>
                  <Typography variant="h5" fontWeight="bold">
                    {riskData.portfolio_sharpe?.toFixed(2) ?? 'N/A'}
                  </Typography>
                  <Typography variant="caption" color="textSecondary">
                    Risk-adjusted return
                  </Typography>
                </CardContent>
              </Card>
            </Grid>
          </Grid>

          {/* Position Risk Contributions */}
              <Paper>
                <Box sx={{ p: 2 }}>
                  <Typography variant="h6" gutterBottom>
                    Risk Contribution by Position
                  </Typography>
                  <Typography variant="body2" color="textSecondary">
                    Positions sorted by risk score (highest risk first)
                  </Typography>
                </Box>

                <TableContainer>
                  <Table>
                    <TableHead>
                      <TableRow>
                        <TableCell>Ticker</TableCell>
                        <TableCell align="right">Market Value</TableCell>
                        <TableCell align="right">Weight</TableCell>
                        <TableCell align="right">Risk Score</TableCell>
                        <TableCell align="center">Risk Level</TableCell>
                        <TableCell align="right">Volatility</TableCell>
                        <TableCell align="right">Drawdown</TableCell>
                        <TableCell align="right">VaR 95%</TableCell>
                        <TableCell align="right">VaR 99%</TableCell>
                        <TableCell align="right">ES 95%</TableCell>
                        <TableCell align="right">Beta</TableCell>
                        <TableCell align="right">Sharpe</TableCell>
                        <TableCell align="right">Sortino</TableCell>
                        <TableCell align="right">Ann. Return</TableCell>
                      </TableRow>
                    </TableHead>
                    <TableBody>
                      {riskData.position_risks.map((position) => {
                        const riskColor = getRiskColor(position.risk_assessment.risk_level);

                        return (
                          <TableRow key={position.ticker} hover>
                            <TableCell>
                              <TickerChip ticker={position.ticker} onNavigate={onTickerNavigate} />
                            </TableCell>
                            <TableCell align="right">
                              {formatCurrency(position.market_value)}
                            </TableCell>
                            <TableCell align="right">
                              <Box display="flex" alignItems="center" justifyContent="flex-end" gap={1}>
                                <Typography variant="body2">
                                  {formatPercentage(position.weight * 100)}
                                </Typography>
                                <Box
                                  sx={{
                                    width: 60,
                                    height: 6,
                                    bgcolor: '#e0e0e0',
                                    borderRadius: 3,
                                    overflow: 'hidden',
                                  }}
                                >
                                  <Box
                                    sx={{
                                      width: `${position.weight * 100}%`,
                                      height: '100%',
                                      bgcolor: 'primary.main',
                                    }}
                                  />
                                </Box>
                              </Box>
                            </TableCell>
                            <TableCell align="right">
                              <Typography variant="body2" fontWeight="bold" color={riskColor}>
                                {position.risk_assessment.risk_score.toFixed(1)}
                              </Typography>
                            </TableCell>
                            <TableCell align="center">
                              <Chip
                                label={position.risk_assessment.risk_level.toUpperCase()}
                                size="small"
                                sx={{
                                  bgcolor: riskColor,
                                  color: 'white',
                                  fontWeight: 'bold',
                                }}
                              />
                            </TableCell>
                            <TableCell align="right">
                              {position.risk_assessment.metrics.volatility.toFixed(2)}%
                            </TableCell>
                            <TableCell align="right" sx={{ color: 'error.main' }}>
                              {position.risk_assessment.metrics.max_drawdown.toFixed(2)}%
                            </TableCell>
                            <TableCell align="right" sx={{ color: 'warning.main' }}>
                              {position.risk_assessment.metrics.var_95?.toFixed(2) ?? '—'}
                            </TableCell>
                            <TableCell align="right" sx={{ color: 'error.main' }}>
                              {position.risk_assessment.metrics.var_99?.toFixed(2) ?? '—'}
                            </TableCell>
                            <TableCell align="right" sx={{ color: 'error.dark' }}>
                              {position.risk_assessment.metrics.expected_shortfall_95?.toFixed(2) ?? '—'}
                            </TableCell>
                            <TableCell align="right">
                              {position.risk_assessment.metrics.beta?.toFixed(2) ?? '—'}
                            </TableCell>
                            <TableCell align="right">
                              {position.risk_assessment.metrics.sharpe?.toFixed(2) ?? '—'}
                            </TableCell>
                            <TableCell align="right">
                              {position.risk_assessment.metrics.sortino?.toFixed(2) ?? '—'}
                            </TableCell>
                            <TableCell align="right" sx={{
                              color: position.risk_assessment.metrics.annualized_return !== null && position.risk_assessment.metrics.annualized_return > 0
                                ? 'success.main'
                                : position.risk_assessment.metrics.annualized_return !== null && position.risk_assessment.metrics.annualized_return < 0
                                ? 'error.main'
                                : 'text.primary'
                            }}>
                              {position.risk_assessment.metrics.annualized_return?.toFixed(2) ? `${position.risk_assessment.metrics.annualized_return.toFixed(2)}%` : '—'}
                            </TableCell>
                          </TableRow>
                        );
                      })}
                    </TableBody>
                  </Table>
                </TableContainer>
              </Paper>
            </TabPanel>

            {/* Tab 2: Analysis */}
            <TabPanel value={activeTab} index={1}>
              <RiskHistoryChart
                portfolioId={selectedPortfolioId!}
                thresholds={riskData?.thresholds}
              />
            </TabPanel>

            {/* Tab 3: AI Insights */}
            <TabPanel value={activeTab} index={2}>
              {/* Sub-tabs for AI Insights */}
              <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 3 }}>
                <Tabs
                  value={aiInsightsSubTab}
                  onChange={(_, newValue) => setAiInsightsSubTab(newValue)}
                  variant="fullWidth"
                >
                  <Tab icon={<Psychology />} label="AI Analysis" iconPosition="start" />
                  <Tab icon={<Newspaper />} label="News & Sentiment" iconPosition="start" />
                  <Tab icon={<QuestionAnswer />} label="Ask AI" iconPosition="start" />
                </Tabs>
              </Box>

              {/* Sub-tab 1: AI Analysis */}
              {aiInsightsSubTab === 0 && (
                <PortfolioNarrative portfolioId={selectedPortfolioId!} timePeriod="90d" />
              )}

              {/* Sub-tab 2: News & Sentiment */}
              {aiInsightsSubTab === 1 && (
                <PortfolioNews portfolioId={selectedPortfolioId!} />
              )}

              {/* Sub-tab 3: Ask AI */}
              {aiInsightsSubTab === 2 && (
                <PortfolioQA portfolioId={selectedPortfolioId!} />
              )}
            </TabPanel>

            {/* Tab 4: Optimization */}
            <TabPanel value={activeTab} index={3}>
              <Typography variant="h5" gutterBottom sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                <Psychology /> Optimization Suggestions
              </Typography>
              <Typography variant="body2" color="textSecondary" sx={{ mb: 2 }}>
                AI-powered recommendations to improve your portfolio's risk-return profile
              </Typography>
              <OptimizationRecommendations portfolioId={selectedPortfolioId!} />
            </TabPanel>
          </Paper>

          {/* Disclaimer */}
          <Alert severity="info" sx={{ mt: 3 }}>
            <Typography variant="body2">
              <strong>Note:</strong> Portfolio risk metrics are calculated using weighted averages based on position size.
              This analysis is based on historical data (90 days) and does not account for correlations between positions.
              Past performance is not indicative of future results.
            </Typography>
          </Alert>
        </>
      )}

      {!selectedPortfolioId && !portfolioRiskQ.isLoading && (
        <Alert severity="info">
          Select a portfolio above to view its risk analysis and position contributions.
        </Alert>
      )}

      {/* Success/Error Snackbar */}
      <Snackbar
        open={snackbarOpen}
        autoHideDuration={6000}
        onClose={() => setSnackbarOpen(false)}
        message={snackbarMessage}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      />

      {/* Risk Threshold Settings Dialog */}
      {selectedPortfolioId && (
        <RiskThresholdSettings
          portfolioId={selectedPortfolioId}
          open={settingsOpen}
          onClose={() => setSettingsOpen(false)}
        />
      )}
    </Box>
  );
}
