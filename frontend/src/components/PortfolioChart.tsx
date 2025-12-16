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
    
    return (
        <Paper sx={{ p: 2, height: 400 }}>
            <ResponsiveContainer width="100%" height="100%">
                <LineChart data={series}>
                    <XAxis dataKey="date" hide />
                    <YAxis width={80} />
                    <Tooltip 
                        contentStyle={{
                            backgroundColor: theme.palette.background.paper,
                            border: `1px solid ${theme.palette.divider}`,
                            borderRadius: theme.shape.borderRadius,
                        }}
                    />
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