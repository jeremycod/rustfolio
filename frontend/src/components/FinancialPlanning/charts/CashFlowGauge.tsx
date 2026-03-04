import { useState } from 'react';
import { Box, Typography, Paper, Grid, Collapse, IconButton } from '@mui/material';
import { ExpandMore, ExpandLess } from '@mui/icons-material';
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
    monthlyTaxes?: number;
    monthlyPayrollDeductions?: number;
    monthlyNetIncome?: number;
    estimatedMonthlyExpenses?: number;
    expensesByCategory?: Array<{ category: string; amount: number }>;
    usingActualExpenses?: boolean;
    housingExcludedFromExpenses?: boolean;
    isHousehold?: boolean;
}

export function CashFlowGauge({
    monthlyCashFlow,
    savingsRate,
    grossAnnualIncome,
    monthlyGrossIncome,
    monthlyTaxes = 0,
    monthlyPayrollDeductions = 0,
    monthlyNetIncome,
    estimatedMonthlyExpenses,
    expensesByCategory = [],
    usingActualExpenses = false,
    housingExcludedFromExpenses = false,
    isHousehold = false,
}: CashFlowGaugeProps) {
    const [showExpenseBreakdown, setShowExpenseBreakdown] = useState(false);
    // Clamp savings rate between 0 and 100 for gauge display
    const displayRate = Math.max(0, Math.min(100, savingsRate));

    const gaugeColor = savingsRate >= 20 ? '#4caf50' :
        savingsRate >= 10 ? '#ff9800' : '#f44336';

    const statusText = savingsRate >= 20 ? 'Excellent' :
        savingsRate >= 10 ? 'Good' : 'Needs Attention';

    const data = [{ name: 'Savings Rate', value: displayRate, fill: gaugeColor }];


    const displayMonthlyGross = monthlyGrossIncome ?? (grossAnnualIncome ? grossAnnualIncome / 12 : null);
    const hasTax = monthlyTaxes > 0;
    const hasDeductions = monthlyPayrollDeductions > 0;
    // Net income after tax and payroll deductions
    const displayMonthlyNet = monthlyNetIncome ?? (displayMonthlyGross != null ? displayMonthlyGross - monthlyTaxes - monthlyPayrollDeductions : null);
    const displayMonthlyExpenses = estimatedMonthlyExpenses ?? (displayMonthlyNet != null ? displayMonthlyNet - monthlyCashFlow : null);

    // Debt payments = net - expenses - cash flow
    const debtPayments = displayMonthlyNet != null && displayMonthlyExpenses != null
        ? Math.max(0, displayMonthlyNet - displayMonthlyExpenses - monthlyCashFlow)
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
                        {displayMonthlyGross != null && (
                            <StatRow
                                label={isHousehold ? 'Household Gross Income' : 'Gross Income'}
                                value={formatCurrency(displayMonthlyGross)}
                                color="primary.main"
                            />
                        )}
                        {hasTax && (
                            <StatRow
                                label="− Income Tax"
                                value={formatCurrency(monthlyTaxes)}
                                color="warning.main"
                            />
                        )}
                        {hasDeductions && (
                            <StatRow
                                label="− Payroll Deductions (CPP, EI…)"
                                value={formatCurrency(monthlyPayrollDeductions)}
                                color="warning.main"
                            />
                        )}
                        {(hasTax || hasDeductions) && displayMonthlyNet != null && (
                            <StatRow
                                label="= Net Income"
                                value={formatCurrency(displayMonthlyNet)}
                                color="primary.main"
                            />
                        )}
                        {displayMonthlyExpenses != null && (
                            <>
                                <Box
                                    display="flex"
                                    justifyContent="space-between"
                                    alignItems="center"
                                    py={0.75}
                                    borderBottom="1px solid"
                                    borderColor="divider"
                                    sx={{ cursor: expensesByCategory.length > 0 ? 'pointer' : 'default' }}
                                    onClick={() => expensesByCategory.length > 0 && setShowExpenseBreakdown(v => !v)}
                                >
                                    <Box display="flex" alignItems="center" gap={0.5}>
                                        <Typography variant="body2" color="text.secondary">
                                            {usingActualExpenses
                                                ? housingExcludedFromExpenses
                                                    ? '− Living Expenses (actual, excl. mortgage)'
                                                    : '− Living Expenses (actual)'
                                                : '− Living Expenses (est. 70%)'}
                                        </Typography>
                                        {expensesByCategory.length > 0 && (
                                            <IconButton size="small" sx={{ p: 0 }}>
                                                {showExpenseBreakdown ? <ExpandLess fontSize="small" /> : <ExpandMore fontSize="small" />}
                                            </IconButton>
                                        )}
                                    </Box>
                                    <Typography variant="body2" fontWeight="bold" color="text.secondary">
                                        {formatCurrency(displayMonthlyExpenses)}
                                    </Typography>
                                </Box>
                                {expensesByCategory.length > 0 && (
                                    <Collapse in={showExpenseBreakdown}>
                                        <Box pl={2} py={0.5} bgcolor="action.hover" borderRadius={1} mt={0.5} mb={0.5}>
                                            {expensesByCategory.map(({ category, amount }) => (
                                                <Box key={category} display="flex" justifyContent="space-between" py={0.25}>
                                                    <Typography variant="caption" color="text.secondary" sx={{ textTransform: 'capitalize' }}>
                                                        {category.replace(/_/g, ' ')}
                                                    </Typography>
                                                    <Typography variant="caption" color="text.secondary">
                                                        {formatCurrency(amount)}
                                                    </Typography>
                                                </Box>
                                            ))}
                                        </Box>
                                    </Collapse>
                                )}
                            </>
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
                        ? 'Net Income − Living Expenses (from survey) − Debt Payments = Available Cash Flow'
                        : 'Net Income − Living Expenses (estimated at 70% of income) − Debt Payments = Available Cash Flow'
                    }
                </Typography>
                {housingExcludedFromExpenses && (
                    <Typography variant="caption" color="warning.main" display="block" mt={0.5}>
                        ⚠ Your mortgage payment appears in both Liabilities and the Housing expense category.
                        Housing has been excluded from Living Expenses to avoid double-counting — your mortgage is counted under Debt Payments only.
                    </Typography>
                )}
                {!usingActualExpenses && (
                    <Typography variant="caption" color="text.secondary" display="block" mt={1}>
                        <strong>Tip:</strong> Add your actual expenses in the survey Expenses step for an accurate breakdown.
                    </Typography>
                )}
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
