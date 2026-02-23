use chrono::{Duration, Utc};
use tracing::{info, warn};

use crate::errors::AppError;
use crate::models::{EnhancedSentimentSignal, ForecastPoint, HistoricalDataPoint, SentimentAwareForecast, SentimentFactors};
use crate::services::price_service;
use sqlx::PgPool;

/// Sentiment momentum over different time windows
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SentimentMomentum {
    pub seven_day_change: f64,
    pub thirty_day_change: f64,
    pub acceleration: f64, // Rate of change of momentum
}

/// Sentiment spike detection
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SentimentSpike {
    pub detected: bool,
    pub magnitude: f64, // How unusual the change is (z-score)
    pub direction: SpikeDirection,
}

#[derive(Debug, Clone)]
pub enum SpikeDirection {
    Positive,
    Negative,
    None,
}

/// Sentiment-price divergence detection
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SentimentDivergence {
    pub detected: bool,
    pub divergence_type: DivergenceType,
    pub divergence_score: f64, // 0.0 to 1.0
    pub reversal_probability: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone)]
pub enum DivergenceType {
    BullishDivergence, // Negative sentiment, positive price trend
    BearishDivergence, // Positive sentiment, negative price trend
    None,
}

/// Calculate sentiment momentum (rate of change)
///
/// This analyzes how quickly sentiment is changing over time.
/// Positive momentum = sentiment improving
/// Negative momentum = sentiment deteriorating
pub async fn calculate_sentiment_momentum(
    pool: &PgPool,
    ticker: &str,
) -> Result<SentimentMomentum, AppError> {
    info!("🔍 [SENTIMENT MOMENTUM] Calculating momentum for {}", ticker);

    // Fetch historical sentiment data
    let sentiment_history = fetch_sentiment_history(pool, ticker, 30).await?;

    if sentiment_history.len() < 7 {
        warn!("⚠️ [SENTIMENT MOMENTUM] Insufficient data for {}: {} points", ticker, sentiment_history.len());
        return Ok(SentimentMomentum {
            seven_day_change: 0.0,
            thirty_day_change: 0.0,
            acceleration: 0.0,
        });
    }

    // Calculate 7-day change
    let recent_avg = sentiment_history[..7.min(sentiment_history.len())]
        .iter()
        .map(|s| s.sentiment)
        .sum::<f64>()
        / 7.0_f64.min(sentiment_history.len() as f64);

    let week_ago_avg = if sentiment_history.len() >= 14 {
        sentiment_history[7..14]
            .iter()
            .map(|s| s.sentiment)
            .sum::<f64>()
            / 7.0
    } else {
        sentiment_history[sentiment_history.len() - 1].sentiment
    };

    let seven_day_change = recent_avg - week_ago_avg;

    // Calculate 30-day change
    let month_ago_avg = if sentiment_history.len() >= 30 {
        sentiment_history[..sentiment_history.len().min(30)]
            .iter()
            .map(|s| s.sentiment)
            .sum::<f64>()
            / sentiment_history.len().min(30) as f64
    } else {
        recent_avg
    };

    let thirty_day_change = recent_avg - month_ago_avg;

    // Acceleration = change in momentum
    let acceleration = if sentiment_history.len() >= 14 {
        seven_day_change - (week_ago_avg - month_ago_avg)
    } else {
        0.0
    };

    info!(
        "✅ [SENTIMENT MOMENTUM] {} - 7d: {:.3}, 30d: {:.3}, accel: {:.3}",
        ticker, seven_day_change, thirty_day_change, acceleration
    );

    Ok(SentimentMomentum {
        seven_day_change,
        thirty_day_change,
        acceleration,
    })
}

