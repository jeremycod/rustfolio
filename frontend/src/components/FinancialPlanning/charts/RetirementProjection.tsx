import { useMemo, useState } from 'react';
import {
    Box,
    Typography,
    Paper,
    Grid,
    Chip,
    TextField,
    InputAdornment,
    IconButton,
    Button,
    Tooltip,
    Divider,
    Table,
    TableHead,
    TableBody,
    TableRow,
    TableCell,
} from '@mui/material';
import { InfoOutlined, Add, Close } from '@mui/icons-material';
import {
    LineChart,
    Line,
    XAxis,
    YAxis,
    CartesianGrid,
    Tooltip as RechartsTooltip,
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
    if (value >= 1_000_000) return `$${(value / 1_000_000).toFixed(2)}M`;
    if (value >= 1_000) return `$${(value / 1_000).toFixed(0)}K`;
    return `$${value.toFixed(0)}`;
}

// Green (#4caf50) is reserved for the goal-based line; custom scenarios cycle through these
const SCENARIO_COLORS = ['#ff9800', '#9c27b0', '#f44336', '#00bcd4', '#795548', '#607d8b', '#e91e63', '#ff5722'];
const SCENARIO_LABELS = ['A', 'B', 'C', 'D', 'E', 'F', 'G', 'H'];

interface Scenario {
    monthlyExtra: number;
    inputValue: string; // raw text while typing
}

interface RetirementProjectionProps {
    projection: RetirementProjectionType;
    currentAge: number | null;
    goalBasedMonthlySavings?: number; // auto-computed from goal monthly needed
    monthlyCashFlowSurplus?: number;  // available cash flow — what if it's all invested?
    desiredAnnualRetirementIncome?: number | null;
    retirementGoalTarget?: number | null;
}

