use crate::models::{
    SentimentTrend, MomentumTrend, DivergenceType, SentimentDataPoint,
    SentimentSignal, NewsTheme, PricePoint, Sentiment,
};
use crate::errors::AppError;
use chrono::{Utc, Duration, NaiveDate};
use sqlx::PgPool;
use std::collections::HashMap;

/// Calculate sentiment score from news themes
/// Returns a score from -1.0 (very negative) to +1.0 (very positive)
pub fn calculate_sentiment_score(themes: &[NewsTheme]) -> f64 {
    if themes.is_empty() {
        return 0.0;
    }

    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for theme in themes {
        // Convert Sentiment enum to numeric value
        let sentiment_value = match theme.sentiment {
            Sentiment::Positive => 0.7,
            Sentiment::Negative => -0.7,
            Sentiment::Neutral => 0.0,
        };

        let weight = theme.relevance_score;
        weighted_sum += sentiment_value * weight;
        total_weight += weight;
    }

    if total_weight == 0.0 {
        return 0.0;
    }

    // Normalize to -1.0 to +1.0 range
    (weighted_sum / total_weight).clamp(-1.0, 1.0)
}

/// Determine sentiment trend by comparing recent vs previous periods
pub fn determine_sentiment_trend(historical: &[SentimentDataPoint]) -> SentimentTrend {
    if historical.len() < 10 {
        return SentimentTrend::Stable;
    }

    // Compare recent 3 days vs previous 7 days
    let recent_days = 3;
    let comparison_days = 7;

    if historical.len() < recent_days + comparison_days {
        return SentimentTrend::Stable;
    }

    // Calculate average sentiment for recent period
    let recent_avg: f64 = historical
        .iter()
        .rev()
        .take(recent_days)
        .map(|p| p.sentiment_score)
        .sum::<f64>()
        / recent_days as f64;

    // Calculate average sentiment for comparison period
    let comparison_avg: f64 = historical
        .iter()
        .rev()
        .skip(recent_days)
        .take(comparison_days)
        .map(|p| p.sentiment_score)
        .sum::<f64>()
        / comparison_days as f64;

    let change = recent_avg - comparison_avg;

    // Threshold for significant change: 0.15 (15% of scale)
    if change > 0.15 {
        SentimentTrend::Improving
    } else if change < -0.15 {
        SentimentTrend::Deteriorating
    } else {
        SentimentTrend::Stable
    }
}

/// Determine momentum trend using simple moving average crossover
pub fn determine_momentum_trend(prices: &[f64]) -> MomentumTrend {
    if prices.len() < 20 {
        return MomentumTrend::Neutral;
    }

    // Calculate 5-day and 20-day simple moving averages
    let sma_5: f64 = prices.iter().rev().take(5).sum::<f64>() / 5.0;
    let sma_20: f64 = prices.iter().rev().take(20).sum::<f64>() / 20.0;

    let diff_pct = (sma_5 - sma_20) / sma_20 * 100.0;

    // Threshold: 2% difference
    if diff_pct > 2.0 {
        MomentumTrend::Bullish
    } else if diff_pct < -2.0 {
        MomentumTrend::Bearish
    } else {
        MomentumTrend::Neutral
    }
}

/// Detect divergence between sentiment and price trends
pub fn detect_divergence(
    sentiment_trend: SentimentTrend,
    momentum_trend: MomentumTrend,
) -> DivergenceType {
    match (sentiment_trend, momentum_trend) {
        // Bullish divergence: sentiment improving while price down
        (SentimentTrend::Improving, MomentumTrend::Bearish) => DivergenceType::Bullish,

        // Bearish divergence: sentiment deteriorating while price up
        (SentimentTrend::Deteriorating, MomentumTrend::Bullish) => DivergenceType::Bearish,

        // Confirmed: both aligned
        (SentimentTrend::Improving, MomentumTrend::Bullish) => DivergenceType::Confirmed,
        (SentimentTrend::Deteriorating, MomentumTrend::Bearish) => DivergenceType::Confirmed,

        // No clear divergence
        _ => DivergenceType::None,
    }
}

