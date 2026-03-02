import { Box, Typography, Paper, Grid } from '@mui/material';
import {
    RadialBarChart,
    RadialBar,
    ResponsiveContainer,
    PolarAngleAxis,
} from 'recharts';

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

interface CashFlowGaugeProps {
    monthlyCashFlow: number;
    savingsRate: number;
    grossAnnualIncome: number | null;
}

export function CashFlowGauge({
    monthlyCashFlow,
    savingsRate,
    grossAnnualIncome,
}: CashFlowGaugeProps) {
    // Clamp savings rate between 0 and 100 for gauge display
    const displayRate = Math.max(0, Math.min(100, savingsRate));

    const gaugeColor = savingsRate >= 20 ? '#4caf50' :
        savingsRate >= 10 ? '#ff9800' : '#f44336';

    const statusText = savingsRate >= 20 ? 'Excellent' :
        savingsRate >= 10 ? 'Good' : 'Needs Attention';

    const data = [{ name: 'Savings Rate', value: displayRate, fill: gaugeColor }];

    const monthlyIncome = grossAnnualIncome ? grossAnnualIncome / 12 : null;
    const monthlyExpenses = monthlyIncome ? monthlyIncome - monthlyCashFlow : null;

    return (
        <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
                Cash Flow & Savings
            </Typography>
            <Grid container spacing={3} alignItems="center">
                <Grid item xs={12} md={5}>
                    <Box display="flex" justifyContent="center">
                        <ResponsiveContainer width={200} height={200}>
                            <RadialBarChart
                                cx="50%"
                                cy="50%"
                                innerRadius="60%"
                                outerRadius="90%"
                                startAngle={180}
                                endAngle={0}
                                data={data}
                            >
                                <PolarAngleAxis
                                    type="number"
                                    domain={[0, 100]}
                                    angleAxisId={0}
                                    tick={false}
                                />
                                <RadialBar
                                    dataKey="value"
                                    cornerRadius={10}
                                    background={{ fill: '#e0e0e0' }}
                                />
                            </RadialBarChart>
                        </ResponsiveContainer>
                    </Box>
                    <Box textAlign="center" mt={-6}>
                        <Typography variant="h4" fontWeight="bold" color={gaugeColor}>
                            {savingsRate.toFixed(0)}%
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                            Savings Rate
                        </Typography>
                        <Typography variant="caption" color={gaugeColor} fontWeight="bold">
                            {statusText}
                        </Typography>
                    </Box>
                </Grid>
                <Grid item xs={12} md={7}>
                    <Box>
                        <StatRow
                            label="Monthly Cash Flow"
                            value={formatCurrency(monthlyCashFlow)}
                            color={monthlyCashFlow >= 0 ? 'success.main' : 'error.main'}
                        />
                        {monthlyIncome != null && (
                            <StatRow
                                label="Est. Monthly Income"
                                value={formatCurrency(monthlyIncome)}
                            />
                        )}
                        {monthlyExpenses != null && (
                            <StatRow
                                label="Est. Monthly Expenses"
                                value={formatCurrency(monthlyExpenses)}
                            />
                        )}
                        {grossAnnualIncome != null && (
                            <StatRow
                                label="Annual Income"
                                value={formatCurrency(grossAnnualIncome)}
                            />
                        )}
                        <StatRow
                            label="Annual Savings"
                            value={formatCurrency(monthlyCashFlow * 12)}
                            color={monthlyCashFlow >= 0 ? 'success.main' : 'error.main'}
                        />
                    </Box>
                </Grid>
            </Grid>
        </Paper>
    );
}

function StatRow({
    label,
    value,
    color,
}: {
    label: string;
    value: string;
    color?: string;
}) {
    return (
        <Box display="flex" justifyContent="space-between" py={0.75} borderBottom="1px solid" borderColor="divider">
            <Typography variant="body2" color="text.secondary">
                {label}
            </Typography>
            <Typography variant="body2" fontWeight="bold" color={color || 'text.primary'}>
                {value}
            </Typography>
        </Box>
    );
}
