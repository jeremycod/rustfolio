import {
    Box,
    Typography,
    Grid,
    Paper,
    CircularProgress,
    Alert,
    Button,
    Chip,
    Divider,
} from '@mui/material';
import { Refresh, PictureAsPdf, Edit } from '@mui/icons-material';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
    getFinancialSnapshot,
    getFinancialSurvey,
    regenerateFinancialSnapshot,
} from '../../lib/endpoints';
import { NetWorthBreakdown } from './charts/NetWorthBreakdown';
import { CashFlowGauge } from './charts/CashFlowGauge';
import { RetirementProjectionChart } from './charts/RetirementProjection';
import { GoalProgressBars } from './charts/GoalProgressBars';

interface FinancialSnapshotViewProps {
    surveyId: string;
    onEdit?: () => void;
}

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

export function FinancialSnapshotView({ surveyId, onEdit }: FinancialSnapshotViewProps) {
    const queryClient = useQueryClient();

    const snapshotQ = useQuery({
        queryKey: ['financial-snapshot', surveyId],
        queryFn: () => getFinancialSnapshot(surveyId),
    });

    const surveyQ = useQuery({
        queryKey: ['financial-survey', surveyId],
        queryFn: () => getFinancialSurvey(surveyId),
    });

    const regenerateMutation = useMutation({
        mutationFn: () => regenerateFinancialSnapshot(surveyId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-snapshot', surveyId] });
        },
    });

    if (snapshotQ.isLoading || surveyQ.isLoading) {
        return (
            <Box display="flex" flexDirection="column" alignItems="center" py={6}>
                <CircularProgress size={48} />
                <Typography variant="body1" color="text.secondary" mt={2}>
                    Generating your financial snapshot...
                </Typography>
            </Box>
        );
    }

    if (snapshotQ.error) {
        return (
            <Alert severity="error">
                Failed to load snapshot: {(snapshotQ.error as Error).message}
            </Alert>
        );
    }

    const snapshot = snapshotQ.data;
    const survey = surveyQ.data;
    if (!snapshot || !snapshot.snapshot_data) {
        return (
            <Alert severity="warning">
                No snapshot data available. Please complete the survey and try refreshing.
            </Alert>
        );
    }

    const snapshotData = snapshot.snapshot_data;
    const userName = survey?.personal_info?.full_name || 'Your';
    const birthYear = survey?.personal_info?.birth_year;
    const currentAge = birthYear ? new Date().getFullYear() - birthYear : null;
    const grossAnnualIncome = survey?.income_info?.gross_annual_income || null;

    // Check if user has entered actual expenses
    const hasActualExpenses = (survey?.expenses?.length || 0) > 0;

    // Calculate total monthly contribution needed for all goals
    const totalMonthlyNeeded = snapshotData.goal_progress.reduce(
        (sum, goal) => sum + (goal.monthly_contribution_needed || 0),
        0
    );

    // Extract retirement goal savings target (if set) for cross-referencing in the projection
    const retirementGoalTarget = snapshotData.goal_progress.find(g => g.goal_type === 'retirement')?.target_amount ?? null;

    return (
        <Box>
            {/* Header */}
            <Paper sx={{ p: 3, mb: 3 }}>
                <Box display="flex" justifyContent="space-between" alignItems="flex-start">
                    <Box>
                        <Typography variant="h5" fontWeight="bold">
                            {userName === 'Your' ? 'Your' : `${userName}'s`} Financial Snapshot
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                            Generated {new Date(snapshot.generated_at).toLocaleDateString()}
                        </Typography>
                    </Box>
                    <Box display="flex" gap={1}>
                        {onEdit && (
                            <Button
                                size="small"
                                variant="outlined"
                                startIcon={<Edit />}
                                onClick={onEdit}
                            >
                                Edit Survey
                            </Button>
                        )}
                        <Button
                            size="small"
                            startIcon={<Refresh />}
                            onClick={() => regenerateMutation.mutate()}
                            disabled={regenerateMutation.isPending}
                        >
                            {regenerateMutation.isPending ? 'Refreshing...' : 'Refresh'}
                        </Button>
                        <Button
                            size="small"
                            startIcon={<PictureAsPdf />}
                            disabled
                        >
                            Export PDF
                        </Button>
                    </Box>
                </Box>

                <Divider sx={{ my: 2 }} />

                {/* Quick Stats */}
                <Grid container spacing={3}>
                    <Grid item xs={6} sm={3}>
                        <Box textAlign="center">
                            <Typography variant="caption" color="text.secondary">
                                NET WORTH
                            </Typography>
                            <Typography
                                variant="h5"
                                fontWeight="bold"
                                color={snapshotData.net_worth_breakdown.net_worth >= 0 ? 'success.main' : 'error.main'}
                            >
                                {formatCurrency(snapshotData.net_worth_breakdown.net_worth)}
                            </Typography>
                        </Box>
                    </Grid>
                    <Grid item xs={6} sm={3}>
                        <Box textAlign="center">
                            <Typography variant="caption" color="text.secondary">
                                TOTAL ASSETS
                            </Typography>
                            <Typography variant="h5" fontWeight="bold" color="primary.main">
                                {formatCurrency(snapshotData.net_worth_breakdown.total_assets)}
                            </Typography>
                        </Box>
                    </Grid>
                    <Grid item xs={6} sm={3}>
                        <Box textAlign="center">
                            <Typography variant="caption" color="text.secondary">
                                TOTAL LIABILITIES
                            </Typography>
                            <Typography variant="h5" fontWeight="bold" color="error.main">
                                {formatCurrency(snapshotData.net_worth_breakdown.total_liabilities)}
                            </Typography>
                        </Box>
                    </Grid>
                    <Grid item xs={6} sm={3}>
                        <Box textAlign="center">
                            <Typography variant="caption" color="text.secondary">
                                MONTHLY CASH FLOW
                            </Typography>
                            <Typography
                                variant="h5"
                                fontWeight="bold"
                                color={snapshotData.cash_flow.monthly_cash_flow >= 0 ? 'success.main' : 'error.main'}
                            >
                                {formatCurrency(snapshotData.cash_flow.monthly_cash_flow)}
                            </Typography>
                        </Box>
                    </Grid>
                </Grid>
            </Paper>

            {/* Charts Grid */}
            <Grid container spacing={3}>
                {/* Net Worth Breakdown */}
                <Grid item xs={12}>
                    <NetWorthBreakdown
                        assetBreakdown={snapshotData.net_worth_breakdown.assets_by_type.reduce((acc, item) => {
                            acc[item.category] = item.amount;
                            return acc;
                        }, {} as Record<string, number>)}
                        liabilityBreakdown={snapshotData.net_worth_breakdown.liabilities_by_type.reduce((acc, item) => {
                            acc[item.category] = item.amount;
                            return acc;
                        }, {} as Record<string, number>)}
                        netWorth={snapshotData.net_worth_breakdown.net_worth}
                        totalAssets={snapshotData.net_worth_breakdown.total_assets}
                        totalLiabilities={snapshotData.net_worth_breakdown.total_liabilities}
                    />
                </Grid>

                {/* Cash Flow Gauge */}
                <Grid item xs={12} md={6}>
                    <CashFlowGauge
                        monthlyCashFlow={snapshotData.cash_flow.monthly_cash_flow}
                        savingsRate={snapshotData.cash_flow.savings_rate}
                        grossAnnualIncome={grossAnnualIncome}
                        monthlyGrossIncome={snapshotData.cash_flow.monthly_gross_income}
                        estimatedMonthlyExpenses={snapshotData.cash_flow.estimated_monthly_expenses}
                        usingActualExpenses={hasActualExpenses}
                    />
                </Grid>

                {/* Goal Progress */}
                <Grid item xs={12} md={6}>
                    <GoalProgressBars goals={snapshotData.goal_progress} />
                </Grid>

                {/* Retirement Projection */}
                {snapshotData.retirement && (
                    <Grid item xs={12}>
                        <RetirementProjectionChart
                            projection={snapshotData.retirement}
                            currentAge={currentAge}
                            goalBasedMonthlySavings={totalMonthlyNeeded}
                            desiredAnnualRetirementIncome={survey?.income_info?.desired_annual_retirement_income || null}
                            retirementGoalTarget={retirementGoalTarget}
                        />
                    </Grid>
                )}

                {/* Recommendations */}
                {snapshotData.recommendations && snapshotData.recommendations.length > 0 && (
                    <Grid item xs={12}>
                        <Paper sx={{ p: 3 }}>
                            <Typography variant="h6" gutterBottom>
                                Key Recommendations
                            </Typography>
                            <Box display="flex" flexDirection="column" gap={1.5}>
                                {snapshotData.recommendations.map((rec, index) => (
                                    <Box key={index} display="flex" alignItems="flex-start" gap={1}>
                                        <Chip
                                            label={index + 1}
                                            size="small"
                                            color="primary"
                                            sx={{ minWidth: 28 }}
                                        />
                                        <Typography variant="body2">{rec}</Typography>
                                    </Box>
                                ))}
                            </Box>
                        </Paper>
                    </Grid>
                )}
            </Grid>

            {/* Disclaimer */}
            <Alert severity="info" sx={{ mt: 3 }}>
                <Typography variant="body2">
                    <strong>Disclaimer:</strong> This financial snapshot is generated based on the
                    information you provided and uses simplified assumptions for projections. It is
                    for informational purposes only and does not constitute financial advice. Please
                    consult a qualified financial advisor for personalized guidance.
                </Typography>
            </Alert>
        </Box>
    );
}
