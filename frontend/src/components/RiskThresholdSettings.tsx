import { useState, useEffect, useMemo } from 'react';
import {
  Box,
  Typography,
  Paper,
  TextField,
  Button,
  Alert,
  Grid,
  Card,
  CardContent,
  Divider,
  CircularProgress,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Collapse,
  List,
  ListItem,
  ListItemText,
  Chip,
} from '@mui/material';
import { Security, Save, RestartAlt, Warning, ExpandMore, ExpandLess } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getRiskThresholds, setRiskThresholds, listPortfolios, getPortfolioRisk } from '../lib/endpoints';
import { RiskThresholds } from '../types';

export function RiskThresholdSettings() {
  const queryClient = useQueryClient();

  const thresholdsQ = useQuery({
    queryKey: ['riskThresholds'],
    queryFn: getRiskThresholds,
  });

  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const [formValues, setFormValues] = useState<RiskThresholds>({
    volatility_threshold: null,
    drawdown_threshold: null,
    beta_threshold: null,
    var_threshold: null,
    risk_score_threshold: null,
  });

  const [hasChanges, setHasChanges] = useState(false);
  const [selectedPortfolioId, setSelectedPortfolioId] = useState<string | null>(null);
  const [expandedWarnings, setExpandedWarnings] = useState<Record<string, boolean>>({});

  const portfolioRiskQ = useQuery({
    queryKey: ['portfolioRisk', selectedPortfolioId],
    queryFn: () => getPortfolioRisk(selectedPortfolioId!, 90, 'SPY'),
    enabled: !!selectedPortfolioId,
  });

  // Initialize form values when data loads
  useEffect(() => {
    if (thresholdsQ.data && !hasChanges) {
      setFormValues(thresholdsQ.data);
    }
  }, [thresholdsQ.data, hasChanges]);

  // Calculate which positions would trigger warnings
  const warningPreview = useMemo(() => {
    if (!portfolioRiskQ.data) return null;

    const warnings: Record<string, string[]> = {
      volatility: [],
      drawdown: [],
      beta: [],
      var: [],
      riskScore: [],
    };

    portfolioRiskQ.data.position_risks.forEach((position) => {
      const metrics = position.risk_assessment.metrics;
      const riskScore = position.risk_assessment.risk_score;

      if (formValues.volatility_threshold !== null && metrics.volatility > formValues.volatility_threshold) {
        warnings.volatility.push(position.ticker);
      }

      if (formValues.drawdown_threshold !== null && metrics.max_drawdown < formValues.drawdown_threshold) {
        warnings.drawdown.push(position.ticker);
      }

      if (formValues.beta_threshold !== null && metrics.beta !== null && metrics.beta > formValues.beta_threshold) {
        warnings.beta.push(position.ticker);
      }

      if (formValues.var_threshold !== null && metrics.value_at_risk !== null && metrics.value_at_risk < formValues.var_threshold) {
        warnings.var.push(position.ticker);
      }

      if (formValues.risk_score_threshold !== null && riskScore > formValues.risk_score_threshold) {
        warnings.riskScore.push(position.ticker);
      }
    });

    const totalWarnings = Object.values(warnings).reduce((sum, arr) => sum + arr.length, 0);

    return {
      warnings,
      totalWarnings,
      totalPositions: portfolioRiskQ.data.position_risks.length,
    };
  }, [portfolioRiskQ.data, formValues]);

  const saveMutation = useMutation({
    mutationFn: (thresholds: RiskThresholds) => setRiskThresholds(thresholds),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['riskThresholds'] });
      setHasChanges(false);
    },
  });

  const handleFieldChange = (field: keyof RiskThresholds, value: string) => {
    const numValue = value === '' ? null : parseFloat(value);
    setFormValues((prev) => ({
      ...prev,
      [field]: numValue,
    }));
    setHasChanges(true);
  };

  const handleSave = () => {
    saveMutation.mutate(formValues);
  };

  const handleReset = () => {
    const defaults: RiskThresholds = {
      volatility_threshold: 30.0,
      drawdown_threshold: -20.0,
      beta_threshold: 1.5,
      var_threshold: -10.0,
      risk_score_threshold: 70.0,
    };
    setFormValues(defaults);
    setHasChanges(true);
  };

  const toggleWarningExpansion = (key: string) => {
    setExpandedWarnings((prev) => ({
      ...prev,
      [key]: !prev[key],
    }));
  };

  if (thresholdsQ.isLoading) {
    return (
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, p: 3 }}>
        <CircularProgress size={24} />
        <Typography>Loading risk threshold settings...</Typography>
      </Box>
    );
  }

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <Security sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Risk Threshold Settings
        </Typography>
      </Box>

      <Alert severity="info" sx={{ mb: 3 }}>
        Configure custom thresholds for risk warnings. Positions exceeding these thresholds will be
        flagged as high risk. Leave blank to use default values.
      </Alert>

      {saveMutation.isSuccess && (
        <Alert severity="success" sx={{ mb: 3 }}>
          Risk thresholds saved successfully!
        </Alert>
      )}

      {saveMutation.isError && (
        <Alert severity="error" sx={{ mb: 3 }}>
          Failed to save risk thresholds. Please try again.
        </Alert>
      )}

      <Grid container spacing={3}>
        {/* Threshold Configuration */}
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Threshold Configuration
            </Typography>
            <Divider sx={{ mb: 3 }} />

            <Box display="flex" flexDirection="column" gap={3}>
              <TextField
                label="Volatility Threshold (%)"
                type="number"
                value={formValues.volatility_threshold ?? ''}
                onChange={(e) => handleFieldChange('volatility_threshold', e.target.value)}
                helperText="Annualized volatility above this level triggers a warning (default: 30%)"
                fullWidth
                inputProps={{ step: 0.1, min: 0, max: 100 }}
              />

              <TextField
                label="Max Drawdown Threshold (%)"
                type="number"
                value={formValues.drawdown_threshold ?? ''}
                onChange={(e) => handleFieldChange('drawdown_threshold', e.target.value)}
                helperText="Maximum drawdown below this level triggers a warning (default: -20%)"
                fullWidth
                inputProps={{ step: 0.1, max: 0 }}
              />

              <TextField
                label="Beta Threshold"
                type="number"
                value={formValues.beta_threshold ?? ''}
                onChange={(e) => handleFieldChange('beta_threshold', e.target.value)}
                helperText="Beta above this level triggers a warning (default: 1.5)"
                fullWidth
                inputProps={{ step: 0.1, min: 0 }}
              />

              <TextField
                label="Value at Risk (VaR) Threshold (%)"
                type="number"
                value={formValues.var_threshold ?? ''}
                onChange={(e) => handleFieldChange('var_threshold', e.target.value)}
                helperText="5% VaR below this level triggers a warning (default: -10%)"
                fullWidth
                inputProps={{ step: 0.1, max: 0 }}
              />

              <TextField
                label="Risk Score Threshold"
                type="number"
                value={formValues.risk_score_threshold ?? ''}
                onChange={(e) => handleFieldChange('risk_score_threshold', e.target.value)}
                helperText="Overall risk score above this level triggers a warning (default: 70)"
                fullWidth
                inputProps={{ step: 1, min: 0, max: 100 }}
              />
            </Box>

            <Box display="flex" gap={2} mt={4}>
              <Button
                variant="contained"
                startIcon={<Save />}
                onClick={handleSave}
                disabled={!hasChanges || saveMutation.isPending}
              >
                {saveMutation.isPending ? 'Saving...' : 'Save Thresholds'}
              </Button>
              <Button
                variant="outlined"
                startIcon={<RestartAlt />}
                onClick={handleReset}
              >
                Reset to Defaults
              </Button>
            </Box>
          </Paper>
        </Grid>

        {/* Threshold Explanation */}
        <Grid item xs={12} md={6}>
          <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Understanding Risk Metrics
            </Typography>
            <Divider sx={{ mb: 3 }} />

            <Box display="flex" flexDirection="column" gap={2}>
              <Card>
                <CardContent>
                  <Typography variant="subtitle2" fontWeight="bold" gutterBottom>
                    Volatility
                  </Typography>
                  <Typography variant="body2" color="textSecondary">
                    Measures how much an asset's price fluctuates. Higher volatility means more
                    price swings and potentially higher risk. Values typically range from 10-50%
                    for stocks.
                  </Typography>
                </CardContent>
              </Card>

              <Card>
                <CardContent>
                  <Typography variant="subtitle2" fontWeight="bold" gutterBottom>
                    Maximum Drawdown
                  </Typography>
                  <Typography variant="body2" color="textSecondary">
                    The largest peak-to-trough decline in value. Shows the worst-case loss
                    historically. A -20% drawdown means the position lost 20% from its peak.
                  </Typography>
                </CardContent>
              </Card>

              <Card>
                <CardContent>
                  <Typography variant="subtitle2" fontWeight="bold" gutterBottom>
                    Beta
                  </Typography>
                  <Typography variant="body2" color="textSecondary">
                    Measures sensitivity to market movements (vs SPY benchmark). Beta of 1.0 moves
                    with the market, &gt;1.0 is more volatile, &lt;1.0 is less volatile.
                  </Typography>
                </CardContent>
              </Card>

              <Card>
                <CardContent>
                  <Typography variant="subtitle2" fontWeight="bold" gutterBottom>
                    Value at Risk (VaR)
                  </Typography>
                  <Typography variant="body2" color="textSecondary">
                    Estimates the maximum expected loss at 95% confidence. A VaR of -10% means
                    there's a 5% chance of losing more than 10% in a given period.
                  </Typography>
                </CardContent>
              </Card>

              <Card>
                <CardContent>
                  <Typography variant="subtitle2" fontWeight="bold" gutterBottom>
                    Risk Score
                  </Typography>
                  <Typography variant="body2" color="textSecondary">
                    A composite score (0-100) combining all risk metrics. Scores below 40 are low
                    risk, 40-60 are moderate, and above 60 are high risk.
                  </Typography>
                </CardContent>
              </Card>
            </Box>
          </Paper>
        </Grid>

        {/* Position Warning Preview */}
        <Grid item xs={12}>
          <Paper sx={{ p: 3 }}>
            <Box display="flex" alignItems="center" gap={2} mb={3}>
              <Warning sx={{ fontSize: 28, color: 'warning.main' }} />
              <Typography variant="h6" fontWeight="bold">
                Preview Impact
              </Typography>
            </Box>

            <Alert severity="info" sx={{ mb: 3 }}>
              Select a portfolio to see which positions would exceed your configured thresholds. This preview updates in real-time as you adjust threshold values.
            </Alert>

            {/* Portfolio Selector */}
            <FormControl fullWidth sx={{ mb: 3 }}>
              <InputLabel>Select Portfolio</InputLabel>
              <Select
                value={selectedPortfolioId ?? ''}
                onChange={(e) => setSelectedPortfolioId(e.target.value || null)}
                label="Select Portfolio"
              >
                <MenuItem value="">
                  <em>None</em>
                </MenuItem>
                {(portfoliosQ.data ?? []).map((p) => (
                  <MenuItem key={p.id} value={p.id}>
                    {p.name}
                  </MenuItem>
                ))}
              </Select>
            </FormControl>

            {/* Loading State */}
            {portfolioRiskQ.isLoading && (
              <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, py: 3 }}>
                <CircularProgress size={20} />
                <Typography>Loading portfolio risk data...</Typography>
              </Box>
            )}

            {/* Error State */}
            {portfolioRiskQ.isError && (
              <Alert severity="error">
                Failed to load portfolio risk data. Please try again.
              </Alert>
            )}

            {/* Warning Preview Results */}
            {warningPreview && (
              <Box>
                <Divider sx={{ mb: 3 }} />

                {/* Summary */}
                <Box sx={{ mb: 3 }}>
                  <Typography variant="body1" fontWeight="bold" gutterBottom>
                    Summary
                  </Typography>
                  <Typography variant="body2" color="textSecondary">
                    {warningPreview.totalWarnings === 0 ? (
                      <>No positions would exceed your thresholds.</>
                    ) : (
                      <>
                        <strong>{warningPreview.totalWarnings}</strong> warning{warningPreview.totalWarnings !== 1 ? 's' : ''} would be triggered across{' '}
                        <strong>{warningPreview.totalPositions}</strong> position{warningPreview.totalPositions !== 1 ? 's' : ''}.
                      </>
                    )}
                  </Typography>
                </Box>

                {/* Detailed Warnings */}
                <Box display="flex" flexDirection="column" gap={2}>
                  {/* Volatility Warnings */}
                  {formValues.volatility_threshold !== null && (
                    <Card variant="outlined">
                      <CardContent>
                        <Box
                          display="flex"
                          justifyContent="space-between"
                          alignItems="center"
                          sx={{ cursor: warningPreview.warnings.volatility.length > 0 ? 'pointer' : 'default' }}
                          onClick={() => warningPreview.warnings.volatility.length > 0 && toggleWarningExpansion('volatility')}
                        >
                          <Box display="flex" alignItems="center" gap={1}>
                            <Typography variant="body2" fontWeight="bold">
                              Volatility Threshold ({formValues.volatility_threshold}%)
                            </Typography>
                            <Chip
                              label={warningPreview.warnings.volatility.length}
                              size="small"
                              color={warningPreview.warnings.volatility.length > 0 ? 'warning' : 'default'}
                            />
                          </Box>
                          {warningPreview.warnings.volatility.length > 0 && (
                            expandedWarnings.volatility ? <ExpandLess /> : <ExpandMore />
                          )}
                        </Box>
                        <Collapse in={expandedWarnings.volatility}>
                          <List dense>
                            {warningPreview.warnings.volatility.map((ticker) => (
                              <ListItem key={ticker}>
                                <ListItemText primary={ticker} />
                              </ListItem>
                            ))}
                          </List>
                        </Collapse>
                      </CardContent>
                    </Card>
                  )}

                  {/* Drawdown Warnings */}
                  {formValues.drawdown_threshold !== null && (
                    <Card variant="outlined">
                      <CardContent>
                        <Box
                          display="flex"
                          justifyContent="space-between"
                          alignItems="center"
                          sx={{ cursor: warningPreview.warnings.drawdown.length > 0 ? 'pointer' : 'default' }}
                          onClick={() => warningPreview.warnings.drawdown.length > 0 && toggleWarningExpansion('drawdown')}
                        >
                          <Box display="flex" alignItems="center" gap={1}>
                            <Typography variant="body2" fontWeight="bold">
                              Max Drawdown Threshold ({formValues.drawdown_threshold}%)
                            </Typography>
                            <Chip
                              label={warningPreview.warnings.drawdown.length}
                              size="small"
                              color={warningPreview.warnings.drawdown.length > 0 ? 'warning' : 'default'}
                            />
                          </Box>
                          {warningPreview.warnings.drawdown.length > 0 && (
                            expandedWarnings.drawdown ? <ExpandLess /> : <ExpandMore />
                          )}
                        </Box>
                        <Collapse in={expandedWarnings.drawdown}>
                          <List dense>
                            {warningPreview.warnings.drawdown.map((ticker) => (
                              <ListItem key={ticker}>
                                <ListItemText primary={ticker} />
                              </ListItem>
                            ))}
                          </List>
                        </Collapse>
                      </CardContent>
                    </Card>
                  )}

                  {/* Beta Warnings */}
                  {formValues.beta_threshold !== null && (
                    <Card variant="outlined">
                      <CardContent>
                        <Box
                          display="flex"
                          justifyContent="space-between"
                          alignItems="center"
                          sx={{ cursor: warningPreview.warnings.beta.length > 0 ? 'pointer' : 'default' }}
                          onClick={() => warningPreview.warnings.beta.length > 0 && toggleWarningExpansion('beta')}
                        >
                          <Box display="flex" alignItems="center" gap={1}>
                            <Typography variant="body2" fontWeight="bold">
                              Beta Threshold ({formValues.beta_threshold})
                            </Typography>
                            <Chip
                              label={warningPreview.warnings.beta.length}
                              size="small"
                              color={warningPreview.warnings.beta.length > 0 ? 'warning' : 'default'}
                            />
                          </Box>
                          {warningPreview.warnings.beta.length > 0 && (
                            expandedWarnings.beta ? <ExpandLess /> : <ExpandMore />
                          )}
                        </Box>
                        <Collapse in={expandedWarnings.beta}>
                          <List dense>
                            {warningPreview.warnings.beta.map((ticker) => (
                              <ListItem key={ticker}>
                                <ListItemText primary={ticker} />
                              </ListItem>
                            ))}
                          </List>
                        </Collapse>
                      </CardContent>
                    </Card>
                  )}

                  {/* VaR Warnings */}
                  {formValues.var_threshold !== null && (
                    <Card variant="outlined">
                      <CardContent>
                        <Box
                          display="flex"
                          justifyContent="space-between"
                          alignItems="center"
                          sx={{ cursor: warningPreview.warnings.var.length > 0 ? 'pointer' : 'default' }}
                          onClick={() => warningPreview.warnings.var.length > 0 && toggleWarningExpansion('var')}
                        >
                          <Box display="flex" alignItems="center" gap={1}>
                            <Typography variant="body2" fontWeight="bold">
                              VaR Threshold ({formValues.var_threshold}%)
                            </Typography>
                            <Chip
                              label={warningPreview.warnings.var.length}
                              size="small"
                              color={warningPreview.warnings.var.length > 0 ? 'warning' : 'default'}
                            />
                          </Box>
                          {warningPreview.warnings.var.length > 0 && (
                            expandedWarnings.var ? <ExpandLess /> : <ExpandMore />
                          )}
                        </Box>
                        <Collapse in={expandedWarnings.var}>
                          <List dense>
                            {warningPreview.warnings.var.map((ticker) => (
                              <ListItem key={ticker}>
                                <ListItemText primary={ticker} />
                              </ListItem>
                            ))}
                          </List>
                        </Collapse>
                      </CardContent>
                    </Card>
                  )}

                  {/* Risk Score Warnings */}
                  {formValues.risk_score_threshold !== null && (
                    <Card variant="outlined">
                      <CardContent>
                        <Box
                          display="flex"
                          justifyContent="space-between"
                          alignItems="center"
                          sx={{ cursor: warningPreview.warnings.riskScore.length > 0 ? 'pointer' : 'default' }}
                          onClick={() => warningPreview.warnings.riskScore.length > 0 && toggleWarningExpansion('riskScore')}
                        >
                          <Box display="flex" alignItems="center" gap={1}>
                            <Typography variant="body2" fontWeight="bold">
                              Risk Score Threshold ({formValues.risk_score_threshold})
                            </Typography>
                            <Chip
                              label={warningPreview.warnings.riskScore.length}
                              size="small"
                              color={warningPreview.warnings.riskScore.length > 0 ? 'warning' : 'default'}
                            />
                          </Box>
                          {warningPreview.warnings.riskScore.length > 0 && (
                            expandedWarnings.riskScore ? <ExpandLess /> : <ExpandMore />
                          )}
                        </Box>
                        <Collapse in={expandedWarnings.riskScore}>
                          <List dense>
                            {warningPreview.warnings.riskScore.map((ticker) => (
                              <ListItem key={ticker}>
                                <ListItemText primary={ticker} />
                              </ListItem>
                            ))}
                          </List>
                        </Collapse>
                      </CardContent>
                    </Card>
                  )}
                </Box>
              </Box>
            )}
          </Paper>
        </Grid>
      </Grid>
    </Box>
  );
}
