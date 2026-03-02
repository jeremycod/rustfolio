import { useMemo } from 'react';
import { Box, Typography, Paper, Grid, Chip } from '@mui/material';
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip,
    Legend,
    ResponsiveContainer,
    ReferenceLine,
} from 'recharts';
import type { RetirementProjection as RetirementProjectionType } from '../../../types';

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

function formatCompact(value: number): string {
    if (value >= 1_000_000) return `$${(value / 1_000_000).toFixed(1)}M`;
    if (value >= 1_000) return `$${(value / 1_000).toFixed(0)}K`;
    return `$${value.toFixed(0)}`;
}

interface RetirementProjectionProps {
    projection: RetirementProjectionType;
    currentAge: number | null;
    additionalMonthlySavings?: number;
    desiredAnnualRetirementIncome?: number | null;
}

export function RetirementProjectionChart({
    projection,
    currentAge,
    additionalMonthlySavings = 0,
    desiredAnnualRetirementIncome = null,
}: RetirementProjectionProps) {
    const chartData = useMemo(() => {
        if (!currentAge) return [];

        const years = projection.years_to_retirement;
        const rate = projection.assumed_return_rate;
        const annualContribution = projection.annual_contribution;
        const additionalAnnual = additionalMonthlySavings * 12;

        const data = [];
        let balance = projection.current_retirement_savings;
        let balanceWithExtra = projection.current_retirement_savings;

        for (let year = 0; year <= years; year++) {
            data.push({
                year: year,
                age: currentAge + year,
                balance: Math.round(balance),
                balanceWithExtra: additionalAnnual > 0 ? Math.round(balanceWithExtra) : undefined,
            });
            balance = balance * (1 + rate) + annualContribution;
            balanceWithExtra = balanceWithExtra * (1 + rate) + annualContribution + additionalAnnual;
        }

        return data;
    }, [projection, currentAge, additionalMonthlySavings]);

    const retirementAge = currentAge ? currentAge + projection.years_to_retirement : null;

    // Calculate what the extra savings would yield
    const finalBalanceWithExtra = chartData.length > 0 ? chartData[chartData.length - 1].balanceWithExtra : null;
    const extraRetirementIncome = finalBalanceWithExtra
        ? (finalBalanceWithExtra * projection.assumed_withdrawal_rate) / 12
        : null;

    return (
        <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
                Retirement Projection
            </Typography>
            {additionalMonthlySavings > 0 && extraRetirementIncome && (
                <Box mb={2} p={2} bgcolor="success.light" borderRadius={1}>
                    <Typography variant="body2" color="success.dark">
                        <strong>Impact of {formatCurrency(additionalMonthlySavings)}/month extra savings:</strong>
                        {' '}At retirement, you'd have an additional{' '}
                        <strong>{formatCurrency(finalBalanceWithExtra! - projection.projected_total_at_retirement)}</strong>
                        {', giving you '}
                        <strong>{formatCurrency(extraRetirementIncome)}/month</strong>
                        {' instead of '}
                        {formatCurrency(projection.estimated_monthly_income)}/month
                        {' ('}
                        {formatCurrency(extraRetirementIncome - projection.estimated_monthly_income)}
                        /month more).
                    </Typography>
                </Box>
            )}

            <Grid container spacing={2} mb={3}>
                <Grid item xs={6} sm={3}>
                    <Box textAlign="center">
                        <Typography variant="caption" color="text.secondary">
                            CURRENT SAVINGS
                        </Typography>
                        <Typography variant="h6" fontWeight="bold">
                            {formatCurrency(projection.current_retirement_savings)}
                        </Typography>
                    </Box>
                </Grid>
                <Grid item xs={6} sm={3}>
                    <Box textAlign="center">
                        <Typography variant="caption" color="text.secondary">
                            PROJECTED AT RETIREMENT
                        </Typography>
                        <Typography variant="h6" fontWeight="bold" color="primary.main">
                            {formatCurrency(projection.projected_total_at_retirement)}
                        </Typography>
                    </Box>
                </Grid>
                <Grid item xs={6} sm={3}>
                    <Box textAlign="center">
                        <Typography variant="caption" color="text.secondary">
                            PROJECTED MONTHLY INCOME
                        </Typography>
                        <Typography variant="h6" fontWeight="bold" color="success.main">
                            {formatCurrency(projection.estimated_monthly_income)}
                        </Typography>
                    </Box>
                </Grid>
                <Grid item xs={6} sm={3}>
                    <Box textAlign="center">
                        <Typography variant="caption" color="text.secondary">
                            YEARS TO RETIREMENT
                        </Typography>
                        <Typography variant="h6" fontWeight="bold">
                            {projection.years_to_retirement}
                        </Typography>
                    </Box>
                </Grid>
            </Grid>

            {/* Retirement Income Goal Analysis */}
            {desiredAnnualRetirementIncome && desiredAnnualRetirementIncome > 0 && (
                <Box mt={3} p={2} bgcolor="grey.50" borderRadius={1}>
                    <Typography variant="subtitle2" gutterBottom fontWeight="bold">
                        Retirement Income Goal Analysis
                    </Typography>
                    <Grid container spacing={2}>
                        <Grid item xs={12} sm={4}>
                            <Typography variant="caption" color="text.secondary">Desired Monthly Income:</Typography>
                            <Typography variant="body2" fontWeight="bold">
                                {formatCurrency(desiredAnnualRetirementIncome / 12)}
                            </Typography>
                        </Grid>
                        <Grid item xs={12} sm={4}>
                            <Typography variant="caption" color="text.secondary">Projected Monthly Income:</Typography>
                            <Typography variant="body2" fontWeight="bold">
                                {formatCurrency(projection.estimated_monthly_income)}
                            </Typography>
                        </Grid>
                        <Grid item xs={12} sm={4}>
                            <Typography variant="caption" color="text.secondary">Monthly Gap:</Typography>
                            <Typography
                                variant="body2"
                                fontWeight="bold"
                                color={
                                    projection.estimated_monthly_income >= desiredAnnualRetirementIncome / 12
                                        ? 'success.main'
                                        : 'error.main'
                                }
                            >
                                {formatCurrency((desiredAnnualRetirementIncome / 12) - projection.estimated_monthly_income)}
                            </Typography>
                        </Grid>
                    </Grid>
                    {projection.estimated_monthly_income >= desiredAnnualRetirementIncome / 12 ? (
                        <Box mt={1} display="flex" alignItems="center" gap={1}>
                            <Chip label="On Track" color="success" size="small" />
                            <Typography variant="caption" color="text.secondary">
                                Your projected retirement income meets your goal!
                            </Typography>
                        </Box>
                    ) : (
                        <Box mt={1} display="flex" alignItems="center" gap={1}>
                            <Chip label="Needs Attention" color="error" size="small" />
                            <Typography variant="caption" color="text.secondary">
                                Consider increasing your retirement contributions to reach your income goal.
                            </Typography>
                        </Box>
                    )}
                </Box>
            )}

            {chartData.length > 0 ? (
                <ResponsiveContainer width="100%" height={300}>
                    <LineChart data={chartData} margin={{ top: 5, right: 30, left: 20, bottom: 5 }}>
                        <CartesianGrid strokeDasharray="3 3" />
                        <XAxis
                            dataKey="age"
                            label={{ value: 'Age', position: 'insideBottomRight', offset: -5 }}
                        />
                        <YAxis
                            tickFormatter={formatCompact}
                            label={{ value: 'Portfolio Value', angle: -90, position: 'insideLeft' }}
                        />
                        <Tooltip
                            formatter={(value: number | undefined) => value !== undefined ? formatCurrency(value) : ''}
                            labelFormatter={(age) => `Age ${age}`}
                        />
                        <Legend />
                        {retirementAge && (
                            <ReferenceLine
                                x={retirementAge}
                                stroke="#ff9800"
                                strokeDasharray="5 5"
                                label={{ value: 'Retirement', position: 'top' }}
                            />
                        )}
                        <Line
                            type="monotone"
                            dataKey="balance"
                            stroke="#2196f3"
                            strokeWidth={2}
                            dot={false}
                            name="Current Plan"
                        />
                        {additionalMonthlySavings > 0 && (
                            <Line
                                type="monotone"
                                dataKey="balanceWithExtra"
                                stroke="#4caf50"
                                strokeWidth={2}
                                strokeDasharray="5 5"
                                dot={false}
                                name={`With +${formatCurrency(additionalMonthlySavings)}/mo`}
                            />
                        )}
                    </LineChart>
                </ResponsiveContainer>
            ) : (
                <Box display="flex" alignItems="center" justifyContent="center" height={300}>
                    <Typography variant="body2" color="text.secondary">
                        Birth year required for retirement projection chart
                    </Typography>
                </Box>
            )}

            <Box display="flex" gap={1} mt={2} flexWrap="wrap">
                <Chip
                    label={`${(projection.assumed_return_rate * 100).toFixed(0)}% assumed return`}
                    size="small"
                    variant="outlined"
                />
                <Chip
                    label={`${(projection.assumed_withdrawal_rate * 100).toFixed(0)}% withdrawal rate`}
                    size="small"
                    variant="outlined"
                />
                <Chip
                    label={`Annual contribution: ${formatCurrency(projection.annual_contribution)}`}
                    size="small"
                    variant="outlined"
                />
            </Box>

            <Box mt={2} p={2} bgcolor="grey.50" borderRadius={1}>
                <Typography variant="caption" color="text.secondary" display="block" gutterBottom>
                    <strong>How it's calculated:</strong>
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                    • Starting with {formatCurrency(projection.current_retirement_savings)} in retirement accounts (RRSP, LIRA, RRIF, TFSA, 401k, IRA)
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                    • Adding {formatCurrency(projection.annual_contribution)}/year in contributions
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                    • Growing at {(projection.assumed_return_rate * 100).toFixed(0)}% annually for {projection.years_to_retirement} years
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                    • Withdrawing {(projection.assumed_withdrawal_rate * 100).toFixed(0)}% per year at retirement (4% rule)
                </Typography>
            </Box>
        </Paper>
    );
}
