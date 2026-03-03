import { Box, Typography, Paper, LinearProgress, Chip, Grid } from '@mui/material';
import type { GoalProgress } from '../../../types';

const GOAL_LABELS: Record<string, string> = {
    retirement: 'Retirement',
    home_purchase: 'Home Purchase',
    education: 'Education',
    travel: 'Travel',
    emergency_fund: 'Emergency Fund',
    other: 'Other',
};

const STATUS_CONFIG: Record<string, { color: 'success' | 'warning' | 'error' | 'info'; label: string }> = {
    on_track: { color: 'success', label: 'On Track' },
    behind: { color: 'warning', label: 'Behind' },
    at_risk: { color: 'error', label: 'At Risk' },
    completed: { color: 'info', label: 'Completed' },
};

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

interface GoalProgressBarsProps {
    goals: GoalProgress[];
}

export function GoalProgressBars({ goals }: GoalProgressBarsProps) {
    if (goals.length === 0) {
        return (
            <Paper sx={{ p: 3 }}>
                <Typography variant="h6" gutterBottom>
                    Goal Progress
                </Typography>
                <Box display="flex" alignItems="center" justifyContent="center" py={4}>
                    <Typography variant="body2" color="text.secondary">
                        No goals set yet. Add goals in the survey to track progress.
                    </Typography>
                </Box>
            </Paper>
        );
    }

    return (
        <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
                Goal Progress ({goals.length})
            </Typography>
            <Box display="flex" flexDirection="column" gap={3}>
                {goals.map((goal) => (
                    <GoalProgressItem key={goal.goal_id} goal={goal} />
                ))}
            </Box>
        </Paper>
    );
}

function GoalProgressItem({ goal }: { goal: GoalProgress }) {
    const statusConfig = STATUS_CONFIG[goal.status] || STATUS_CONFIG.behind;
    const progressColor = statusConfig.color;
    const progressValue = Math.min(goal.progress_percentage, 100);

    return (
        <Box>
            <Box display="flex" justifyContent="space-between" alignItems="center" mb={0.5}>
                <Box display="flex" alignItems="center" gap={1}>
                    <Typography variant="subtitle2">
                        {goal.description || GOAL_LABELS[goal.goal_type] || goal.goal_type}
                    </Typography>
                    <Chip
                        label={statusConfig.label}
                        size="small"
                        color={progressColor}
                        variant="outlined"
                    />
                </Box>
                <Typography variant="subtitle2" fontWeight="bold">
                    {goal.progress_percentage.toFixed(0)}%
                </Typography>
            </Box>
            <LinearProgress
                variant="determinate"
                value={progressValue}
                color={progressColor}
                sx={{ height: 10, borderRadius: 5, mb: 1 }}
            />
            <Grid container spacing={2}>
                <Grid item xs={4}>
                    <Typography variant="caption" color="text.secondary">
                        Saved
                    </Typography>
                    <Typography variant="body2" fontWeight="bold">
                        {formatCurrency(goal.current_savings)}
                    </Typography>
                </Grid>
                <Grid item xs={4}>
                    <Typography variant="caption" color="text.secondary">
                        Target
                    </Typography>
                    <Typography variant="body2" fontWeight="bold">
                        {formatCurrency(goal.target_amount)}
                    </Typography>
                </Grid>
                <Grid item xs={4}>
                    <Typography variant="caption" color="text.secondary">
                        {goal.monthly_contribution_needed != null ? 'Monthly Needed' : 'Time Left'}
                    </Typography>
                    <Typography variant="body2" fontWeight="bold">
                        {goal.monthly_contribution_needed != null
                            ? formatCurrency(goal.monthly_contribution_needed)
                            : goal.months_remaining != null
                                ? `${goal.months_remaining} months`
                                : 'N/A'}
                    </Typography>
                    {goal.monthly_contribution_needed != null && (
                        <Typography variant="caption" color="text.secondary" display="block">
                            {goal.contribution_uses_growth
                                ? 'incl. 6% annual growth'
                                : 'no growth assumed'}
                        </Typography>
                    )}
                </Grid>
            </Grid>
        </Box>
    );
}