/// Calculate Pearson correlation between sentiment and price changes
/// Returns (correlation_coefficient, optimal_lag_days)
pub fn calculate_sentiment_price_correlation(
    historical: &[SentimentDataPoint],
) -> (Option<f64>, Option<i32>) {
    if historical.len() < 10 {
        return (None, None);
    }

    let mut best_correlation: f64 = 0.0;
    let mut best_lag = 0;

    // Test different lags (0-7 days)
    for lag in 0..=7 {
        if historical.len() <= lag {
            continue;
        }

        let mut sentiment_values = Vec::new();
        let mut price_changes = Vec::new();

        // Build aligned vectors with lag
        for i in 0..(historical.len() - lag) {
            let price_index = i + lag;

            // Need previous price to calculate change - skip if at first index
            if price_index == 0 {
                continue;
            }

            if let Some(price_current) = historical[price_index].price {
                if let Some(price_prev) = historical.get(price_index - 1).and_then(|p| p.price) {
                    sentiment_values.push(historical[i].sentiment_score);
                    price_changes.push((price_current - price_prev) / price_prev);
                }
            }
        }

        if sentiment_values.len() < 5 {
            continue;
        }

        // Calculate Pearson correlation
        let corr = pearson_correlation(&sentiment_values, &price_changes);

        if corr.abs() > best_correlation.abs() {
            best_correlation = corr;
            best_lag = lag as i32;
        }
    }

    if best_correlation.abs() < 0.01 {
        return (None, None);
    }

    (Some(best_correlation), Some(best_lag))
}

/// Calculate Pearson correlation coefficient
fn pearson_correlation(x: &[f64], y: &[f64]) -> f64 {
    if x.len() != y.len() || x.is_empty() {
        return 0.0;
    }

    let n = x.len() as f64;
    let mean_x: f64 = x.iter().sum::<f64>() / n;
    let mean_y: f64 = y.iter().sum::<f64>() / n;

    let mut numerator = 0.0;
    let mut sum_sq_x = 0.0;
    let mut sum_sq_y = 0.0;

    for i in 0..x.len() {
        let diff_x = x[i] - mean_x;
        let diff_y = y[i] - mean_y;
        numerator += diff_x * diff_y;
        sum_sq_x += diff_x * diff_x;
        sum_sq_y += diff_y * diff_y;
    }

    if sum_sq_x == 0.0 || sum_sq_y == 0.0 {
        return 0.0;
    }

    numerator / (sum_sq_x.sqrt() * sum_sq_y.sqrt())
}

/// Classify correlation strength
fn classify_correlation_strength(correlation: f64) -> String {
    let abs_corr = correlation.abs();
    if abs_corr >= 0.7 {
        "strong".to_string()
    } else if abs_corr >= 0.4 {
        "moderate".to_string()
    } else {
        "weak".to_string()
    }
}

/// Generate warnings based on signal quality
fn generate_warnings(
    articles_count: i32,
    correlation: Option<f64>,
    historical_len: usize,
    sentiment_volatility: f64,
) -> Vec<String> {
    let mut warnings = Vec::new();

    if articles_count < 5 {
        warnings.push("Low confidence: Limited news data available".to_string());
    }

    if let Some(corr) = correlation {
        if corr.abs() < 0.3 {
            warnings.push("Weak correlation: Sentiment may not predict price movements".to_string());
        }
    }

    if historical_len < 10 {
        warnings.push("Insufficient historical data for reliable analysis".to_string());
    }

    if sentiment_volatility > 0.4 {
        warnings.push("High sentiment volatility: Frequent sentiment swings detected".to_string());
    }

    warnings
}

