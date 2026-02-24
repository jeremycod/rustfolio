use chrono::Utc;
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

use crate::models::screening::*;
use crate::services::indicators::{sma, rsi};

pub struct ScreeningService {
    pool: PgPool,
}

impl ScreeningService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // -----------------------------------------------------------------------
    // Public entry point
    // -----------------------------------------------------------------------

    pub async fn screen(&self, req: &ScreeningRequest) -> Result<ScreeningResponse, String> {
        let weights = req.weights.resolve(req.risk_appetite, req.horizon_months);
        let cache_key = self.build_cache_key(req);

        // Try cache first
        if !req.refresh {
            if let Some(cached) = self.get_cached(&cache_key).await {
                info!("Screening cache hit for key {}", cache_key);
                return Ok(cached);
            }
        }

        // 1. Build the universe of tickers
        let tickers = self.resolve_universe(&req.symbols).await?;
        let total_screened = tickers.len();
        info!("Screening universe: {} tickers", total_screened);

        // 2. Fetch data for all tickers (prices + any fundamental/sentiment data)
        let ticker_data = self.fetch_ticker_data(&tickers).await?;

        // 3. Apply pre-scoring filters
        let filtered: Vec<_> = ticker_data
            .into_iter()
            .filter(|d| self.passes_filters(d, &req.filters))
            .collect();
        let total_passed = filtered.len();
        info!("{} tickers passed filters", total_passed);

        // 4. Score every ticker across all factor dimensions
        let mut scored: Vec<ScreeningResult> = filtered
            .iter()
            .map(|d| self.score_ticker(d, &weights))
            .collect();

        // 5. Sort descending by composite score and assign ranks
        scored.sort_by(|a, b| b.composite_score.partial_cmp(&a.composite_score).unwrap_or(std::cmp::Ordering::Equal));
        for (i, r) in scored.iter_mut().enumerate() {
            r.rank = i + 1;
        }

        // 6. Paginate
        let page: Vec<ScreeningResult> = scored
            .into_iter()
            .skip(req.offset)
            .take(req.limit)
            .collect();

        let response = ScreeningResponse {
            results: page,
            total_screened,
            total_passed_filters: total_passed,
            weights_used: weights,
            screened_at: Utc::now(),
            cache_hit: false,
            limit: req.limit,
            offset: req.offset,
        };

        // Store in cache (fire-and-forget)
        if let Err(e) = self.store_cache(&cache_key, &response).await {
            warn!("Failed to store screening cache: {}", e);
        }

        Ok(response)
    }

    // -----------------------------------------------------------------------
    // Universe resolution
    // -----------------------------------------------------------------------

    async fn resolve_universe(&self, explicit: &[String]) -> Result<Vec<String>, String> {
        if !explicit.is_empty() {
            return Ok(explicit.iter().map(|s| s.to_uppercase()).collect());
        }
        // Fetch all distinct tickers that have price data
        let rows: Vec<(String,)> = sqlx::query_as(
            "SELECT DISTINCT ticker FROM price_points ORDER BY ticker"
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Failed to fetch ticker universe: {}", e))?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    // -----------------------------------------------------------------------
    // Data fetching
    // -----------------------------------------------------------------------

    async fn fetch_ticker_data(&self, tickers: &[String]) -> Result<Vec<TickerData>, String> {
        let mut result = Vec::with_capacity(tickers.len());

        for ticker in tickers {
            match self.fetch_single_ticker(ticker).await {
                Ok(d) => result.push(d),
                Err(e) => {
                    warn!("Skipping ticker {} due to data error: {}", ticker, e);
                }
            }
        }

        Ok(result)
    }

    async fn fetch_single_ticker(&self, ticker: &str) -> Result<TickerData, String> {
        // Fetch prices (newest first in DB, we reverse for indicator math)
        let price_rows: Vec<(f64,)> = sqlx::query_as(
            r#"SELECT CAST(close_price AS DOUBLE PRECISION)
               FROM price_points
               WHERE ticker = $1
               ORDER BY date ASC
               LIMIT 365"#,
        )
        .bind(ticker)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("price query: {}", e))?;

        if price_rows.len() < 30 {
            return Err(format!("Insufficient price data for {}", ticker));
        }

        let prices: Vec<f64> = price_rows.iter().map(|r| r.0).collect();
        let current_price = *prices.last().unwrap_or(&0.0);

        // Fetch latest sentiment score if available
        let sentiment_row: Option<(f64,)> = sqlx::query_as(
            r#"SELECT CAST(sentiment_score AS DOUBLE PRECISION)
               FROM sentiment_signal_cache
               WHERE ticker = $1
               ORDER BY generated_at DESC
               LIMIT 1"#,
        )
        .bind(ticker)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None);

        let sentiment_score = sentiment_row.map(|r| r.0);

        // Fetch sector/industry from latest_account_holdings view if available
        let sector_row: Option<(Option<String>,)> = sqlx::query_as(
            r#"SELECT industry
               FROM latest_account_holdings
               WHERE ticker = $1
               LIMIT 1"#,
        )
        .bind(ticker)
        .fetch_optional(&self.pool)
        .await
        .unwrap_or(None);

        let sector = sector_row.and_then(|r| r.0);

        Ok(TickerData {
            symbol: ticker.to_string(),
            prices,
            current_price,
            sentiment_score,
            sector,
            // We don't have real volume data in this schema, so we'll skip volume-based filters.
            avg_volume: None,
            market_cap: None,
            geography: None,
        })
    }

    // -----------------------------------------------------------------------
    // Filtering
    // -----------------------------------------------------------------------

    fn passes_filters(&self, data: &TickerData, filters: &ScreeningFilters) -> bool {
        // Sector filter
        if !filters.sectors.is_empty() {
            if let Some(ref sector) = data.sector {
                let matches = filters.sectors.iter().any(|s| {
                    sector.to_lowercase().contains(&s.to_lowercase())
                });
                if !matches {
                    return false;
                }
            } else {
                // No sector info -- exclude if sector filter is active
                return false;
            }
        }

        // Price filter
        if let Some(min) = filters.min_price {
            if data.current_price < min {
                return false;
            }
        }
        if let Some(max) = filters.max_price {
            if data.current_price > max {
                return false;
            }
        }

        // Volume filter
        if let Some(min_vol) = filters.min_avg_volume {
            if let Some(avg) = data.avg_volume {
                if avg < min_vol {
                    return false;
                }
            }
            // If we don't have volume data, skip this filter
        }

        // Market cap filter (when data available)
        if let Some(ref cap_range) = filters.market_cap {
            if let Some(cap) = data.market_cap {
                let pass = match cap_range {
                    MarketCapRange::Small => cap < 2_000_000_000.0,
                    MarketCapRange::Mid => (2_000_000_000.0..10_000_000_000.0).contains(&cap),
                    MarketCapRange::Large => (10_000_000_000.0..200_000_000_000.0).contains(&cap),
                    MarketCapRange::Mega => cap >= 200_000_000_000.0,
                };
                if !pass {
                    return false;
                }
            }
        }

        // Geography filter
        if !filters.geographies.is_empty() {
            if let Some(ref geo) = data.geography {
                if !filters.geographies.iter().any(|g| g.eq_ignore_ascii_case(geo)) {
                    return false;
                }
            }
        }

        true
    }

    // -----------------------------------------------------------------------
    // Scoring
    // -----------------------------------------------------------------------

    fn score_ticker(&self, data: &TickerData, weights: &ResolvedWeights) -> ScreeningResult {
        let fundamental = self.score_fundamentals(data);
        let technical = self.score_technicals(data);
        let sentiment = self.score_sentiment(data);
        let momentum = self.score_momentum(data);

        let composite = fundamental.composite * weights.fundamental
            + technical.composite * weights.technical
            + sentiment.composite * weights.sentiment
            + momentum.composite * weights.momentum;

        let explanation = self.build_explanation(data, &fundamental, &technical, &sentiment, &momentum, composite);

        ScreeningResult {
            symbol: data.symbol.clone(),
            composite_score: composite,
            rank: 0, // assigned after sorting
            fundamental,
            technical,
            sentiment,
            momentum,
            weights_used: weights.clone(),
            explanation,
        }
    }

    // ---- Fundamental metrics ----

    fn score_fundamentals(&self, data: &TickerData) -> FundamentalScore {
        let prices = &data.prices;
        let mut details = Vec::new();
        let mut scores: Vec<f64> = Vec::new();

        // P/E proxy: We don't have actual earnings data, so we derive a pseudo-valuation
        // metric from price change stability (low volatility = value-like).
        let pe_score = self.pseudo_pe_score(prices);
        details.push(ScoreDetail {
            metric: "P/E Proxy (volatility-adjusted)".into(),
            raw_value: Some(pe_score),
            score: pe_score,
            interpretation: if pe_score > 60.0 {
                "Attractively valued (low volatility, steady growth)".into()
            } else if pe_score > 40.0 {
                "Fairly valued".into()
            } else {
                "Appears expensive or volatile".into()
            },
        });
        scores.push(pe_score);

        // P/B proxy: price relative to long-term average
        let pb_score = self.price_to_avg_score(prices);
        details.push(ScoreDetail {
            metric: "P/B Proxy (price vs long-term avg)".into(),
            raw_value: Some(pb_score),
            score: pb_score,
            interpretation: if pb_score > 60.0 {
                "Trading below long-term average".into()
            } else {
                "Trading near or above long-term average".into()
            },
        });
        scores.push(pb_score);

        // PEG proxy: growth-adjusted valuation
        let peg_score = self.peg_proxy_score(prices);
        details.push(ScoreDetail {
            metric: "PEG Proxy (growth-adjusted)".into(),
            raw_value: Some(peg_score),
            score: peg_score,
            interpretation: if peg_score > 60.0 {
                "Good growth relative to price movement".into()
            } else {
                "Growth may not justify current price".into()
            },
        });
        scores.push(peg_score);

        // Debt-to-equity proxy: drawdown severity
        let de_score = self.drawdown_stability_score(prices);
        details.push(ScoreDetail {
            metric: "D/E Proxy (max drawdown stability)".into(),
            raw_value: Some(de_score),
            score: de_score,
            interpretation: if de_score > 60.0 {
                "Low drawdown suggests financial stability".into()
            } else {
                "Higher drawdowns may indicate leverage risk".into()
            },
        });
        scores.push(de_score);

        // Earnings growth proxy: rolling return consistency
        let eg_score = self.earnings_growth_proxy(prices);
        details.push(ScoreDetail {
            metric: "Earnings Growth Proxy (return consistency)".into(),
            raw_value: Some(eg_score),
            score: eg_score,
            interpretation: if eg_score > 60.0 {
                "Consistent positive returns".into()
            } else {
                "Inconsistent or negative returns".into()
            },
        });
        scores.push(eg_score);

        let composite = if scores.is_empty() {
            50.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        FundamentalScore {
            pe_score: Some(pe_score),
            pb_score: Some(pb_score),
            peg_score: Some(peg_score),
            debt_to_equity_score: Some(de_score),
            earnings_growth_score: Some(eg_score),
            composite,
            details,
        }
    }

    /// Lower volatility relative to return => higher "value" score (0-100).
    fn pseudo_pe_score(&self, prices: &[f64]) -> f64 {
        if prices.len() < 30 {
            return 50.0;
        }
        let returns: Vec<f64> = prices.windows(2).map(|w| (w[1] - w[0]) / w[0]).collect();
        let mean_ret = returns.iter().sum::<f64>() / returns.len() as f64;
        let var = returns.iter().map(|r| (r - mean_ret).powi(2)).sum::<f64>() / returns.len() as f64;
        let std_dev = var.sqrt();

        if std_dev == 0.0 {
            return 50.0;
        }
        // Sharpe-like: higher ratio => higher score
        let ratio = mean_ret / std_dev;
        // Map ratio roughly in [-0.3, 0.3] to [0, 100]
        ((ratio + 0.3) / 0.6 * 100.0).clamp(0.0, 100.0)
    }

    /// Price below long-term average => good "P/B" signal.
    fn price_to_avg_score(&self, prices: &[f64]) -> f64 {
        if prices.is_empty() {
            return 50.0;
        }
        let avg = prices.iter().sum::<f64>() / prices.len() as f64;
        let current = *prices.last().unwrap();
        if avg == 0.0 {
            return 50.0;
        }
        let ratio = current / avg;
        // ratio 0.8 => score 70, ratio 1.0 => 50, ratio 1.2 => 30
        (100.0 - (ratio - 0.5) * 100.0).clamp(0.0, 100.0)
    }

    /// Growth-adjusted: positive trend + low dispersion => high score.
    fn peg_proxy_score(&self, prices: &[f64]) -> f64 {
        if prices.len() < 60 {
            return 50.0;
        }
        let half = prices.len() / 2;
        let first_half_avg = prices[..half].iter().sum::<f64>() / half as f64;
        let second_half_avg = prices[half..].iter().sum::<f64>() / (prices.len() - half) as f64;

        if first_half_avg == 0.0 {
            return 50.0;
        }
        let growth = (second_half_avg - first_half_avg) / first_half_avg;
        // Map growth [-0.2, 0.4] to [0, 100]
        ((growth + 0.2) / 0.6 * 100.0).clamp(0.0, 100.0)
    }

    /// Lower max drawdown => higher financial stability score.
    fn drawdown_stability_score(&self, prices: &[f64]) -> f64 {
        if prices.is_empty() {
            return 50.0;
        }
        let mut peak = prices[0];
        let mut max_dd = 0.0_f64;
        for &p in prices {
            if p > peak {
                peak = p;
            }
            let dd = (peak - p) / peak;
            if dd > max_dd {
                max_dd = dd;
            }
        }
        // max_dd 0% => 100, 50% => 0
        (100.0 - max_dd * 200.0).clamp(0.0, 100.0)
    }

    /// Fraction of positive monthly returns.
    fn earnings_growth_proxy(&self, prices: &[f64]) -> f64 {
        if prices.len() < 22 {
            return 50.0;
        }
        let step = 21; // ~1 month of trading days
        let monthly: Vec<f64> = prices
            .windows(step + 1)
            .step_by(step)
            .map(|w| (w[step] - w[0]) / w[0])
            .collect();

        if monthly.is_empty() {
            return 50.0;
        }
        let positive_frac = monthly.iter().filter(|&&r| r > 0.0).count() as f64 / monthly.len() as f64;
        (positive_frac * 100.0).clamp(0.0, 100.0)
    }

    // ---- Technical indicators ----

    fn score_technicals(&self, data: &TickerData) -> TechnicalScore {
        let prices = &data.prices;
        let mut details = Vec::new();
        let mut scores: Vec<f64> = Vec::new();

        // Moving average crossover (SMA 50 vs SMA 200)
        let ma_score = self.ma_crossover_score(prices);
        details.push(ScoreDetail {
            metric: "MA Crossover (SMA50 vs SMA200)".into(),
            raw_value: Some(ma_score),
            score: ma_score,
            interpretation: if ma_score > 60.0 {
                "Golden cross / bullish alignment".into()
            } else if ma_score < 40.0 {
                "Death cross / bearish alignment".into()
            } else {
                "Neutral alignment".into()
            },
        });
        scores.push(ma_score);

        // RSI score
        let rsi_score_val = self.rsi_scoring(prices);
        details.push(ScoreDetail {
            metric: "RSI (14-period)".into(),
            raw_value: Some(rsi_score_val),
            score: rsi_score_val,
            interpretation: if rsi_score_val > 65.0 {
                "RSI in favorable zone".into()
            } else if rsi_score_val < 35.0 {
                "RSI signals caution".into()
            } else {
                "RSI neutral".into()
            },
        });
        scores.push(rsi_score_val);

        // Relative strength vs market proxy (comparison to own long-term avg)
        let rs_score = self.relative_strength_score(prices);
        details.push(ScoreDetail {
            metric: "Relative Strength".into(),
            raw_value: Some(rs_score),
            score: rs_score,
            interpretation: if rs_score > 60.0 {
                "Outperforming trend".into()
            } else {
                "Underperforming or in line".into()
            },
        });
        scores.push(rs_score);

        // Volume (we don't have real volume, score 50 neutral)
        let vol_score = 50.0;
        details.push(ScoreDetail {
            metric: "Volume Trend".into(),
            raw_value: None,
            score: vol_score,
            interpretation: "Volume data not available; neutral score applied".into(),
        });
        scores.push(vol_score);

        let composite = if scores.is_empty() {
            50.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        TechnicalScore {
            ma_crossover_score: Some(ma_score),
            rsi_score: Some(rsi_score_val),
            relative_strength_score: Some(rs_score),
            volume_score: Some(vol_score),
            composite,
            details,
        }
    }

    fn ma_crossover_score(&self, prices: &[f64]) -> f64 {
        let sma50 = sma(prices, 50);
        let sma200 = sma(prices, 200);

        if let (Some(s50), Some(s200)) = (
            sma50.last().and_then(|v| *v),
            sma200.last().and_then(|v| *v),
        ) {
            if s200 == 0.0 {
                return 50.0;
            }
            let diff_pct = (s50 - s200) / s200;
            // diff_pct in [-0.1, 0.1] => score [0, 100]
            ((diff_pct + 0.1) / 0.2 * 100.0).clamp(0.0, 100.0)
        } else {
            // Not enough data for SMA200 -- try shorter
            let sma20 = sma(prices, 20);
            let sma50_v = sma(prices, 50);
            if let (Some(s20), Some(s50)) = (
                sma20.last().and_then(|v| *v),
                sma50_v.last().and_then(|v| *v),
            ) {
                if s50 == 0.0 {
                    return 50.0;
                }
                let diff_pct = (s20 - s50) / s50;
                ((diff_pct + 0.05) / 0.10 * 100.0).clamp(0.0, 100.0)
            } else {
                50.0
            }
        }
    }

    fn rsi_scoring(&self, prices: &[f64]) -> f64 {
        let rsi_values = rsi(prices, 14);
        if let Some(current_rsi) = rsi_values.last().and_then(|v| *v) {
            // RSI 30-70 is the "normal" range.
            // Oversold (<30) is bullish opportunity => high score
            // Overbought (>70) is bearish risk => lower score
            // Ideal RSI for buying is around 40-60
            if current_rsi < 30.0 {
                // Oversold -- great buying opportunity
                80.0 + (30.0 - current_rsi) / 30.0 * 20.0
            } else if current_rsi < 50.0 {
                // Mildly bullish
                60.0 + (50.0 - current_rsi) / 20.0 * 20.0
            } else if current_rsi < 70.0 {
                // Neutral to slightly overbought
                60.0 - (current_rsi - 50.0) / 20.0 * 20.0
            } else {
                // Overbought -- caution
                40.0 - (current_rsi - 70.0) / 30.0 * 40.0
            }
            .clamp(0.0, 100.0)
        } else {
            50.0
        }
    }

    fn relative_strength_score(&self, prices: &[f64]) -> f64 {
        if prices.len() < 60 {
            return 50.0;
        }
        let recent = prices.len().saturating_sub(20);
        let recent_avg = prices[recent..].iter().sum::<f64>() / (prices.len() - recent) as f64;
        let long_avg = prices.iter().sum::<f64>() / prices.len() as f64;

        if long_avg == 0.0 {
            return 50.0;
        }
        let rs = recent_avg / long_avg;
        // rs > 1 = outperforming, map [0.8, 1.2] => [0, 100]
        ((rs - 0.8) / 0.4 * 100.0).clamp(0.0, 100.0)
    }

    // ---- Sentiment scoring ----

    fn score_sentiment(&self, data: &TickerData) -> SentimentScore {
        let mut details = Vec::new();
        let mut scores: Vec<f64> = Vec::new();

        // News sentiment score
        if let Some(sent) = data.sentiment_score {
            // sent is in [-1.0, 1.0], map to [0, 100]
            let news_score = (sent + 1.0) / 2.0 * 100.0;
            details.push(ScoreDetail {
                metric: "News Sentiment".into(),
                raw_value: Some(sent),
                score: news_score,
                interpretation: if sent > 0.3 {
                    "Positive news sentiment".into()
                } else if sent < -0.3 {
                    "Negative news sentiment".into()
                } else {
                    "Neutral news sentiment".into()
                },
            });
            scores.push(news_score);

            // Sentiment trend (we only have latest, so use price trend as proxy)
            let trend_score = self.sentiment_trend_from_price(data);
            details.push(ScoreDetail {
                metric: "Sentiment Trend (price-proxy)".into(),
                raw_value: Some(trend_score),
                score: trend_score,
                interpretation: if trend_score > 60.0 {
                    "Improving sentiment trajectory".into()
                } else {
                    "Flat or declining sentiment trajectory".into()
                },
            });
            scores.push(trend_score);
        } else {
            details.push(ScoreDetail {
                metric: "News Sentiment".into(),
                raw_value: None,
                score: 50.0,
                interpretation: "No sentiment data available; neutral score applied".into(),
            });
            scores.push(50.0);
        }

        let composite = if scores.is_empty() {
            50.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        SentimentScore {
            news_sentiment_score: data.sentiment_score.map(|s| (s + 1.0) / 2.0 * 100.0),
            sentiment_trend_score: Some(self.sentiment_trend_from_price(data)),
            composite,
            details,
        }
    }

    fn sentiment_trend_from_price(&self, data: &TickerData) -> f64 {
        let prices = &data.prices;
        if prices.len() < 10 {
            return 50.0;
        }
        let recent_5 = &prices[prices.len() - 5..];
        let prev_5 = &prices[prices.len() - 10..prices.len() - 5];
        let recent_avg = recent_5.iter().sum::<f64>() / 5.0;
        let prev_avg = prev_5.iter().sum::<f64>() / 5.0;

        if prev_avg == 0.0 {
            return 50.0;
        }
        let change = (recent_avg - prev_avg) / prev_avg;
        // Map [-0.05, 0.05] => [0, 100]
        ((change + 0.05) / 0.10 * 100.0).clamp(0.0, 100.0)
    }

    // ---- Momentum factors ----

    fn score_momentum(&self, data: &TickerData) -> MomentumScore {
        let prices = &data.prices;
        let mut details = Vec::new();
        let mut scores: Vec<f64> = Vec::new();

        // 1-month momentum (~21 trading days)
        let mom_1m = self.price_momentum(prices, 21);
        details.push(ScoreDetail {
            metric: "1-Month Momentum".into(),
            raw_value: Some(mom_1m),
            score: mom_1m,
            interpretation: format!("1M return-based momentum score: {:.0}", mom_1m),
        });
        scores.push(mom_1m);

        // 3-month momentum (~63 trading days)
        let mom_3m = self.price_momentum(prices, 63);
        details.push(ScoreDetail {
            metric: "3-Month Momentum".into(),
            raw_value: Some(mom_3m),
            score: mom_3m,
            interpretation: format!("3M return-based momentum score: {:.0}", mom_3m),
        });
        scores.push(mom_3m);

        // 6-month momentum (~126 trading days)
        let mom_6m = self.price_momentum(prices, 126);
        details.push(ScoreDetail {
            metric: "6-Month Momentum".into(),
            raw_value: Some(mom_6m),
            score: mom_6m,
            interpretation: format!("6M return-based momentum score: {:.0}", mom_6m),
        });
        scores.push(mom_6m);

        // 12-month momentum (~252 trading days)
        let mom_12m = self.price_momentum(prices, 252);
        details.push(ScoreDetail {
            metric: "12-Month Momentum".into(),
            raw_value: Some(mom_12m),
            score: mom_12m,
            interpretation: format!("12M return-based momentum score: {:.0}", mom_12m),
        });
        scores.push(mom_12m);

        // Acceleration: recent momentum accelerating?
        let accel = self.momentum_acceleration(prices);
        details.push(ScoreDetail {
            metric: "Momentum Acceleration".into(),
            raw_value: Some(accel),
            score: accel,
            interpretation: if accel > 60.0 {
                "Momentum accelerating".into()
            } else if accel < 40.0 {
                "Momentum decelerating".into()
            } else {
                "Steady momentum".into()
            },
        });
        scores.push(accel);

        let composite = if scores.is_empty() {
            50.0
        } else {
            scores.iter().sum::<f64>() / scores.len() as f64
        };

        MomentumScore {
            momentum_1m: Some(mom_1m),
            momentum_3m: Some(mom_3m),
            momentum_6m: Some(mom_6m),
            momentum_12m: Some(mom_12m),
            volume_momentum: None, // No volume data
            acceleration: Some(accel),
            composite,
            details,
        }
    }

    fn price_momentum(&self, prices: &[f64], lookback: usize) -> f64 {
        if prices.len() <= lookback {
            return 50.0;
        }
        let current = *prices.last().unwrap();
        let past = prices[prices.len() - 1 - lookback];
        if past == 0.0 {
            return 50.0;
        }
        let ret = (current - past) / past;
        // Map return [-0.3, 0.3] => [0, 100]
        ((ret + 0.3) / 0.6 * 100.0).clamp(0.0, 100.0)
    }

    fn momentum_acceleration(&self, prices: &[f64]) -> f64 {
        // Compare 1-month momentum to 3-month average momentum
        let mom_1m = self.price_momentum(prices, 21);
        let mom_3m = self.price_momentum(prices, 63);

        // If recent momentum exceeds longer-term, acceleration is positive
        let diff = mom_1m - mom_3m;
        // Map diff [-30, 30] => [0, 100]
        ((diff + 30.0) / 60.0 * 100.0).clamp(0.0, 100.0)
    }

    // -----------------------------------------------------------------------
    // Explanation generation
    // -----------------------------------------------------------------------

    fn build_explanation(
        &self,
        data: &TickerData,
        fund: &FundamentalScore,
        tech: &TechnicalScore,
        sent: &SentimentScore,
        mom: &MomentumScore,
        composite: f64,
    ) -> String {
        let mut parts = Vec::new();

        // Find the strongest factor
        let factors = [
            ("Fundamentals", fund.composite),
            ("Technicals", tech.composite),
            ("Sentiment", sent.composite),
            ("Momentum", mom.composite),
        ];

        let mut sorted = factors.to_vec();
        sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        parts.push(format!(
            "{} scores {:.0}/100 overall.",
            data.symbol, composite,
        ));

        parts.push(format!(
            "Strongest factor: {} ({:.0}/100). ",
            sorted[0].0, sorted[0].1,
        ));

        // Top details
        if let Some(top_detail) = fund.details.first() {
            parts.push(format!("Fundamental: {}.", top_detail.interpretation));
        }
        if let Some(top_detail) = tech.details.first() {
            parts.push(format!("Technical: {}.", top_detail.interpretation));
        }
        if let Some(top_detail) = mom.details.first() {
            parts.push(format!("Momentum: {}.", top_detail.interpretation));
        }

        parts.join(" ")
    }

    // -----------------------------------------------------------------------
    // Caching
    // -----------------------------------------------------------------------

    fn build_cache_key(&self, req: &ScreeningRequest) -> String {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        let mut h = DefaultHasher::new();
        // Include relevant request fields in the key
        let mut symbols = req.symbols.clone();
        symbols.sort();
        for s in &symbols {
            s.hash(&mut h);
        }
        format!("{:?}", req.risk_appetite).hash(&mut h);
        req.horizon_months.hash(&mut h);
        format!("{:?}", req.filters.sectors).hash(&mut h);
        format!("{:?}", req.filters.market_cap).hash(&mut h);

        format!("screen_{:x}", h.finish())
    }

    async fn get_cached(&self, cache_key: &str) -> Option<ScreeningResponse> {
        let row: Option<(serde_json::Value, i32, i32)> = sqlx::query_as(
            r#"SELECT results_json, total_screened, total_passed_filters
               FROM screening_cache
               WHERE cache_key = $1 AND expires_at > NOW()
               ORDER BY created_at DESC
               LIMIT 1"#,
        )
        .bind(cache_key)
        .fetch_optional(&self.pool)
        .await
        .ok()?;

        let (json_val, total_screened, total_passed) = row?;
        let results: Vec<ScreeningResult> = serde_json::from_value(json_val).ok()?;

        Some(ScreeningResponse {
            results,
            total_screened: total_screened as usize,
            total_passed_filters: total_passed as usize,
            weights_used: ResolvedWeights {
                fundamental: 0.0,
                technical: 0.0,
                sentiment: 0.0,
                momentum: 0.0,
            },
            screened_at: Utc::now(),
            cache_hit: true,
            limit: 0,
            offset: 0,
        })
    }

    async fn store_cache(&self, cache_key: &str, response: &ScreeningResponse) -> Result<(), String> {
        let json_val = serde_json::to_value(&response.results)
            .map_err(|e| format!("JSON serialization: {}", e))?;

        sqlx::query(
            r#"INSERT INTO screening_cache (id, cache_key, results_json, total_screened, total_passed_filters, created_at, expires_at)
               VALUES ($1, $2, $3, $4, $5, NOW(), NOW() + INTERVAL '15 minutes')
               ON CONFLICT (cache_key) DO UPDATE
                 SET results_json = EXCLUDED.results_json,
                     total_screened = EXCLUDED.total_screened,
                     total_passed_filters = EXCLUDED.total_passed_filters,
                     created_at = EXCLUDED.created_at,
                     expires_at = EXCLUDED.expires_at"#,
        )
        .bind(Uuid::new_v4())
        .bind(cache_key)
        .bind(&json_val)
        .bind(response.total_screened as i32)
        .bind(response.total_passed_filters as i32)
        .execute(&self.pool)
        .await
        .map_err(|e| format!("Cache insert: {}", e))?;

        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Internal data carrier
// ---------------------------------------------------------------------------

struct TickerData {
    symbol: String,
    prices: Vec<f64>,
    current_price: f64,
    sentiment_score: Option<f64>,
    sector: Option<String>,
    avg_volume: Option<f64>,
    market_cap: Option<f64>,
    geography: Option<String>,
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_prices(n: usize, base: f64, trend: f64) -> Vec<f64> {
        (0..n).map(|i| base + trend * i as f64 + (i as f64 * 0.3).sin() * 2.0).collect()
    }

    fn make_ticker(prices: Vec<f64>) -> TickerData {
        let current_price = *prices.last().unwrap_or(&100.0);
        TickerData {
            symbol: "TEST".into(),
            prices,
            current_price,
            sentiment_score: Some(0.3),
            sector: Some("Technology".into()),
            avg_volume: Some(1_000_000.0),
            market_cap: Some(50_000_000_000.0),
            geography: Some("US".into()),
        }
    }

    fn test_service() -> ScreeningService {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let pool = rt.block_on(async {
            PgPool::connect_lazy("postgres://test:test@localhost/test").unwrap()
        });
        ScreeningService { pool }
    }

    #[test]
    fn test_factor_weights_resolve_defaults() {
        let w = FactorWeights::default();
        let resolved = w.resolve(None, None);
        assert!((resolved.fundamental + resolved.technical + resolved.sentiment + resolved.momentum - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_factor_weights_conservative() {
        let w = FactorWeights::default();
        let resolved = w.resolve(Some(RiskAppetite::Conservative), None);
        assert!(resolved.fundamental > resolved.momentum, "Conservative should weight fundamentals more");
    }

    #[test]
    fn test_factor_weights_aggressive() {
        let w = FactorWeights::default();
        let resolved = w.resolve(Some(RiskAppetite::Aggressive), None);
        assert!(resolved.momentum > resolved.fundamental, "Aggressive should weight momentum more");
    }

    #[test]
    fn test_pseudo_pe_score_stable() {
        let service = test_service();
        let prices = make_prices(200, 100.0, 0.1);
        let score = service.pseudo_pe_score(&prices);
        assert!(score > 50.0, "Stable uptrend should have high PE score, got {}", score);
    }

    #[test]
    fn test_ma_crossover_uptrend() {
        let service = test_service();
        let prices = make_prices(250, 50.0, 0.2);
        let score = service.ma_crossover_score(&prices);
        assert!(score > 50.0, "Strong uptrend MA crossover should score high, got {}", score);
    }

    #[test]
    fn test_rsi_scoring_oversold() {
        let service = test_service();
        let prices: Vec<f64> = (0..60).map(|i| 100.0 - i as f64 * 0.5).collect();
        let score = service.rsi_scoring(&prices);
        assert!(score > 60.0, "Oversold stock should get high RSI score (buy signal), got {}", score);
    }

    #[test]
    fn test_price_momentum() {
        let service = test_service();
        let prices = make_prices(100, 100.0, 0.5);
        let score = service.price_momentum(&prices, 21);
        assert!(score > 50.0, "Uptrend momentum should score above 50, got {}", score);
    }

    #[test]
    fn test_price_momentum_downtrend() {
        let service = test_service();
        let prices = make_prices(100, 200.0, -0.5);
        let score = service.price_momentum(&prices, 21);
        assert!(score < 50.0, "Downtrend momentum should score below 50, got {}", score);
    }

    #[test]
    fn test_drawdown_stability() {
        let service = test_service();
        let stable = vec![100.0; 100];
        let score_stable = service.drawdown_stability_score(&stable);
        assert!(score_stable > 90.0, "No drawdown should score very high, got {}", score_stable);

        let volatile: Vec<f64> = (0..100).map(|i| if i % 2 == 0 { 100.0 } else { 60.0 }).collect();
        let score_vol = service.drawdown_stability_score(&volatile);
        assert!(score_vol < score_stable, "Volatile should score lower than stable");
    }

    #[test]
    fn test_filters_sector() {
        let service = test_service();
        let data = make_ticker(vec![100.0; 50]);

        let mut filters = ScreeningFilters::default();
        filters.sectors = vec!["Technology".into()];
        assert!(service.passes_filters(&data, &filters));

        filters.sectors = vec!["Healthcare".into()];
        assert!(!service.passes_filters(&data, &filters));
    }

    #[test]
    fn test_filters_price_range() {
        let service = test_service();
        let data = make_ticker(vec![50.0; 30]);

        let mut filters = ScreeningFilters::default();
        filters.min_price = Some(40.0);
        filters.max_price = Some(60.0);
        assert!(service.passes_filters(&data, &filters));

        filters.min_price = Some(60.0);
        assert!(!service.passes_filters(&data, &filters));
    }

    #[test]
    fn test_filters_market_cap() {
        let service = test_service();
        let data = make_ticker(vec![100.0; 50]);

        let mut filters = ScreeningFilters::default();
        filters.market_cap = Some(MarketCapRange::Large);
        assert!(service.passes_filters(&data, &filters), "50B market cap should pass Large filter");

        filters.market_cap = Some(MarketCapRange::Small);
        assert!(!service.passes_filters(&data, &filters), "50B market cap should not pass Small filter");
    }

    #[test]
    fn test_scoring_produces_valid_composite() {
        let service = test_service();
        let data = make_ticker(make_prices(300, 100.0, 0.1));
        let weights = FactorWeights::default().resolve(None, None);
        let result = service.score_ticker(&data, &weights);

        assert!(result.composite_score >= 0.0 && result.composite_score <= 100.0,
                "Composite score should be 0-100, got {}", result.composite_score);
        assert!(!result.explanation.is_empty());
    }

    #[test]
    fn test_score_ticker_ranking() {
        let service = test_service();
        let weights = FactorWeights::default().resolve(None, None);

        let mut up_data = make_ticker(make_prices(300, 100.0, 0.5));
        up_data.symbol = "UP".into();
        let up_result = service.score_ticker(&up_data, &weights);

        let mut down_data = make_ticker(make_prices(300, 300.0, -0.5));
        down_data.symbol = "DOWN".into();
        let down_result = service.score_ticker(&down_data, &weights);

        assert!(up_result.composite_score > down_result.composite_score,
                "Uptrend ({:.1}) should score higher than downtrend ({:.1})",
                up_result.composite_score, down_result.composite_score);
    }

    #[test]
    fn test_sentiment_with_data() {
        let service = test_service();
        let mut data = make_ticker(make_prices(100, 100.0, 0.1));
        data.sentiment_score = Some(0.8);
        let score = service.score_sentiment(&data);
        assert!(score.composite > 60.0, "Positive sentiment should score high, got {}", score.composite);
    }

    #[test]
    fn test_sentiment_without_data() {
        let service = test_service();
        let mut data = make_ticker(make_prices(100, 100.0, 0.1));
        data.sentiment_score = None;
        let score = service.score_sentiment(&data);
        assert!((score.composite - 50.0).abs() < 10.0, "Missing sentiment should be near neutral, got {}", score.composite);
    }

    #[test]
    fn test_cache_key_deterministic() {
        let service = test_service();
        let req = ScreeningRequest {
            symbols: vec!["AAPL".into(), "GOOG".into()],
            weights: FactorWeights::default(),
            filters: ScreeningFilters::default(),
            limit: 20,
            offset: 0,
            risk_appetite: Some(RiskAppetite::Moderate),
            horizon_months: Some(6),
            refresh: false,
        };

        let k1 = service.build_cache_key(&req);
        let k2 = service.build_cache_key(&req);
        assert_eq!(k1, k2, "Cache key should be deterministic");
    }
}