/// Detect unusual sentiment spikes
///
/// A spike is detected when sentiment changes significantly more than usual.
/// Uses z-score to determine if change is statistically significant.
pub async fn detect_sentiment_spike(
    pool: &PgPool,
    ticker: &str,
) -> Result<SentimentSpike, AppError> {
    info!("🔍 [SENTIMENT SPIKE] Detecting spikes for {}", ticker);

    let sentiment_history = fetch_sentiment_history(pool, ticker, 30).await?;

    if sentiment_history.len() < 5 {
        return Ok(SentimentSpike {
            detected: false,
            magnitude: 0.0,
            direction: SpikeDirection::None,
        });
    }

    // Calculate daily changes
    let mut changes = Vec::new();
    for i in 1..sentiment_history.len() {
        changes.push(sentiment_history[i - 1].sentiment - sentiment_history[i].sentiment);
    }

    // Calculate mean and std dev of changes
    let mean_change = changes.iter().sum::<f64>() / changes.len() as f64;
    let variance = changes
        .iter()
        .map(|c| (c - mean_change).powi(2))
        .sum::<f64>()
        / changes.len() as f64;
    let std_dev = variance.sqrt();

    // Check most recent change
    let latest_change = if changes.is_empty() { 0.0 } else { changes[0] };

    // Calculate z-score
    let z_score = if std_dev > 0.0 {
        (latest_change - mean_change) / std_dev
    } else {
        0.0
    };

    // Spike detected if |z-score| > 2.0 (2 standard deviations)
    let detected = z_score.abs() > 2.0;
    let direction = if detected {
        if z_score > 0.0 {
            SpikeDirection::Positive
        } else {
            SpikeDirection::Negative
        }
    } else {
        SpikeDirection::None
    };

    info!(
        "✅ [SENTIMENT SPIKE] {} - detected: {}, z-score: {:.2}, direction: {:?}",
        ticker, detected, z_score, direction
    );

    Ok(SentimentSpike {
        detected,
        magnitude: z_score.abs(),
        direction,
    })
}

/// Detect sentiment-price divergence
///
/// Divergences often precede price reversals:
/// - Bullish divergence: Negative sentiment but price rising (opportunity)
/// - Bearish divergence: Positive sentiment but price falling (warning)
pub async fn detect_sentiment_price_divergence(
    pool: &PgPool,
    ticker: &str,
    sentiment: f64,
) -> Result<SentimentDivergence, AppError> {
    info!("🔍 [DIVERGENCE] Detecting divergence for {} (sentiment: {:.2})", ticker, sentiment);

    // Fetch recent price history
    let prices = price_service::get_history(pool, ticker).await?;

    if prices.len() < 30 {
        warn!("⚠️ [DIVERGENCE] Insufficient price data for {}: {} points", ticker, prices.len());
        return Ok(SentimentDivergence {
            detected: false,
            divergence_type: DivergenceType::None,
            divergence_score: 0.0,
            reversal_probability: 0.0,
        });
    }

    // Calculate price trend (30-day)
    let recent_prices: Vec<f64> = prices[..30.min(prices.len())]
        .iter()
        .map(|p| p.close_price.to_string().parse::<f64>().unwrap_or(0.0))
        .collect();

    let price_change = if !recent_prices.is_empty() {
        (recent_prices[0] - recent_prices[recent_prices.len() - 1]) / recent_prices[recent_prices.len() - 1]
    } else {
        0.0
    };

    info!("📊 [DIVERGENCE] {} price change (30d): {:.2}%", ticker, price_change * 100.0);

    // Detect divergence
    let threshold = 0.3; // Sentiment and price must be > 0.3 in opposite directions

    let (detected, divergence_type, divergence_score) = if sentiment < -threshold && price_change > threshold {
        // Bullish divergence: negative sentiment, positive price
        let score = (sentiment.abs() + price_change) / 2.0;
        (true, DivergenceType::BullishDivergence, score.min(1.0))
    } else if sentiment > threshold && price_change < -threshold {
        // Bearish divergence: positive sentiment, negative price
        let score = (sentiment + price_change.abs()) / 2.0;
        (true, DivergenceType::BearishDivergence, score.min(1.0))
    } else {
        (false, DivergenceType::None, 0.0)
    };

    // Calculate reversal probability based on divergence strength
    // Historical data suggests 40-60% probability of reversal with strong divergence
    let reversal_probability = if detected {
        0.4 + (divergence_score * 0.3) // 40-70% based on strength
    } else {
        0.0
    };

    info!(
        "✅ [DIVERGENCE] {} - detected: {}, type: {:?}, score: {:.2}, reversal prob: {:.2}",
        ticker, detected, divergence_type, divergence_score, reversal_probability
    );

    Ok(SentimentDivergence {
        detected,
        divergence_type,
        divergence_score,
        reversal_probability,
    })
}

