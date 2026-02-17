use crate::errors::AppError;
use crate::models::{SecFiling, InsiderTransaction, InsiderTransactionType, FilingType};
use crate::services::llm_service::LlmService;
use chrono::{NaiveDate, Utc, Duration};
use reqwest::Client;
use serde::Deserialize;
use tracing::{info, warn, error};
use uuid::Uuid;
use regex::Regex;

pub struct SecEdgarService {
    client: Client,
    user_agent: String,
}

/// SEC Edgar RSS feed entry
#[derive(Debug, Deserialize)]
struct EdgarRssItem {
    title: String,
    link: String,
    #[serde(rename = "pubDate")]
    pub_date: String,
}

impl SecEdgarService {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .expect("Failed to build HTTP client"),
            // SEC requires: Company Name + Contact Email
            user_agent: "Rustfolio/1.0 (portfolio.analytics@rustfolio.com)".to_string(),
        }
    }

    /// Fetch recent 8-K filings for a ticker
    pub async fn fetch_8k_filings(
        &self,
        ticker: &str,
        days_back: i32,
    ) -> Result<Vec<SecFiling>, AppError> {
        info!("Fetching 8-K filings for {} (last {} days)", ticker, days_back);

        // SEC Edgar RSS feed URL
        // Format: https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK={ticker}&type=8-K&count=10&output=atom
        let url = format!(
            "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK={}&type=8-K&count=20&output=atom",
            ticker.to_uppercase()
        );

        let response = self.client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Failed to fetch 8-K filings: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::External(format!(
                "SEC Edgar returned status: {}",
                response.status()
            )));
        }

        let body = response.text().await
            .map_err(|e| AppError::External(format!("Failed to read response: {}", e)))?;

        // Parse RSS/Atom feed
        let filings = self.parse_edgar_feed(&body, ticker, days_back)?;

        info!("Found {} 8-K filings for {}", filings.len(), ticker);
        Ok(filings)
    }

    /// Fetch recent Form 4 insider transactions
    pub async fn fetch_form4_transactions(
        &self,
        ticker: &str,
        days_back: i32,
    ) -> Result<Vec<InsiderTransaction>, AppError> {
        info!("Fetching Form 4 transactions for {} (last {} days)", ticker, days_back);

        // Similar to 8-K but with type=4
        let url = format!(
            "https://www.sec.gov/cgi-bin/browse-edgar?action=getcompany&CIK={}&type=4&count=20&output=atom",
            ticker.to_uppercase()
        );

        let response = self.client
            .get(&url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Failed to fetch Form 4 filings: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::External(format!(
                "SEC Edgar returned status: {}",
                response.status()
            )));
        }

        let body = response.text().await
            .map_err(|e| AppError::External(format!("Failed to read response: {}", e)))?;

        // Parse Form 4 transactions
        let transactions = self.parse_form4_feed(&body, ticker, days_back)?;

        info!("Found {} Form 4 transactions for {}", transactions.len(), ticker);
        Ok(transactions)
    }

    /// Parse Edgar RSS/Atom feed
    fn parse_edgar_feed(
        &self,
        feed_xml: &str,
        ticker: &str,
        days_back: i32,
    ) -> Result<Vec<SecFiling>, AppError> {
        let cutoff_date = Utc::now().date_naive() - Duration::days(days_back as i64);
        let mut filings = Vec::new();

        // Parse Atom feed entries
        // Pattern: <entry>...</entry> (with multiline support using (?s) flag)
        let entry_re = Regex::new(r"(?s)<entry>(.*?)</entry>").unwrap();

        for entry_cap in entry_re.captures_iter(feed_xml) {
            let entry = &entry_cap[1];

            // Extract title: <title>8-K - GOOGLE INC</title>
            let title = self.extract_xml_tag(entry, "title").unwrap_or_default();

            // Extract link: <link rel="alternate" href="..." />
            let link = self.extract_xml_attribute(entry, "link", "href").unwrap_or_default();

            // Extract filing date: <filing-date>2026-01-29</filing-date>
            let filing_date_str = self.extract_xml_tag(entry, "filing-date").unwrap_or_default();

            // Parse date (YYYY-MM-DD format)
            let filing_date = if !filing_date_str.is_empty() {
                NaiveDate::parse_from_str(&filing_date_str, "%Y-%m-%d").ok()
            } else {
                None
            };

            // Extract accession number from link
            // Format: https://www.sec.gov/cgi-bin/viewer?action=view&cik=...&accession_number=0000320193-26-000012
            let accession_number = self.extract_accession_from_url(&link);

            // Skip if too old
            if let Some(date) = filing_date {
                if date < cutoff_date {
                    continue;
                }

                filings.push(SecFiling {
                    ticker: ticker.to_uppercase(),
                    filing_type: FilingType::EightK,
                    filing_date: date,
                    accession_number,
                    filing_url: link,
                    description: Some(title),
                });
            }
        }

        Ok(filings)
    }

    /// Parse Form 4 feed and extract transaction details
    fn parse_form4_feed(
        &self,
        feed_xml: &str,
        ticker: &str,
        days_back: i32,
    ) -> Result<Vec<InsiderTransaction>, AppError> {
        let cutoff_date = Utc::now().date_naive() - Duration::days(days_back as i64);
        let mut transactions = Vec::new();

        // Parse Atom feed entries (with multiline support using (?s) flag)
        let entry_re = Regex::new(r"(?s)<entry>(.*?)</entry>").unwrap();

        for entry_cap in entry_re.captures_iter(feed_xml) {
            let entry = &entry_cap[1];

            // Extract filing date: <filing-date>2026-01-29</filing-date>
            let filing_date_str = self.extract_xml_tag(entry, "filing-date").unwrap_or_default();
            let filing_date = if !filing_date_str.is_empty() {
                NaiveDate::parse_from_str(&filing_date_str, "%Y-%m-%d").ok()
            } else {
                None
            };

            // Skip if too old
            if let Some(date) = filing_date {
                if date < cutoff_date {
                    continue;
                }
            } else {
                continue;
            }

            // Extract title to get reporting person
            // Format: "4 - John Doe (CEO)"
            let title = self.extract_xml_tag(entry, "title").unwrap_or_default();

            // Simple parsing: extract name between "4 - " and " ("
            let reporting_person = if let Some(start_idx) = title.find("4 - ") {
                let after_dash = &title[start_idx + 4..];
                if let Some(paren_idx) = after_dash.find(" (") {
                    after_dash[..paren_idx].to_string()
                } else {
                    after_dash.to_string()
                }
            } else {
                "Unknown".to_string()
            };

            // Extract title (position)
            let position_title = if let Some(start) = title.find(" (") {
                if let Some(end) = title.find(")") {
                    Some(title[start + 2..end].to_string())
                } else {
                    None
                }
            } else {
                None
            };

            // Create a simplified transaction
            // Note: Full Form 4 XML parsing requires downloading the actual filing
            // For now, we'll create placeholder transactions
            transactions.push(InsiderTransaction {
                ticker: ticker.to_uppercase(),
                transaction_date: filing_date.unwrap(),
                reporting_person,
                title: position_title,
                transaction_type: InsiderTransactionType::Purchase, // Placeholder
                shares: 0, // Would need to parse full XML
                price_per_share: None,
                ownership_after: None,
            });
        }

        Ok(transactions)
    }

    /// Download and extract text content from a filing
    pub async fn fetch_filing_content(
        &self,
        filing_url: &str,
    ) -> Result<String, AppError> {
        info!("Fetching filing content from: {}", filing_url);

        // Convert index URLs to plain text version
        // Index: /Archives/edgar/data/.../0000320193-26-000005/0000320193-26-000005-index.htm
        // Text:  /Archives/edgar/data/.../0000320193-26-000005/0000320193-26-000005.txt
        let content_url = if filing_url.contains("-index.htm") {
            let txt_url = filing_url.replace("-index.htm", ".txt");
            info!("📄 [SEC] Converting index URL to text file: {}", txt_url);
            txt_url
        } else {
            filing_url.to_string()
        };

        let response = self.client
            .get(&content_url)
            .header("User-Agent", &self.user_agent)
            .send()
            .await
            .map_err(|e| AppError::External(format!("Failed to fetch filing: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::External(format!(
                "Filing URL returned status: {}",
                response.status()
            )));
        }

        let content = response.text().await
            .map_err(|e| AppError::External(format!("Failed to read filing content: {}", e)))?;

        info!("📄 [SEC] Downloaded filing content: {} chars", content.len());

        // Extract meaningful content from the filing
        // Skip XBRL/XML headers and extract exhibits (press releases) or Item sections
        let meaningful_content = self.extract_meaningful_content(&content);

        info!("📄 [SEC] Extracted meaningful content: {} chars", meaningful_content.len());

        Ok(meaningful_content)
    }

    /// Extract meaningful content from SEC filing text
    /// Skips XBRL headers and extracts exhibits (press releases) or Item sections
    fn extract_meaningful_content(&self, content: &str) -> String {
        // Strategy: Look for exhibits first (press releases), then Item sections

        // Try to find EX-99.1 or similar exhibits (earnings press releases)
        if let Some(exhibit_start) = content.find("<TYPE>EX-99") {
            // Find the <TEXT> section after this exhibit type
            if let Some(text_start) = content[exhibit_start..].find("<TEXT>") {
                let content_start = exhibit_start + text_start + 6; // +6 for "<TEXT>"

                // Find the end of this exhibit (next <DOCUMENT> or </TEXT>)
                let remaining = &content[content_start..];
                let exhibit_end = remaining.find("</TEXT>")
                    .or_else(|| remaining.find("<DOCUMENT>"))
                    .unwrap_or(remaining.len().min(50000)); // Take up to 50KB

                let exhibit_text = &remaining[..exhibit_end];

                // Clean up HTML tags and XBRL
                let cleaned = self.extract_text_from_html(exhibit_text);

                info!("📄 [SEC] Extracted EX-99 exhibit content");
                return cleaned;
            }
        }

        // Fallback: Look for "Item 2.02" section (Results of Operations)
        if let Some(item_start) = content.find("Item 2.02") {
            let content_start = item_start;
            let remaining = &content[content_start..];

            // Take next 20KB after Item 2.02
            let item_end = remaining.len().min(20000);
            let item_text = &remaining[..item_end];

            let cleaned = self.extract_text_from_html(item_text);

            info!("📄 [SEC] Extracted Item 2.02 section content");
            return cleaned;
        }

        // Last resort: Skip first 10KB of XBRL junk and take next 20KB
        let skip_amount = content.len().min(10000);
        let take_amount = (content.len() - skip_amount).min(20000);

        let text = &content[skip_amount..skip_amount + take_amount];
        let cleaned = self.extract_text_from_html(text);

        info!("📄 [SEC] Using fallback content extraction (skipped first 10KB)");
        cleaned
    }

    /// Extract plain text from HTML filing
    fn extract_text_from_html(&self, html: &str) -> String {
        // Simple HTML tag removal (will improve later)
        // TODO: Use proper HTML parser
        let text = html
            .replace("<br>", "\n")
            .replace("<BR>", "\n")
            .replace("<br/>", "\n")
            .replace("<p>", "\n")
            .replace("<P>", "\n")
            .replace("</p>", "\n")
            .replace("</P>", "\n");

        // Remove HTML tags using regex
        let tag_re = Regex::new(r"<[^>]+>").unwrap();
        let text = tag_re.replace_all(&text, " ");

        // Clean up whitespace
        text.lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && line.len() > 10) // Skip very short lines
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Extract text content from XML tag
    fn extract_xml_tag(&self, xml: &str, tag: &str) -> Option<String> {
        let pattern = format!(r"<{}[^>]*>(.*?)</{}>", tag, tag);
        let re = Regex::new(&pattern).ok()?;
        re.captures(xml)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().trim().to_string())
    }

    /// Extract attribute value from XML tag
    fn extract_xml_attribute(&self, xml: &str, tag: &str, attr: &str) -> Option<String> {
        let pattern = format!(r#"<{}\s[^>]*{}="([^"]+)"#, tag, attr);
        let re = Regex::new(&pattern).ok()?;
        re.captures(xml)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract accession number from SEC URL
    fn extract_accession_from_url(&self, url: &str) -> String {
        // Pattern: accession_number=0000320193-26-000012
        let re = Regex::new(r"accession_number=([0-9-]+)").unwrap();
        re.captures(url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }
}

/// Analyze 8-K filing content using LLM
pub async fn analyze_material_event(
    filing: &SecFiling,
    content: &str,
    llm_service: &LlmService,
    user_id: Uuid,
) -> Result<crate::models::MaterialEvent, AppError> {
    info!("Analyzing 8-K filing for {} (content length: {} chars)", filing.ticker, content.len());

    // Extract content we'll send to LLM (first 8000 chars)
    let content_for_llm = content.chars().take(8000).collect::<String>();

    // Log the actual content being sent to LLM in chunks so we can see what it contains
    info!("📄 [SEC] Content being sent to LLM - PART 1 (chars 0-2000):\n{}",
          content_for_llm.chars().take(2000).collect::<String>());
    info!("📄 [SEC] Content being sent to LLM - PART 2 (chars 2000-4000):\n{}",
          content_for_llm.chars().skip(2000).take(2000).collect::<String>());
    info!("📄 [SEC] Content being sent to LLM - PART 3 (chars 4000-6000):\n{}",
          content_for_llm.chars().skip(4000).take(2000).collect::<String>());
    info!("📄 [SEC] Content being sent to LLM - PART 4 (chars 6000-8000):\n{}",
          content_for_llm.chars().skip(6000).take(2000).collect::<String>());

    // Create prompt for LLM
    let prompt = format!(
        r#"Analyze this SEC 8-K filing for {} and provide sentiment analysis.

Filing Date: {}
Description: {}

Content (first 8000 chars):
{}

IMPORTANT: Return ONLY valid JSON with no additional text, explanation, or markdown formatting.

Return this exact JSON structure:
{{
    "event_type": "earnings_beat|acquisition|management_change|lawsuit|other",
    "sentiment_score": 0.5,
    "importance": "critical|high|medium|low",
    "summary": "Brief 1-2 sentence summary"
}}

Rules:
- event_type: Choose one: earnings_beat, acquisition, management_change, lawsuit, or other
- sentiment_score: Number from -1.0 (very negative) to +1.0 (very positive)
- importance: Choose one: critical, high, medium, or low
- summary: 1-2 sentences about what happened and investor impact

Focus on: What happened, why it matters, and whether it's positive or negative for investors."#,
        filing.ticker,
        filing.filing_date,
        filing.description.as_ref().unwrap_or(&"N/A".to_string()),
        content_for_llm
    );

    // Call LLM
    let response = llm_service.generate_completion_for_user(
        user_id,
        prompt.to_string(),
    ).await?;

    info!("📝 [SEC] Raw LLM response (first 500 chars): {}",
          response.chars().take(500).collect::<String>());

    // Clean response: remove markdown code blocks if present
    let json_str = response.trim();
    let json_str = if json_str.starts_with("```json") {
        info!("🔧 [SEC] Removing ```json markdown wrapper");
        // Remove ```json at start and ``` at end
        json_str.trim_start_matches("```json")
            .trim_end_matches("```")
            .trim()
    } else if json_str.starts_with("```") {
        info!("🔧 [SEC] Removing ``` markdown wrapper");
        // Remove ``` at start and end
        json_str.trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
    } else {
        info!("✅ [SEC] No markdown wrapper detected");
        json_str
    };

    info!("📝 [SEC] Cleaned JSON string (first 300 chars): {}",
          json_str.chars().take(300).collect::<String>());

    // Parse response
    let analysis: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| {
            warn!("❌ [SEC] JSON parse error: {}", e);
            warn!("❌ [SEC] Failed string: {}", json_str);
            AppError::Validation(format!("Failed to parse LLM response: {}", e))
        })?;

    info!("📊 [SEC] Parsed JSON: {}", serde_json::to_string_pretty(&analysis).unwrap_or_default());

    let event_type = analysis["event_type"]
        .as_str()
        .unwrap_or("other")
        .to_string();
    info!("📊 [SEC] Extracted event_type: {}", event_type);

    let sentiment_score = analysis["sentiment_score"]
        .as_f64()
        .unwrap_or(0.0)
        .clamp(-1.0, 1.0);
    info!("📊 [SEC] Extracted sentiment_score: {} (raw: {:?})", sentiment_score, analysis["sentiment_score"]);

    let importance_str = analysis["importance"]
        .as_str()
        .unwrap_or("medium");
    info!("📊 [SEC] Extracted importance: {}", importance_str);

    let importance = match importance_str {
        "critical" => crate::models::EventImportance::Critical,
        "high" => crate::models::EventImportance::High,
        "medium" => crate::models::EventImportance::Medium,
        _ => crate::models::EventImportance::Low,
    };

    let summary = analysis["summary"]
        .as_str()
        .unwrap_or("No summary available")
        .to_string();
    info!("📊 [SEC] Extracted summary: {}", summary.chars().take(100).collect::<String>());

    let material_event = crate::models::MaterialEvent {
        ticker: filing.ticker.clone(),
        event_date: filing.filing_date,
        event_type: event_type.clone(),
        sentiment_score,
        summary: summary.clone(),
        importance,
        filing_url: filing.filing_url.clone(),
    };

    info!("✅ [SEC] Created MaterialEvent: ticker={}, sentiment={}, type={}, importance={}",
          material_event.ticker, material_event.sentiment_score, event_type, importance_str);

    Ok(material_event)
}

/// Calculate aggregated insider sentiment from transactions
pub fn calculate_insider_sentiment(
    ticker: &str,
    transactions: Vec<InsiderTransaction>,
    period_days: i32,
) -> crate::models::InsiderSentiment {
    if transactions.is_empty() {
        return crate::models::InsiderSentiment {
            ticker: ticker.to_string(),
            period_days,
            ..Default::default()
        };
    }

    let mut net_shares = 0i64;
    let mut buying_count = 0;
    let mut selling_count = 0;
    let mut notable_transactions = Vec::new();

    for txn in transactions.iter() {
        match txn.transaction_type {
            InsiderTransactionType::Purchase => {
                net_shares += txn.shares;
                buying_count += 1;
            }
            InsiderTransactionType::Sale => {
                net_shares -= txn.shares;
                selling_count += 1;
            }
            _ => {} // Ignore grants and exercises for sentiment
        }

        // Flag large transactions (>10k shares) as notable
        if txn.shares.abs() > 10_000 {
            notable_transactions.push(txn.clone());
        }
    }

    let total_transactions = transactions.len() as i32;

    // Calculate sentiment score
    // If we have actual share counts, use volume-based scoring
    // Otherwise, use count-based heuristic (since we don't parse Form 4 XML yet)
    let sentiment_score = if net_shares != 0 {
        // Volume-based: Heavy buying = +1.0, Heavy selling = -1.0
        let max_expected_volume = 100_000i64;
        (net_shares as f64 / max_expected_volume as f64).clamp(-1.0, 1.0)
    } else {
        // Count-based heuristic when shares are unknown
        // Use transaction counts as proxy: more buys = positive, more sells = negative
        let net_count = buying_count - selling_count;
        let max_count = 10.0; // Normalize to 10 transactions
        (net_count as f64 / max_count).clamp(-1.0, 1.0)
    };

    // Determine confidence
    let confidence = if net_shares != 0 {
        // When we have actual share data
        if total_transactions >= 5 && net_shares.abs() > 50_000 {
            crate::models::InsiderConfidence::High
        } else if total_transactions >= 2 {
            crate::models::InsiderConfidence::Medium
        } else if total_transactions > 0 {
            crate::models::InsiderConfidence::Low
        } else {
            crate::models::InsiderConfidence::None
        }
    } else {
        // When using count-based heuristic, confidence is lower
        if total_transactions >= 5 {
            crate::models::InsiderConfidence::Medium
        } else if total_transactions >= 2 {
            crate::models::InsiderConfidence::Low
        } else {
            crate::models::InsiderConfidence::None
        }
    };

    crate::models::InsiderSentiment {
        ticker: ticker.to_string(),
        period_days,
        net_shares_traded: net_shares,
        total_transactions,
        buying_transactions: buying_count,
        selling_transactions: selling_count,
        sentiment_score,
        confidence,
        notable_transactions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_insider_sentiment_buying() {
        let transactions = vec![
            InsiderTransaction {
                ticker: "AAPL".to_string(),
                transaction_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                reporting_person: "Tim Cook".to_string(),
                title: Some("CEO".to_string()),
                transaction_type: InsiderTransactionType::Purchase,
                shares: 50_000,
                price_per_share: None,
                ownership_after: None,
            },
        ];

        let sentiment = calculate_insider_sentiment("AAPL", transactions, 30);

        assert_eq!(sentiment.net_shares_traded, 50_000);
        assert_eq!(sentiment.buying_transactions, 1);
        assert_eq!(sentiment.selling_transactions, 0);
        assert!(sentiment.sentiment_score > 0.0);
    }

    #[test]
    fn test_calculate_insider_sentiment_selling() {
        let transactions = vec![
            InsiderTransaction {
                ticker: "AAPL".to_string(),
                transaction_date: NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(),
                reporting_person: "Tim Cook".to_string(),
                title: Some("CEO".to_string()),
                transaction_type: InsiderTransactionType::Sale,
                shares: 30_000,
                price_per_share: None,
                ownership_after: None,
            },
        ];

        let sentiment = calculate_insider_sentiment("AAPL", transactions, 30);

        assert_eq!(sentiment.net_shares_traded, -30_000);
        assert_eq!(sentiment.buying_transactions, 0);
        assert_eq!(sentiment.selling_transactions, 1);
        assert!(sentiment.sentiment_score < 0.0);
    }
}
