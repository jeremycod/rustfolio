import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  Button,
  Grid,
  Stepper,
  Step,
  StepLabel,
  Card,
  CardContent,
  Chip,
  CircularProgress,
  Alert,
  Slider,
  RadioGroup,
  Radio,
  FormControlLabel,
  FormControl,
  FormLabel,
  LinearProgress,
  Divider,
  Tooltip,
} from '@mui/material';
import {
  Savings,
  School,
  TrendingUp,
  AccountBalanceWallet,
  ArrowForward,
  ArrowBack,
  Star,
  Info,
} from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getLongTermGuidance, listPortfolios } from '../lib/endpoints';
import type {
  InvestmentGoal,
  RiskAppetite,
  LongTermGuidance,
  LongTermRecommendation,
  QualityScore,
} from '../types';

const GOAL_OPTIONS: { value: InvestmentGoal; label: string; description: string; icon: React.ReactNode }[] = [
  {
    value: 'retirement',
    label: 'Retirement',
    description: 'Build a diversified portfolio for long-term retirement savings with focus on stability and income.',
    icon: <Savings sx={{ fontSize: 40 }} />,
  },
  {
    value: 'college',
    label: 'College Fund',
    description: 'Save for education expenses with a balanced approach to growth and preservation.',
    icon: <School sx={{ fontSize: 40 }} />,
  },
  {
    value: 'wealth',
    label: 'Wealth Building',
    description: 'Maximize long-term capital appreciation through growth-oriented investments.',
    icon: <TrendingUp sx={{ fontSize: 40 }} />,
  },
];

const RISK_OPTIONS: { value: RiskAppetite; label: string; description: string }[] = [
  {
    value: 'conservative',
    label: 'Conservative',
    description: 'Lower risk, focus on capital preservation. Suitable for shorter horizons or risk-averse investors.',
  },
  {
    value: 'moderate',
    label: 'Moderate',
    description: 'Moderate risk with a mix of growth and stability. Suitable for most long-term investors.',
  },
  {
    value: 'aggressive',
    label: 'Aggressive',
    description: 'Higher risk, maximum growth potential. Suitable for longer horizons and risk-tolerant investors.',
  },
];

const steps = ['Investment Goal', 'Time Horizon', 'Risk Tolerance', 'Review & Generate'];