/// Apply sentiment adjustments to forecast confidence intervals
///
/// Strategy:
/// - High positive sentiment: widen upper bound, narrow lower bound
/// - High negative sentiment: widen lower bound, narrow upper bound
/// - Divergence detected: flag potential reversal, wider confidence bands
pub fn apply_sentiment_adjustments(
    base_forecast: &[ForecastPoint],
    sentiment: f64,
    momentum: &SentimentMomentum,
    spike: &SentimentSpike,
    divergence: &SentimentDivergence,
) -> Vec<ForecastPoint> {
    let mut adjusted_forecast = Vec::new();

    // Sentiment weight: 20-30% influence on confidence intervals
    let sentiment_weight = 0.25;

    for (i, point) in base_forecast.iter().enumerate() {
        let days_ahead = i + 1;

        // Time decay: sentiment effect reduces over longer horizons
        let time_decay = 1.0 / (1.0 + (days_ahead as f64 / 30.0));
        let effective_sentiment = sentiment * time_decay;

        // Momentum amplifies or dampens sentiment effect
        let momentum_factor = 1.0 + (momentum.seven_day_change * 0.5);
        let adjusted_sentiment = effective_sentiment * momentum_factor;

        // Calculate adjustment factors
        let upper_adjustment = if adjusted_sentiment > 0.0 {
            // Positive sentiment: widen upper bound
            1.0 + (adjusted_sentiment * sentiment_weight * 1.5)
        } else {
            // Negative sentiment: narrow upper bound
            1.0 + (adjusted_sentiment * sentiment_weight * 0.5)
        };

        let lower_adjustment = if adjusted_sentiment < 0.0 {
            // Negative sentiment: widen lower bound (more downside risk)
            1.0 + (adjusted_sentiment.abs() * sentiment_weight * 1.5)
        } else {
            // Positive sentiment: narrow lower bound
            1.0 - (adjusted_sentiment * sentiment_weight * 0.5)
        };

        // Apply spike adjustment (wider confidence bands)
        let spike_factor = if spike.detected {
            1.0 + (spike.magnitude * 0.1)
        } else {
            1.0
        };

        // Apply divergence adjustment (much wider confidence bands due to uncertainty)
        let divergence_factor = if divergence.detected {
            1.0 + (divergence.divergence_score * 0.3)
        } else {
            1.0
        };

        // Calculate adjusted bounds
        let predicted_value = point.predicted_value;
        let base_upper_range = point.upper_bound - predicted_value;
        let base_lower_range = predicted_value - point.lower_bound;

        let adjusted_upper = predicted_value
            + (base_upper_range * upper_adjustment * spike_factor * divergence_factor);
        let adjusted_lower = predicted_value
            - (base_lower_range * lower_adjustment * spike_factor * divergence_factor);

        adjusted_forecast.push(ForecastPoint {
            date: point.date.clone(),
            predicted_value,
            lower_bound: adjusted_lower.max(0.0),
            upper_bound: adjusted_upper,
            confidence_level: point.confidence_level,
        });
    }

    adjusted_forecast
}

