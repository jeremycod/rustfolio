import {
    ResponsiveContainer,
    LineChart,
    Line,
    XAxis,
    YAxis,
    Tooltip,
} from "recharts";
import type { ChartPoint } from "../types";

export function PortfolioChart({ series }: { series: ChartPoint[] }) {
    return (
        <div style={{ width: "100%", height: 360, minHeight: 360 }}>
            <ResponsiveContainer width="100%" height="100%">
                <LineChart data={series}>
                    <XAxis dataKey="date" hide />
                    <YAxis width={80} />
                    <Tooltip />
                    <Line type="monotone" dataKey="value" dot={false} />
                    <Line type="monotone" dataKey="sma20" dot={false} />
                    <Line type="monotone" dataKey="ema20" dot={false} />
                    <Line type="monotone" dataKey="trend" dot={false} />
                </LineChart>
            </ResponsiveContainer>
        </div>
    );
}