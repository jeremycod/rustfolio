import { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  Slider,
  Grid,
  Alert,
  CircularProgress,
  Paper,
} from '@mui/material';
import { Settings, Save, RestartAlt } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getRiskThresholds, updateRiskThresholds } from '../lib/endpoints';
import type { RiskThresholdSettings as ThresholdSettings, UpdateRiskThresholds } from '../types';

interface RiskThresholdSettingsProps {
  portfolioId: string;
  open: boolean;
  onClose: () => void;
}

const DEFAULT_THRESHOLDS = {
  volatility_warning_threshold: 30.0,
  volatility_critical_threshold: 50.0,
  drawdown_warning_threshold: -20.0,
  drawdown_critical_threshold: -35.0,
  beta_warning_threshold: 1.5,
  beta_critical_threshold: 2.0,
  risk_score_warning_threshold: 60.0,
  risk_score_critical_threshold: 80.0,
  var_warning_threshold: -5.0,
  var_critical_threshold: -10.0,
};

export function RiskThresholdSettings({
  portfolioId,
  open,
  onClose,
}: RiskThresholdSettingsProps) {
  const queryClient = useQueryClient();

  // Local state for threshold values
  const [volatilityWarning, setVolatilityWarning] = useState(30.0);
  const [volatilityCritical, setVolatilityCritical] = useState(50.0);
  const [drawdownWarning, setDrawdownWarning] = useState(-20.0);
  const [drawdownCritical, setDrawdownCritical] = useState(-35.0);
  const [betaWarning, setBetaWarning] = useState(1.5);
  const [betaCritical, setBetaCritical] = useState(2.0);
  const [riskScoreWarning, setRiskScoreWarning] = useState(60.0);
  const [riskScoreCritical, setRiskScoreCritical] = useState(80.0);
  const [varWarning, setVarWarning] = useState(-5.0);
  const [varCritical, setVarCritical] = useState(-10.0);

  // Fetch current thresholds
  const { data: thresholds, isLoading } = useQuery({
    queryKey: ['riskThresholds', portfolioId],
    queryFn: () => getRiskThresholds(portfolioId),
    enabled: open && !!portfolioId,
  });

  // Update mutation
  const updateMutation = useMutation({
    mutationFn: (updates: UpdateRiskThresholds) =>
      updateRiskThresholds(portfolioId, updates),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['riskThresholds', portfolioId] });
      queryClient.invalidateQueries({ queryKey: ['portfolioRisk', portfolioId] });
      onClose();
    },
  });

  // Initialize local state when thresholds are loaded
  useEffect(() => {
    if (thresholds) {
      setVolatilityWarning(thresholds.volatility_warning_threshold);
      setVolatilityCritical(thresholds.volatility_critical_threshold);
      setDrawdownWarning(thresholds.drawdown_warning_threshold);
      setDrawdownCritical(thresholds.drawdown_critical_threshold);
      setBetaWarning(thresholds.beta_warning_threshold);
      setBetaCritical(thresholds.beta_critical_threshold);
      setRiskScoreWarning(thresholds.risk_score_warning_threshold);
      setRiskScoreCritical(thresholds.risk_score_critical_threshold);
      setVarWarning(thresholds.var_warning_threshold);
      setVarCritical(thresholds.var_critical_threshold);
    }
  }, [thresholds]);

  const handleSave = () => {
    const updates: UpdateRiskThresholds = {
      volatility_warning_threshold: volatilityWarning,
      volatility_critical_threshold: volatilityCritical,
      drawdown_warning_threshold: drawdownWarning,
      drawdown_critical_threshold: drawdownCritical,
      beta_warning_threshold: betaWarning,
      beta_critical_threshold: betaCritical,
      risk_score_warning_threshold: riskScoreWarning,
      risk_score_critical_threshold: riskScoreCritical,
      var_warning_threshold: varWarning,
      var_critical_threshold: varCritical,
    };
    updateMutation.mutate(updates);
  };

  const handleReset = () => {
    setVolatilityWarning(DEFAULT_THRESHOLDS.volatility_warning_threshold);
    setVolatilityCritical(DEFAULT_THRESHOLDS.volatility_critical_threshold);
    setDrawdownWarning(DEFAULT_THRESHOLDS.drawdown_warning_threshold);
    setDrawdownCritical(DEFAULT_THRESHOLDS.drawdown_critical_threshold);
    setBetaWarning(DEFAULT_THRESHOLDS.beta_warning_threshold);
    setBetaCritical(DEFAULT_THRESHOLDS.beta_critical_threshold);
    setRiskScoreWarning(DEFAULT_THRESHOLDS.risk_score_warning_threshold);
    setRiskScoreCritical(DEFAULT_THRESHOLDS.risk_score_critical_threshold);
    setVarWarning(DEFAULT_THRESHOLDS.var_warning_threshold);
    setVarCritical(DEFAULT_THRESHOLDS.var_critical_threshold);
  };

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
      <DialogTitle>
        <Box display="flex" alignItems="center" gap={1}>
          <Settings />
          <Typography variant="h6">Risk Threshold Settings</Typography>
        </Box>
        <Typography variant="body2" color="textSecondary" sx={{ mt: 1 }}>
          Customize risk warning and critical thresholds for this portfolio
        </Typography>
      </DialogTitle>

      <DialogContent>
        {isLoading && (
          <Box display="flex" justifyContent="center" py={4}>
            <CircularProgress />
          </Box>
        )}

        {updateMutation.isError && (
          <Alert severity="error" sx={{ mb: 2 }}>
            Failed to save threshold settings. Please try again.
          </Alert>
        )}

        {!isLoading && (
          <Box>
            {/* Volatility Thresholds */}
            <Paper elevation={1} sx={{ p: 3, mb: 2 }}>
              <Typography variant="h6" gutterBottom>
                Volatility Thresholds (%)
              </Typography>
              <Typography variant="body2" color="textSecondary" gutterBottom>
                Annualized volatility percentage thresholds
              </Typography>

              <Grid container spacing={3} sx={{ mt: 1 }}>
                <Grid item xs={6}>
                  <Typography variant="subtitle2" gutterBottom>
                    Warning: {volatilityWarning.toFixed(1)}%
                  </Typography>
                  <Slider
                    value={volatilityWarning}
                    onChange={(_, val) => setVolatilityWarning(val as number)}
                    min={10}
                    max={100}
                    step={1}
                    marks={[
                      { value: 10, label: '10%' },
                      { value: 50, label: '50%' },
                      { value: 100, label: '100%' },
                    ]}
                    valueLabelDisplay="auto"
                    color="warning"
                  />
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="subtitle2" gutterBottom>
                    Critical: {volatilityCritical.toFixed(1)}%
                  </Typography>
                  <Slider
                    value={volatilityCritical}
                    onChange={(_, val) => setVolatilityCritical(val as number)}
                    min={20}
                    max={150}
                    step={1}
                    marks={[
                      { value: 20, label: '20%' },
                      { value: 75, label: '75%' },
                      { value: 150, label: '150%' },
                    ]}
                    valueLabelDisplay="auto"
                    color="error"
                  />
                </Grid>
              </Grid>
            </Paper>

            {/* Drawdown Thresholds */}
            <Paper elevation={1} sx={{ p: 3, mb: 2 }}>
              <Typography variant="h6" gutterBottom>
                Maximum Drawdown Thresholds (%)
              </Typography>
              <Typography variant="body2" color="textSecondary" gutterBottom>
                Peak-to-trough decline percentage thresholds (negative values)
              </Typography>

              <Grid container spacing={3} sx={{ mt: 1 }}>
                <Grid item xs={6}>
                  <Typography variant="subtitle2" gutterBottom>
                    Warning: {drawdownWarning.toFixed(1)}%
                  </Typography>
                  <Slider
                    value={drawdownWarning}
                    onChange={(_, val) => setDrawdownWarning(val as number)}
                    min={-50}
                    max={0}
                    step={1}
                    marks={[
                      { value: -50, label: '-50%' },
                      { value: -25, label: '-25%' },
                      { value: 0, label: '0%' },
                    ]}
                    valueLabelDisplay="auto"
                    color="warning"
                  />
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="subtitle2" gutterBottom>
                    Critical: {drawdownCritical.toFixed(1)}%
                  </Typography>
                  <Slider
                    value={drawdownCritical}
                    onChange={(_, val) => setDrawdownCritical(val as number)}
                    min={-70}
                    max={-10}
                    step={1}
                    marks={[
                      { value: -70, label: '-70%' },
                      { value: -40, label: '-40%' },
                      { value: -10, label: '-10%' },
                    ]}
                    valueLabelDisplay="auto"
                    color="error"
                  />
                </Grid>
              </Grid>
            </Paper>

            {/* Beta Thresholds */}
            <Paper elevation={1} sx={{ p: 3, mb: 2 }}>
              <Typography variant="h6" gutterBottom>
                Beta Thresholds
              </Typography>
              <Typography variant="body2" color="textSecondary" gutterBottom>
                Volatility relative to market benchmark (1.0 = market volatility)
              </Typography>

              <Grid container spacing={3} sx={{ mt: 1 }}>
                <Grid item xs={6}>
                  <Typography variant="subtitle2" gutterBottom>
                    Warning: {betaWarning.toFixed(2)}
                  </Typography>
                  <Slider
                    value={betaWarning}
                    onChange={(_, val) => setBetaWarning(val as number)}
                    min={0.5}
                    max={3.0}
                    step={0.1}
                    marks={[
                      { value: 0.5, label: '0.5' },
                      { value: 1.5, label: '1.5' },
                      { value: 3.0, label: '3.0' },
                    ]}
                    valueLabelDisplay="auto"
                    color="warning"
                  />
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="subtitle2" gutterBottom>
                    Critical: {betaCritical.toFixed(2)}
                  </Typography>
                  <Slider
                    value={betaCritical}
                    onChange={(_, val) => setBetaCritical(val as number)}
                    min={1.0}
                    max={5.0}
                    step={0.1}
                    marks={[
                      { value: 1.0, label: '1.0' },
                      { value: 3.0, label: '3.0' },
                      { value: 5.0, label: '5.0' },
                    ]}
                    valueLabelDisplay="auto"
                    color="error"
                  />
                </Grid>
              </Grid>
            </Paper>

            {/* Risk Score Thresholds */}
            <Paper elevation={1} sx={{ p: 3, mb: 2 }}>
              <Typography variant="h6" gutterBottom>
                Risk Score Thresholds (0-100)
              </Typography>
              <Typography variant="body2" color="textSecondary" gutterBottom>
                Overall risk score thresholds (higher = riskier)
              </Typography>

              <Grid container spacing={3} sx={{ mt: 1 }}>
                <Grid item xs={6}>
                  <Typography variant="subtitle2" gutterBottom>
                    Warning: {riskScoreWarning.toFixed(0)}
                  </Typography>
                  <Slider
                    value={riskScoreWarning}
                    onChange={(_, val) => setRiskScoreWarning(val as number)}
                    min={0}
                    max={100}
                    step={1}
                    marks={[
                      { value: 0, label: '0' },
                      { value: 50, label: '50' },
                      { value: 100, label: '100' },
                    ]}
                    valueLabelDisplay="auto"
                    color="warning"
                  />
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="subtitle2" gutterBottom>
                    Critical: {riskScoreCritical.toFixed(0)}
                  </Typography>
                  <Slider
                    value={riskScoreCritical}
                    onChange={(_, val) => setRiskScoreCritical(val as number)}
                    min={0}
                    max={100}
                    step={1}
                    marks={[
                      { value: 0, label: '0' },
                      { value: 50, label: '50' },
                      { value: 100, label: '100' },
                    ]}
                    valueLabelDisplay="auto"
                    color="error"
                  />
                </Grid>
              </Grid>
            </Paper>

            {/* VaR Thresholds */}
            <Paper elevation={1} sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                Value at Risk (VaR) Thresholds (%)
              </Typography>
              <Typography variant="body2" color="textSecondary" gutterBottom>
                Maximum expected loss at 95% confidence (negative values)
              </Typography>

              <Grid container spacing={3} sx={{ mt: 1 }}>
                <Grid item xs={6}>
                  <Typography variant="subtitle2" gutterBottom>
                    Warning: {varWarning.toFixed(1)}%
                  </Typography>
                  <Slider
                    value={varWarning}
                    onChange={(_, val) => setVarWarning(val as number)}
                    min={-15}
                    max={0}
                    step={0.5}
                    marks={[
                      { value: -15, label: '-15%' },
                      { value: -7.5, label: '-7.5%' },
                      { value: 0, label: '0%' },
                    ]}
                    valueLabelDisplay="auto"
                    color="warning"
                  />
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="subtitle2" gutterBottom>
                    Critical: {varCritical.toFixed(1)}%
                  </Typography>
                  <Slider
                    value={varCritical}
                    onChange={(_, val) => setVarCritical(val as number)}
                    min={-25}
                    max={-5}
                    step={0.5}
                    marks={[
                      { value: -25, label: '-25%' },
                      { value: -15, label: '-15%' },
                      { value: -5, label: '-5%' },
                    ]}
                    valueLabelDisplay="auto"
                    color="error"
                  />
                </Grid>
              </Grid>
            </Paper>

            <Alert severity="info" sx={{ mt: 2 }}>
              <Typography variant="body2">
                <strong>Warning thresholds</strong> will highlight positions in yellow.{' '}
                <strong>Critical thresholds</strong> will highlight positions in red.
                These thresholds help you identify positions that may require attention based on your risk tolerance.
              </Typography>
            </Alert>
          </Box>
        )}
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button onClick={onClose} disabled={updateMutation.isPending}>
          Cancel
        </Button>
        <Button
          onClick={handleReset}
          startIcon={<RestartAlt />}
          disabled={updateMutation.isPending}
        >
          Reset to Defaults
        </Button>
        <Button
          onClick={handleSave}
          variant="contained"
          startIcon={<Save />}
          disabled={updateMutation.isPending}
        >
          {updateMutation.isPending ? 'Saving...' : 'Save Changes'}
        </Button>
      </DialogActions>
    </Dialog>
  );
}
