import {
    ResponsiveContainer,
    LineChart,
    Line,
    XAxis,
    YAxis,
    Tooltip,
} from "recharts";
import { Paper, useTheme } from '@mui/material';
import type { ChartPoint } from "../types";

export function PortfolioChart({ series }: { series: ChartPoint[] }) {
    const theme = useTheme();

    const formatCurrency = (value: number) => {
        return new Intl.NumberFormat('en-US', {
            style: 'currency',
            currency: 'USD',
            minimumFractionDigits: 2,
            maximumFractionDigits: 2,
        }).format(value);
    };

    const CustomTooltip = ({ active, payload, label }: any) => {
        if (!active || !payload || !payload.length) return null;

        return (
            <div style={{
                backgroundColor: theme.palette.background.paper,
                border: `1px solid ${theme.palette.divider}`,
                borderRadius: theme.shape.borderRadius,
                padding: '12px',
            }}>
                <p style={{ margin: '0 0 8px 0', fontWeight: 600 }}>{label}</p>
                {payload.map((entry: any) => {
                    if (entry.value === null || entry.value === undefined) return null;

                    let displayValue = formatCurrency(entry.value);
                    let label = entry.name;

                    // Customize labels
                    if (entry.name === 'value') label = 'Portfolio Value';
                    if (entry.name === 'sma20') label = 'SMA (20)';
                    if (entry.name === 'ema20') label = 'EMA (20)';
                    if (entry.name === 'trend') label = 'Trendline';

                    return (
                        <p key={entry.name} style={{
                            margin: '4px 0',
                            color: entry.color,
                            fontSize: '14px',
                        }}>
                            {label}: <strong>{displayValue}</strong>
                        </p>
                    );
                })}
            </div>
        );
    };

    return (
        <Paper sx={{ p: 2, height: 400 }}>
            <ResponsiveContainer width="100%" height="100%">
                <LineChart data={series}>
                    <XAxis dataKey="date" hide />
                    <YAxis width={80} tickFormatter={(value) => `$${(value / 1000).toFixed(0)}k`} />
                    <Tooltip content={<CustomTooltip />} />
                    <Line 
                        type="monotone" 
                        dataKey="value" 
                        dot={false} 
                        stroke={theme.palette.primary.main}
                        strokeWidth={2}
                    />
                    <Line 
                        type="monotone" 
                        dataKey="sma20" 
                        dot={false} 
                        stroke={theme.palette.secondary.main}
                        strokeWidth={1}
                    />
                    <Line 
                        type="monotone" 
                        dataKey="ema20" 
                        dot={false} 
                        stroke={theme.palette.success.main}
                        strokeWidth={1}
                    />
                    <Line 
                        type="monotone" 
                        dataKey="trend" 
                        dot={false} 
                        stroke={theme.palette.error.main}
                        strokeWidth={1}
                        strokeDasharray="5 5"
                    />
                </LineChart>
            </ResponsiveContainer>
        </Paper>
    );
}