/// Generate sentiment-aware forecast for a stock
pub async fn generate_sentiment_aware_stock_forecast(
    pool: &PgPool,
    ticker: &str,
    enhanced_sentiment: &EnhancedSentimentSignal,
    days_ahead: i32,
) -> Result<SentimentAwareForecast, AppError> {
    info!(
        "🚀 [SENTIMENT FORECAST] Generating sentiment-aware forecast for {} ({} days)",
        ticker, days_ahead
    );

    // 1. Calculate sentiment features
    let momentum = calculate_sentiment_momentum(pool, ticker).await?;
    let spike = detect_sentiment_spike(pool, ticker).await?;
    let divergence = detect_sentiment_price_divergence(
        pool,
        ticker,
        enhanced_sentiment.combined_sentiment,
    )
    .await?;

    // 2. Generate base technical forecast
    let prices = price_service::get_history(pool, ticker).await?;
    let historical_data: Vec<HistoricalDataPoint> = prices
        .iter()
        .map(|p| HistoricalDataPoint {
            date: p.date.to_string(),
            value: p.close_price.to_string().parse::<f64>().unwrap_or(0.0),
        })
        .collect();

    if historical_data.len() < 30 {
        return Err(AppError::Validation(format!(
            "Insufficient historical data for {}: need at least 30 points, got {}",
            ticker,
            historical_data.len()
        )));
    }

    // Use simple linear regression for base forecast
    let base_forecast = generate_simple_price_forecast(&historical_data, days_ahead)?;

    // 3. Apply sentiment adjustments
    let sentiment_adjusted_forecast = apply_sentiment_adjustments(
        &base_forecast,
        enhanced_sentiment.combined_sentiment,
        &momentum,
        &spike,
        &divergence,
    );

    // 4. Create sentiment factors summary
    let sentiment_factors = SentimentFactors {
        news_sentiment: enhanced_sentiment.news_sentiment,
        sec_filing_sentiment: enhanced_sentiment.sec_filing_score,
        insider_sentiment: enhanced_sentiment.insider_sentiment.sentiment_score,
        combined_sentiment: enhanced_sentiment.combined_sentiment,
        sentiment_momentum: momentum.seven_day_change,
        spike_detected: spike.detected,
        divergence_detected: divergence.detected,
    };

    // 5. Calculate confidence adjustment
    let confidence_adjustment = calculate_confidence_adjustment(
        enhanced_sentiment.combined_sentiment,
        &momentum,
        &spike,
        &divergence,
    );

    info!(
        "✅ [SENTIMENT FORECAST] Completed for {}: sentiment={:.2}, momentum={:.3}, divergence={}, reversal_prob={:.2}",
        ticker,
        enhanced_sentiment.combined_sentiment,
        momentum.seven_day_change,
        divergence.detected,
        divergence.reversal_probability
    );

    Ok(SentimentAwareForecast {
        ticker: ticker.to_string(),
        base_forecast,
        sentiment_adjusted_forecast,
        sentiment_factors,
        divergence_flags: enhanced_sentiment.divergence_flags.clone(),
        reversal_probability: divergence.reversal_probability,
        confidence_adjustment,
        methodology: "Sentiment-Augmented Linear Regression".to_string(),
        generated_at: Utc::now(),
    })
}

// Helper functions

