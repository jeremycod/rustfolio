use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartPoint {
    pub date: NaiveDate,
    pub value: f64,
    pub sma20: Option<f64>,
    pub ema20: Option<f64>,
    pub trend: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationPoint {
    pub ticker: String,
    pub value: f64,
    pub weight: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsMeta {
    pub points: usize,
    pub start: Option<NaiveDate>,
    pub end: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsResponse {
    pub series: Vec<ChartPoint>,
    pub allocations: Vec<AllocationPoint>,
    pub meta: AnalyticsMeta,
}