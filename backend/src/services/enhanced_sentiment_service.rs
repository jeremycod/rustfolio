use crate::errors::AppError;
use crate::models::{
    SentimentSignal, EnhancedSentimentSignal, MaterialEvent, InsiderSentiment,
    ConfidenceLevel, EventImportance, InsiderConfidence,
};
use crate::services::{sentiment_service, sec_edgar_service};
use chrono::{Utc, Duration};
use sqlx::PgPool;
use tracing::{info, warn};
use uuid::Uuid;

/// Generate enhanced sentiment combining news + SEC + insider data
pub async fn generate_enhanced_sentiment(
    pool: &PgPool,
    edgar_service: &sec_edgar_service::SecEdgarService,
    llm_service: &crate::services::llm_service::LlmService,
    news_service: &crate::services::news_service::NewsService,
    ticker: &str,
    days: i32,
) -> Result<EnhancedSentimentSignal, AppError> {
    info!("🚀 [ENHANCED SENTIMENT] ========== STARTING ENHANCED SENTIMENT GENERATION FOR {} ==========", ticker);
    info!("🚀 [ENHANCED SENTIMENT] Parameters: ticker={}, days={}", ticker, days);

    // Check cache first
    info!("🔍 [ENHANCED SENTIMENT] Checking cache...");
    if let Some(cached) = get_enhanced_sentiment_from_cache(pool, ticker).await? {
        info!("✅ [ENHANCED SENTIMENT] Using cached enhanced sentiment for {}", ticker);
        return Ok(cached);
    }
    info!("⚠️ [ENHANCED SENTIMENT] No cache found, generating fresh sentiment...");

    // 1. Get base news sentiment (existing implementation)
    info!("📰 [ENHANCED SENTIMENT] === PHASE 1: NEWS SENTIMENT ===");
    let news_signal = get_or_create_news_sentiment(pool, news_service, ticker, days).await?;
    info!("📰 [ENHANCED SENTIMENT] News sentiment result: score={:.2}, articles={}",
        news_signal.current_sentiment, news_signal.news_articles_analyzed);

    // 2. Fetch and analyze material events (8-K filings)
    info!("📄 [ENHANCED SENTIMENT] === PHASE 2: SEC FILINGS ===");
    let material_events = fetch_and_analyze_material_events(
        pool,
        edgar_service,
        llm_service,
        ticker,
        days,
    ).await?;
    info!("📄 [ENHANCED SENTIMENT] SEC filings result: {} material events found", material_events.len());

    let sec_score = calculate_sec_filing_score(&material_events);
    info!("📄 [ENHANCED SENTIMENT] SEC filing score calculated: {:?}", sec_score);

    // 3. Fetch and analyze insider transactions (Form 4)
    info!("👤 [ENHANCED SENTIMENT] === PHASE 3: INSIDER ACTIVITY ===");
    let insider_sentiment = fetch_and_analyze_insider_activity(
        pool,
        edgar_service,
        ticker,
        days,
    ).await?;
    info!("👤 [ENHANCED SENTIMENT] Insider activity result: score={:.2}, transactions={}, buys={}, sells={}",
        insider_sentiment.sentiment_score, insider_sentiment.total_transactions,
        insider_sentiment.buying_transactions, insider_sentiment.selling_transactions);

    // 4. Calculate combined sentiment with weights
    // News: 40%, SEC: 30%, Insider: 30%
    info!("🧮 [ENHANCED SENTIMENT] === PHASE 4: COMBINING SENTIMENTS ===");
    info!("🧮 [ENHANCED SENTIMENT] Input scores: news={:.2}, sec={:?}, insider={:.2}",
        news_signal.current_sentiment, sec_score, insider_sentiment.sentiment_score);
    let combined = calculate_weighted_sentiment(
        news_signal.current_sentiment,
        sec_score,
        insider_sentiment.sentiment_score,
    );
    info!("🧮 [ENHANCED SENTIMENT] Combined sentiment calculated: {:.2}", combined);

    // 5. Detect divergences between sources
    info!("🔍 [ENHANCED SENTIMENT] Detecting divergences between sources...");
    let divergence_flags = detect_multi_source_divergences(
        news_signal.current_sentiment,
        sec_score,
        insider_sentiment.sentiment_score,
        &material_events,
    );
    info!("🔍 [ENHANCED SENTIMENT] Divergences detected: {}", divergence_flags.len());

    // 6. Determine overall confidence level
    info!("🎯 [ENHANCED SENTIMENT] Determining confidence level...");
    let confidence = determine_confidence_level(
        &news_signal,
        &material_events,
        &insider_sentiment,
        &divergence_flags,
    );
    info!("🎯 [ENHANCED SENTIMENT] Confidence level: {:?}", confidence);

    // Log final results before moving values
    let insider_score = insider_sentiment.sentiment_score;
    let news_score = news_signal.current_sentiment;

    let enhanced_signal = EnhancedSentimentSignal {
        ticker: ticker.to_string(),
        news_sentiment: news_signal.current_sentiment,
        news_confidence: format!("{} articles analyzed", news_signal.news_articles_analyzed),
        material_events,
        sec_filing_score: sec_score,
        insider_sentiment,
        combined_sentiment: combined,
        confidence_level: confidence.clone(),
        divergence_flags,
        calculated_at: Utc::now(),
    };

    // Cache the result (12-hour TTL)
    info!("💾 [ENHANCED SENTIMENT] Caching result with 12-hour TTL...");
    save_enhanced_sentiment_to_cache(pool, &enhanced_signal).await?;
    info!("✅ [ENHANCED SENTIMENT] Result cached successfully");

    info!(
        "🎉 [ENHANCED SENTIMENT] ========== COMPLETED: combined={:.2}, confidence={:?}, news={:.2}, sec={:?}, insider={:.2} ==========",
        combined, &confidence, news_score, sec_score, insider_score
    );

    Ok(enhanced_signal)
}

