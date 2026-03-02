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
}

export function RetirementProjectionChart({
    projection,
    currentAge,
}: RetirementProjectionProps) {
    const chartData = useMemo(() => {
        if (!currentAge) return [];

        const years = projection.years_to_retirement;
        const rate = projection.assumed_return_rate;
        const annualContribution = projection.annual_contribution;

        const data = [];
        let balance = projection.current_retirement_savings;

        for (let year = 0; year <= years; year++) {
            data.push({
                year: year,
                age: currentAge + year,
                balance: Math.round(balance),
            });
            balance = balance * (1 + rate) + annualContribution;
        }

        return data;
    }, [projection, currentAge]);

    const retirementAge = currentAge ? currentAge + projection.years_to_retirement : null;

    return (
        <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
                Retirement Projection
            </Typography>

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
                            EST. MONTHLY INCOME
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
                            name="Projected Balance"
                        />
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
        </Paper>
    );
}
