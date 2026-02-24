/// Phase 3: Factor Calculation Accuracy Tests
///
/// Tests for fundamental, technical, momentum, and quality factor calculations
/// used by the multi-factor screening engine and factor-based recommendation system.

// ---------------------------------------------------------------------------
// Fundamental Factor Calculations
// ---------------------------------------------------------------------------

#[cfg(test)]
mod fundamental_factors {
    /// PE ratio = price / earnings_per_share
    fn calculate_pe_ratio(price: f64, eps: f64) -> Option<f64> {
        if eps <= 0.0 { None } else { Some(price / eps) }
    }

    /// PB ratio = price / book_value_per_share
    fn calculate_pb_ratio(price: f64, bvps: f64) -> Option<f64> {
        if bvps <= 0.0 { None } else { Some(price / bvps) }
    }

    /// PEG ratio = PE / earnings_growth_rate (in percent)
    fn calculate_peg_ratio(pe: f64, earnings_growth_pct: f64) -> Option<f64> {
        if earnings_growth_pct <= 0.0 { None } else { Some(pe / earnings_growth_pct) }
    }

    /// Debt-to-equity ratio = total_debt / total_equity
    fn calculate_debt_to_equity(total_debt: f64, total_equity: f64) -> Option<f64> {
        if total_equity <= 0.0 { None } else { Some(total_debt / total_equity) }
    }

    /// Earnings growth year-over-year
    fn calculate_earnings_growth_yoy(current_eps: f64, previous_eps: f64) -> Option<f64> {
        if previous_eps == 0.0 { None } else { Some((current_eps - previous_eps) / previous_eps.abs() * 100.0) }
    }

    #[test]
    fn test_pe_ratio_positive_eps() {
        let pe = calculate_pe_ratio(150.0, 6.0);
        assert_eq!(pe, Some(25.0));
    }

    #[test]
    fn test_pe_ratio_negative_eps_returns_none() {
        let pe = calculate_pe_ratio(150.0, -3.0);
        assert_eq!(pe, None);
    }

    #[test]
    fn test_pe_ratio_zero_eps_returns_none() {
        let pe = calculate_pe_ratio(150.0, 0.0);
        assert_eq!(pe, None);
    }

    #[test]
    fn test_pe_ratio_known_value() {
        // AAPL-like: ~$180, EPS ~$6.50 -> PE ~27.7
        let pe = calculate_pe_ratio(180.0, 6.50).unwrap();
        assert!((pe - 27.692).abs() < 0.01);
    }

    #[test]
    fn test_pb_ratio_positive_bvps() {
        let pb = calculate_pb_ratio(100.0, 25.0);
        assert_eq!(pb, Some(4.0));
    }

    #[test]
    fn test_pb_ratio_negative_bvps_returns_none() {
        let pb = calculate_pb_ratio(100.0, -10.0);
        assert_eq!(pb, None);
    }

    #[test]
    fn test_peg_ratio_standard() {
        // PE of 25, earnings growth of 15% -> PEG = 1.67
        let peg = calculate_peg_ratio(25.0, 15.0).unwrap();
        assert!((peg - 1.667).abs() < 0.01);
    }

    #[test]
    fn test_peg_ratio_negative_growth_returns_none() {
        let peg = calculate_peg_ratio(25.0, -5.0);
        assert_eq!(peg, None);
    }

    #[test]
    fn test_peg_ratio_below_one_signals_undervalued() {
        // PE of 10, growth of 20% -> PEG = 0.5 (undervalued)
        let peg = calculate_peg_ratio(10.0, 20.0).unwrap();
        assert!(peg < 1.0);
    }

    #[test]
    fn test_debt_to_equity_low_leverage() {
        let de = calculate_debt_to_equity(500_000.0, 2_000_000.0).unwrap();
        assert_eq!(de, 0.25);
    }

    #[test]
    fn test_debt_to_equity_high_leverage() {
        let de = calculate_debt_to_equity(5_000_000.0, 1_000_000.0).unwrap();
        assert_eq!(de, 5.0);
    }

