use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde::Deserialize;
use sqlx::PgPool;
use std::path::Path;
use std::str::FromStr;
use tracing::info;
use uuid::Uuid;

use crate::db::{account_queries, detected_transaction_queries};
use crate::models::{CreateDetectedTransaction, TransactionType};

#[derive(Debug, Deserialize)]
struct ActivityRow {
    #[serde(rename = "Processed")]
    #[allow(dead_code)]
    processed: String,
    #[serde(rename = "Settled")]
    settled: String,
    #[serde(rename = "Tran Types")]
    tran_types: String,
    #[serde(rename = "Symbol")]
    symbol: String,
    #[serde(rename = "Description")]
    description: String,
    #[serde(rename = "Price")]
    price: String,
    #[serde(rename = "Quantity")]
    quantity: String,
    #[serde(rename = "Amount")]
    amount: String,
}

#[derive(Debug)]
pub struct ActivityImportResult {
    pub transactions_imported: usize,
    pub errors: Vec<String>,
}

fn parse_money_string(s: &str) -> Result<BigDecimal> {
    let cleaned = s
        .replace("$", "")
        .replace(",", "")
        .replace("\"", "")
        .trim()
        .to_string();

    if cleaned.is_empty() || cleaned == "-" {
        return Ok(BigDecimal::from(0));
    }

    BigDecimal::from_str(&cleaned)
        .with_context(|| format!("Failed to parse money string: {}", s))
}

fn parse_date(s: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d")
        .with_context(|| format!("Failed to parse date: {}", s))
}

fn extract_account_number_from_filename(filename: &str) -> Result<String> {
    // Expected format: AccountActivities-{account_number}-{date}.csv
    let parts: Vec<&str> = filename.split('-').collect();
    if parts.len() < 3 {
        anyhow::bail!("Invalid filename format: {}", filename);
    }
    Ok(parts[1].to_string())
}

fn map_transaction_type(tran_type: &str) -> Option<TransactionType> {
    let tran_type_upper = tran_type.to_uppercase();

    if tran_type_upper.contains("BUY") {
        Some(TransactionType::Buy)
    } else if tran_type_upper.contains("SELL") {
        Some(TransactionType::Sell)
    } else if tran_type_upper.contains("DIVIDEND") ||
              tran_type_upper.contains("DISTRIBUTION") ||
              tran_type_upper.contains("REINVESTED") {
        Some(TransactionType::Dividend)
    } else if tran_type_upper.contains("SPLIT") {
        Some(TransactionType::Split)
    } else if tran_type_upper.contains("CASH RECEIPT") ||
              tran_type_upper.contains("CONTRIBUTION") ||
              tran_type_upper.contains("FEE") ||
              tran_type_upper.contains("TAX") {
        // These are cash flows, not trades - skip them
        None
    } else {
        // Other transaction types we don't categorize
        Some(TransactionType::Other)
    }
}

pub async fn import_activities_file(
    pool: &PgPool,
    portfolio_id: Uuid,
    file_path: &Path,
) -> Result<ActivityImportResult> {
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid filename")?;

    let account_number = extract_account_number_from_filename(filename)?;

    info!("Importing activities for account: {}", account_number);

    // Find the account
    let account = account_queries::find_by_account_number(pool, portfolio_id, &account_number)
        .await?
        .context(format!("Account not found: {}", account_number))?;

    let file_content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file_content.as_bytes());

    let mut transactions_imported = 0;
    let mut errors = Vec::new();

    for (line_num, result) in reader.deserialize::<ActivityRow>().enumerate() {
        match result {
            Ok(row) => {
                match process_activity_row(pool, account.id, row).await {
                    Ok(imported) => {
                        if imported {
                            transactions_imported += 1;
                        }
                    }
                    Err(e) => {
                        errors.push(format!("Line {}: {}", line_num + 2, e));
                    }
                }
            }
            Err(e) => {
                errors.push(format!("Line {}: Failed to parse CSV row: {}", line_num + 2, e));
            }
        }
    }

    info!(
        "Activity import completed for account {}: {} transactions imported, {} errors",
        account_number, transactions_imported, errors.len()
    );

    Ok(ActivityImportResult {
        transactions_imported,
        errors,
    })
}

async fn process_activity_row(
    pool: &PgPool,
    account_id: Uuid,
    row: ActivityRow,
) -> Result<bool> {
    // Map transaction type
    let transaction_type = match map_transaction_type(&row.tran_types) {
        Some(tt) => tt,
        None => {
            // Skip cash flows and other non-trade activities
            return Ok(false);
        }
    };

    // Parse dates
    let settled_date = parse_date(&row.settled)?;

    // Parse ticker
    let ticker = row.symbol.trim().to_string();
    if ticker.is_empty() {
        // Skip transactions without a ticker (pure cash transactions)
        return Ok(false);
    }

    // Parse price, quantity, and amount
    let price = parse_money_string(&row.price).ok();
    let quantity = parse_money_string(&row.quantity).ok();
    let amount = parse_money_string(&row.amount).ok();

    // Create transaction
    let transaction = CreateDetectedTransaction {
        transaction_type,
        ticker: ticker.clone(),
        quantity,
        price,
        amount,
        from_snapshot_date: None,
        to_snapshot_date: None,
        description: Some(format!("{}: {}", row.tran_types, row.description)),
    };

    detected_transaction_queries::create(pool, account_id, settled_date, transaction).await?;

    Ok(true)
}
