mod portfolio;
mod price_point;
mod analytics;
mod account;
mod holding_snapshot;
mod cash_flow;
mod detected_transaction;
pub mod risk;
pub mod risk_snapshot;
pub mod optimization;
pub mod llm;
pub mod narrative;
pub mod news;
pub mod qa;
pub mod forecast;
pub mod sentiment;
pub mod sec_filing;
pub mod alert;
pub mod market_regime;

pub use portfolio::Portfolio;
pub use portfolio::CreatePortfolio;
pub use portfolio::UpdatePortfolio;
pub use price_point::PricePoint;
pub use analytics::*;
pub use account::{Account, CreateAccount};
pub use holding_snapshot::{HoldingSnapshot, CreateHoldingSnapshot, LatestAccountHolding, AccountValueHistory};
pub use cash_flow::{CashFlow, CreateCashFlow, FlowType};
pub use detected_transaction::{DetectedTransaction, CreateDetectedTransaction, TransactionType, AccountActivity, AccountTruePerformance};
pub use risk::{
    PositionRisk, RiskAssessment, RiskLevel, PortfolioRisk, PositionRiskContribution,
    CorrelationPair, CorrelationMatrix,
};
pub use risk_snapshot::{RiskSnapshot, RiskAlert, RiskHistoryParams, AlertQueryParams};
pub use optimization::{
    OptimizationRecommendation, OptimizationAnalysis, PositionAdjustment, ExpectedImpact,
    RecommendationType, Severity, AdjustmentAction, CurrentMetrics, AnalysisSummary,
    PortfolioHealth, RiskContribution,
};
pub use llm::{
    LlmUsage, CreateLlmUsage, UserPreferences, UpdateUserPreferences, LlmUsageStats,
};
pub use narrative::{PortfolioNarrative, GenerateNarrativeRequest};
pub use news::{NewsArticle, Sentiment, NewsTheme, PortfolioNewsAnalysis, NewsQueryParams};
pub use qa::{PortfolioQuestion, PortfolioAnswer, Confidence};
pub use forecast::{PortfolioForecast, ForecastPoint, ForecastMethod, HistoricalDataPoint};
pub use sentiment::{
    SentimentTrend, MomentumTrend, DivergenceType, SentimentDataPoint,
    SentimentSignal, PortfolioSentimentAnalysis,
};
pub use sec_filing::{
    FilingType, SecFiling, EventImportance, MaterialEvent,
    InsiderTransactionType, InsiderTransaction, InsiderConfidence, InsiderSentiment,
    ConfidenceLevel, EnhancedSentimentSignal,
};
pub use market_regime::{
    MarketRegime, CreateMarketRegime, RegimeType, RegimeDetectionParams,
    RegimeHistoryParams, CurrentRegimeWithThresholds, AdjustedThresholds,
};
// Alert module models are used internally by routes/services
// Re-export only when needed by other modules