/// Get news sentiment, using cache if available
async fn get_or_create_news_sentiment(
    pool: &PgPool,
    news_service: &crate::services::news_service::NewsService,
    ticker: &str,
    days: i32,
) -> Result<SentimentSignal, AppError> {
    info!("🔍 [NEWS SENTIMENT] Starting news sentiment fetch for {} (days={})", ticker, days);

    // 1. Fetch news articles
    info!("🔍 [NEWS SENTIMENT] Step 1: Fetching news articles...");
    let articles = match news_service.fetch_ticker_news(ticker, days).await {
        Ok(articles) => {
            info!("✅ [NEWS SENTIMENT] Successfully fetched {} articles for {}", articles.len(), ticker);
            articles
        },
        Err(e) => {
            warn!("❌ [NEWS SENTIMENT] Failed to fetch news for {}: {}", ticker, e);
            return Ok(SentimentSignal {
                ticker: ticker.to_string(),
                current_sentiment: 0.0,
                sentiment_trend: crate::models::SentimentTrend::Stable,
                momentum_trend: crate::models::MomentumTrend::Neutral,
                divergence: crate::models::DivergenceType::None,
                sentiment_price_correlation: None,
                correlation_lag_days: None,
                correlation_strength: None,
                historical_sentiment: Vec::new(),
                news_articles_analyzed: 0,
                calculated_at: Utc::now(),
                warnings: vec![format!("Failed to fetch news: {}", e)],
            });
        }
    };

    if articles.is_empty() {
        warn!("⚠️ [NEWS SENTIMENT] No news articles found for {} in last {} days", ticker, days);
        return Ok(SentimentSignal {
            ticker: ticker.to_string(),
            current_sentiment: 0.0,
            sentiment_trend: crate::models::SentimentTrend::Stable,
            momentum_trend: crate::models::MomentumTrend::Neutral,
            divergence: crate::models::DivergenceType::None,
            sentiment_price_correlation: None,
            correlation_lag_days: None,
            correlation_strength: None,
            historical_sentiment: Vec::new(),
            news_articles_analyzed: 0,
            calculated_at: Utc::now(),
            warnings: vec!["No news articles found".to_string()],
        });
    }

    info!("✅ [NEWS SENTIMENT] Found {} news articles for {}", articles.len(), ticker);

    // 2. Cluster articles into themes using LLM
    info!("🔍 [NEWS SENTIMENT] Step 2: Clustering articles into themes using LLM...");
    let demo_user_id = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("Failed to parse demo user UUID");

    let themes = match news_service.cluster_into_themes(articles, demo_user_id).await {
        Ok(themes) => {
            info!("✅ [NEWS SENTIMENT] Successfully extracted {} themes for {}", themes.len(), ticker);
            themes
        },
        Err(e) => {
            warn!("❌ [NEWS SENTIMENT] Failed to cluster themes for {}: {}", ticker, e);
            return Ok(SentimentSignal {
                ticker: ticker.to_string(),
                current_sentiment: 0.0,
                sentiment_trend: crate::models::SentimentTrend::Stable,
                momentum_trend: crate::models::MomentumTrend::Neutral,
                divergence: crate::models::DivergenceType::None,
                sentiment_price_correlation: None,
                correlation_lag_days: None,
                correlation_strength: None,
                historical_sentiment: Vec::new(),
                news_articles_analyzed: 0,
                calculated_at: Utc::now(),
                warnings: vec![format!("Failed to analyze themes: {}", e)],
            });
        }
    };

    if themes.is_empty() {
        warn!("⚠️ [NEWS SENTIMENT] No themes extracted for {} (empty result from LLM)", ticker);
        return Ok(SentimentSignal {
            ticker: ticker.to_string(),
            current_sentiment: 0.0,
            sentiment_trend: crate::models::SentimentTrend::Stable,
            momentum_trend: crate::models::MomentumTrend::Neutral,
            divergence: crate::models::DivergenceType::None,
            sentiment_price_correlation: None,
            correlation_lag_days: None,
            correlation_strength: None,
            historical_sentiment: Vec::new(),
            news_articles_analyzed: 0,
            calculated_at: Utc::now(),
            warnings: vec!["No themes extracted from news".to_string()],
        });
    }

    info!("✅ [NEWS SENTIMENT] Extracted {} themes for {}", themes.len(), ticker);

    // 3. Fetch price history for correlation analysis
    info!("🔍 [NEWS SENTIMENT] Step 3: Fetching price history for correlation analysis...");
    let prices = match crate::services::price_service::get_history(pool, ticker).await {
        Ok(prices) => {
            info!("✅ [NEWS SENTIMENT] Successfully fetched {} price points for {}", prices.len(), ticker);
            prices
        },
        Err(e) => {
            warn!("❌ [NEWS SENTIMENT] Failed to fetch price history for {}: {}", ticker, e);
            return Ok(SentimentSignal {
                ticker: ticker.to_string(),
                current_sentiment: 0.0,
                sentiment_trend: crate::models::SentimentTrend::Stable,
                momentum_trend: crate::models::MomentumTrend::Neutral,
                divergence: crate::models::DivergenceType::None,
                sentiment_price_correlation: None,
                correlation_lag_days: None,
                correlation_strength: None,
                historical_sentiment: Vec::new(),
                news_articles_analyzed: themes.iter().map(|t| t.articles.len()).sum::<usize>() as i32,
                calculated_at: Utc::now(),
                warnings: vec![format!("No price history available: {}", e)],
            });
        }
    };

    if prices.is_empty() {
        warn!("⚠️ [NEWS SENTIMENT] No price history found for {} (empty result from DB)", ticker);
        return Ok(SentimentSignal {
            ticker: ticker.to_string(),
            current_sentiment: 0.0,
            sentiment_trend: crate::models::SentimentTrend::Stable,
            momentum_trend: crate::models::MomentumTrend::Neutral,
            divergence: crate::models::DivergenceType::None,
            sentiment_price_correlation: None,
            correlation_lag_days: None,
            correlation_strength: None,
            historical_sentiment: Vec::new(),
            news_articles_analyzed: themes.iter().map(|t| t.articles.len()).sum::<usize>() as i32,
            calculated_at: Utc::now(),
            warnings: vec!["No price history available".to_string()],
        });
    }

    info!("✅ [NEWS SENTIMENT] Found {} price points for {}", prices.len(), ticker);

    // 4. Generate sentiment signal using existing service
    info!("🔍 [NEWS SENTIMENT] Step 4: Generating sentiment signal with correlation analysis...");
    let signal = match crate::services::sentiment_service::generate_sentiment_signal(
        pool,
        ticker,
        themes,
        prices,
    ).await {
        Ok(signal) => {
            info!("✅ [NEWS SENTIMENT] Successfully generated sentiment signal for {}: score={:.2}, trend={:?}, articles={}",
                ticker, signal.current_sentiment, signal.sentiment_trend, signal.news_articles_analyzed);
            signal
        },
        Err(e) => {
            warn!("❌ [NEWS SENTIMENT] Failed to generate sentiment signal for {}: {}", ticker, e);
            return Ok(SentimentSignal {
                ticker: ticker.to_string(),
                current_sentiment: 0.0,
                sentiment_trend: crate::models::SentimentTrend::Stable,
                momentum_trend: crate::models::MomentumTrend::Neutral,
                divergence: crate::models::DivergenceType::None,
                sentiment_price_correlation: None,
                correlation_lag_days: None,
                correlation_strength: None,
                historical_sentiment: Vec::new(),
                news_articles_analyzed: 0,
                calculated_at: Utc::now(),
                warnings: vec![format!("Failed to generate signal: {}", e)],
            });
        }
    };

    info!(
        "🎉 [NEWS SENTIMENT] COMPLETED news sentiment for {}: score={:.2}, trend={:?}, articles={}",
        ticker, signal.current_sentiment, signal.sentiment_trend, signal.news_articles_analyzed
    );

    Ok(signal)
}

