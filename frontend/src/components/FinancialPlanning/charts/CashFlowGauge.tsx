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
    monthlyGrossIncome?: number;
    estimatedMonthlyExpenses?: number;
    usingActualExpenses?: boolean;
}

export function CashFlowGauge({
    monthlyCashFlow,
    savingsRate,
    grossAnnualIncome,
    monthlyGrossIncome,
    estimatedMonthlyExpenses,
    usingActualExpenses = false,
}: CashFlowGaugeProps) {
    // Clamp savings rate between 0 and 100 for gauge display
    const displayRate = Math.max(0, Math.min(100, savingsRate));

    const gaugeColor = savingsRate >= 20 ? '#4caf50' :
        savingsRate >= 10 ? '#ff9800' : '#f44336';

    const statusText = savingsRate >= 20 ? 'Excellent' :
        savingsRate >= 10 ? 'Good' : 'Needs Attention';

    const data = [{ name: 'Savings Rate', value: displayRate, fill: gaugeColor }];

    // Use provided values or calculate from annual income
    const displayMonthlyIncome = monthlyGrossIncome ?? (grossAnnualIncome ? grossAnnualIncome / 12 : null);
    const displayMonthlyExpenses = estimatedMonthlyExpenses ?? (displayMonthlyIncome ? displayMonthlyIncome - monthlyCashFlow : null);

    // Calculate debt payments from the difference
    const debtPayments = displayMonthlyIncome && displayMonthlyExpenses
        ? displayMonthlyIncome - displayMonthlyExpenses - monthlyCashFlow
        : 0;

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
                        <Typography variant="subtitle2" color="text.secondary" mb={1}>
                            Monthly Breakdown:
                        </Typography>
                        {displayMonthlyIncome != null && (
                            <StatRow
                                label="Gross Income"
                                value={formatCurrency(displayMonthlyIncome)}
                                color="primary.main"
                            />
                        )}
                        {displayMonthlyExpenses != null && (
                            <StatRow
                                label={usingActualExpenses ? "− Living Expenses (actual)" : "− Living Expenses (est. 70%)"}
                                value={formatCurrency(displayMonthlyExpenses)}
                                color="text.secondary"
                            />
                        )}
                        {debtPayments > 0 && (
                            <StatRow
                                label="− Debt Payments"
                                value={formatCurrency(debtPayments)}
                                color="text.secondary"
                            />
                        )}
                        <Box my={1} borderTop="2px solid" borderColor="divider" />
                        <StatRow
                            label="= Available Cash Flow"
                            value={formatCurrency(monthlyCashFlow)}
                            color={monthlyCashFlow >= 0 ? 'success.main' : 'error.main'}
                        />
                        <StatRow
                            label="Annual Savings Potential"
                            value={formatCurrency(monthlyCashFlow * 12)}
                            color={monthlyCashFlow >= 0 ? 'success.main' : 'error.main'}
                        />
                    </Box>
                </Grid>
            </Grid>

            <Box mt={2} p={2} bgcolor="grey.50" borderRadius={1}>
                <Typography variant="caption" color="text.secondary" display="block" gutterBottom>
                    <strong>How Cash Flow is calculated:</strong>
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                    {usingActualExpenses
                        ? 'Monthly Income − Actual Living Expenses − Debt Payments = Available Cash Flow'
                        : 'Monthly Income − Living Expenses (estimated at 70% of income) − Debt Payments = Available Cash Flow'
                    }
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block" mt={1}>
                    <strong>Tip:</strong> {usingActualExpenses
                        ? 'You are using actual expense data for accurate cash flow calculations.'
                        : 'Add your actual expenses in the survey for more accurate cash flow calculations.'
                    }
                </Typography>
            </Box>
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
