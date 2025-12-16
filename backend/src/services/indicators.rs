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