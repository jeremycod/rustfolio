import { Pie, PieChart, Tooltip, ResponsiveContainer } from "recharts";
import type { AllocationPoint } from "../types";

export function AllocationDonut({ allocations }: { allocations: AllocationPoint[] }) {
    const data = allocations.map((a) => ({ name: a.ticker, value: a.value }));

    return (
        <div style={{ width: "100%", height: 280 }}>
            <ResponsiveContainer>
                <PieChart>
                    <Pie data={data} dataKey="value" nameKey="name" innerRadius={60} outerRadius={100} />
                    <Tooltip />
                </PieChart>
            </ResponsiveContainer>
        </div>
    );
}