    #[test]
    fn test_debt_to_equity_zero_equity_returns_none() {
        let de = calculate_debt_to_equity(1_000_000.0, 0.0);
        assert_eq!(de, None);
    }

    #[test]
    fn test_earnings_growth_positive() {
        let growth = calculate_earnings_growth_yoy(8.0, 6.0).unwrap();
        assert!((growth - 33.333).abs() < 0.01);
    }

    #[test]
    fn test_earnings_growth_negative() {
        let growth = calculate_earnings_growth_yoy(4.0, 6.0).unwrap();
        assert!((growth - (-33.333)).abs() < 0.01);
    }

    #[test]
    fn test_earnings_growth_from_loss_to_profit() {
        let growth = calculate_earnings_growth_yoy(3.0, -2.0).unwrap();
        // (3 - (-2)) / abs(-2) * 100 = 250%
        assert!((growth - 250.0).abs() < 0.01);
    }

    #[test]
    fn test_earnings_growth_zero_base_returns_none() {
        let growth = calculate_earnings_growth_yoy(5.0, 0.0);
        assert_eq!(growth, None);
    }
}

// ---------------------------------------------------------------------------
// Technical Factor Calculations
// ---------------------------------------------------------------------------

#[cfg(test)]
mod technical_factors {
    /// Simple Moving Average
    fn sma(values: &[f64], window: usize) -> Vec<Option<f64>> {
        if window == 0 {
            return vec![None; values.len()];
        }
        values
            .iter()
            .enumerate()
            .scan(0.0_f64, move |sum, (i, &v)| {
                *sum += v;
                if i >= window {
                    *sum -= values[i - window];
                }
                let out = if i + 1 >= window {
                    Some(*sum / window as f64)
                } else {
                    None
                };
                Some(out)
            })
            .collect()
    }

    /// Detect golden cross: short SMA crosses above long SMA
    fn detect_golden_cross(short_sma: &[Option<f64>], long_sma: &[Option<f64>]) -> bool {
        if short_sma.len() < 2 || long_sma.len() < 2 {
            return false;
        }
        let n = short_sma.len();
        match (short_sma[n-2], long_sma[n-2], short_sma[n-1], long_sma[n-1]) {
            (Some(prev_short), Some(prev_long), Some(curr_short), Some(curr_long)) => {
                prev_short <= prev_long && curr_short > curr_long
            }
            _ => false,
        }
    }

    /// Detect death cross: short SMA crosses below long SMA
    fn detect_death_cross(short_sma: &[Option<f64>], long_sma: &[Option<f64>]) -> bool {
        if short_sma.len() < 2 || long_sma.len() < 2 {
            return false;
        }
        let n = short_sma.len();
        match (short_sma[n-2], long_sma[n-2], short_sma[n-1], long_sma[n-1]) {
            (Some(prev_short), Some(prev_long), Some(curr_short), Some(curr_long)) => {
                prev_short >= prev_long && curr_short < curr_long
            }
            _ => false,
        }
    }

    /// RSI calculation
    fn rsi(prices: &[f64], period: usize) -> Vec<Option<f64>> {
        if prices.len() < 2 || period == 0 {
            return vec![None; prices.len()];
        }
        let mut result = vec![None; prices.len()];
        let changes: Vec<f64> = prices.windows(2).map(|w| w[1] - w[0]).collect();
        if changes.is_empty() {
            return result;
        }
        let gains: Vec<f64> = changes.iter().map(|&c| if c > 0.0 { c } else { 0.0 }).collect();
        let losses: Vec<f64> = changes.iter().map(|&c| if c < 0.0 { -c } else { 0.0 }).collect();

        let alpha = 1.0 / period as f64;
        let mut avg_gain = gains[..period.min(gains.len())].iter().sum::<f64>() / period as f64;
        let mut avg_loss = losses[..period.min(losses.len())].iter().sum::<f64>() / period as f64;

        if period < prices.len() {
            let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
            result[period] = Some(100.0 - (100.0 / (1.0 + rs)));
        }

        for i in period..changes.len() {
            avg_gain = alpha * gains[i] + (1.0 - alpha) * avg_gain;
            avg_loss = alpha * losses[i] + (1.0 - alpha) * avg_loss;
            let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
            result[i + 1] = Some(100.0 - (100.0 / (1.0 + rs)));
        }
        result
    }

