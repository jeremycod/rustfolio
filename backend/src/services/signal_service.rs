use chrono::{Duration, Utc};
use sqlx::{PgPool, types::Json};
use uuid::Uuid;

use crate::models::{
    TradingSignal, SignalType, SignalDirection, SignalFactors, SignalFactor,
    SignalConfidenceLevel, SignalGenerationParams, SignalResponse,
};
use crate::services::indicators::{rsi, macd, bollinger_bands, volume_trend, sma, ema};

/// Signal service for generating probability-based trading signals
pub struct SignalService {
    pool: PgPool,
}

impl SignalService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Generate trading signals for a stock symbol
    pub async fn generate_signals(
        &self,
        params: SignalGenerationParams,
    ) -> Result<SignalResponse, String> {
        let mut signals = Vec::new();

        // Generate signals for each horizon
        for &horizon in &params.horizons {
            // Generate momentum signal
            if let Ok(momentum_signal) = self.generate_momentum_signal(&params, horizon) {
                signals.push(momentum_signal);
            }

            // Generate mean reversion signal
            if let Ok(mean_reversion_signal) = self.generate_mean_reversion_signal(&params, horizon) {
                signals.push(mean_reversion_signal);
            }

            // Generate trend signal
            if let Ok(trend_signal) = self.generate_trend_signal(&params, horizon) {
                signals.push(trend_signal);
            }

            // Generate combined signal
            if let Ok(combined_signal) = self.generate_combined_signal(&params, horizon) {
                signals.push(combined_signal);
            }
        }

        // Cache signals in database
        for signal in &signals {
            if let Err(e) = self.cache_signal(signal).await {
                eprintln!("Warning: Failed to cache signal: {}", e);
            }
        }

        // Determine overall recommendation (strongest signal)
        let (recommendation, confidence) = self.calculate_overall_recommendation(&signals);

        Ok(SignalResponse {
            ticker: params.ticker.clone(),
            current_price: Some(params.current_price),
            signals,
            recommendation: Some(recommendation),
            confidence: Some(confidence),
            analyzed_at: Utc::now(),
        })
    }

    /// Generate momentum-based signal (RSI, MACD, price momentum)
    fn generate_momentum_signal(
        &self,
        params: &SignalGenerationParams,
        horizon: i32,
    ) -> Result<TradingSignal, String> {
        let prices = &params.prices;
        if prices.len() < 30 {
            return Err("Insufficient price data for momentum signal".to_string());
        }

        let mut factors = Vec::new();
        let mut bullish_score = 0.0;
        let mut bearish_score = 0.0;
        let mut total_weight = 0.0;

        // Calculate RSI (14-period)
        let rsi_values = rsi(prices, 14);
        if let Some(current_rsi) = rsi_values.last().and_then(|&v| v) {
            let weight = self.get_horizon_weight(horizon, 0.35); // RSI more important for short-term
            total_weight += weight;

            let (direction, score) = self.interpret_rsi(current_rsi);
            let interpretation = format!("RSI: {:.1} - {}", current_rsi, self.rsi_interpretation(current_rsi));

            factors.push(SignalFactor {
                indicator: "RSI_14".to_string(),
                value: current_rsi,
                weight,
                direction,
                interpretation,
            });

            match direction {
                SignalDirection::Bullish => bullish_score += score * weight,
                SignalDirection::Bearish => bearish_score += score * weight,
                SignalDirection::Neutral => {}
            }
        }

        // Calculate MACD (12, 26, 9)
        let (macd_line, signal_line, histogram) = macd(prices, 12, 26, 9);
        if let (Some(macd_val), Some(signal_val), Some(hist_val)) = (
            macd_line.last().and_then(|&v| v),
            signal_line.last().and_then(|&v| v),
            histogram.last().and_then(|&v| v),
        ) {
            let weight = self.get_horizon_weight(horizon, 0.35);
            total_weight += weight;

            let (direction, score) = self.interpret_macd(macd_val, signal_val, hist_val);
            let interpretation = format!("MACD: {:.2}, Signal: {:.2}, Histogram: {:.2}", macd_val, signal_val, hist_val);

            factors.push(SignalFactor {
                indicator: "MACD".to_string(),
                value: hist_val,
                weight,
                direction,
                interpretation,
            });

            match direction {
                SignalDirection::Bullish => bullish_score += score * weight,
                SignalDirection::Bearish => bearish_score += score * weight,
                SignalDirection::Neutral => {}
            }
        }

        // Calculate price momentum (rate of change)
        if prices.len() >= 20 {
            let recent_price = prices[0];
            let past_price = prices[19];
            let momentum = (recent_price - past_price) / past_price;

            let weight = self.get_horizon_weight(horizon, 0.30);
            total_weight += weight;

            let (direction, score) = self.interpret_momentum(momentum);
            let interpretation = format!("20-day momentum: {:.2}%", momentum * 100.0);

            factors.push(SignalFactor {
                indicator: "Momentum_20d".to_string(),
                value: momentum,
                weight,
                direction,
                interpretation,
            });

            match direction {
                SignalDirection::Bullish => bullish_score += score * weight,
                SignalDirection::Bearish => bearish_score += score * weight,
                SignalDirection::Neutral => {}
            }
        }

        // Normalize scores
        if total_weight > 0.0 {
            bullish_score /= total_weight;
            bearish_score /= total_weight;
        }

        let signal_factors = SignalFactors {
            factors,
            bullish_score,
            bearish_score,
            total_factors: 3,
        };

        let (direction, probability) = signal_factors.calculate_signal();
        let confidence_level = SignalConfidenceLevel::from_probability(probability);
        let explanation = self.generate_explanation(&signal_factors, SignalType::Momentum);

        Ok(TradingSignal {
            id: Uuid::new_v4(),
            ticker: params.ticker.clone(),
            signal_type: SignalType::Momentum,
            horizon_months: horizon,
            probability,
            direction,
            confidence_level,
            contributing_factors: Json(signal_factors),
            explanation,
            generated_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::hours(4)),
        })
    }

    /// Generate mean reversion signal (Bollinger Bands, oversold/overbought)
    fn generate_mean_reversion_signal(
        &self,
        params: &SignalGenerationParams,
        horizon: i32,
    ) -> Result<TradingSignal, String> {
        let prices = &params.prices;
        if prices.len() < 30 {
            return Err("Insufficient price data for mean reversion signal".to_string());
        }

        let mut factors = Vec::new();
        let mut bullish_score = 0.0;
        let mut bearish_score = 0.0;
        let mut total_weight = 0.0;

        // Calculate Bollinger Bands (20-period, 2 std dev)
        let (middle_band, upper_band, lower_band) = bollinger_bands(prices, 20, 2.0);
        let current_price = params.current_price;

        if let (Some(mid), Some(upper), Some(lower)) = (
            middle_band.last().and_then(|&v| v),
            upper_band.last().and_then(|&v| v),
            lower_band.last().and_then(|&v| v),
        ) {
            let weight = 0.45; // High weight for mean reversion
            total_weight += weight;

            let (direction, score) = self.interpret_bollinger_bands(current_price, mid, upper, lower);
            let bb_percent = if upper - lower > 0.0 {
                (current_price - lower) / (upper - lower)
            } else {
                0.5
            };

            let interpretation = format!(
                "Bollinger Band %B: {:.2}% - Price relative to bands",
                bb_percent * 100.0
            );

            factors.push(SignalFactor {
                indicator: "Bollinger_Bands".to_string(),
                value: bb_percent,
                weight,
                direction,
                interpretation,
            });

            match direction {
                SignalDirection::Bullish => bullish_score += score * weight,
                SignalDirection::Bearish => bearish_score += score * weight,
                SignalDirection::Neutral => {}
            }
        }

        // RSI for oversold/overbought conditions
        let rsi_values = rsi(prices, 14);
        if let Some(current_rsi) = rsi_values.last().and_then(|&v| v) {
            let weight = 0.35;
            total_weight += weight;

            // For mean reversion, extreme RSI values signal reversal
            let (direction, score) = self.interpret_rsi_mean_reversion(current_rsi);
            let interpretation = format!("RSI: {:.1} - {}", current_rsi,
                if current_rsi < 30.0 { "Oversold - potential bounce" }
                else if current_rsi > 70.0 { "Overbought - potential pullback" }
                else { "Neutral zone" });

            factors.push(SignalFactor {
                indicator: "RSI_MeanReversion".to_string(),
                value: current_rsi,
                weight,
                direction,
                interpretation,
            });

            match direction {
                SignalDirection::Bullish => bullish_score += score * weight,
                SignalDirection::Bearish => bearish_score += score * weight,
                SignalDirection::Neutral => {}
            }
        }

        // Price deviation from moving average
        let sma_values = sma(prices, 50);
        if let Some(sma_50) = sma_values.last().and_then(|&v| v) {
            let deviation = (current_price - sma_50) / sma_50;
            let weight = 0.20;
            total_weight += weight;

            let (direction, score) = self.interpret_price_deviation(deviation);
            let interpretation = format!(
                "Price vs 50-day SMA: {:.2}% - {}",
                deviation * 100.0,
                if deviation > 0.1 { "Far above average" }
                else if deviation < -0.1 { "Far below average" }
                else { "Near average" }
            );

            factors.push(SignalFactor {
                indicator: "Price_Deviation_SMA50".to_string(),
                value: deviation,
                weight,
                direction,
                interpretation,
            });

            match direction {
                SignalDirection::Bullish => bullish_score += score * weight,
                SignalDirection::Bearish => bearish_score += score * weight,
                SignalDirection::Neutral => {}
            }
        }

        // Normalize scores
        if total_weight > 0.0 {
            bullish_score /= total_weight;
            bearish_score /= total_weight;
        }

        let signal_factors = SignalFactors {
            factors,
            bullish_score,
            bearish_score,
            total_factors: 3,
        };

        let (direction, probability) = signal_factors.calculate_signal();
        let confidence_level = SignalConfidenceLevel::from_probability(probability);
        let explanation = self.generate_explanation(&signal_factors, SignalType::MeanReversion);

        Ok(TradingSignal {
            id: Uuid::new_v4(),
            ticker: params.ticker.clone(),
            signal_type: SignalType::MeanReversion,
            horizon_months: horizon,
            probability,
            direction,
            confidence_level,
            contributing_factors: Json(signal_factors),
            explanation,
            generated_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::hours(4)),
        })
    }

    /// Generate trend-following signal (SMA crossovers, EMA alignment)
    fn generate_trend_signal(
        &self,
        params: &SignalGenerationParams,
        horizon: i32,
    ) -> Result<TradingSignal, String> {
        let prices = &params.prices;
        if prices.len() < 60 {
            return Err("Insufficient price data for trend signal".to_string());
        }

        let mut factors = Vec::new();
        let mut bullish_score = 0.0;
        let mut bearish_score = 0.0;
        let mut total_weight = 0.0;

        // SMA 50 vs SMA 200 (golden cross / death cross)
        let sma_50 = sma(prices, 50);
        let sma_200 = sma(prices, 200);

        if let (Some(sma50_val), Some(sma200_val)) = (
            sma_50.last().and_then(|&v| v),
            sma_200.last().and_then(|&v| v),
        ) {
            let weight = self.get_horizon_weight(horizon, 0.4); // More important for long-term
            total_weight += weight;

            let (direction, score) = self.interpret_sma_crossover(sma50_val, sma200_val);
            let interpretation = format!(
                "SMA 50/200: {:.2}/{:.2} - {}",
                sma50_val,
                sma200_val,
                if sma50_val > sma200_val { "Golden Cross (bullish)" } else { "Death Cross (bearish)" }
            );

            factors.push(SignalFactor {
                indicator: "SMA_50_200_Cross".to_string(),
                value: (sma50_val - sma200_val) / sma200_val,
                weight,
                direction,
                interpretation,
            });

            match direction {
                SignalDirection::Bullish => bullish_score += score * weight,
                SignalDirection::Bearish => bearish_score += score * weight,
                SignalDirection::Neutral => {}
            }
        }

        // EMA alignment (12, 26, 50)
        let ema_12 = ema(prices, 12);
        let ema_26 = ema(prices, 26);
        let ema_50 = ema(prices, 50);

        if let (Some(e12), Some(e26), Some(e50)) = (
            ema_12.last().and_then(|&v| v),
            ema_26.last().and_then(|&v| v),
            ema_50.last().and_then(|&v| v),
        ) {
            let weight = 0.35;
            total_weight += weight;

            let (direction, score) = self.interpret_ema_alignment(e12, e26, e50);
            let interpretation = format!(
                "EMA Alignment: 12={:.2}, 26={:.2}, 50={:.2} - {}",
                e12, e26, e50,
                if e12 > e26 && e26 > e50 { "Bullish alignment" }
                else if e12 < e26 && e26 < e50 { "Bearish alignment" }
                else { "Mixed alignment" }
            );

            factors.push(SignalFactor {
                indicator: "EMA_Alignment".to_string(),
                value: (e12 - e50) / e50,
                weight,
                direction,
                interpretation,
            });

            match direction {
                SignalDirection::Bullish => bullish_score += score * weight,
                SignalDirection::Bearish => bearish_score += score * weight,
                SignalDirection::Neutral => {}
            }
        }

        // Volume trend confirmation
        if !params.volumes.is_empty() && params.volumes.len() >= 20 {
            let (_, vol_ratio) = volume_trend(&params.volumes, 20);
            if let Some(current_vol_ratio) = vol_ratio.last().and_then(|&v| v) {
                let weight = 0.25;
                total_weight += weight;

                let (direction, score) = self.interpret_volume_trend(current_vol_ratio, params.prices[0], params.prices[19]);
                let interpretation = format!(
                    "Volume: {:.0}% of average - {}",
                    current_vol_ratio * 100.0,
                    if current_vol_ratio > 1.5 { "High conviction" }
                    else if current_vol_ratio < 0.5 { "Low conviction" }
                    else { "Average volume" }
                );

                factors.push(SignalFactor {
                    indicator: "Volume_Trend".to_string(),
                    value: current_vol_ratio,
                    weight,
                    direction,
                    interpretation,
                });

                match direction {
                    SignalDirection::Bullish => bullish_score += score * weight,
                    SignalDirection::Bearish => bearish_score += score * weight,
                    SignalDirection::Neutral => {}
                }
            }
        }

        // Normalize scores
        if total_weight > 0.0 {
            bullish_score /= total_weight;
            bearish_score /= total_weight;
        }

        let signal_factors = SignalFactors {
            factors,
            bullish_score,
            bearish_score,
            total_factors: 3,
        };

        let (direction, probability) = signal_factors.calculate_signal();
        let confidence_level = SignalConfidenceLevel::from_probability(probability);
        let explanation = self.generate_explanation(&signal_factors, SignalType::Trend);

        Ok(TradingSignal {
            id: Uuid::new_v4(),
            ticker: params.ticker.clone(),
            signal_type: SignalType::Trend,
            horizon_months: horizon,
            probability,
            direction,
            confidence_level,
            contributing_factors: Json(signal_factors),
            explanation,
            generated_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::hours(4)),
        })
    }

    /// Generate combined multi-factor signal
    fn generate_combined_signal(
        &self,
        params: &SignalGenerationParams,
        horizon: i32,
    ) -> Result<TradingSignal, String> {
        // Generate individual signals
        let momentum = self.generate_momentum_signal(params, horizon)?;
        let mean_reversion = self.generate_mean_reversion_signal(params, horizon)?;
        let trend = self.generate_trend_signal(params, horizon)?;

        // Combine factors with horizon-based weighting
        let momentum_weight = self.get_signal_type_weight(horizon, SignalType::Momentum);
        let mean_reversion_weight = self.get_signal_type_weight(horizon, SignalType::MeanReversion);
        let trend_weight = self.get_signal_type_weight(horizon, SignalType::Trend);
        let total_weight = momentum_weight + mean_reversion_weight + trend_weight;

        let mut combined_factors = Vec::new();
        let mut bullish_score = 0.0;
        let mut bearish_score = 0.0;

        // Weight momentum signal
        let momentum_factors = &momentum.contributing_factors.0;
        bullish_score += momentum_factors.bullish_score * momentum_weight;
        bearish_score += momentum_factors.bearish_score * momentum_weight;
        combined_factors.push(SignalFactor {
            indicator: "Momentum_Signal".to_string(),
            value: momentum.probability,
            weight: momentum_weight / total_weight,
            direction: momentum.direction,
            interpretation: format!("{} momentum ({})", momentum.direction, momentum.confidence_level),
        });

        // Weight mean reversion signal
        let mean_reversion_factors = &mean_reversion.contributing_factors.0;
        bullish_score += mean_reversion_factors.bullish_score * mean_reversion_weight;
        bearish_score += mean_reversion_factors.bearish_score * mean_reversion_weight;
        combined_factors.push(SignalFactor {
            indicator: "MeanReversion_Signal".to_string(),
            value: mean_reversion.probability,
            weight: mean_reversion_weight / total_weight,
            direction: mean_reversion.direction,
            interpretation: format!("{} mean reversion ({})", mean_reversion.direction, mean_reversion.confidence_level),
        });

        // Weight trend signal
        let trend_factors = &trend.contributing_factors.0;
        bullish_score += trend_factors.bullish_score * trend_weight;
        bearish_score += trend_factors.bearish_score * trend_weight;
        combined_factors.push(SignalFactor {
            indicator: "Trend_Signal".to_string(),
            value: trend.probability,
            weight: trend_weight / total_weight,
            direction: trend.direction,
            interpretation: format!("{} trend ({})", trend.direction, trend.confidence_level),
        });

        // Normalize scores
        bullish_score /= total_weight;
        bearish_score /= total_weight;

        let signal_factors = SignalFactors {
            factors: combined_factors,
            bullish_score,
            bearish_score,
            total_factors: 3,
        };

        let (direction, probability) = signal_factors.calculate_signal();
        let confidence_level = SignalConfidenceLevel::from_probability(probability);
        let explanation = self.generate_explanation(&signal_factors, SignalType::Combined);

        Ok(TradingSignal {
            id: Uuid::new_v4(),
            ticker: params.ticker.clone(),
            signal_type: SignalType::Combined,
            horizon_months: horizon,
            probability,
            direction,
            confidence_level,
            contributing_factors: Json(signal_factors),
            explanation,
            generated_at: Utc::now(),
            expires_at: Some(Utc::now() + Duration::hours(4)),
        })
    }

    /// Get signal type weight based on horizon
    fn get_signal_type_weight(&self, horizon_months: i32, signal_type: SignalType) -> f64 {
        match signal_type {
            SignalType::Momentum => {
                // Momentum more important for short-term
                if horizon_months <= 3 {
                    0.5
                } else if horizon_months <= 6 {
                    0.3
                } else {
                    0.2
                }
            }
            SignalType::MeanReversion => {
                // Mean reversion relatively consistent across horizons
                0.3
            }
            SignalType::Trend => {
                // Trend more important for long-term
                if horizon_months <= 3 {
                    0.2
                } else if horizon_months <= 6 {
                    0.4
                } else {
                    0.5
                }
            }
            SignalType::Combined => 1.0,
        }
    }

    /// Adjust indicator weight based on horizon
    fn get_horizon_weight(&self, horizon_months: i32, base_weight: f64) -> f64 {
        // Short-term (1-3 months): Favor momentum
        // Long-term (6-12 months): Favor trend
        let multiplier = if horizon_months <= 3 {
            1.2
        } else if horizon_months <= 6 {
            1.0
        } else {
            0.8
        };

        base_weight * multiplier
    }

    /// Interpret RSI value for momentum
    fn interpret_rsi(&self, rsi: f64) -> (SignalDirection, f64) {
        if rsi > 70.0 {
            (SignalDirection::Bearish, (rsi - 70.0) / 30.0)
        } else if rsi < 30.0 {
            (SignalDirection::Bullish, (30.0 - rsi) / 30.0)
        } else {
            (SignalDirection::Neutral, 0.1)
        }
    }

    /// Interpret RSI for mean reversion (inverted logic)
    fn interpret_rsi_mean_reversion(&self, rsi: f64) -> (SignalDirection, f64) {
        if rsi < 30.0 {
            // Oversold - expect bounce (bullish)
            (SignalDirection::Bullish, (30.0 - rsi) / 30.0)
        } else if rsi > 70.0 {
            // Overbought - expect pullback (bearish)
            (SignalDirection::Bearish, (rsi - 70.0) / 30.0)
        } else {
            (SignalDirection::Neutral, 0.1)
        }
    }

    fn rsi_interpretation(&self, rsi: f64) -> &str {
        if rsi > 80.0 {
            "Extremely overbought"
        } else if rsi > 70.0 {
            "Overbought"
        } else if rsi > 60.0 {
            "Moderately bullish"
        } else if rsi > 40.0 {
            "Neutral"
        } else if rsi > 30.0 {
            "Moderately bearish"
        } else if rsi > 20.0 {
            "Oversold"
        } else {
            "Extremely oversold"
        }
    }

    /// Interpret MACD values
    fn interpret_macd(&self, macd: f64, signal: f64, histogram: f64) -> (SignalDirection, f64) {
        // MACD above signal is bullish, below is bearish
        if macd > signal && histogram > 0.0 {
            let strength = (histogram.abs() / macd.abs()).min(1.0);
            (SignalDirection::Bullish, strength * 0.8)
        } else if macd < signal && histogram < 0.0 {
            let strength = (histogram.abs() / macd.abs()).min(1.0);
            (SignalDirection::Bearish, strength * 0.8)
        } else {
            (SignalDirection::Neutral, 0.1)
        }
    }

    /// Interpret price momentum
    fn interpret_momentum(&self, momentum: f64) -> (SignalDirection, f64) {
        if momentum > 0.05 {
            (SignalDirection::Bullish, (momentum * 10.0).min(1.0))
        } else if momentum < -0.05 {
            (SignalDirection::Bearish, (momentum.abs() * 10.0).min(1.0))
        } else {
            (SignalDirection::Neutral, 0.1)
        }
    }

    /// Interpret Bollinger Bands position
    fn interpret_bollinger_bands(&self, price: f64, _middle: f64, upper: f64, lower: f64) -> (SignalDirection, f64) {
        let band_width = upper - lower;
        if band_width <= 0.0 {
            return (SignalDirection::Neutral, 0.1);
        }

        let position = (price - lower) / band_width;

        if position < 0.1 {
            // Near lower band - oversold, potential bounce
            (SignalDirection::Bullish, 0.8)
        } else if position > 0.9 {
            // Near upper band - overbought, potential pullback
            (SignalDirection::Bearish, 0.8)
        } else if position < 0.3 {
            (SignalDirection::Bullish, 0.5)
        } else if position > 0.7 {
            (SignalDirection::Bearish, 0.5)
        } else {
            (SignalDirection::Neutral, 0.1)
        }
    }

    /// Interpret price deviation from SMA
    fn interpret_price_deviation(&self, deviation: f64) -> (SignalDirection, f64) {
        // For mean reversion, extreme deviations signal reversal
        if deviation > 0.15 {
            // Far above average - expect pullback
            (SignalDirection::Bearish, (deviation * 3.0).min(1.0))
        } else if deviation < -0.15 {
            // Far below average - expect bounce
            (SignalDirection::Bullish, (deviation.abs() * 3.0).min(1.0))
        } else {
            (SignalDirection::Neutral, 0.1)
        }
    }

    /// Interpret SMA crossover
    fn interpret_sma_crossover(&self, sma_50: f64, sma_200: f64) -> (SignalDirection, f64) {
        let diff = (sma_50 - sma_200) / sma_200;

        if diff > 0.02 {
            (SignalDirection::Bullish, (diff * 20.0).min(1.0))
        } else if diff < -0.02 {
            (SignalDirection::Bearish, (diff.abs() * 20.0).min(1.0))
        } else {
            (SignalDirection::Neutral, 0.1)
        }
    }

    /// Interpret EMA alignment
    fn interpret_ema_alignment(&self, ema_12: f64, ema_26: f64, ema_50: f64) -> (SignalDirection, f64) {
        let bullish_aligned = ema_12 > ema_26 && ema_26 > ema_50;
        let bearish_aligned = ema_12 < ema_26 && ema_26 < ema_50;

        if bullish_aligned {
            let strength = ((ema_12 - ema_50) / ema_50).min(0.2) * 5.0;
            (SignalDirection::Bullish, strength)
        } else if bearish_aligned {
            let strength = ((ema_50 - ema_12) / ema_50).min(0.2) * 5.0;
            (SignalDirection::Bearish, strength)
        } else {
            (SignalDirection::Neutral, 0.1)
        }
    }

    /// Interpret volume trend
    fn interpret_volume_trend(&self, vol_ratio: f64, current_price: f64, past_price: f64) -> (SignalDirection, f64) {
        let price_up = current_price > past_price;

        // High volume confirms the price trend
        if vol_ratio > 1.5 {
            if price_up {
                (SignalDirection::Bullish, 0.7)
            } else {
                (SignalDirection::Bearish, 0.7)
            }
        } else if vol_ratio < 0.5 {
            // Low volume weakens the signal
            (SignalDirection::Neutral, 0.1)
        } else {
            // Average volume - mild confirmation
            if price_up {
                (SignalDirection::Bullish, 0.4)
            } else {
                (SignalDirection::Bearish, 0.4)
            }
        }
    }

    /// Generate human-readable explanation
    fn generate_explanation(&self, factors: &SignalFactors, signal_type: SignalType) -> String {
        let (direction, probability) = factors.calculate_signal();
        let confidence = SignalConfidenceLevel::from_probability(probability);

        let mut explanation = format!(
            "{} ({:.0}% probability, {} confidence) - {}: ",
            direction,
            probability * 100.0,
            confidence,
            signal_type
        );

        // List top contributing factors
        let mut sorted_factors = factors.factors.clone();
        sorted_factors.sort_by(|a, b| b.weight.partial_cmp(&a.weight).unwrap());

        let top_factors: Vec<String> = sorted_factors
            .iter()
            .take(3)
            .map(|f| f.interpretation.clone())
            .collect();

        explanation.push_str(&top_factors.join(", "));

        explanation
    }

    /// Calculate overall recommendation from all signals
    pub fn calculate_overall_recommendation(&self, signals: &[TradingSignal]) -> (SignalDirection, SignalConfidenceLevel) {
        if signals.is_empty() {
            return (SignalDirection::Neutral, SignalConfidenceLevel::Low);
        }

        let mut bullish_score = 0.0;
        let mut bearish_score = 0.0;
        let mut total_weight = 0.0;

        for signal in signals {
            let weight = match signal.signal_type {
                SignalType::Combined => 2.0, // Combined signal gets double weight
                _ => 1.0,
            };

            total_weight += weight;

            match signal.direction {
                SignalDirection::Bullish => bullish_score += signal.probability * weight,
                SignalDirection::Bearish => bearish_score += signal.probability * weight,
                SignalDirection::Neutral => {}
            }
        }

        if total_weight > 0.0 {
            bullish_score /= total_weight;
            bearish_score /= total_weight;
        }

        let net_score = bullish_score - bearish_score;
        let direction = if net_score > 0.1 {
            SignalDirection::Bullish
        } else if net_score < -0.1 {
            SignalDirection::Bearish
        } else {
            SignalDirection::Neutral
        };

        let probability = 0.5 + (net_score.abs() * 0.5);
        let confidence = SignalConfidenceLevel::from_probability(probability.min(1.0));

        (direction, confidence)
    }

    /// Cache signal in database
    async fn cache_signal(&self, signal: &TradingSignal) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO stock_signals (
                id, ticker, signal_type, horizon_months, probability,
                direction, confidence_level, contributing_factors,
                explanation, generated_at, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ON CONFLICT (ticker, signal_type, horizon_months, generated_at)
            DO UPDATE SET
                probability = EXCLUDED.probability,
                direction = EXCLUDED.direction,
                confidence_level = EXCLUDED.confidence_level,
                contributing_factors = EXCLUDED.contributing_factors,
                explanation = EXCLUDED.explanation,
                expires_at = EXCLUDED.expires_at
            "#,
        )
        .bind(signal.id)
        .bind(&signal.ticker)
        .bind(signal.signal_type.to_string())
        .bind(signal.horizon_months)
        .bind(signal.probability)
        .bind(signal.direction.to_string())
        .bind(signal.confidence_level.to_string())
        .bind(serde_json::to_value(&signal.contributing_factors.0).unwrap())
        .bind(&signal.explanation)
        .bind(signal.generated_at)
        .bind(signal.expires_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Retrieve cached signals
    pub async fn get_cached_signals(
        &self,
        ticker: &str,
        horizon: Option<i32>,
        signal_type: Option<SignalType>,
    ) -> Result<Vec<TradingSignal>, sqlx::Error> {
        let query = if let (Some(h), Some(st)) = (horizon, signal_type) {
            sqlx::query_as::<_, TradingSignal>(
                r#"
                SELECT id, ticker, signal_type, horizon_months, probability,
                       direction, confidence_level, contributing_factors,
                       explanation, generated_at, expires_at
                FROM stock_signals
                WHERE ticker = $1 AND horizon_months = $2 AND signal_type = $3
                  AND (expires_at IS NULL OR expires_at > NOW())
                ORDER BY generated_at DESC
                LIMIT 10
                "#
            )
            .bind(ticker)
            .bind(h)
            .bind(st.to_string())
            .fetch_all(&self.pool)
            .await
        } else if let Some(h) = horizon {
            sqlx::query_as::<_, TradingSignal>(
                r#"
                SELECT id, ticker, signal_type, horizon_months, probability,
                       direction, confidence_level, contributing_factors,
                       explanation, generated_at, expires_at
                FROM stock_signals
                WHERE ticker = $1 AND horizon_months = $2
                  AND (expires_at IS NULL OR expires_at > NOW())
                ORDER BY generated_at DESC
                LIMIT 10
                "#
            )
            .bind(ticker)
            .bind(h)
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as::<_, TradingSignal>(
                r#"
                SELECT id, ticker, signal_type, horizon_months, probability,
                       direction, confidence_level, contributing_factors,
                       explanation, generated_at, expires_at
                FROM stock_signals
                WHERE ticker = $1
                  AND (expires_at IS NULL OR expires_at > NOW())
                ORDER BY generated_at DESC
                LIMIT 10
                "#
            )
            .bind(ticker)
            .fetch_all(&self.pool)
            .await
        };

        query
    }
}
