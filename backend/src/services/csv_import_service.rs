use anyhow::{Context, Result};
use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use csv::ReaderBuilder;
use serde::Deserialize;
use sqlx::PgPool;
use std::path::Path;
use std::str::FromStr;
use uuid::Uuid;

use crate::db::{account_queries, holding_snapshot_queries};
use crate::models::{CreateAccount, CreateHoldingSnapshot};

#[derive(Debug, Deserialize)]
struct CsvRow {
    #[serde(rename = "Client Name")]
    client_name: String,
    #[serde(rename = "Client Id")]
    client_id: String,
    #[serde(rename = "Account Nickname")]
    account_nickname: String,
    #[serde(rename = "Account Number")]
    account_number: String,
    #[serde(rename = "Asset Category")]
    asset_category: String,
    #[serde(rename = "Industry")]
    industry: String,
    #[serde(rename = "Symbol")]
    symbol: String,
    #[serde(rename = "Holding")]
    holding: String,
    #[serde(rename = "Quantity")]
    quantity: String,
    #[serde(rename = "Price")]
    price: String,
    #[serde(rename = "Fund")]
    fund: String,
    #[serde(rename = "Average Cost")]
    average_cost: String,
    #[serde(rename = "Book Value")]
    book_value: String,
    #[serde(rename = "Market Value")]
    market_value: String,
    #[serde(rename = "Accrued Interest")]
    accrued_interest: String,
    #[serde(rename = "G/L")]
    gain_loss: String,
    #[serde(rename = "G/L (%)")]
    gain_loss_pct: String,
    #[serde(rename = "Percentage of Assets")]
    percentage_of_assets: String,
}

fn parse_money_string(s: &str) -> Result<BigDecimal> {
    let cleaned = s
        .replace("$", "")
        .replace(",", "")
        .replace("%", "")
        .trim()
        .to_string();

    if cleaned.is_empty() || cleaned == "-" {
        return Ok(BigDecimal::from(0));
    }

    BigDecimal::from_str(&cleaned)
        .with_context(|| format!("Failed to parse money string: {}", s))
}

fn extract_date_from_filename(filename: &str) -> Result<NaiveDate> {
    // Expected format: AccountsHoldings-YYYYMMDD.csv
    let parts: Vec<&str> = filename.split('-').collect();
    if parts.len() < 2 {
        anyhow::bail!("Invalid filename format: {}", filename);
    }

    let date_part = parts[1].replace(".csv", "");
    if date_part.len() != 8 {
        anyhow::bail!("Invalid date format in filename: {}", filename);
    }

    NaiveDate::parse_from_str(&date_part, "%Y%m%d")
        .with_context(|| format!("Failed to parse date from filename: {}", filename))
}

pub async fn import_csv_file(
    pool: &PgPool,
    portfolio_id: Uuid,
    file_path: &Path,
) -> Result<ImportResult> {
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid filename")?;

    let snapshot_date = extract_date_from_filename(filename)?;

    let file_content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {:?}", file_path))?;

    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(file_content.as_bytes());

    let mut accounts_created = 0;
    let mut holdings_created = 0;
    let mut errors = Vec::new();

    for (line_num, result) in reader.deserialize::<CsvRow>().enumerate() {
        match result {
            Ok(row) => {
                match process_row(pool, portfolio_id, snapshot_date, row).await {
                    Ok((account_new, holding_new)) => {
                        if account_new {
                            accounts_created += 1;
                        }
                        if holding_new {
                            holdings_created += 1;
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

    Ok(ImportResult {
        accounts_created,
        holdings_created,
        errors,
        snapshot_date,
    })
}

async fn process_row(
    pool: &PgPool,
    portfolio_id: Uuid,
    snapshot_date: NaiveDate,
    row: CsvRow,
) -> Result<(bool, bool)> {
    // Skip "Cash" entries without a ticker symbol
    if row.symbol.trim().is_empty() && row.holding.to_lowercase().contains("cash") {
        // Still create/update account but don't create holding
        let account_data = CreateAccount {
            account_number: row.account_number.clone(),
            account_nickname: row.account_nickname.clone(),
            client_id: Some(row.client_id.clone()),
            client_name: Some(row.client_name.clone()),
        };

        let existing = account_queries::find_by_account_number(
            pool,
            portfolio_id,
            &row.account_number,
        ).await?;

        let account_new = existing.is_none();

        account_queries::upsert(pool, portfolio_id, account_data).await?;

        return Ok((account_new, false));
    }

    // Skip rows without a ticker
    if row.symbol.trim().is_empty() {
        return Ok((false, false));
    }

    // Create or get account
    let account_data = CreateAccount {
        account_number: row.account_number.clone(),
        account_nickname: row.account_nickname.clone(),
        client_id: Some(row.client_id.clone()),
        client_name: Some(row.client_name.clone()),
    };

    let existing_account = account_queries::find_by_account_number(
        pool,
        portfolio_id,
        &row.account_number,
    ).await?;

    let account_new = existing_account.is_none();

    let account = account_queries::upsert(pool, portfolio_id, account_data).await?;

    // Parse holding data
    let holding_name = if row.holding.trim().is_empty() {
        None
    } else {
        Some(row.holding.clone())
    };

    let asset_category = if row.asset_category.trim().is_empty() {
        None
    } else {
        Some(row.asset_category.clone())
    };

    let industry = if row.industry.trim().is_empty() {
        None
    } else {
        Some(row.industry.clone())
    };

    let fund = if row.fund.trim().is_empty() {
        None
    } else {
        Some(row.fund.clone())
    };

    let quantity = parse_money_string(&row.quantity)?;
    let price = parse_money_string(&row.price)?;
    let average_cost = parse_money_string(&row.average_cost)?;
    let book_value = parse_money_string(&row.book_value)?;
    let market_value = parse_money_string(&row.market_value)?;

    let accrued_interest = if row.accrued_interest.trim().is_empty() {
        None
    } else {
        Some(parse_money_string(&row.accrued_interest)?)
    };

    let gain_loss = if row.gain_loss.trim().is_empty() {
        None
    } else {
        Some(parse_money_string(&row.gain_loss)?)
    };

    let gain_loss_pct = if row.gain_loss_pct.trim().is_empty() {
        None
    } else {
        Some(parse_money_string(&row.gain_loss_pct)?)
    };

    let percentage_of_assets = if row.percentage_of_assets.trim().is_empty() {
        None
    } else {
        Some(parse_money_string(&row.percentage_of_assets)?)
    };

    let holding_data = CreateHoldingSnapshot {
        ticker: row.symbol.clone(),
        holding_name,
        asset_category,
        industry,
        quantity,
        price,
        average_cost,
        book_value,
        market_value,
        fund,
        accrued_interest,
        gain_loss,
        gain_loss_pct,
        percentage_of_assets,
    };

    // Check if holding already exists
    let existing_holdings = holding_snapshot_queries::fetch_by_account_and_date(
        pool,
        account.id,
        snapshot_date,
    ).await?;

    let holding_new = !existing_holdings.iter().any(|h| h.ticker == row.symbol);

    holding_snapshot_queries::upsert(pool, account.id, snapshot_date, holding_data).await?;

    Ok((account_new, holding_new))
}

#[derive(Debug)]
pub struct ImportResult {
    pub accounts_created: usize,
    pub holdings_created: usize,
    pub errors: Vec<String>,
    pub snapshot_date: NaiveDate,
}