    /// Relative strength vs benchmark
    fn relative_strength(stock_return: f64, benchmark_return: f64) -> f64 {
        if benchmark_return == 0.0 {
            return if stock_return > 0.0 { f64::INFINITY } else { 0.0 };
        }
        (1.0 + stock_return) / (1.0 + benchmark_return)
    }

    #[test]
    fn test_sma_basic() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let result = sma(&values, 3);
        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
        assert!((result[2].unwrap() - 20.0).abs() < 0.001);
        assert!((result[3].unwrap() - 30.0).abs() < 0.001);
        assert!((result[4].unwrap() - 40.0).abs() < 0.001);
    }

    #[test]
    fn test_sma_window_1_equals_values() {
        let values = vec![5.0, 10.0, 15.0];
        let result = sma(&values, 1);
        assert_eq!(result[0], Some(5.0));
        assert_eq!(result[1], Some(10.0));
        assert_eq!(result[2], Some(15.0));
    }

    #[test]
    fn test_sma_window_larger_than_data() {
        let values = vec![10.0, 20.0];
        let result = sma(&values, 5);
        assert_eq!(result[0], None);
        assert_eq!(result[1], None);
    }

    #[test]
    fn test_golden_cross_detected() {
        // Short SMA goes from below long SMA to above
        let short = vec![Some(48.0), Some(50.0), Some(52.0)];
        let long  = vec![Some(49.0), Some(51.0), Some(51.0)];
        assert!(detect_golden_cross(&short, &long));
    }

    #[test]
    fn test_death_cross_detected() {
        let short = vec![Some(52.0), Some(50.0), Some(48.0)];
        let long  = vec![Some(49.0), Some(49.0), Some(49.0)];
        assert!(detect_death_cross(&short, &long));
    }

    #[test]
    fn test_no_cross_when_parallel() {
        let short = vec![Some(55.0), Some(56.0), Some(57.0)];
        let long  = vec![Some(50.0), Some(51.0), Some(52.0)];
        assert!(!detect_golden_cross(&short, &long));
        assert!(!detect_death_cross(&short, &long));
    }

    #[test]
    fn test_rsi_strong_uptrend() {
        let uptrend: Vec<f64> = (0..30).map(|i| 50.0 + i as f64).collect();
        let result = rsi(&uptrend, 14);
        let last = result.last().and_then(|&v| v).unwrap();
        assert!(last > 70.0, "Strong uptrend RSI should be > 70, got {}", last);
    }

    #[test]
    fn test_rsi_strong_downtrend() {
        let downtrend: Vec<f64> = (0..30).map(|i| 80.0 - i as f64).collect();
        let result = rsi(&downtrend, 14);
        let last = result.last().and_then(|&v| v).unwrap();
        assert!(last < 30.0, "Strong downtrend RSI should be < 30, got {}", last);
    }

    #[test]
    fn test_rsi_range_always_0_to_100() {
        let volatile: Vec<f64> = (0..50).map(|i| 50.0 + (i as f64).sin() * 20.0).collect();
        let result = rsi(&volatile, 14);
        for val in result.iter().flatten() {
            assert!(*val >= 0.0 && *val <= 100.0, "RSI out of range: {}", val);
        }
    }

    #[test]
    fn test_rsi_empty_prices() {
        let result = rsi(&[], 14);
        assert!(result.is_empty());
    }

    #[test]
    fn test_rsi_insufficient_data() {
        let result = rsi(&[100.0], 14);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], None);
    }

    #[test]
    fn test_relative_strength_outperform() {
        let rs = relative_strength(0.15, 0.10);
        assert!(rs > 1.0, "Stock outperforming benchmark should have RS > 1.0");
    }

    #[test]
    fn test_relative_strength_underperform() {
        let rs = relative_strength(0.05, 0.10);
        assert!(rs < 1.0, "Stock underperforming benchmark should have RS < 1.0");
    }

    #[test]
    fn test_relative_strength_equal() {
        let rs = relative_strength(0.10, 0.10);
        assert!((rs - 1.0).abs() < 0.001);
    }
}

