import { api } from "./api";
import type { AnalyticsResponse, Portfolio, Position, PricePoint, TickerMatch } from "../types";

export async function listPortfolios(): Promise<Portfolio[]> {
    const res = await api.get("/api/portfolios");
    return res.data;
}

export async function createPortfolio(name: string): Promise<Portfolio> {
    const res = await api.post("/api/portfolios", { name });
    return res.data;
}

export async function listPositions(portfolioId: string): Promise<Position[]> {
    const res = await api.get(`/api/portfolios/${portfolioId}/positions`);
    return res.data;
}

export async function createPosition(portfolioId: string, payload: {
    ticker: string;
    shares: number;
    avg_buy_price: number;
}): Promise<Position> {
    const res = await api.post(`/api/portfolios/${portfolioId}/positions`, payload);
    return res.data;
}

export async function getAnalytics(portfolioId: string): Promise<AnalyticsResponse> {
    const res = await api.get(`/api/analytics/${portfolioId}`);
    return res.data;
}

export async function updatePrices(ticker: string): Promise<void> {
    await api.post(`/api/prices/${ticker}/update`);
}

export async function getLatestPrice(ticker: string): Promise<PricePoint> {
    const res = await api.get(`/api/prices/${ticker}/latest`);
    return res.data;
}

export async function deletePosition(positionId: string): Promise<void> {
    await api.delete(`/api/positions/${positionId}`);
}

export async function updatePosition(positionId: string, payload: {
    shares: number;
    avg_buy_price: number;
}): Promise<Position> {
    const res = await api.put(`/api/positions/${positionId}`, payload);
    return res.data;
}

export async function searchTickers(keyword: string): Promise<TickerMatch[]> {
    const res = await api.get(`/api/prices/search/${encodeURIComponent(keyword)}`);
    return res.data;
}