/// Generate sentiment signal for a ticker from provided themes and prices
/// This is the main function called by API endpoints
pub async fn generate_sentiment_signal(
    pool: &PgPool,
    ticker: &str,
    themes: Vec<NewsTheme>,
    prices: Vec<PricePoint>,
) -> Result<SentimentSignal, AppError> {
    // Check cache first
    if let Some(cached) = get_sentiment_from_cache(pool, ticker).await? {
        return Ok(cached);
    }

    if themes.is_empty() {
        return Err(AppError::Validation(
            format!("No news data available for {}", ticker)
        ));
    }

    // Build historical sentiment timeline
    let days = 30; // Default to 30 days
    let historical_sentiment = build_sentiment_timeline(&themes, &prices, days);

    // Calculate current sentiment
    let current_sentiment = calculate_sentiment_score(&themes);

    // Determine trends
    let sentiment_trend = determine_sentiment_trend(&historical_sentiment);

    let price_values: Vec<f64> = prices.iter()
        .filter_map(|p| {
            use bigdecimal::ToPrimitive;
            p.close_price.to_f64()
        })
        .collect();
    let momentum_trend = determine_momentum_trend(&price_values);

    // Detect divergence
    let divergence = detect_divergence(sentiment_trend, momentum_trend);

    // Calculate correlation
    let (correlation, lag) = calculate_sentiment_price_correlation(&historical_sentiment);
    let correlation_strength = correlation.map(classify_correlation_strength);

    // Calculate sentiment volatility
    let sentiment_volatility = if historical_sentiment.len() > 1 {
        let mean: f64 = historical_sentiment.iter()
            .map(|p| p.sentiment_score)
            .sum::<f64>() / historical_sentiment.len() as f64;

        let variance: f64 = historical_sentiment.iter()
            .map(|p| (p.sentiment_score - mean).powi(2))
            .sum::<f64>() / historical_sentiment.len() as f64;

        variance.sqrt()
    } else {
        0.0
    };

    // Generate warnings
    let warnings = generate_warnings(
        themes.len() as i32,
        correlation,
        historical_sentiment.len(),
        sentiment_volatility,
    );

    let signal = SentimentSignal {
        ticker: ticker.to_string(),
        current_sentiment,
        sentiment_trend,
        momentum_trend,
        divergence,
        sentiment_price_correlation: correlation,
        correlation_lag_days: lag,
        correlation_strength,
        historical_sentiment,
        news_articles_analyzed: themes.len() as i32,
        calculated_at: Utc::now(),
        warnings,
    };

    // Cache the result
    save_sentiment_to_cache(pool, &signal).await?;

    Ok(signal)
}

/// Build sentiment timeline by aggregating news by date
/// Note: Since NewsTheme doesn't have a date field, we aggregate all themes as a single point
fn build_sentiment_timeline(
    themes: &[NewsTheme],
    prices: &[PricePoint],
    days: i32,
) -> Vec<SentimentDataPoint> {
    use bigdecimal::ToPrimitive;

    // Build price map
    let price_map: HashMap<NaiveDate, f64> = prices
        .iter()
        .filter_map(|p| p.close_price.to_f64().map(|price| (p.date, price)))
        .collect();

    // Generate timeline with sentiment score applied uniformly
    // (This is a simplification since themes don't have individual dates)
    let end_date = Utc::now().date_naive();
    let start_date = end_date - Duration::days(days as i64);

    let mut timeline = Vec::new();
    let overall_sentiment = calculate_sentiment_score(themes);
    let mut current_date = start_date;

    while current_date <= end_date {
        if let Some(price) = price_map.get(&current_date).copied() {
            timeline.push(SentimentDataPoint {
                date: current_date.format("%Y-%m-%d").to_string(),
                sentiment_score: overall_sentiment,
                news_volume: themes.len() as i32,
                price: Some(price),
            });
        }

        current_date += Duration::days(1);
    }

    timeline
}

/// Get sentiment signal from cache
async fn get_sentiment_from_cache(
    pool: &PgPool,
    ticker: &str,
) -> Result<Option<SentimentSignal>, AppError> {
    let result = sqlx::query!(
        r#"
        SELECT
            ticker,
            calculated_at,
            current_sentiment,
            sentiment_trend,
            momentum_trend,
            divergence,
            sentiment_price_correlation,
            correlation_lag_days,
            historical_sentiment,
            news_articles_analyzed,
            warnings
        FROM sentiment_signal_cache
        WHERE ticker = $1
          AND expires_at > NOW()
        "#,
        ticker
    )
    .fetch_optional(pool)
    .await?;

    if let Some(row) = result {
        let historical_sentiment: Vec<SentimentDataPoint> = serde_json::from_value(
            row.historical_sentiment.clone()
        ).map_err(|e| AppError::Validation(format!("Failed to parse historical_sentiment: {}", e)))?;

        let warnings: Vec<String> = serde_json::from_value(
            row.warnings.clone()
        ).map_err(|e| AppError::Validation(format!("Failed to parse warnings: {}", e)))?;

        let correlation_strength = row.sentiment_price_correlation
            .map(classify_correlation_strength);

        let signal = SentimentSignal {
            ticker: row.ticker,
            current_sentiment: row.current_sentiment,
            sentiment_trend: serde_json::from_str(&format!("\"{}\"", row.sentiment_trend))
                .unwrap_or(SentimentTrend::Stable),
            momentum_trend: serde_json::from_str(&format!("\"{}\"", row.momentum_trend))
                .unwrap_or(MomentumTrend::Neutral),
            divergence: serde_json::from_str(&format!("\"{}\"", row.divergence))
                .unwrap_or(DivergenceType::None),
            sentiment_price_correlation: row.sentiment_price_correlation,
            correlation_lag_days: row.correlation_lag_days,
            correlation_strength,
            historical_sentiment,
            news_articles_analyzed: row.news_articles_analyzed,
            calculated_at: row.calculated_at.and_utc(),
            warnings,
        };

        Ok(Some(signal))
    } else {
        Ok(None)
    }
}

