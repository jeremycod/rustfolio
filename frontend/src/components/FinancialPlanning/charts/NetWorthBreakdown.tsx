import { Box, Typography, Paper, Grid } from '@mui/material';
import {
    PieChart,
    Pie,
    Cell,
    Tooltip,
    Legend,
    ResponsiveContainer,
} from 'recharts';

const ASSET_COLORS = ['#4caf50', '#2196f3', '#ff9800', '#9c27b0', '#607d8b'];
const LIABILITY_COLORS = ['#f44336', '#e91e63', '#ff5722', '#d32f2f', '#c62828'];

const ASSET_LABELS: Record<string, string> = {
    liquid: 'Liquid Assets',
    investment: 'Investments',
    retirement: 'Retirement',
    real_estate: 'Real Estate',
    other: 'Other Assets',
};

const LIABILITY_LABELS: Record<string, string> = {
    mortgage: 'Mortgage',
    student_loan: 'Student Loans',
    auto_loan: 'Auto Loans',
    credit_card: 'Credit Cards',
    other: 'Other Debts',
};

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

interface NetWorthBreakdownProps {
    assetBreakdown: Record<string, number>;
    liabilityBreakdown: Record<string, number>;
    netWorth: number;
    totalAssets: number;
    totalLiabilities: number;
}

export function NetWorthBreakdown({
    assetBreakdown,
    liabilityBreakdown,
    netWorth,
    totalAssets,
    totalLiabilities,
}: NetWorthBreakdownProps) {
    const assetData = Object.entries(assetBreakdown)
        .filter(([, value]) => value > 0)
        .map(([key, value]) => ({
            name: ASSET_LABELS[key] || key,
            value,
        }));

    const liabilityData = Object.entries(liabilityBreakdown)
        .filter(([, value]) => value > 0)
        .map(([key, value]) => ({
            name: LIABILITY_LABELS[key] || key,
            value,
        }));

    const renderLabel = ({ percent }: { name: string; percent: number }) => {
        return percent > 0.05 ? `${(percent * 100).toFixed(0)}%` : '';
    };

    return (
        <Paper sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
                Net Worth Breakdown
            </Typography>
            <Box textAlign="center" mb={2}>
                <Typography variant="h4" fontWeight="bold" color={netWorth >= 0 ? 'success.main' : 'error.main'}>
                    {formatCurrency(netWorth)}
                </Typography>
                <Typography variant="body2" color="text.secondary">
                    Net Worth
                </Typography>
            </Box>
            <Grid container spacing={3}>
                <Grid item xs={12} md={6}>
                    <Typography variant="subtitle2" align="center" gutterBottom>
                        Assets ({formatCurrency(totalAssets)})
                    </Typography>
                    {assetData.length > 0 ? (
                        <ResponsiveContainer width="100%" height={250}>
                            <PieChart>
                                <Pie
                                    data={assetData}
                                    cx="50%"
                                    cy="50%"
                                    innerRadius={50}
                                    outerRadius={90}
                                    dataKey="value"
                                    label={renderLabel as any}
                                >
                                    {assetData.map((_, index) => (
                                        <Cell key={`asset-${index}`} fill={ASSET_COLORS[index % ASSET_COLORS.length]} />
                                    ))}
                                </Pie>
                                <Tooltip formatter={(value: number | undefined) => value !== undefined ? formatCurrency(value) : ''} />
                                <Legend />
                            </PieChart>
                        </ResponsiveContainer>
                    ) : (
                        <Box display="flex" alignItems="center" justifyContent="center" height={250}>
                            <Typography variant="body2" color="text.secondary">
                                No assets entered
                            </Typography>
                        </Box>
                    )}
                </Grid>
                <Grid item xs={12} md={6}>
                    <Typography variant="subtitle2" align="center" gutterBottom>
                        Liabilities ({formatCurrency(totalLiabilities)})
                    </Typography>
                    {liabilityData.length > 0 ? (
                        <ResponsiveContainer width="100%" height={250}>
                            <PieChart>
                                <Pie
                                    data={liabilityData}
                                    cx="50%"
                                    cy="50%"
                                    innerRadius={50}
                                    outerRadius={90}
                                    dataKey="value"
                                    label={renderLabel as any}
                                >
                                    {liabilityData.map((_, index) => (
                                        <Cell key={`liability-${index}`} fill={LIABILITY_COLORS[index % LIABILITY_COLORS.length]} />
                                    ))}
                                </Pie>
                                <Tooltip formatter={(value: number | undefined) => value !== undefined ? formatCurrency(value) : ''} />
                                <Legend />
                            </PieChart>
                        </ResponsiveContainer>
                    ) : (
                        <Box display="flex" alignItems="center" justifyContent="center" height={250}>
                            <Typography variant="body2" color="text.secondary">
                                No liabilities entered
                            </Typography>
                        </Box>
                    )}
                </Grid>
            </Grid>
        </Paper>
    );
}