// ---------------------------------------------------------------------------
// Momentum Factor Calculations
// ---------------------------------------------------------------------------

#[cfg(test)]
mod momentum_factors {
    /// Price momentum: percentage return over N periods
    fn price_momentum(prices: &[f64], periods: usize) -> Option<f64> {
        if prices.len() <= periods {
            return None;
        }
        let current = *prices.last()?;
        let past = prices[prices.len() - 1 - periods];
        if past == 0.0 { return None; }
        Some((current - past) / past * 100.0)
    }

    /// Acceleration: change in momentum (second derivative)
    fn momentum_acceleration(
        prices: &[f64],
        short_period: usize,
        long_period: usize,
    ) -> Option<f64> {
        let short_mom = price_momentum(prices, short_period)?;
        let long_mom = price_momentum(prices, long_period)?;
        Some(short_mom - long_mom / (long_period as f64 / short_period as f64))
    }

    /// Volume momentum: ratio of recent avg volume to longer-term avg
    fn volume_momentum(volumes: &[f64], short_window: usize, long_window: usize) -> Option<f64> {
        if volumes.len() < long_window || short_window == 0 || long_window == 0 {
            return None;
        }
        let n = volumes.len();
        let short_avg: f64 = volumes[n - short_window..].iter().sum::<f64>() / short_window as f64;
        let long_avg: f64 = volumes[n - long_window..].iter().sum::<f64>() / long_window as f64;
        if long_avg == 0.0 { return None; }
        Some(short_avg / long_avg)
    }

    #[test]
    fn test_price_momentum_1_month() {
        // 20 trading days for ~1 month
        let mut prices: Vec<f64> = vec![100.0; 20];
        prices.push(110.0); // 10% gain
        let mom = price_momentum(&prices, 20).unwrap();
        assert!((mom - 10.0).abs() < 0.01);
    }

    #[test]
    fn test_price_momentum_negative() {
        let mut prices: Vec<f64> = vec![100.0; 20];
        prices.push(90.0); // -10% loss
        let mom = price_momentum(&prices, 20).unwrap();
        assert!((mom - (-10.0)).abs() < 0.01);
    }

    #[test]
    fn test_price_momentum_insufficient_data() {
        let prices = vec![100.0; 5];
        let mom = price_momentum(&prices, 10);
        assert_eq!(mom, None);
    }

    #[test]
    fn test_price_momentum_3_month() {
        let mut prices: Vec<f64> = (0..60).map(|i| 100.0 + i as f64 * 0.5).collect();
        prices.push(135.0);
        let mom = price_momentum(&prices, 60).unwrap();
        assert!(mom > 0.0, "Upward trending prices should show positive 3M momentum");
    }

    #[test]
    fn test_volume_momentum_increasing() {
        // 40 periods at 1M, then 10 periods at 1.5M
        // Short avg (last 10) = 1.5M, Long avg (all 50) = (40*1M + 10*1.5M)/50 = 1.1M
        // Ratio = 1.5M / 1.1M = ~1.36
        let mut volumes: Vec<f64> = vec![1_000_000.0; 50];
        for v in volumes.iter_mut().rev().take(10) {
            *v = 1_500_000.0;
        }
        let vm = volume_momentum(&volumes, 10, 50).unwrap();
        assert!(vm > 1.0, "Volume momentum should be above 1.0 for increasing volume, got {}", vm);
    }