export function LongTermGuidancePage() {
  const [activeStep, setActiveStep] = useState(0);
  const [goal, setGoal] = useState<InvestmentGoal>('retirement');
  const [horizon, setHorizon] = useState(10);
  const [riskTolerance, setRiskTolerance] = useState<RiskAppetite>('moderate');
  const [submitted, setSubmitted] = useState(false);

  // Fetch portfolios to get first portfolio ID
  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  // Auto-select first portfolio
  const portfolioId = portfoliosQ.data?.[0]?.id;

  const guidanceQ = useQuery({
    queryKey: ['long-term-guidance', portfolioId, goal, horizon, riskTolerance],
    queryFn: () => {
      if (!portfolioId) throw new Error('No portfolio available. Please create a portfolio first.');
      return getLongTermGuidance(portfolioId, goal, horizon, riskTolerance);
    },
    enabled: submitted && !!portfolioId,
  });

  const handleNext = () => {
    if (activeStep === steps.length - 1) {
      setSubmitted(true);
    } else {
      setActiveStep(prev => prev + 1);
    }
  };

  const handleBack = () => {
    if (submitted) {
      setSubmitted(false);
    } else {
      setActiveStep(prev => prev - 1);
    }
  };

  const handleReset = () => {
    setActiveStep(0);
    setSubmitted(false);
  };

  const renderStepContent = () => {
    switch (activeStep) {
      case 0:
        return (
          <Box>
            <Typography variant="h6" gutterBottom>
              What is your investment goal?
            </Typography>
            <Typography variant="body2" color="text.secondary" mb={3}>
              Select the primary purpose for your investment. This helps us tailor recommendations to your specific needs.
            </Typography>
            <Grid container spacing={2}>
              {GOAL_OPTIONS.map(opt => (
                <Grid item xs={12} sm={6} key={opt.value}>
                  <Card
                    sx={{
                      cursor: 'pointer',
                      border: 2,
                      borderColor: goal === opt.value ? 'primary.main' : 'transparent',
                      transition: 'all 0.2s',
                      '&:hover': { borderColor: goal === opt.value ? 'primary.main' : 'divider' },
                    }}
                    onClick={() => setGoal(opt.value)}
                  >
                    <CardContent>
                      <Box display="flex" alignItems="center" gap={2}>
                        <Box color={goal === opt.value ? 'primary.main' : 'text.secondary'}>
                          {opt.icon}
                        </Box>
                        <Box>
                          <Typography variant="subtitle1" fontWeight="bold">
                            {opt.label}
                          </Typography>
                          <Typography variant="body2" color="text.secondary">
                            {opt.description}
                          </Typography>
                        </Box>
                      </Box>
                    </CardContent>
                  </Card>
                </Grid>
              ))}
            </Grid>
          </Box>
        );

      case 1:
        return (
          <Box>
            <Typography variant="h6" gutterBottom>
              What is your investment time horizon?
            </Typography>
            <Typography variant="body2" color="text.secondary" mb={3}>
              Select how many years you plan to invest. Longer horizons allow for more growth-oriented strategies.
            </Typography>
            <Box sx={{ px: 4, py: 2 }}>
              <Slider
                value={horizon}
                onChange={(_, v) => setHorizon(v as number)}
                min={1}
                max={40}
                step={1}
                marks={[
                  { value: 1, label: '1yr' },
                  { value: 5, label: '5yr' },
                  { value: 10, label: '10yr' },
                  { value: 20, label: '20yr' },
                  { value: 30, label: '30yr' },
                  { value: 40, label: '40yr' },
                ]}
                valueLabelDisplay="on"
                valueLabelFormat={(v) => `${v} years`}
              />
            </Box>
            <Box textAlign="center" mt={2}>
              <Typography variant="h4" fontWeight="bold" color="primary.main">
                {horizon} {horizon === 1 ? 'Year' : 'Years'}
              </Typography>
              <Typography variant="body2" color="text.secondary">
                {horizon <= 3 ? 'Short-term (focus on preservation)' :
                 horizon <= 10 ? 'Medium-term (balanced growth)' :
                 'Long-term (maximum growth potential)'}
              </Typography>
            </Box>
          </Box>
        );

      case 2:
        return (
          <Box>
            <Typography variant="h6" gutterBottom>
              What is your risk tolerance?
            </Typography>
            <Typography variant="body2" color="text.secondary" mb={3}>
              This determines the balance between potential returns and acceptable volatility in your recommendations.
            </Typography>
            <FormControl component="fieldset">
              <RadioGroup
                value={riskTolerance}
                onChange={(e) => setRiskTolerance(e.target.value as RiskAppetite)}
              >
                {RISK_OPTIONS.map(opt => (
                  <Paper
                    key={opt.value}
                    variant="outlined"
                    sx={{
                      p: 2,
                      mb: 1.5,
                      border: 2,
                      borderColor: riskTolerance === opt.value ? 'primary.main' : 'divider',
                      cursor: 'pointer',
                    }}
                    onClick={() => setRiskTolerance(opt.value)}
                  >
                    <FormControlLabel
                      value={opt.value}
                      control={<Radio />}
                      label={
                        <Box>
                          <Typography fontWeight="bold">{opt.label}</Typography>
                          <Typography variant="body2" color="text.secondary">
                            {opt.description}
                          </Typography>
                        </Box>
                      }
                    />
                  </Paper>
                ))}
              </RadioGroup>
            </FormControl>
          </Box>
        );

      case 3:
        return (
          <Box>
            <Typography variant="h6" gutterBottom>
              Review Your Preferences
            </Typography>
            <Typography variant="body2" color="text.secondary" mb={3}>
              Confirm your selections before generating personalized long-term investment guidance.
            </Typography>
            <Paper variant="outlined" sx={{ p: 3 }}>
              <Grid container spacing={2}>
                <Grid item xs={12} sm={4}>
                  <Typography variant="caption" color="text.secondary">
                    INVESTMENT GOAL
                  </Typography>
                  <Typography variant="body1" fontWeight="bold">
                    {GOAL_OPTIONS.find(g => g.value === goal)?.label}
                  </Typography>
                </Grid>
                <Grid item xs={12} sm={4}>
                  <Typography variant="caption" color="text.secondary">
                    TIME HORIZON
                  </Typography>
                  <Typography variant="body1" fontWeight="bold">
                    {horizon} Years
                  </Typography>
                </Grid>
                <Grid item xs={12} sm={4}>
                  <Typography variant="caption" color="text.secondary">
                    RISK TOLERANCE
                  </Typography>
                  <Typography variant="body1" fontWeight="bold">
                    {riskTolerance}
                  </Typography>
                </Grid>
              </Grid>
            </Paper>
          </Box>
        );

      default:
        return null;
    }
  };

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={3}>
        <Savings sx={{ fontSize: 32, color: 'primary.main' }} />
        <Typography variant="h4" fontWeight="bold">
          Long-Term Investment Guidance
        </Typography>
      </Box>

      {!submitted ? (
        <Paper sx={{ p: 3 }}>
          <Stepper activeStep={activeStep} sx={{ mb: 4 }}>
            {steps.map(label => (
              <Step key={label}>
                <StepLabel>{label}</StepLabel>
              </Step>
            ))}
          </Stepper>

          {renderStepContent()}

          <Box display="flex" justifyContent="space-between" mt={4}>
            <Button
              disabled={activeStep === 0}
              onClick={handleBack}
              startIcon={<ArrowBack />}
            >
              Back
            </Button>
            <Button
              variant="contained"
              onClick={handleNext}
              endIcon={activeStep === steps.length - 1 ? undefined : <ArrowForward />}
            >
              {activeStep === steps.length - 1 ? 'Generate Guidance' : 'Next'}
            </Button>
          </Box>
        </Paper>
      ) : (
        <Box>
          {/* Summary Bar */}
          <Paper sx={{ p: 2, mb: 3, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
            <Box display="flex" gap={2}>
              <Chip label={GOAL_OPTIONS.find(g => g.value === goal)?.label} color="primary" />
              <Chip label={`${horizon} Years`} variant="outlined" />
              <Chip label={riskTolerance} variant="outlined" />
            </Box>
            <Button size="small" onClick={handleReset}>
              Change Preferences
            </Button>
          </Paper>

          {guidanceQ.isLoading && (
            <Box display="flex" flexDirection="column" alignItems="center" py={6}>
              <CircularProgress size={48} />
              <Typography variant="body1" color="text.secondary" mt={2}>
                Generating personalized investment guidance...
              </Typography>
            </Box>
          )}

          {guidanceQ.error && (
            <Alert severity="error">
              Failed to generate guidance: {(guidanceQ.error as Error).message}
            </Alert>
          )}

          {guidanceQ.data && (
            <GuidanceResults guidance={guidanceQ.data} />
          )}
        </Box>
      )}
    </Box>
  );
}

function GuidanceResults({ guidance }: { guidance: LongTermGuidance }) {
  return (
    <Box>
      {/* Allocation Strategy */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Recommended Asset Allocation
        </Typography>
        <Grid container spacing={3}>
          <Grid item xs={12} sm={4}>
            <AllocationBar
              label="Low Risk"
              value={guidance.allocation_strategy.low_risk_allocation * 100}
              color="success.main"
            />
          </Grid>
          <Grid item xs={12} sm={4}>
            <AllocationBar
              label="Medium Risk"
              value={guidance.allocation_strategy.medium_risk_allocation * 100}
              color="warning.main"
            />
          </Grid>
          <Grid item xs={12} sm={4}>
            <AllocationBar
              label="High Risk"
              value={guidance.allocation_strategy.high_risk_allocation * 100}
              color="error.main"
            />
          </Grid>
        </Grid>
      </Paper>

      {/* Projected Growth */}
      {guidance.projected_growth && (
        <Paper sx={{ p: 3, mb: 3 }}>
          <Typography variant="h6" gutterBottom>
            Projected Growth (per $10,000 invested)
          </Typography>
          <Grid container spacing={3}>
            <Grid item xs={12} sm={4}>
              <Box textAlign="center">
                <Typography variant="caption" color="text.secondary">
                  CONSERVATIVE
                </Typography>
                <Typography variant="h5" color="warning.main" fontWeight="bold">
                  ${guidance.projected_growth.conservative.toLocaleString()}
                </Typography>
              </Box>
            </Grid>
            <Grid item xs={12} sm={4}>
              <Box textAlign="center">
                <Typography variant="caption" color="text.secondary">
                  EXPECTED
                </Typography>
                <Typography variant="h5" color="primary.main" fontWeight="bold">
                  ${guidance.projected_growth.expected.toLocaleString()}
                </Typography>
              </Box>
            </Grid>
            <Grid item xs={12} sm={4}>
              <Box textAlign="center">
                <Typography variant="caption" color="text.secondary">
                  OPTIMISTIC
                </Typography>
                <Typography variant="h5" color="success.main" fontWeight="bold">
                  ${guidance.projected_growth.optimistic.toLocaleString()}
                </Typography>
              </Box>
            </Grid>
          </Grid>
        </Paper>
      )}

      {/* Recommendations */}
      <Paper sx={{ p: 3, mb: 3 }}>
        <Typography variant="h6" gutterBottom>
          Recommended Holdings ({guidance.recommendations.length})
        </Typography>
        <Grid container spacing={2}>
          {guidance.recommendations.map(rec => (
            <Grid item xs={12} sm={6} md={4} key={rec.ticker}>
              <LongTermRecCard recommendation={rec} />
            </Grid>
          ))}
        </Grid>
      </Paper>

      {/* Disclaimer */}
      <Alert severity="info" icon={<Info />}>
        <Typography variant="body2">
          <strong>Disclaimer:</strong> Long-term investment recommendations are for informational purposes only
          and do not constitute financial advice. Past performance does not guarantee future results.
          Consider consulting a financial advisor before making investment decisions. All projections are
          estimates based on historical data and may not reflect actual future performance.
        </Typography>
      </Alert>
    </Box>
  );
}

function AllocationBar({ label, value, color }: { label: string; value: number; color: string }) {
  return (
    <Box>
      <Box display="flex" justifyContent="space-between" mb={0.5}>
        <Typography variant="body2" fontWeight="bold">{label}</Typography>
        <Typography variant="body2" fontWeight="bold">{value.toFixed(0)}%</Typography>
      </Box>
      <LinearProgress
        variant="determinate"
        value={value}
        sx={{
          height: 12,
          borderRadius: 6,
          bgcolor: 'grey.200',
          '& .MuiLinearProgress-bar': { bgcolor: color, borderRadius: 6 },
        }}
      />
    </Box>
  );
}

function LongTermRecCard({ recommendation }: { recommendation: LongTermRecommendation }) {
  const qs = recommendation.quality_score;
  const riskColor = recommendation.risk_class === 'low' ? 'success' :
    recommendation.risk_class === 'medium' ? 'warning' : 'error';

  return (
    <Card variant="outlined" sx={{ height: '100%' }}>
      <CardContent>
        <Box display="flex" justifyContent="space-between" alignItems="flex-start" mb={1}>
          <Box>
            <Typography variant="subtitle1" fontWeight="bold">
              {recommendation.ticker}
            </Typography>
            <Typography variant="caption" color="text.secondary" noWrap>
              {recommendation.holding_name || recommendation.ticker}
            </Typography>
          </Box>
          <Box textAlign="right">
            <Chip
              label={`${(recommendation.suggested_weight * 100).toFixed(0)}%`}
              size="small"
              color="primary"
            />
          </Box>
        </Box>

        <Box display="flex" gap={0.5} mb={1.5} flexWrap="wrap">
          <Chip
            label={`Risk: ${recommendation.risk_class.charAt(0).toUpperCase() + recommendation.risk_class.slice(1)}`}
            size="small"
            color={riskColor}
            variant="outlined"
          />
          {qs.industry && (
            <Chip label={qs.industry} size="small" variant="outlined" />
          )}
        </Box>

        {/* Quality Scores */}
        <Typography variant="caption" color="text.secondary" display="block" mb={0.5}>
          Quality Scores
        </Typography>
        <QualityScoreBar label="Overall" value={qs.composite_score} />
        <QualityScoreBar label="Growth" value={qs.growth_score} />
        <QualityScoreBar label="Dividend" value={qs.dividend_score} />
        <QualityScoreBar label="Moat" value={qs.moat_score} />

        {/* Key Metrics */}
        <Box display="flex" gap={1} mt={1.5} flexWrap="wrap">
          {qs.dividend_yield != null && (
            <Chip
              label={`Yield: ${qs.dividend_yield.toFixed(1)}%`}
              size="small"
              variant="outlined"
            />
          )}
          {qs.roe != null && (
            <Chip
              label={`ROE: ${qs.roe.toFixed(1)}%`}
              size="small"
              variant="outlined"
            />
          )}
        </Box>

        <Divider sx={{ my: 1.5 }} />
        <Typography variant="body2" color="text.secondary">
          {recommendation.rationale}
        </Typography>
      </CardContent>
    </Card>
  );
}

function QualityScoreBar({ label, value }: { label: string; value: number }) {
  const color = value >= 70 ? 'success' : value >= 40 ? 'warning' : 'error';

  return (
    <Box display="flex" alignItems="center" gap={1} mb={0.3}>
      <Typography variant="caption" sx={{ width: 60, flexShrink: 0 }}>
        {label}
      </Typography>
      <LinearProgress
        variant="determinate"
        value={Math.min(value, 100)}
        color={color}
        sx={{ flex: 1, height: 5, borderRadius: 3 }}
      />
      <Typography variant="caption" fontWeight="bold" sx={{ width: 30, textAlign: 'right' }}>
        {value.toFixed(0)}
      </Typography>
    </Box>
  );
}