/// Save sentiment signal to cache
async fn save_sentiment_to_cache(
    pool: &PgPool,
    signal: &SentimentSignal,
) -> Result<(), AppError> {
    let expires_at = Utc::now() + Duration::hours(6); // 6-hour TTL

    let historical_json = serde_json::to_value(&signal.historical_sentiment)
        .map_err(|e| AppError::Validation(format!("Failed to serialize historical_sentiment: {}", e)))?;

    let warnings_json = serde_json::to_value(&signal.warnings)
        .map_err(|e| AppError::Validation(format!("Failed to serialize warnings: {}", e)))?;

    let sentiment_trend_str = match signal.sentiment_trend {
        SentimentTrend::Improving => "improving",
        SentimentTrend::Stable => "stable",
        SentimentTrend::Deteriorating => "deteriorating",
    };

    let momentum_trend_str = match signal.momentum_trend {
        MomentumTrend::Bullish => "bullish",
        MomentumTrend::Neutral => "neutral",
        MomentumTrend::Bearish => "bearish",
    };

    let divergence_str = match signal.divergence {
        DivergenceType::Bullish => "bullish",
        DivergenceType::Bearish => "bearish",
        DivergenceType::Confirmed => "confirmed",
        DivergenceType::None => "none",
    };

    sqlx::query!(
        r#"
        INSERT INTO sentiment_signal_cache (
            ticker,
            calculated_at,
            expires_at,
            current_sentiment,
            sentiment_trend,
            momentum_trend,
            divergence,
            sentiment_price_correlation,
            correlation_lag_days,
            historical_sentiment,
            news_articles_analyzed,
            warnings
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
        ON CONFLICT (ticker)
        DO UPDATE SET
            calculated_at = EXCLUDED.calculated_at,
            expires_at = EXCLUDED.expires_at,
            current_sentiment = EXCLUDED.current_sentiment,
            sentiment_trend = EXCLUDED.sentiment_trend,
            momentum_trend = EXCLUDED.momentum_trend,
            divergence = EXCLUDED.divergence,
            sentiment_price_correlation = EXCLUDED.sentiment_price_correlation,
            correlation_lag_days = EXCLUDED.correlation_lag_days,
            historical_sentiment = EXCLUDED.historical_sentiment,
            news_articles_analyzed = EXCLUDED.news_articles_analyzed,
            warnings = EXCLUDED.warnings
        "#,
        signal.ticker,
        signal.calculated_at.naive_utc(),
        expires_at.naive_utc(),
        signal.current_sentiment,
        sentiment_trend_str,
        momentum_trend_str,
        divergence_str,
        signal.sentiment_price_correlation,
        signal.correlation_lag_days,
        historical_json,
        signal.news_articles_analyzed,
        warnings_json,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_sentiment_score_empty() {
        let themes = vec![];
        assert_eq!(calculate_sentiment_score(&themes), 0.0);
    }

    #[test]
    fn test_detect_divergence() {
        // Bullish divergence
        assert_eq!(
            detect_divergence(SentimentTrend::Improving, MomentumTrend::Bearish),
            DivergenceType::Bullish
        );

        // Bearish divergence
        assert_eq!(
            detect_divergence(SentimentTrend::Deteriorating, MomentumTrend::Bullish),
            DivergenceType::Bearish
        );

        // Confirmed
        assert_eq!(
            detect_divergence(SentimentTrend::Improving, MomentumTrend::Bullish),
            DivergenceType::Confirmed
        );

        // None
        assert_eq!(
            detect_divergence(SentimentTrend::Stable, MomentumTrend::Neutral),
            DivergenceType::None
        );
    }

    #[test]
    fn test_pearson_correlation() {
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.0, 4.0, 6.0, 8.0, 10.0];

        let corr = pearson_correlation(&x, &y);
        assert!((corr - 1.0).abs() < 0.01); // Perfect positive correlation
    }

    #[test]
    fn test_classify_correlation_strength() {
        assert_eq!(classify_correlation_strength(0.8), "strong");
        assert_eq!(classify_correlation_strength(0.5), "moderate");
        assert_eq!(classify_correlation_strength(0.2), "weak");
        assert_eq!(classify_correlation_strength(-0.75), "strong");
    }
}