    #[test]
    fn test_volume_momentum_stable() {
        let volumes = vec![1_000_000.0; 50];
        let vm = volume_momentum(&volumes, 10, 50).unwrap();
        assert!((vm - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_acceleration_positive() {
        // Prices accelerating upward
        let prices: Vec<f64> = (0..100).map(|i| 100.0 + (i as f64).powi(2) * 0.01).collect();
        let accel = momentum_acceleration(&prices, 20, 60);
        assert!(accel.is_some());
        // Short-term momentum should exceed scaled long-term momentum for accelerating prices
    }

    #[test]
    fn test_momentum_zero_base_price() {
        let prices = vec![0.0, 0.0, 0.0, 10.0];
        let mom = price_momentum(&prices, 3);
        assert_eq!(mom, None);
    }
}

// ---------------------------------------------------------------------------
// Scoring Algorithm Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod scoring_algorithms {
    /// Normalize a value to [0, 1] using min-max normalization
    fn min_max_normalize(value: f64, min: f64, max: f64) -> f64 {
        if (max - min).abs() < f64::EPSILON {
            return 0.5;
        }
        ((value - min) / (max - min)).clamp(0.0, 1.0)
    }

    /// Z-score normalization
    fn z_score_normalize(value: f64, mean: f64, std_dev: f64) -> f64 {
        if std_dev.abs() < f64::EPSILON {
            return 0.0;
        }
        (value - mean) / std_dev
    }

    /// Calculate composite score from weighted factors
    fn composite_score(factors: &[(f64, f64)]) -> f64 {
        let total_weight: f64 = factors.iter().map(|(_, w)| w).sum();
        if total_weight == 0.0 {
            return 0.0;
        }
        let weighted_sum: f64 = factors.iter().map(|(score, weight)| score * weight).sum();
        (weighted_sum / total_weight).clamp(0.0, 100.0)
    }

    /// Rank stocks by score, return sorted indices (highest first)
    fn rank_by_score(scores: &[f64]) -> Vec<usize> {
        let mut indices: Vec<usize> = (0..scores.len()).collect();
        indices.sort_by(|&a, &b| scores[b].partial_cmp(&scores[a]).unwrap());
        indices
    }

    #[test]
    fn test_min_max_normalize_middle() {
        let result = min_max_normalize(50.0, 0.0, 100.0);
        assert!((result - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_min_max_normalize_at_min() {
        let result = min_max_normalize(0.0, 0.0, 100.0);
        assert!((result - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_min_max_normalize_at_max() {
        let result = min_max_normalize(100.0, 0.0, 100.0);
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_min_max_normalize_clamped_above() {
        let result = min_max_normalize(150.0, 0.0, 100.0);
        assert_eq!(result, 1.0);
    }

    #[test]
    fn test_min_max_normalize_clamped_below() {
        let result = min_max_normalize(-50.0, 0.0, 100.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_min_max_normalize_same_min_max() {
        let result = min_max_normalize(50.0, 50.0, 50.0);
        assert_eq!(result, 0.5);
    }

    #[test]
    fn test_z_score_normalize_at_mean() {
        let result = z_score_normalize(50.0, 50.0, 10.0);
        assert!((result - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_z_score_normalize_one_std_above() {
        let result = z_score_normalize(60.0, 50.0, 10.0);
        assert!((result - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_z_score_normalize_zero_std() {
        let result = z_score_normalize(50.0, 50.0, 0.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_composite_score_equal_weights() {
        let factors = vec![(80.0, 1.0), (60.0, 1.0), (70.0, 1.0)];
        let score = composite_score(&factors);
        assert!((score - 70.0).abs() < 0.01);
    }

    #[test]
    fn test_composite_score_weighted() {
        // Value (80, weight 2), Momentum (60, weight 1), Quality (90, weight 3)
        let factors = vec![(80.0, 2.0), (60.0, 1.0), (90.0, 3.0)];
        let score = composite_score(&factors);
        // (160 + 60 + 270) / 6 = 81.67
        assert!((score - 81.667).abs() < 0.01);
    }

    #[test]
    fn test_composite_score_zero_weights() {
        let factors = vec![(80.0, 0.0), (60.0, 0.0)];
        let score = composite_score(&factors);
        assert_eq!(score, 0.0);
    }

    #[test]
    fn test_composite_score_clamped() {
        let factors = vec![(150.0, 1.0)]; // above 100
        let score = composite_score(&factors);
        assert_eq!(score, 100.0);
    }

    #[test]
    fn test_ranking_basic() {
        let scores = vec![50.0, 90.0, 30.0, 70.0];
        let ranked = rank_by_score(&scores);
        assert_eq!(ranked[0], 1); // 90.0 first
        assert_eq!(ranked[1], 3); // 70.0 second
        assert_eq!(ranked[2], 0); // 50.0 third
        assert_eq!(ranked[3], 2); // 30.0 last
    }

    #[test]
    fn test_ranking_empty() {
        let scores: Vec<f64> = vec![];
        let ranked = rank_by_score(&scores);
        assert!(ranked.is_empty());
    }

    #[test]
    fn test_ranking_single() {
        let scores = vec![85.0];
        let ranked = rank_by_score(&scores);
        assert_eq!(ranked, vec![0]);
    }

    #[test]
    fn test_ranking_equal_scores() {
        let scores = vec![75.0, 75.0, 75.0];
        let ranked = rank_by_score(&scores);
        assert_eq!(ranked.len(), 3);
    }

    #[test]
    fn test_top_n_selection() {
        let scores = vec![50.0, 90.0, 30.0, 70.0, 85.0, 60.0];
        let ranked = rank_by_score(&scores);
        let top_3: Vec<usize> = ranked.into_iter().take(3).collect();
        assert_eq!(top_3, vec![1, 4, 3]); // 90, 85, 70
    }
}

// ---------------------------------------------------------------------------
// Quality Metrics Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod quality_metrics {
    /// Return on Equity
    fn calculate_roe(net_income: f64, shareholders_equity: f64) -> Option<f64> {
        if shareholders_equity <= 0.0 { None } else { Some(net_income / shareholders_equity * 100.0) }
    }

    /// Return on Invested Capital
    fn calculate_roic(nopat: f64, invested_capital: f64) -> Option<f64> {
        if invested_capital <= 0.0 { None } else { Some(nopat / invested_capital * 100.0) }
    }

    /// Dividend yield
    fn calculate_dividend_yield(annual_dividend: f64, price: f64) -> Option<f64> {
        if price <= 0.0 { None } else { Some(annual_dividend / price * 100.0) }
    }

    /// Dividend payout ratio
    fn calculate_payout_ratio(dividends_per_share: f64, eps: f64) -> Option<f64> {
        if eps <= 0.0 { None } else { Some(dividends_per_share / eps * 100.0) }
    }

    /// Gross margin
    fn calculate_gross_margin(revenue: f64, cogs: f64) -> Option<f64> {
        if revenue <= 0.0 { None } else { Some((revenue - cogs) / revenue * 100.0) }
    }

    /// Revenue growth consistency (standard deviation of YoY growth rates)
    fn revenue_growth_consistency(yoy_growth_rates: &[f64]) -> f64 {
        if yoy_growth_rates.is_empty() {
            return f64::MAX;
        }
        let mean: f64 = yoy_growth_rates.iter().sum::<f64>() / yoy_growth_rates.len() as f64;
        let variance: f64 = yoy_growth_rates.iter()
            .map(|&g| (g - mean).powi(2))
            .sum::<f64>() / yoy_growth_rates.len() as f64;
        variance.sqrt()
    }

    /// Composite quality score (0-100)
    fn quality_score(roe: f64, roic: f64, gross_margin: f64, growth_consistency: f64) -> f64 {
        let mut score = 0.0;

        // ROE component (max 25 points)
        score += (roe / 30.0 * 25.0).clamp(0.0, 25.0);

        // ROIC component (max 25 points)
        score += (roic / 20.0 * 25.0).clamp(0.0, 25.0);

        // Gross margin component (max 25 points)
        score += (gross_margin / 60.0 * 25.0).clamp(0.0, 25.0);

        // Growth consistency (lower std = better, max 25 points)
        let consistency_score = if growth_consistency < 5.0 {
            25.0
        } else if growth_consistency < 15.0 {
            25.0 * (1.0 - (growth_consistency - 5.0) / 10.0)
        } else {
            0.0
        };
        score += consistency_score;

        score.clamp(0.0, 100.0)
    }

    #[test]
    fn test_roe_high_quality() {
        let roe = calculate_roe(500_000.0, 2_000_000.0).unwrap();
        assert_eq!(roe, 25.0);
    }

    #[test]
    fn test_roe_loss_making() {
        let roe = calculate_roe(-100_000.0, 500_000.0).unwrap();
        assert!(roe < 0.0);
    }

    #[test]
    fn test_roe_zero_equity() {
        let roe = calculate_roe(100_000.0, 0.0);
        assert_eq!(roe, None);
    }

    #[test]
    fn test_roic_strong() {
        let roic = calculate_roic(200_000.0, 1_000_000.0).unwrap();
        assert_eq!(roic, 20.0);
    }

    #[test]
    fn test_dividend_yield() {
        let dy = calculate_dividend_yield(4.0, 100.0).unwrap();
        assert_eq!(dy, 4.0);
    }

    #[test]
    fn test_dividend_yield_zero_price() {
        let dy = calculate_dividend_yield(4.0, 0.0);
        assert_eq!(dy, None);
    }

    #[test]
    fn test_payout_ratio_sustainable() {
        let pr = calculate_payout_ratio(2.0, 6.0).unwrap();
        assert!((pr - 33.333).abs() < 0.01);
        assert!(pr < 75.0, "Payout ratio below 75% is sustainable");
    }

    #[test]
    fn test_payout_ratio_unsustainable() {
        let pr = calculate_payout_ratio(8.0, 6.0).unwrap();
        assert!(pr > 100.0, "Payout exceeds earnings");
    }

    #[test]
    fn test_gross_margin_high() {
        let gm = calculate_gross_margin(1_000_000.0, 300_000.0).unwrap();
        assert_eq!(gm, 70.0);
    }

    #[test]
    fn test_revenue_growth_consistency_stable() {
        let rates = vec![10.0, 12.0, 11.0, 9.0, 10.5];
        let consistency = revenue_growth_consistency(&rates);
        assert!(consistency < 5.0, "Stable growth should have low std dev");
    }

    #[test]
    fn test_revenue_growth_consistency_volatile() {
        let rates = vec![30.0, -10.0, 25.0, -5.0, 40.0];
        let consistency = revenue_growth_consistency(&rates);
        assert!(consistency > 15.0, "Volatile growth should have high std dev");
    }

    #[test]
    fn test_quality_score_excellent() {
        let score = quality_score(30.0, 20.0, 60.0, 3.0);
        assert!(score >= 90.0, "Excellent company should score >= 90, got {}", score);
    }

    #[test]
    fn test_quality_score_poor() {
        let score = quality_score(5.0, 3.0, 15.0, 20.0);
        assert!(score < 30.0, "Poor company should score < 30, got {}", score);
    }

    #[test]
    fn test_quality_score_range() {
        // Verify score is always in [0, 100]
        for roe in [0.0, 10.0, 25.0, 50.0] {
            for roic in [0.0, 10.0, 25.0] {
                for gm in [0.0, 30.0, 60.0, 80.0] {
                    for gc in [0.0, 5.0, 15.0, 30.0] {
                        let s = quality_score(roe, roic, gm, gc);
                        assert!(s >= 0.0 && s <= 100.0, "Score out of range: {}", s);
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Filter and Screening Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod screening_filters {
    #[derive(Debug, Clone)]
    struct Stock {
        symbol: String,
        sector: String,
        market_cap: f64,   // in billions
        pe_ratio: f64,
        avg_volume: f64,   // shares/day
        price: f64,
    }

    fn filter_by_sector<'a>(stocks: &'a [Stock], sector: &str) -> Vec<&'a Stock> {
        stocks.iter().filter(|s| s.sector == sector).collect()
    }

    fn filter_by_market_cap_range<'a>(stocks: &'a [Stock], min: f64, max: f64) -> Vec<&'a Stock> {
        stocks.iter().filter(|s| s.market_cap >= min && s.market_cap <= max).collect()
    }

    fn filter_by_liquidity<'a>(stocks: &'a [Stock], min_volume: f64) -> Vec<&'a Stock> {
        stocks.iter().filter(|s| s.avg_volume >= min_volume).collect()
    }

    fn filter_by_price_range<'a>(stocks: &'a [Stock], min: f64, max: f64) -> Vec<&'a Stock> {
        stocks.iter().filter(|s| s.price >= min && s.price <= max).collect()
    }

    fn test_universe() -> Vec<Stock> {
        vec![
            Stock { symbol: "AAPL".into(), sector: "Technology".into(), market_cap: 3000.0, pe_ratio: 28.0, avg_volume: 50_000_000.0, price: 185.0 },
            Stock { symbol: "MSFT".into(), sector: "Technology".into(), market_cap: 2800.0, pe_ratio: 35.0, avg_volume: 25_000_000.0, price: 410.0 },
            Stock { symbol: "JNJ".into(), sector: "Healthcare".into(), market_cap: 400.0, pe_ratio: 15.0, avg_volume: 8_000_000.0, price: 155.0 },
            Stock { symbol: "KO".into(), sector: "Consumer Staples".into(), market_cap: 250.0, pe_ratio: 22.0, avg_volume: 12_000_000.0, price: 60.0 },
            Stock { symbol: "PLTR".into(), sector: "Technology".into(), market_cap: 50.0, pe_ratio: 100.0, avg_volume: 30_000_000.0, price: 22.0 },
            Stock { symbol: "SMLL".into(), sector: "Technology".into(), market_cap: 1.5, pe_ratio: 12.0, avg_volume: 200_000.0, price: 5.0 },
        ]
    }

    #[test]
    fn test_filter_by_sector() {
        let stocks = test_universe();
        let tech = filter_by_sector(&stocks, "Technology");
        assert_eq!(tech.len(), 4);
        assert!(tech.iter().all(|s| s.sector == "Technology"));
    }

    #[test]
    fn test_filter_by_sector_no_match() {
        let stocks = test_universe();
        let energy = filter_by_sector(&stocks, "Energy");
        assert!(energy.is_empty());
    }

    #[test]
    fn test_filter_large_cap() {
        let stocks = test_universe();
        let large = filter_by_market_cap_range(&stocks, 200.0, f64::MAX);
        assert_eq!(large.len(), 4); // AAPL, MSFT, JNJ, KO
    }

    #[test]
    fn test_filter_small_cap() {
        let stocks = test_universe();
        let small = filter_by_market_cap_range(&stocks, 0.0, 10.0);
        assert_eq!(small.len(), 1); // SMLL
        assert_eq!(small[0].symbol, "SMLL");
    }

    #[test]
    fn test_filter_by_liquidity() {
        let stocks = test_universe();
        let liquid = filter_by_liquidity(&stocks, 10_000_000.0);
        assert_eq!(liquid.len(), 4); // Excludes JNJ (8M) and SMLL (200K)
    }

    #[test]
    fn test_filter_by_price_range() {
        let stocks = test_universe();
        let affordable = filter_by_price_range(&stocks, 10.0, 200.0);
        assert_eq!(affordable.len(), 4); // AAPL, JNJ, KO, PLTR
    }

    #[test]
    fn test_combined_filters() {
        let stocks = test_universe();
        // Tech + Large cap + High liquidity
        let result: Vec<&Stock> = stocks.iter()
            .filter(|s| s.sector == "Technology")
            .filter(|s| s.market_cap >= 200.0)
            .filter(|s| s.avg_volume >= 10_000_000.0)
            .collect();
        assert_eq!(result.len(), 2); // AAPL, MSFT
    }

    #[test]
    fn test_empty_universe() {
        let stocks: Vec<Stock> = vec![];
        let result = filter_by_sector(&stocks, "Technology");
        assert!(result.is_empty());
    }
}