export function RetirementProjectionChart({
    projection,
    currentAge,
    goalBasedMonthlySavings = 0,
    monthlyCashFlowSurplus,
    desiredAnnualRetirementIncome = null,
    retirementGoalTarget = null,
}: RetirementProjectionProps) {
    const [scenarios, setScenarios] = useState<Scenario[]>([]);

    const years = projection.years_to_retirement;
    const rate = projection.assumed_return_rate;
    const annualContribution = projection.annual_contribution;
    const retirementAge = currentAge ? currentAge + years : null;

    const showGoalLine = goalBasedMonthlySavings > 0;
    const showSavingsLine = monthlyCashFlowSurplus != null && monthlyCashFlowSurplus > 0;

    // Build chart data — base line + optional goal/savings lines + user-defined scenarios
    const chartData = useMemo(() => {
        if (!currentAge) return [];

        const data = [];
        let base = projection.current_retirement_savings;
        let goal = projection.current_retirement_savings;
        let savings = projection.current_retirement_savings;
        const scenarioBalances = scenarios.map(() => projection.current_retirement_savings);

        for (let year = 0; year <= years; year++) {
            const point: Record<string, number> = {
                year,
                age: currentAge + year,
                base: Math.round(base),
            };
            if (showGoalLine) {
                point.goal_based = Math.round(goal);
            }
            if (showSavingsLine) {
                point.savings_potential = Math.round(savings);
            }
            scenarios.forEach((s, i) => {
                point[`scenario_${i}`] = Math.round(scenarioBalances[i]);
            });
            data.push(point);

            base = base * (1 + rate) + annualContribution;
            goal = goal * (1 + rate) + annualContribution + goalBasedMonthlySavings * 12;
            savings = savings * (1 + rate) + annualContribution + (monthlyCashFlowSurplus ?? 0) * 12;
            scenarios.forEach((s, i) => {
                scenarioBalances[i] =
                    scenarioBalances[i] * (1 + rate) + annualContribution + s.monthlyExtra * 12;
            });
        }
        return data;
    }, [projection, currentAge, scenarios, years, rate, annualContribution, goalBasedMonthlySavings, showGoalLine, monthlyCashFlowSurplus, showSavingsLine]);

    // Final balances for each row
    const finalBase = chartData.length > 0 ? chartData[chartData.length - 1].base : projection.projected_total_at_retirement;
    const finalGoal = chartData.length > 0 ? (chartData[chartData.length - 1].goal_based ?? 0) : 0;
    const finalSavings = chartData.length > 0 ? (chartData[chartData.length - 1].savings_potential ?? 0) : 0;
    const scenarioFinals = scenarios.map((_, i) =>
        chartData.length > 0 ? chartData[chartData.length - 1][`scenario_${i}`] : 0
    );

    const monthlyIncome = (total: number) =>
        (total * projection.assumed_withdrawal_rate) / 12;

    const addScenario = () => {
        setScenarios([...scenarios, { monthlyExtra: 0, inputValue: '' }]);
    };

    const removeScenario = (i: number) => {
        setScenarios(scenarios.filter((_, idx) => idx !== i));
    };

    const updateScenario = (i: number, raw: string) => {
        const parsed = parseFloat(raw);
        setScenarios(scenarios.map((s, idx) =>
            idx === i
                ? { monthlyExtra: isNaN(parsed) ? 0 : parsed, inputValue: raw }
                : s
        ));
    };

    return (
        <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
                Retirement Projection
            </Typography>

            {/* ── Key stats ── */}
            <Grid container spacing={2} mb={3}>
                <Grid item xs={6} sm={3}>
                    <Box textAlign="center">
                        <Typography variant="caption" color="text.secondary">
                            CURRENT SAVINGS
                        </Typography>
                        <Typography variant="h6" fontWeight="bold">
                            {formatCurrency(projection.current_retirement_savings)}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                            from retirement goal
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
                        {retirementGoalTarget != null && retirementGoalTarget > 0 && (
                            <Typography
                                variant="caption"
                                color={projection.projected_total_at_retirement >= retirementGoalTarget ? 'success.main' : 'error.main'}
                                display="block"
                            >
                                Goal: {formatCompact(retirementGoalTarget)}
                                {' '}
                                {projection.projected_total_at_retirement >= retirementGoalTarget
                                    ? `· +${formatCompact(projection.projected_total_at_retirement - retirementGoalTarget)} above`
                                    : `· ${formatCompact(retirementGoalTarget - projection.projected_total_at_retirement)} short`}
                            </Typography>
                        )}
                    </Box>
                </Grid>
                <Grid item xs={6} sm={3}>
                    <Box textAlign="center">
                        <Box display="flex" alignItems="center" justifyContent="center" gap={0.5}>
                            <Typography variant="caption" color="text.secondary">
                                PROJECTED MONTHLY INCOME
                            </Typography>
                            <Tooltip
                                title={
                                    <Box sx={{ maxWidth: 320 }}>
                                        <Typography variant="body2" fontWeight="bold" gutterBottom>
                                            Projected Monthly Income — how it's calculated
                                        </Typography>
                                        <Typography variant="body2" gutterBottom>
                                            <strong>Projected Total × 4% ÷ 12</strong>
                                        </Typography>
                                        <Typography variant="body2" gutterBottom>
                                            This uses the <strong>4% Safe Withdrawal Rate</strong> (Trinity Study, 1998).
                                            The research shows that withdrawing 4% of your portfolio per year
                                            gives a ~95% chance the money lasts <strong>30 years</strong> — e.g.
                                            retiring at 65 and having income through age 95.
                                        </Typography>
                                        <Typography variant="body2" gutterBottom>
                                            The portfolio continues growing while you withdraw, so it doesn't
                                            simply count down to zero. Growth partially offsets withdrawals.
                                        </Typography>
                                        <Typography variant="body2" gutterBottom>
                                            If you plan to retire earlier (e.g. at 55), consider using a
                                            lower rate like 3–3.5% for a 40+ year horizon.
                                        </Typography>
                                        <Typography variant="body2" sx={{ fontStyle: 'italic' }}>
                                            This is portfolio withdrawals only — it excludes pension,
                                            CPP/OAS, Social Security, or any other income source.
                                        </Typography>
                                    </Box>
                                }
                                arrow
                                placement="top"
                            >
                                <InfoOutlined sx={{ fontSize: 14, color: 'text.secondary', cursor: 'help' }} />
                            </Tooltip>
                        </Box>
                        <Typography variant="h6" fontWeight="bold" color="success.main">
                            {formatCurrency(projection.estimated_monthly_income)}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                            portfolio withdrawals only
                        </Typography>
                    </Box>
                </Grid>
                <Grid item xs={6} sm={3}>
                    <Box textAlign="center">
                        <Typography variant="caption" color="text.secondary">
                            MONTHLY CONTRIBUTION
                        </Typography>
                        <Typography variant="h6" fontWeight="bold">
                            {formatCurrency(projection.annual_contribution / 12)}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                            salary × contribution %
                        </Typography>
                    </Box>
                </Grid>
            </Grid>

            {/* ── Retirement income goal analysis ── */}
            {desiredAnnualRetirementIncome && desiredAnnualRetirementIncome > 0 && (
                <Box mb={3} p={2} bgcolor="grey.50" borderRadius={1}>
                    <Typography variant="subtitle2" gutterBottom fontWeight="bold">
                        Retirement Income Goal
                    </Typography>
                    <Grid container spacing={2}>
                        <Grid item xs={12} sm={4}>
                            <Typography variant="caption" color="text.secondary">Desired Monthly</Typography>
                            <Typography variant="body2" fontWeight="bold">
                                {formatCurrency(desiredAnnualRetirementIncome / 12)}
                            </Typography>
                        </Grid>
                        <Grid item xs={12} sm={4}>
                            <Typography variant="caption" color="text.secondary">Projected Monthly</Typography>
                            <Typography variant="body2" fontWeight="bold">
                                {formatCurrency(projection.estimated_monthly_income)}
                            </Typography>
                        </Grid>
                        <Grid item xs={12} sm={4}>
                            <Typography variant="caption" color="text.secondary">Monthly Gap</Typography>
                            <Typography
                                variant="body2"
                                fontWeight="bold"
                                color={
                                    projection.estimated_monthly_income >= desiredAnnualRetirementIncome / 12
                                        ? 'success.main'
                                        : 'error.main'
                                }
                            >
                                {formatCurrency(Math.abs((desiredAnnualRetirementIncome / 12) - projection.estimated_monthly_income))}
                                {' '}
                                {projection.estimated_monthly_income >= desiredAnnualRetirementIncome / 12 ? 'surplus' : 'shortfall'}
                            </Typography>
                        </Grid>
                    </Grid>
                    <Box mt={1}>
                        <Chip
                            label={projection.estimated_monthly_income >= desiredAnnualRetirementIncome / 12 ? 'On Track' : 'Needs Attention'}
                            color={projection.estimated_monthly_income >= desiredAnnualRetirementIncome / 12 ? 'success' : 'error'}
                            size="small"
                        />
                    </Box>
                </Box>
            )}

            {/* ── Chart ── */}
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
                        <RechartsTooltip
                            formatter={(value: unknown) => typeof value === 'number' ? formatCurrency(value) : String(value)}
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
                        {retirementGoalTarget != null && retirementGoalTarget > 0 && (
                            <ReferenceLine
                                y={retirementGoalTarget}
                                stroke="#e91e63"
                                strokeDasharray="4 4"
                                label={{ value: `Goal ${formatCompact(retirementGoalTarget)}`, position: 'right' }}
                            />
                        )}
                        <Line
                            type="monotone"
                            dataKey="base"
                            stroke="#2196f3"
                            strokeWidth={2}
                            dot={false}
                            name="Current Plan"
                        />
                        {showGoalLine && (
                            <Line
                                type="monotone"
                                dataKey="goal_based"
                                stroke="#4caf50"
                                strokeWidth={2}
                                strokeDasharray="6 3"
                                dot={false}
                                name={`Monthly Needed (+${formatCurrency(goalBasedMonthlySavings)}/mo)`}
                            />
                        )}
                        {showSavingsLine && (
                            <Line
                                type="monotone"
                                dataKey="savings_potential"
                                stroke="#ff6f00"
                                strokeWidth={2}
                                strokeDasharray="3 3"
                                dot={false}
                                name={`If Surplus Invested (+${formatCurrency(monthlyCashFlowSurplus!)}/mo)`}
                            />
                        )}
                        {scenarios.map((s, i) => (
                            <Line
                                key={i}
                                type="monotone"
                                dataKey={`scenario_${i}`}
                                stroke={SCENARIO_COLORS[i]}
                                strokeWidth={2}
                                strokeDasharray="5 5"
                                dot={false}
                                name={`Scenario ${SCENARIO_LABELS[i]} (+${formatCurrency(s.monthlyExtra)}/mo)`}
                            />
                        ))}
                    </LineChart>
                </ResponsiveContainer>
            ) : (
                <Box display="flex" alignItems="center" justifyContent="center" height={300}>
                    <Typography variant="body2" color="text.secondary">
                        Birth year required for retirement projection chart
                    </Typography>
                </Box>
            )}

            {/* ── What-If Scenarios ── */}
            <Divider sx={{ my: 3 }} />
            <Box>
                <Typography variant="subtitle1" fontWeight="bold" gutterBottom>
                    What-If Scenarios
                </Typography>
                <Typography variant="body2" color="text.secondary" mb={2}>
                    Add extra monthly savings on top of your current contribution to see the impact on your projected outcome.
                </Typography>

                <Box display="flex" flexDirection="column" gap={1.5} mb={2}>
                    {scenarios.map((s, i) => (
                        <Box key={i} display="flex" alignItems="center" gap={2} flexWrap="wrap">
                            <Box
                                sx={{
                                    width: 12,
                                    height: 12,
                                    borderRadius: '50%',
                                    bgcolor: SCENARIO_COLORS[i],
                                    flexShrink: 0,
                                }}
                            />
                            <Typography variant="body2" sx={{ minWidth: 90 }}>
                                Scenario {SCENARIO_LABELS[i]}
                            </Typography>
                            <TextField
                                size="small"
                                label="Extra monthly savings"
                                type="number"
                                value={s.inputValue}
                                onChange={(e) => updateScenario(i, e.target.value)}
                                InputProps={{
                                    startAdornment: <InputAdornment position="start">+$</InputAdornment>,
                                    endAdornment: <InputAdornment position="end">/mo</InputAdornment>,
                                }}
                                inputProps={{ min: 0 }}
                                sx={{ width: 220 }}
                                placeholder="e.g. 3500"
                                autoFocus
                            />
                            <IconButton size="small" onClick={() => removeScenario(i)}>
                                <Close fontSize="small" />
                            </IconButton>
                        </Box>
                    ))}
                </Box>

                <Button
                    variant="outlined"
                    size="small"
                    startIcon={<Add />}
                    onClick={addScenario}
                >
                    Add Scenario
                </Button>
            </Box>

            {/* ── Scenario comparison table ── */}
            {(showGoalLine || showSavingsLine || scenarios.length > 0) && (
                <Box mt={3}>
                    <Typography variant="subtitle2" gutterBottom fontWeight="bold">
                        Scenario Comparison at Retirement (age {retirementAge ?? '—'})
                    </Typography>
                    <Table size="small">
                        <TableHead>
                            <TableRow>
                                <TableCell>Scenario</TableCell>
                                <TableCell align="right">Extra/month</TableCell>
                                <TableCell align="right">Projected Total</TableCell>
                                <TableCell align="right">Monthly Income</TableCell>
                                {retirementGoalTarget != null && retirementGoalTarget > 0 && (
                                    <TableCell align="right">vs Goal ({formatCompact(retirementGoalTarget)})</TableCell>
                                )}
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            {/* Base row */}
                            <TableRow>
                                <TableCell>
                                    <Box display="flex" alignItems="center" gap={1}>
                                        <Box sx={{ width: 10, height: 10, borderRadius: '50%', bgcolor: '#2196f3', flexShrink: 0 }} />
                                        Current Plan
                                    </Box>
                                </TableCell>
                                <TableCell align="right" sx={{ color: 'text.secondary' }}>—</TableCell>
                                <TableCell align="right">{formatCurrency(finalBase)}</TableCell>
                                <TableCell align="right" sx={{ color: 'success.main' }}>
                                    {formatCurrency(monthlyIncome(finalBase))}
                                </TableCell>
                                {retirementGoalTarget != null && retirementGoalTarget > 0 && (
                                    <TableCell
                                        align="right"
                                        sx={{ color: finalBase >= retirementGoalTarget ? 'success.main' : 'error.main' }}
                                    >
                                        {finalBase >= retirementGoalTarget
                                            ? `+${formatCompact(finalBase - retirementGoalTarget)}`
                                            : `-${formatCompact(retirementGoalTarget - finalBase)}`}
                                    </TableCell>
                                )}
                            </TableRow>
                            {/* Goal-based row (Monthly Needed from retirement goal) */}
                            {showGoalLine && (
                                <TableRow sx={{ bgcolor: '#4caf5010' }}>
                                    <TableCell>
                                        <Box display="flex" alignItems="center" gap={1}>
                                            <Box sx={{ width: 10, height: 10, borderRadius: '50%', bgcolor: '#4caf50', flexShrink: 0 }} />
                                            Monthly Needed
                                        </Box>
                                    </TableCell>
                                    <TableCell align="right" sx={{ fontWeight: 'bold' }}>
                                        +{formatCurrency(goalBasedMonthlySavings)}/mo
                                    </TableCell>
                                    <TableCell align="right" sx={{ fontWeight: 'bold' }}>
                                        {formatCurrency(finalGoal)}
                                    </TableCell>
                                    <TableCell align="right" sx={{ color: 'success.main', fontWeight: 'bold' }}>
                                        {formatCurrency(monthlyIncome(finalGoal))}
                                    </TableCell>
                                    {retirementGoalTarget != null && retirementGoalTarget > 0 && (
                                        <TableCell
                                            align="right"
                                            sx={{ color: finalGoal >= retirementGoalTarget ? 'success.main' : 'error.main', fontWeight: 'bold' }}
                                        >
                                            {finalGoal >= retirementGoalTarget
                                                ? `+${formatCompact(finalGoal - retirementGoalTarget)}`
                                                : `-${formatCompact(retirementGoalTarget - finalGoal)}`}
                                        </TableCell>
                                    )}
                                </TableRow>
                            )}
                            {/* Annual savings potential row */}
                            {showSavingsLine && (
                                <TableRow sx={{ bgcolor: '#ff6f0010' }}>
                                    <TableCell>
                                        <Box display="flex" alignItems="center" gap={1}>
                                            <Box sx={{ width: 10, height: 10, borderRadius: '50%', bgcolor: '#ff6f00', flexShrink: 0 }} />
                                            If Surplus Invested
                                        </Box>
                                    </TableCell>
                                    <TableCell align="right" sx={{ fontWeight: 'bold' }}>
                                        +{formatCurrency(monthlyCashFlowSurplus!)}/mo
                                        <Typography variant="caption" display="block" color="text.secondary">
                                            {formatCurrency((monthlyCashFlowSurplus ?? 0) * 12)}/yr
                                        </Typography>
                                    </TableCell>
                                    <TableCell align="right" sx={{ fontWeight: 'bold' }}>
                                        {formatCurrency(finalSavings)}
                                    </TableCell>
                                    <TableCell align="right" sx={{ color: 'success.main', fontWeight: 'bold' }}>
                                        {formatCurrency(monthlyIncome(finalSavings))}
                                    </TableCell>
                                    {retirementGoalTarget != null && retirementGoalTarget > 0 && (
                                        <TableCell
                                            align="right"
                                            sx={{ color: finalSavings >= retirementGoalTarget ? 'success.main' : 'error.main', fontWeight: 'bold' }}
                                        >
                                            {finalSavings >= retirementGoalTarget
                                                ? `+${formatCompact(finalSavings - retirementGoalTarget)}`
                                                : `-${formatCompact(retirementGoalTarget - finalSavings)}`}
                                        </TableCell>
                                    )}
                                </TableRow>
                            )}
                            {/* User-defined scenario rows */}
                            {scenarios.map((s, i) => {
                                const total = scenarioFinals[i] ?? 0;
                                const income = monthlyIncome(total);
                                return (
                                    <TableRow key={i} sx={{ bgcolor: `${SCENARIO_COLORS[i]}10` }}>
                                        <TableCell>
                                            <Box display="flex" alignItems="center" gap={1}>
                                                <Box sx={{ width: 10, height: 10, borderRadius: '50%', bgcolor: SCENARIO_COLORS[i], flexShrink: 0 }} />
                                                Scenario {SCENARIO_LABELS[i]}
                                            </Box>
                                        </TableCell>
                                        <TableCell align="right">
                                            +{formatCurrency(s.monthlyExtra)}/mo
                                        </TableCell>
                                        <TableCell align="right" sx={{ fontWeight: 'bold' }}>
                                            {formatCurrency(total)}
                                        </TableCell>
                                        <TableCell align="right" sx={{ color: 'success.main', fontWeight: 'bold' }}>
                                            {formatCurrency(income)}
                                        </TableCell>
                                        {retirementGoalTarget != null && retirementGoalTarget > 0 && (
                                            <TableCell
                                                align="right"
                                                sx={{ color: total >= retirementGoalTarget ? 'success.main' : 'error.main', fontWeight: 'bold' }}
                                            >
                                                {total >= retirementGoalTarget
                                                    ? `+${formatCompact(total - retirementGoalTarget)}`
                                                    : `-${formatCompact(retirementGoalTarget - total)}`}
                                            </TableCell>
                                        )}
                                    </TableRow>
                                );
                            })}
                        </TableBody>
                    </Table>
                </Box>
            )}

            {/* ── Assumptions ── */}
            <Box display="flex" gap={1} mt={3} flexWrap="wrap">
                <Chip
                    label={`${(projection.assumed_return_rate * 100).toFixed(0)}% assumed annual return`}
                    size="small"
                    variant="outlined"
                />
                <Chip
                    label="4% withdrawal rate · ~30 yr runway"
                    size="small"
                    variant="outlined"
                />
                <Chip
                    label={`${projection.years_to_retirement} years to retirement`}
                    size="small"
                    variant="outlined"
                />
            </Box>

            <Box mt={2} p={2} bgcolor="grey.50" borderRadius={1}>
                <Typography variant="caption" color="text.secondary" display="block" gutterBottom>
                    <strong>How it's calculated:</strong>
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                    • Starting with {formatCurrency(projection.current_retirement_savings)} (from your Retirement goal's current savings)
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                    • Adding {formatCurrency(projection.annual_contribution / 12)}/month ({formatCurrency(projection.annual_contribution)}/year)
                    {' — salary × (contribution rate + employer match)'}
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                    • Growing at {(projection.assumed_return_rate * 100).toFixed(0)}% annually for {projection.years_to_retirement} years
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                    • Monthly income = projected total × {(projection.assumed_withdrawal_rate * 100).toFixed(0)}% ÷ 12
                    {' (4% safe withdrawal rate — designed to last ~30 years with ~95% historical success; portfolio withdrawals only, excludes pension/CPP/OAS/Social Security)'}
                </Typography>
                {scenarios.length > 0 && (
                    <Typography variant="caption" color="text.secondary" display="block">
                        • Scenarios show additional monthly savings on top of the base contribution
                    </Typography>
                )}
            </Box>
        </Paper>
    );
}