/// Fetch sentiment history from cache
async fn fetch_sentiment_history(
    pool: &PgPool,
    ticker: &str,
    days: i32,
) -> Result<Vec<SentimentHistoryPoint>, AppError> {
    let cutoff_date = (Utc::now() - Duration::days(days as i64)).naive_utc();

    let rows = sqlx::query!(
        r#"
        SELECT calculated_at, combined_sentiment
        FROM enhanced_sentiment_cache
        WHERE ticker = $1
          AND calculated_at >= $2
        ORDER BY calculated_at DESC
        "#,
        ticker,
        cutoff_date
    )
    .fetch_all(pool)
    .await
    .map_err(AppError::Db)?;

    Ok(rows
        .into_iter()
        .map(|r| SentimentHistoryPoint {
            date: r.calculated_at,
            sentiment: r.combined_sentiment,
        })
        .collect())
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
struct SentimentHistoryPoint {
    date: chrono::NaiveDateTime,
    sentiment: f64,
}

/// Simple linear regression forecast for stock prices
fn generate_simple_price_forecast(
    historical_data: &[HistoricalDataPoint],
    days_ahead: i32,
) -> Result<Vec<ForecastPoint>, AppError> {
    let n = historical_data.len() as f64;
    let values: Vec<f64> = historical_data.iter().map(|p| p.value).collect();

    // Calculate linear regression: y = mx + b
    let x_mean = (n - 1.0) / 2.0;
    let y_mean = values.iter().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut denominator = 0.0;

    for (i, &y) in values.iter().enumerate() {
        let x = i as f64;
        numerator += (x - x_mean) * (y - y_mean);
        denominator += (x - x_mean) * (x - x_mean);
    }

    let slope = numerator / denominator;
    let intercept = y_mean - slope * x_mean;

    // Calculate residual standard error
    let mut sum_squared_residuals = 0.0;
    for (i, &y) in values.iter().enumerate() {
        let x = i as f64;
        let predicted = slope * x + intercept;
        sum_squared_residuals += (y - predicted).powi(2);
    }
    let std_error = (sum_squared_residuals / (n - 2.0)).sqrt();

    // Generate forecast points
    let last_date = chrono::NaiveDate::parse_from_str(
        &historical_data.last().unwrap().date,
        "%Y-%m-%d",
    )
    .map_err(|e| AppError::Validation(format!("Invalid date format: {}", e)))?;

    let mut forecast_points = Vec::new();

    for day in 1..=days_ahead {
        let x = n + day as f64 - 1.0;
        let predicted_value = slope * x + intercept;

        // Confidence intervals widen with forecast horizon
        let confidence_factor = 1.96 * std_error * (1.0 + (day as f64 / days_ahead as f64));

        let forecast_date = last_date + Duration::days(day as i64);

        forecast_points.push(ForecastPoint {
            date: forecast_date.to_string(),
            predicted_value: predicted_value.max(0.0),
            lower_bound: (predicted_value - confidence_factor).max(0.0),
            upper_bound: predicted_value + confidence_factor,
            confidence_level: 0.95,
        });
    }

    Ok(forecast_points)
}

/// Calculate overall confidence adjustment based on sentiment factors
fn calculate_confidence_adjustment(
    sentiment: f64,
    momentum: &SentimentMomentum,
    spike: &SentimentSpike,
    divergence: &SentimentDivergence,
) -> f64 {
    let mut adjustment = 1.0;

    // Strong sentiment increases confidence
    adjustment += sentiment.abs() * 0.1;

    // Consistent momentum increases confidence
    if momentum.seven_day_change.signum() == momentum.thirty_day_change.signum() {
        adjustment += 0.05;
    }

    // Spikes reduce confidence (uncertainty)
    if spike.detected {
        adjustment -= spike.magnitude * 0.05;
    }

    // Divergence reduces confidence significantly
    if divergence.detected {
        adjustment -= divergence.divergence_score * 0.2;
    }

    adjustment.max(0.5).min(1.5) // Cap between 0.5x and 1.5x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_sentiment_adjustments_positive_sentiment() {
        let base_forecast = vec![
            ForecastPoint {
                date: "2026-03-01".to_string(),
                predicted_value: 100.0,
                lower_bound: 90.0,
                upper_bound: 110.0,
                confidence_level: 0.95,
            },
            ForecastPoint {
                date: "2026-03-08".to_string(),
                predicted_value: 105.0,
                lower_bound: 93.0,
                upper_bound: 117.0,
                confidence_level: 0.95,
            },
        ];

        let momentum = SentimentMomentum {
            seven_day_change: 0.1,
            thirty_day_change: 0.15,
            acceleration: 0.05,
        };

        let spike = SentimentSpike {
            detected: false,
            magnitude: 0.0,
            direction: SpikeDirection::None,
        };

        let divergence = SentimentDivergence {
            detected: false,
            divergence_type: DivergenceType::None,
            divergence_score: 0.0,
            reversal_probability: 0.0,
        };

        let adjusted = apply_sentiment_adjustments(
            &base_forecast,
            0.5, // Strong positive sentiment
            &momentum,
            &spike,
            &divergence,
        );

        // With positive sentiment, upper bound should be wider
        assert!(adjusted[0].upper_bound > base_forecast[0].upper_bound);
        // Lower bound should be narrower (less downside risk)
        assert!(adjusted[0].lower_bound > base_forecast[0].lower_bound);
        // Predicted value should remain the same
        assert_eq!(adjusted[0].predicted_value, base_forecast[0].predicted_value);
    }

    #[test]
    fn test_apply_sentiment_adjustments_negative_sentiment() {
        let base_forecast = vec![
            ForecastPoint {
                date: "2026-03-01".to_string(),
                predicted_value: 100.0,
                lower_bound: 90.0,
                upper_bound: 110.0,
                confidence_level: 0.95,
            },
        ];

        let momentum = SentimentMomentum {
            seven_day_change: -0.1,
            thirty_day_change: -0.15,
            acceleration: -0.05,
        };

        let spike = SentimentSpike {
            detected: false,
            magnitude: 0.0,
            direction: SpikeDirection::None,
        };

        let divergence = SentimentDivergence {
            detected: false,
            divergence_type: DivergenceType::None,
            divergence_score: 0.0,
            reversal_probability: 0.0,
        };

        let adjusted = apply_sentiment_adjustments(
            &base_forecast,
            -0.5, // Strong negative sentiment
            &momentum,
            &spike,
            &divergence,
        );

        // With negative sentiment, lower bound should be wider (more downside risk)
        assert!(adjusted[0].lower_bound < base_forecast[0].lower_bound);
        // Upper bound should be narrower
        assert!(adjusted[0].upper_bound < base_forecast[0].upper_bound);
    }

    #[test]
    fn test_apply_sentiment_adjustments_with_spike() {
        let base_forecast = vec![
            ForecastPoint {
                date: "2026-03-01".to_string(),
                predicted_value: 100.0,
                lower_bound: 90.0,
                upper_bound: 110.0,
                confidence_level: 0.95,
            },
        ];

        let momentum = SentimentMomentum {
            seven_day_change: 0.0,
            thirty_day_change: 0.0,
            acceleration: 0.0,
        };

        let spike = SentimentSpike {
            detected: true,
            magnitude: 2.5, // Strong spike
            direction: SpikeDirection::Positive,
        };

        let divergence = SentimentDivergence {
            detected: false,
            divergence_type: DivergenceType::None,
            divergence_score: 0.0,
            reversal_probability: 0.0,
        };

        let adjusted = apply_sentiment_adjustments(
            &base_forecast,
            0.0, // Neutral sentiment
            &momentum,
            &spike,
            &divergence,
        );

        // Spike increases uncertainty, so both bounds should be wider
        let base_range = base_forecast[0].upper_bound - base_forecast[0].lower_bound;
        let adjusted_range = adjusted[0].upper_bound - adjusted[0].lower_bound;
        assert!(adjusted_range > base_range);
    }

    #[test]
    fn test_apply_sentiment_adjustments_with_divergence() {
        let base_forecast = vec![
            ForecastPoint {
                date: "2026-03-01".to_string(),
                predicted_value: 100.0,
                lower_bound: 90.0,
                upper_bound: 110.0,
                confidence_level: 0.95,
            },
        ];

        let momentum = SentimentMomentum {
            seven_day_change: 0.0,
            thirty_day_change: 0.0,
            acceleration: 0.0,
        };

        let spike = SentimentSpike {
            detected: false,
            magnitude: 0.0,
            direction: SpikeDirection::None,
        };

        let divergence = SentimentDivergence {
            detected: true,
            divergence_type: DivergenceType::BullishDivergence,
            divergence_score: 0.7, // Strong divergence
            reversal_probability: 0.6,
        };

        let adjusted = apply_sentiment_adjustments(
            &base_forecast,
            -0.4, // Negative sentiment
            &momentum,
            &spike,
            &divergence,
        );

        // Divergence significantly increases uncertainty
        let base_range = base_forecast[0].upper_bound - base_forecast[0].lower_bound;
        let adjusted_range = adjusted[0].upper_bound - adjusted[0].lower_bound;
        assert!(adjusted_range > base_range * 1.15); // At least 15% wider
    }

    #[test]
    fn test_calculate_confidence_adjustment() {
        // Test 1: Strong sentiment increases confidence
        let momentum = SentimentMomentum {
            seven_day_change: 0.1,
            thirty_day_change: 0.15,
            acceleration: 0.05,
        };
        let spike = SentimentSpike {
            detected: false,
            magnitude: 0.0,
            direction: SpikeDirection::None,
        };
        let divergence = SentimentDivergence {
            detected: false,
            divergence_type: DivergenceType::None,
            divergence_score: 0.0,
            reversal_probability: 0.0,
        };

        let adjustment = calculate_confidence_adjustment(0.7, &momentum, &spike, &divergence);
        assert!(adjustment > 1.0); // Confidence increased

        // Test 2: Spike reduces confidence
        let spike_detected = SentimentSpike {
            detected: true,
            magnitude: 2.5,
            direction: SpikeDirection::Positive,
        };
        let adjustment = calculate_confidence_adjustment(0.0, &momentum, &spike_detected, &divergence);
        assert!(adjustment < 1.0); // Confidence reduced

        // Test 3: Divergence significantly reduces confidence
        let divergence_detected = SentimentDivergence {
            detected: true,
            divergence_type: DivergenceType::BearishDivergence,
            divergence_score: 0.8,
            reversal_probability: 0.65,
        };
        let adjustment = calculate_confidence_adjustment(0.0, &momentum, &spike, &divergence_detected);
        assert!(adjustment < 0.85); // Significant reduction

        // Test 4: Adjustment always within bounds
        let adjustment = calculate_confidence_adjustment(1.0, &momentum, &spike_detected, &divergence_detected);
        assert!(adjustment >= 0.5 && adjustment <= 1.5); // Within bounds
    }

    #[test]
    fn test_simple_price_forecast_generation() {
        let historical_data = vec![
            HistoricalDataPoint {
                date: "2026-01-01".to_string(),
                value: 100.0,
            },
            HistoricalDataPoint {
                date: "2026-01-08".to_string(),
                value: 102.0,
            },
            HistoricalDataPoint {
                date: "2026-01-15".to_string(),
                value: 104.0,
            },
            HistoricalDataPoint {
                date: "2026-01-22".to_string(),
                value: 106.0,
            },
            HistoricalDataPoint {
                date: "2026-01-29".to_string(),
                value: 108.0,
            },
        ];

        let forecast = generate_simple_price_forecast(&historical_data, 7).unwrap();

        assert_eq!(forecast.len(), 7);

        // Verify forecast is increasing (since historical trend is positive)
        assert!(forecast[6].predicted_value > forecast[0].predicted_value);

        // Verify confidence intervals widen with horizon
        let first_range = forecast[0].upper_bound - forecast[0].lower_bound;
        let last_range = forecast[6].upper_bound - forecast[6].lower_bound;
        assert!(last_range > first_range);

        // Verify all values are non-negative
        for point in &forecast {
            assert!(point.predicted_value >= 0.0);
            assert!(point.lower_bound >= 0.0);
            assert!(point.upper_bound >= 0.0);
        }
    }

    #[test]
    fn test_forecast_insufficient_data() {
        let historical_data = vec![
            HistoricalDataPoint {
                date: "2026-01-01".to_string(),
                value: 100.0,
            },
            HistoricalDataPoint {
                date: "2026-01-08".to_string(),
                value: 102.0,
            },
        ];

        // Should succeed with 2 points (minimum for linear regression)
        let result = generate_simple_price_forecast(&historical_data, 7);
        assert!(result.is_ok());
    }
}