/// Fetch and analyze 8-K material events
async fn fetch_and_analyze_material_events(
    pool: &PgPool,
    edgar_service: &sec_edgar_service::SecEdgarService,
    llm_service: &crate::services::llm_service::LlmService,
    ticker: &str,
    days: i32,
) -> Result<Vec<MaterialEvent>, AppError> {
    info!("🔍 [SEC] Checking database cache for material events...");
    // Check database cache first
    let cached_events = fetch_material_events_from_db(pool, ticker, days).await?;

    if !cached_events.is_empty() {
        info!("✅ [SEC] Found {} cached material events for {}", cached_events.len(), ticker);
        return Ok(cached_events);
    }
    info!("⚠️ [SEC] No cached events found");

    // Fetch fresh 8-K filings from SEC Edgar
    info!("🔍 [SEC] Fetching fresh 8-K filings from SEC Edgar API...");
    let filings = edgar_service.fetch_8k_filings(ticker, days).await?;

    if filings.is_empty() {
        info!("⚠️ [SEC] No 8-K filings found for {} in last {} days", ticker, days);
        return Ok(Vec::new());
    }

    info!("✅ [SEC] Found {} 8-K filings for {}, analyzing with LLM", filings.len(), ticker);

    // Demo user ID for LLM rate limiting
    let demo_user_id = uuid::Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .expect("Failed to parse demo user UUID");

    let mut events = Vec::new();

    // Analyze up to 3 most recent filings (to avoid timeouts)
    for filing in filings.iter().take(3) {
        info!("🔍 [SEC] Analyzing 8-K filing from {} for {}", filing.filing_date, ticker);

        // Try to fetch and analyze filing content
        match edgar_service.fetch_filing_content(&filing.filing_url).await {
            Ok(content) => {
                info!("✅ [SEC] Fetched filing content ({} chars)", content.len());

                // Analyze with LLM
                match sec_edgar_service::analyze_material_event(filing, &content, llm_service, demo_user_id).await {
                    Ok(event) => {
                        info!("✅ [SEC] LLM analysis complete: sentiment={:.2}, importance={:?}",
                              event.sentiment_score, event.importance);

                        // Save to database
                        if let Err(e) = save_material_event_to_db(pool, &event).await {
                            warn!("Failed to save material event to DB: {}", e);
                        }

                        events.push(event);
                    }
                    Err(e) => {
                        warn!("⚠️ [SEC] LLM analysis failed for {}: {}", filing.filing_date, e);
                        // Create placeholder event as fallback
                        let event = MaterialEvent {
                            ticker: ticker.to_string(),
                            event_date: filing.filing_date,
                            event_type: "8-K Filing".to_string(),
                            sentiment_score: 0.0,
                            summary: filing.description.clone().unwrap_or_else(|| "Material event filed with SEC".to_string()),
                            importance: EventImportance::Medium,
                            filing_url: filing.filing_url.clone(),
                        };
                        events.push(event);
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ [SEC] Failed to fetch filing content for {}: {}", filing.filing_date, e);
                // Create placeholder event as fallback
                let event = MaterialEvent {
                    ticker: ticker.to_string(),
                    event_date: filing.filing_date,
                    event_type: "8-K Filing".to_string(),
                    sentiment_score: 0.0,
                    summary: filing.description.clone().unwrap_or_else(|| "Material event filed with SEC".to_string()),
                    importance: EventImportance::Medium,
                    filing_url: filing.filing_url.clone(),
                };
                events.push(event);
            }
        }
    }

    info!("Created {} material events for {} ({} analyzed)", events.len(), ticker,
          events.iter().filter(|e| e.sentiment_score != 0.0).count());
    Ok(events)
}

/// Fetch and analyze insider trading activity
async fn fetch_and_analyze_insider_activity(
    pool: &PgPool,
    edgar_service: &sec_edgar_service::SecEdgarService,
    ticker: &str,
    days: i32,
) -> Result<InsiderSentiment, AppError> {
    info!("🔍 [INSIDER] Checking database cache for insider transactions...");
    // Check database cache first
    let cached_transactions = fetch_insider_transactions_from_db(pool, ticker, days).await?;

    let transactions = if !cached_transactions.is_empty() {
        info!("✅ [INSIDER] Found {} cached insider transactions for {}", cached_transactions.len(), ticker);
        cached_transactions
    } else {
        info!("⚠️ [INSIDER] No cached transactions found");
        // Fetch fresh Form 4 filings
        info!("🔍 [INSIDER] Fetching fresh Form 4 filings from SEC Edgar API...");
        let txns = edgar_service.fetch_form4_transactions(ticker, days).await?;

        if txns.is_empty() {
            info!("⚠️ [INSIDER] No Form 4 filings found for {} in last {} days", ticker, days);
        } else {
            info!("✅ [INSIDER] Found {} Form 4 transactions for {}", txns.len(), ticker);
            // Save to database
            for txn in &txns {
                if let Err(e) = save_insider_transaction_to_db(pool, txn).await {
                    warn!("Failed to save insider transaction to DB: {}", e);
                }
            }
        }

        txns
    };

    // Calculate aggregate sentiment
    let sentiment = sec_edgar_service::calculate_insider_sentiment(ticker, transactions, days);

    Ok(sentiment)
}

/// Calculate weighted sentiment from multiple sources
/// Weights: News 40%, SEC 30%, Insider 30%
fn calculate_weighted_sentiment(news: f64, sec: Option<f64>, insider: f64) -> f64 {
    match sec {
        Some(s) => {
            // All three sources available
            (news * 0.4 + s * 0.3 + insider * 0.3).clamp(-1.0, 1.0)
        }
        None => {
            // Only news and insider
            (news * 0.6 + insider * 0.4).clamp(-1.0, 1.0)
        }
    }
}

/// Calculate SEC filing sentiment score from material events
fn calculate_sec_filing_score(events: &[MaterialEvent]) -> Option<f64> {
    if events.is_empty() {
        return None;
    }

    // Weight by importance and recency
    let mut weighted_sum = 0.0;
    let mut total_weight = 0.0;

    for event in events {
        let importance_weight = match event.importance {
            EventImportance::Critical => 2.0,
            EventImportance::High => 1.5,
            EventImportance::Medium => 1.0,
            EventImportance::Low => 0.5,
        };

        weighted_sum += event.sentiment_score * importance_weight;
        total_weight += importance_weight;
    }

    if total_weight > 0.0 {
        Some((weighted_sum / total_weight).clamp(-1.0, 1.0))
    } else {
        None
    }
}

/// Detect divergences between different data sources
fn detect_multi_source_divergences(
    news: f64,
    sec: Option<f64>,
    insider: f64,
    events: &[MaterialEvent],
) -> Vec<String> {
    let mut flags = Vec::new();

    // Insider selling despite positive news
    if news > 0.3 && insider < -0.3 {
        flags.push("⚠️ Positive news but heavy insider selling - potential red flag".to_string());
    }

    // Insider buying despite negative news (opportunity signal)
    if news < -0.3 && insider > 0.3 {
        flags.push("🟢 Opportunity: Negative news but insiders are buying confidently".to_string());
    }

    // SEC filings contradict news sentiment
    if let Some(s) = sec {
        if news > 0.3 && s < -0.3 {
            flags.push("⚠️ Positive news contradicts negative corporate filings (8-K)".to_string());
        }
        if news < -0.3 && s > 0.3 {
            flags.push("🟢 Negative news but positive corporate developments (8-K)".to_string());
        }
    }

    // Critical material events warrant attention
    let critical_events = events.iter().filter(|e| matches!(e.importance, EventImportance::Critical)).count();
    if critical_events > 0 {
        flags.push(format!("📢 {} critical corporate event(s) detected - review 8-K filings", critical_events));
    }

    // All sources strongly bearish
    if news < -0.5 && sec.unwrap_or(0.0) < -0.5 && insider < -0.5 {
        flags.push("🔴 Strong bearish signal across all sources - high conviction".to_string());
    }

    // All sources strongly bullish
    if news > 0.5 && sec.unwrap_or(0.0) > 0.5 && insider > 0.5 {
        flags.push("🟢 Strong bullish signal across all sources - high conviction".to_string());
    }

    flags
}

/// Determine overall confidence level based on data quality and agreement
fn determine_confidence_level(
    news: &SentimentSignal,
    events: &[MaterialEvent],
    insider: &InsiderSentiment,
    divergences: &[String],
) -> ConfidenceLevel {
    let mut score = 0;

    // Strong news data
    if news.news_articles_analyzed >= 10 {
        score += 2;
    } else if news.news_articles_analyzed >= 5 {
        score += 1;
    }

    // Recent material events
    if !events.is_empty() {
        score += 2;
    }

    // Strong insider activity
    match insider.confidence {
        InsiderConfidence::High => score += 3,
        InsiderConfidence::Medium => score += 2,
        InsiderConfidence::Low => score += 1,
        InsiderConfidence::None => {},
    }

    // Penalize divergences (conflicting signals reduce confidence)
    score -= divergences.len() as i32;

    // Map score to confidence level
    match score {
        7.. => ConfidenceLevel::VeryHigh,
        5..=6 => ConfidenceLevel::High,
        3..=4 => ConfidenceLevel::Medium,
        _ => ConfidenceLevel::Low,
    }
}

// Database query functions

async fn get_enhanced_sentiment_from_cache(
    pool: &PgPool,
    ticker: &str,
) -> Result<Option<EnhancedSentimentSignal>, AppError> {
    #[derive(sqlx::FromRow)]
    struct CacheRow {
        ticker: String,
        calculated_at: chrono::NaiveDateTime,
        news_sentiment: f64,
        news_confidence: String,
        sec_filing_score: Option<f64>,
        insider_sentiment_score: f64,
        combined_sentiment: f64,
        confidence_level: String,
        material_events: serde_json::Value,
        insider_activity: serde_json::Value,
        divergence_flags: serde_json::Value,
    }

    let result = sqlx::query_as::<_, CacheRow>(
        r#"
        SELECT
            ticker,
            calculated_at,
            news_sentiment,
            news_confidence,
            sec_filing_score,
            insider_sentiment_score,
            combined_sentiment,
            confidence_level::TEXT as confidence_level,
            material_events,
            insider_activity,
            divergence_flags
        FROM enhanced_sentiment_cache
        WHERE ticker = $1
          AND expires_at > NOW()
        "#,
    )
    .bind(ticker)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = result {
        let material_events: Vec<MaterialEvent> = serde_json::from_value(row.material_events)
            .map_err(|e| AppError::Validation(format!("Failed to parse material_events: {}", e)))?;

        let insider_sentiment: InsiderSentiment = serde_json::from_value(row.insider_activity)
            .map_err(|e| AppError::Validation(format!("Failed to parse insider_activity: {}", e)))?;

        let divergence_flags: Vec<String> = serde_json::from_value(row.divergence_flags)
            .map_err(|e| AppError::Validation(format!("Failed to parse divergence_flags: {}", e)))?;

        // Parse confidence level from string
        let confidence_level = match row.confidence_level.as_str() {
            "very_high" => ConfidenceLevel::VeryHigh,
            "high" => ConfidenceLevel::High,
            "medium" => ConfidenceLevel::Medium,
            _ => ConfidenceLevel::Low,
        };

        Ok(Some(EnhancedSentimentSignal {
            ticker: row.ticker,
            news_sentiment: row.news_sentiment,
            news_confidence: row.news_confidence,
            material_events,
            sec_filing_score: row.sec_filing_score,
            insider_sentiment,
            combined_sentiment: row.combined_sentiment,
            confidence_level,
            divergence_flags,
            calculated_at: row.calculated_at.and_utc(),
        }))
    } else {
        Ok(None)
    }
}

async fn save_enhanced_sentiment_to_cache(
    pool: &PgPool,
    signal: &EnhancedSentimentSignal,
) -> Result<(), AppError> {
    let expires_at = Utc::now() + chrono::Duration::hours(12); // 12-hour TTL

    let material_events_json = serde_json::to_value(&signal.material_events)
        .map_err(|e| AppError::Validation(format!("Failed to serialize material_events: {}", e)))?;

    let insider_activity_json = serde_json::to_value(&signal.insider_sentiment)
        .map_err(|e| AppError::Validation(format!("Failed to serialize insider_activity: {}", e)))?;

    let divergence_flags_json = serde_json::to_value(&signal.divergence_flags)
        .map_err(|e| AppError::Validation(format!("Failed to serialize divergence_flags: {}", e)))?;

    let confidence_str = match signal.confidence_level {
        ConfidenceLevel::VeryHigh => "very_high",
        ConfidenceLevel::High => "high",
        ConfidenceLevel::Medium => "medium",
        ConfidenceLevel::Low => "low",
    };

    sqlx::query(
        r#"
        INSERT INTO enhanced_sentiment_cache (
            ticker,
            calculated_at,
            expires_at,
            news_sentiment,
            news_confidence,
            sec_filing_score,
            insider_sentiment_score,
            combined_sentiment,
            confidence_level,
            material_events,
            insider_activity,
            divergence_flags
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::confidence_level, $10, $11, $12)
        ON CONFLICT (ticker)
        DO UPDATE SET
            calculated_at = EXCLUDED.calculated_at,
            expires_at = EXCLUDED.expires_at,
            news_sentiment = EXCLUDED.news_sentiment,
            news_confidence = EXCLUDED.news_confidence,
            sec_filing_score = EXCLUDED.sec_filing_score,
            insider_sentiment_score = EXCLUDED.insider_sentiment_score,
            combined_sentiment = EXCLUDED.combined_sentiment,
            confidence_level = EXCLUDED.confidence_level,
            material_events = EXCLUDED.material_events,
            insider_activity = EXCLUDED.insider_activity,
            divergence_flags = EXCLUDED.divergence_flags
        "#,
    )
    .bind(&signal.ticker)
    .bind(signal.calculated_at.naive_utc())
    .bind(expires_at.naive_utc())
    .bind(signal.news_sentiment)
    .bind(&signal.news_confidence)
    .bind(signal.sec_filing_score)
    .bind(signal.insider_sentiment.sentiment_score)
    .bind(signal.combined_sentiment)
    .bind(confidence_str)
    .bind(material_events_json)
    .bind(insider_activity_json)
    .bind(divergence_flags_json)
    .execute(pool)
    .await?;

    Ok(())
}

async fn fetch_material_events_from_db(
    pool: &PgPool,
    ticker: &str,
    days: i32,
) -> Result<Vec<MaterialEvent>, AppError> {
    let cutoff_date = Utc::now().date_naive() - chrono::Duration::days(days as i64);

    #[derive(sqlx::FromRow)]
    struct MaterialEventRow {
        ticker: String,
        event_date: chrono::NaiveDate,
        event_type: String,
        sentiment_score: f64,
        summary: String,
        importance: String,
        filing_url: String,
    }

    let rows = sqlx::query_as::<_, MaterialEventRow>(
        r#"
        SELECT
            ticker,
            event_date,
            event_type,
            sentiment_score,
            summary,
            importance::TEXT as importance,
            filing_url
        FROM material_events
        WHERE ticker = $1
          AND event_date >= $2
        ORDER BY event_date DESC
        "#,
    )
    .bind(ticker)
    .bind(cutoff_date)
    .fetch_all(pool)
    .await?;

    let events = rows
        .into_iter()
        .map(|row| {
            let importance = match row.importance.as_str() {
                "critical" => EventImportance::Critical,
                "high" => EventImportance::High,
                "medium" => EventImportance::Medium,
                _ => EventImportance::Low,
            };

            MaterialEvent {
                ticker: row.ticker,
                event_date: row.event_date,
                event_type: row.event_type,
                sentiment_score: row.sentiment_score,
                summary: row.summary,
                importance,
                filing_url: row.filing_url,
            }
        })
        .collect();

    Ok(events)
}

async fn save_material_event_to_db(
    pool: &PgPool,
    event: &MaterialEvent,
) -> Result<(), AppError> {
    // Extract accession number from URL if possible
    // Will be None if URL doesn't contain accession_number parameter
    let accession_number = extract_accession_from_url(&event.filing_url);

    let importance_str = match event.importance {
        EventImportance::Critical => "critical",
        EventImportance::High => "high",
        EventImportance::Medium => "medium",
        EventImportance::Low => "low",
    };

    sqlx::query(
        r#"
        INSERT INTO material_events (
            ticker,
            event_date,
            event_type,
            sentiment_score,
            summary,
            importance,
            filing_url,
            accession_number
        ) VALUES ($1, $2, $3, $4, $5, $6::event_importance, $7, $8)
        ON CONFLICT (ticker, event_date, event_type)
        DO NOTHING
        "#,
    )
    .bind(&event.ticker)
    .bind(event.event_date)
    .bind(&event.event_type)
    .bind(event.sentiment_score)
    .bind(&event.summary)
    .bind(importance_str)
    .bind(&event.filing_url)
    .bind(accession_number)
    .execute(pool)
    .await?;

    Ok(())
}

async fn fetch_insider_transactions_from_db(
    pool: &PgPool,
    ticker: &str,
    days: i32,
) -> Result<Vec<crate::models::InsiderTransaction>, AppError> {
    let cutoff_date = Utc::now().date_naive() - chrono::Duration::days(days as i64);

    #[derive(sqlx::FromRow)]
    struct InsiderTransactionRow {
        ticker: String,
        transaction_date: chrono::NaiveDate,
        reporting_person: String,
        title: Option<String>,
        transaction_type: String,
        shares: i64,
        price_per_share: Option<bigdecimal::BigDecimal>,
        ownership_after: Option<i64>,
    }

    let rows = sqlx::query_as::<_, InsiderTransactionRow>(
        r#"
        SELECT
            ticker,
            transaction_date,
            reporting_person,
            title,
            transaction_type::TEXT as transaction_type,
            shares,
            price_per_share,
            ownership_after
        FROM insider_transactions
        WHERE ticker = $1
          AND transaction_date >= $2
        ORDER BY transaction_date DESC
        "#,
    )
    .bind(ticker)
    .bind(cutoff_date)
    .fetch_all(pool)
    .await?;

    let transactions = rows
        .into_iter()
        .map(|row| {
            let transaction_type = match row.transaction_type.as_str() {
                "purchase" => crate::models::InsiderTransactionType::Purchase,
                "sale" => crate::models::InsiderTransactionType::Sale,
                "grant" => crate::models::InsiderTransactionType::Grant,
                _ => crate::models::InsiderTransactionType::Exercise,
            };

            crate::models::InsiderTransaction {
                ticker: row.ticker,
                transaction_date: row.transaction_date,
                reporting_person: row.reporting_person,
                title: row.title,
                transaction_type,
                shares: row.shares,
                price_per_share: row.price_per_share,
                ownership_after: row.ownership_after,
            }
        })
        .collect();

    Ok(transactions)
}

async fn save_insider_transaction_to_db(
    pool: &PgPool,
    txn: &crate::models::InsiderTransaction,
) -> Result<(), AppError> {
    let transaction_type_str = match txn.transaction_type {
        crate::models::InsiderTransactionType::Purchase => "purchase",
        crate::models::InsiderTransactionType::Sale => "sale",
        crate::models::InsiderTransactionType::Grant => "grant",
        crate::models::InsiderTransactionType::Exercise => "exercise",
    };

    sqlx::query(
        r#"
        INSERT INTO insider_transactions (
            ticker,
            transaction_date,
            reporting_person,
            title,
            transaction_type,
            shares,
            price_per_share,
            ownership_after,
            accession_number
        ) VALUES ($1, $2, $3, $4, $5::transaction_type, $6, $7, $8, $9)
        ON CONFLICT (ticker, transaction_date, reporting_person)
        DO NOTHING
        "#,
    )
    .bind(&txn.ticker)
    .bind(txn.transaction_date)
    .bind(&txn.reporting_person)
    .bind(&txn.title)
    .bind(transaction_type_str)
    .bind(txn.shares)
    .bind(&txn.price_per_share)
    .bind(txn.ownership_after)
    .bind(None::<String>) // NULL accession number (not fetching parent filing yet)
    .execute(pool)
    .await?;

    Ok(())
}

/// Helper to extract accession number from URL
fn extract_accession_from_url(url: &str) -> Option<String> {
    use regex::Regex;
    let re = Regex::new(r"accession_number=([0-9-]+)").ok()?;
    re.captures(url)
        .and_then(|cap| cap.get(1))
        .map(|m| m.as_str().to_string())
}
