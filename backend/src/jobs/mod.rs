//! Background Jobs Module
//!
//! This module contains implementations of background jobs that are scheduled
//! and executed by the job scheduler service. These jobs perform periodic
//! maintenance tasks, data updates, and calculations that run independently
//! of user requests.
//!
//! # Available Jobs
//!
//! - `portfolio_risk_job` - Calculates and updates portfolio risk metrics cache
//! - `portfolio_correlations_job` - Computes asset correlation matrices
//! - `daily_risk_snapshots_job` - Creates historical risk snapshots for tracking
//! - `populate_sentiment_cache_job` - Pre-caches sentiment signals for portfolio tickers
//! - `populate_optimization_cache_job` - Pre-caches optimization recommendations
//!
//! # Job Architecture
//!
//! Jobs in this module are designed to be:
//! - Idempotent: Can be safely re-run without side effects
//! - Fault-tolerant: Handle errors gracefully and log failures
//! - Efficient: Minimize database queries and API calls
//! - Observable: Provide detailed logging for monitoring
//!
//! Each job is registered with the job scheduler and executed on a defined schedule.

pub mod portfolio_risk_job;
pub mod portfolio_correlations_job;
pub mod daily_risk_snapshots_job;
// pub mod populate_sentiment_cache_job; // TODO: Needs refactoring to match current architecture
pub mod populate_optimization_cache_job;
pub mod market_regime_update_job;
pub mod hmm_training_job;
pub mod regime_forecast_job;
pub mod rolling_beta_cache_job;
pub mod downside_risk_cache_job;
pub mod watchlist_monitoring_job;
