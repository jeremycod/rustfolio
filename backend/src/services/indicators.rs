/// Simple Moving Average (SMA)
/// Returns a vector aligned with `values`:
/// - `None` until enough values exist
/// - `Some(avg)` after `window` values
pub fn sma(values: &[f64], window: usize) -> Vec<Option<f64>> {
    if window == 0 {
        return vec![None; values.len()];
    }

    // We build a running sum using scan, and subtract the value that falls out of the window.
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

/// Exponential Moving Average (EMA)
/// Uses scan to carry previous EMA as state.
/// - returns `None` until enough values exist (optional behavior)
pub fn ema(values: &[f64], window: usize) -> Vec<Option<f64>> {
    if values.is_empty() || window == 0 {
        return vec![None; values.len()];
    }

    let alpha = 2.0 / (window as f64 + 1.0);

    values
        .iter()
        .enumerate()
        .scan(values[0], move |prev_ema, (i, &v)| {
            let next = alpha * v + (1.0 - alpha) * *prev_ema;
            *prev_ema = next;

            // hide early values until window reached (same behavior as before)
            let out = if i + 1 >= window { Some(next) } else { None };
            Some(out)
        })
        .collect()
}

/// Linear regression trend line for y-values using x = 0..n-1
/// Returns (slope m, intercept b) for y = m*x + b
///
/// Uses iterator folds rather than mutable loops.
pub fn regression_trend(values: &[f64]) -> (f64, f64) {
    let n = values.len();
    if n == 0 {
        return (0.0, 0.0);
    }
    if n == 1 {
        return (0.0, values[0]);
    }

    let n_f = n as f64;

    // Fold over enumerated points to get sums.
    let (sum_x, sum_y, sum_xy, sum_x2) = values
        .iter()
        .enumerate()
        .fold((0.0, 0.0, 0.0, 0.0), |(sx, sy, sxy, sx2), (i, &y)| {
            let x = i as f64;
            (
                sx + x,
                sy + y,
                sxy + x * y,
                sx2 + x * x,
            )
        });

    let denom = n_f * sum_x2 - sum_x * sum_x;
    if denom == 0.0 {
        // fallback: horizontal line at mean
        return (0.0, sum_y / n_f);
    }

    let m = (n_f * sum_xy - sum_x * sum_y) / denom;
    let b = (sum_y - m * sum_x) / n_f;

    (m, b)
}

/// Relative Strength Index (RSI)
///
/// Measures momentum by comparing recent gains to recent losses.
/// RSI values range from 0 to 100:
/// - Below 30: Oversold condition (potential buy signal)
/// - Above 70: Overbought condition (potential sell signal)
/// - 50: Neutral momentum
///
/// Calculation:
/// 1. Calculate price changes (gains and losses)
/// 2. Use EMA to smooth gains and losses over the period
/// 3. RS = Average Gain / Average Loss
/// 4. RSI = 100 - (100 / (1 + RS))
///
/// Returns `None` for the first `period` values, then `Some(rsi)` for subsequent values.
pub fn rsi(prices: &[f64], period: usize) -> Vec<Option<f64>> {
    if prices.len() < 2 || period == 0 {
        return vec![None; prices.len()];
    }

    let mut result = vec![None; prices.len()];

    // Calculate price changes
    let changes: Vec<f64> = prices
        .windows(2)
        .map(|w| w[1] - w[0])
        .collect();

    if changes.is_empty() {
        return result;
    }

    // Separate gains and losses
    let gains: Vec<f64> = changes.iter().map(|&c| if c > 0.0 { c } else { 0.0 }).collect();
    let losses: Vec<f64> = changes.iter().map(|&c| if c < 0.0 { -c } else { 0.0 }).collect();

    // Calculate average gain and loss using EMA
    let alpha = 1.0 / period as f64;

    let mut avg_gain = gains[..period.min(gains.len())]
        .iter()
        .sum::<f64>() / period as f64;
    let mut avg_loss = losses[..period.min(losses.len())]
        .iter()
        .sum::<f64>() / period as f64;

    // First RSI value (at index period)
    if period < prices.len() {
        let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
        result[period] = Some(100.0 - (100.0 / (1.0 + rs)));
    }

    // Calculate subsequent RSI values using EMA smoothing
    for i in period..changes.len() {
        avg_gain = alpha * gains[i] + (1.0 - alpha) * avg_gain;
        avg_loss = alpha * losses[i] + (1.0 - alpha) * avg_loss;

        let rs = if avg_loss == 0.0 { 100.0 } else { avg_gain / avg_loss };
        let rsi_value = 100.0 - (100.0 / (1.0 + rs));

        result[i + 1] = Some(rsi_value);
    }

    result
}

/// Moving Average Convergence Divergence (MACD)
///
/// Trend-following momentum indicator showing relationship between two moving averages.
///
/// Components:
/// - MACD Line: 12-period EMA - 26-period EMA
/// - Signal Line: 9-period EMA of MACD Line
/// - Histogram: MACD Line - Signal Line
///
/// Signals:
/// - MACD crosses above signal: Bullish signal
/// - MACD crosses below signal: Bearish signal
/// - Histogram widens: Strengthening trend
/// - Histogram narrows: Weakening trend
///
/// Returns: (macd_line, signal_line, histogram) as three separate Vec<Option<f64>>
pub fn macd(
    prices: &[f64],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    if prices.is_empty() {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let len = prices.len();

    // Calculate fast and slow EMAs
    let fast_ema = ema(prices, fast_period);
    let slow_ema = ema(prices, slow_period);

    // Calculate MACD line (fast EMA - slow EMA)
    let mut macd_line: Vec<Option<f64>> = vec![None; len];
    for i in 0..len {
        if let (Some(fast), Some(slow)) = (fast_ema[i], slow_ema[i]) {
            macd_line[i] = Some(fast - slow);
        }
    }

    // Extract MACD values for signal line calculation
    let macd_values: Vec<f64> = macd_line
        .iter()
        .filter_map(|&v| v)
        .collect();

    // Calculate signal line (EMA of MACD line)
    let signal_values = ema(&macd_values, signal_period);

    // Map signal values back to original indices
    let mut signal_line: Vec<Option<f64>> = vec![None; len];
    let mut signal_idx = 0;
    for i in 0..len {
        if macd_line[i].is_some() {
            if signal_idx < signal_values.len() {
                signal_line[i] = signal_values[signal_idx];
                signal_idx += 1;
            }
        }
    }

    // Calculate histogram (MACD - signal)
    let mut histogram: Vec<Option<f64>> = vec![None; len];
    for i in 0..len {
        if let (Some(macd), Some(signal)) = (macd_line[i], signal_line[i]) {
            histogram[i] = Some(macd - signal);
        }
    }

    (macd_line, signal_line, histogram)
}

/// Bollinger Bands
///
/// Volatility indicator consisting of a moving average with upper and lower bands
/// based on standard deviation.
///
/// Components:
/// - Middle Band: SMA of prices
/// - Upper Band: Middle Band + (std_dev * num_std_dev)
/// - Lower Band: Middle Band - (std_dev * num_std_dev)
///
/// Standard configuration: 20-period SMA, 2 standard deviations
///
/// Signals:
/// - Price touches upper band: Overbought (potential reversal)
/// - Price touches lower band: Oversold (potential reversal)
/// - Bands squeeze: Low volatility, potential breakout coming
/// - Bands widen: High volatility, strong trend
///
/// Returns: (middle_band, upper_band, lower_band) as three separate Vec<Option<f64>>
pub fn bollinger_bands(
    prices: &[f64],
    period: usize,
    num_std_dev: f64,
) -> (Vec<Option<f64>>, Vec<Option<f64>>, Vec<Option<f64>>) {
    if prices.is_empty() || period == 0 {
        return (Vec::new(), Vec::new(), Vec::new());
    }

    let len = prices.len();

    // Calculate middle band (SMA)
    let middle_band = sma(prices, period);

    // Calculate standard deviation for each window
    let mut upper_band: Vec<Option<f64>> = vec![None; len];
    let mut lower_band: Vec<Option<f64>> = vec![None; len];

    for i in 0..len {
        if i + 1 >= period {
            let window = &prices[i + 1 - period..=i];
            let mean = middle_band[i].unwrap_or(0.0);

            // Calculate standard deviation
            let variance = window
                .iter()
                .map(|&x| {
                    let diff = x - mean;
                    diff * diff
                })
                .sum::<f64>() / period as f64;

            let std_dev = variance.sqrt();

            upper_band[i] = Some(mean + num_std_dev * std_dev);
            lower_band[i] = Some(mean - num_std_dev * std_dev);
        }
    }

    (middle_band, upper_band, lower_band)
}

/// Volume Trend Analysis
///
/// Analyzes volume patterns to confirm price movements and identify potential reversals.
///
/// Components:
/// - Volume SMA: Moving average of volume
/// - Volume Ratio: Current volume / Volume SMA
/// - Volume Trend: Direction and strength of volume changes
///
/// Interpretation:
/// - High volume + price increase: Strong bullish confirmation
/// - High volume + price decrease: Strong bearish confirmation
/// - Low volume + price change: Weak trend, potential reversal
/// - Volume ratio > 1.5: Significantly above average (strong conviction)
/// - Volume ratio < 0.5: Significantly below average (weak conviction)
///
/// Returns: (volume_sma, volume_ratio) as two separate Vec<Option<f64>>
/// - volume_sma: Moving average of volume
/// - volume_ratio: Current volume divided by volume_sma (1.0 = average, >1.0 = above average)
pub fn volume_trend(
    volumes: &[f64],
    period: usize,
) -> (Vec<Option<f64>>, Vec<Option<f64>>) {
    if volumes.is_empty() || period == 0 {
        return (Vec::new(), Vec::new());
    }

    let len = volumes.len();

    // Calculate volume SMA
    let volume_sma = sma(volumes, period);

    // Calculate volume ratio (current volume / average volume)
    let mut volume_ratio: Vec<Option<f64>> = vec![None; len];

    for i in 0..len {
        if let Some(avg_vol) = volume_sma[i] {
            if avg_vol > 0.0 {
                volume_ratio[i] = Some(volumes[i] / avg_vol);
            }
        }
    }

    (volume_sma, volume_ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_basic() {
        // Test data: alternating gains and losses
        let prices = vec![44.0, 44.5, 44.0, 45.0, 44.5, 45.5, 45.0, 46.0, 46.5, 46.0,
                         47.0, 46.5, 47.5, 47.0, 48.0, 48.5];
        let rsi_values = rsi(&prices, 14);

        // First 14 values should be None
        for i in 0..14 {
            assert!(rsi_values[i].is_none());
        }

        // After period, should have RSI values between 0 and 100
        for i in 14..rsi_values.len() {
            if let Some(rsi_val) = rsi_values[i] {
                assert!(rsi_val >= 0.0 && rsi_val <= 100.0);
            }
        }
    }

    #[test]
    fn test_rsi_oversold_overbought() {
        // Strong uptrend should give high RSI (overbought)
        let uptrend: Vec<f64> = (0..30).map(|i| 50.0 + i as f64).collect();
        let rsi_values = rsi(&uptrend, 14);

        if let Some(last_rsi) = rsi_values.last().and_then(|&v| v) {
            assert!(last_rsi > 70.0, "Strong uptrend should show overbought RSI");
        }

        // Strong downtrend should give low RSI (oversold)
        let downtrend: Vec<f64> = (0..30).map(|i| 80.0 - i as f64).collect();
        let rsi_values = rsi(&downtrend, 14);

        if let Some(last_rsi) = rsi_values.last().and_then(|&v| v) {
            assert!(last_rsi < 30.0, "Strong downtrend should show oversold RSI");
        }
    }

    #[test]
    fn test_macd_basic() {
        let prices: Vec<f64> = (0..50).map(|i| 100.0 + (i as f64 * 0.5)).collect();
        let (macd_line, signal_line, histogram) = macd(&prices, 12, 26, 9);

        assert_eq!(macd_line.len(), prices.len());
        assert_eq!(signal_line.len(), prices.len());
        assert_eq!(histogram.len(), prices.len());

        // In uptrend, MACD should eventually be positive
        if let Some(last_macd) = macd_line.last().and_then(|&v| v) {
            assert!(last_macd > 0.0, "Uptrend should have positive MACD");
        }
    }

    #[test]
    fn test_bollinger_bands_basic() {
        let prices: Vec<f64> = vec![100.0; 30]; // Flat prices
        let (middle, upper, lower) = bollinger_bands(&prices, 20, 2.0);

        assert_eq!(middle.len(), prices.len());
        assert_eq!(upper.len(), prices.len());
        assert_eq!(lower.len(), prices.len());

        // For flat prices, middle should equal price, upper should be above, lower below
        if let (Some(mid), Some(up), Some(low)) = (middle[25], upper[25], lower[25]) {
            assert!((mid - 100.0).abs() < 0.01);
            assert!(up >= mid);
            assert!(low <= mid);
        }
    }

    #[test]
    fn test_bollinger_bands_volatility() {
        // High volatility data
        let volatile: Vec<f64> = (0..30).map(|i| 100.0 + ((i as f64 * 2.0).sin() * 10.0)).collect();
        let (_, upper_vol, lower_vol) = bollinger_bands(&volatile, 20, 2.0);

        // Low volatility data
        let stable: Vec<f64> = vec![100.0; 30];
        let (_, upper_stable, lower_stable) = bollinger_bands(&stable, 20, 2.0);

        // Volatile data should have wider bands
        if let (Some(vol_width), Some(stable_width)) = (
            upper_vol[25].and_then(|u| lower_vol[25].map(|l| u - l)),
            upper_stable[25].and_then(|u| lower_stable[25].map(|l| u - l)),
        ) {
            assert!(vol_width > stable_width, "Volatile data should have wider Bollinger Bands");
        }
    }

    #[test]
    fn test_volume_trend_basic() {
        let volumes = vec![1000.0, 1100.0, 1200.0, 1050.0, 1150.0, 2000.0, 1800.0, 1000.0];
        let (vol_sma, vol_ratio) = volume_trend(&volumes, 5);

        assert_eq!(vol_sma.len(), volumes.len());
        assert_eq!(vol_ratio.len(), volumes.len());

        // Volume spike at index 5 (2000.0) should show high ratio
        if let Some(ratio) = vol_ratio[6] {
            assert!(ratio > 1.0, "High volume should show ratio > 1.0");
        }
    }

    #[test]
    fn test_volume_trend_ratio() {
        // Consistent volume should have ratio around 1.0
        let consistent = vec![1000.0; 20];
        let (_, vol_ratio) = volume_trend(&consistent, 10);

        if let Some(ratio) = vol_ratio[15] {
            assert!((ratio - 1.0).abs() < 0.01, "Consistent volume should have ratio ≈ 1.0");
        }
    }
}