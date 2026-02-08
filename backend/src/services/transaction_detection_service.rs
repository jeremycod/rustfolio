use anyhow::Result;
use bigdecimal::{BigDecimal, ToPrimitive, FromPrimitive};
use chrono::NaiveDate;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

use crate::db::{detected_transaction_queries, holding_snapshot_queries, cash_flow_queries};
use crate::models::{CreateDetectedTransaction, HoldingSnapshot, TransactionType, CreateCashFlow, FlowType};

pub async fn detect_transactions_between_snapshots(
    pool: &PgPool,
    account_id: Uuid,
    from_date: NaiveDate,
    to_date: NaiveDate,
) -> Result<usize> {
    // Fetch holdings from both snapshots
    let from_holdings = holding_snapshot_queries::fetch_by_account_and_date(pool, account_id, from_date).await?;
    let to_holdings = holding_snapshot_queries::fetch_by_account_and_date(pool, account_id, to_date).await?;

    // Build maps for easier comparison
    let from_map: HashMap<String, &HoldingSnapshot> = from_holdings
        .iter()
        .map(|h| (h.ticker.clone(), h))
        .collect();

    let to_map: HashMap<String, &HoldingSnapshot> = to_holdings
        .iter()
        .map(|h| (h.ticker.clone(), h))
        .collect();

    // Delete any existing transactions for this snapshot
    detected_transaction_queries::delete_transactions_for_snapshot(pool, account_id, to_date).await?;

    let mut transactions_created = 0;

    // Detect cash flow changes (deposits/withdrawals)
    // Cash holdings have empty ticker
    let from_cash = from_holdings.iter().find(|h| h.ticker.is_empty());
    let to_cash = to_holdings.iter().find(|h| h.ticker.is_empty());

    if let (Some(from_cash), Some(to_cash)) = (from_cash, to_cash) {
        let from_amount = from_cash.quantity.to_f64().unwrap_or(0.0);
        let to_amount = to_cash.quantity.to_f64().unwrap_or(0.0);
        let cash_change = to_amount - from_amount;

        // Only create cash flow if change is significant (more than $1)
        if cash_change.abs() > 1.0 {
            let flow_type = if cash_change > 0.0 {
                FlowType::Deposit
            } else {
                FlowType::Withdrawal
            };

            let cash_flow = CreateCashFlow {
                flow_type,
                amount: BigDecimal::from_f64(cash_change.abs()).unwrap_or_else(|| BigDecimal::from(0)),
                flow_date: to_date,
                description: Some(format!(
                    "Auto-detected {} from snapshot comparison: ${:.2} -> ${:.2}",
                    if cash_change > 0.0 { "deposit" } else { "withdrawal" },
                    from_amount,
                    to_amount
                )),
            };

            cash_flow_queries::create(pool, account_id, cash_flow).await?;

            // Update account totals
            cash_flow_queries::update_account_totals(pool, account_id).await?;

            transactions_created += 1;
        }
    } else if let Some(to_cash) = to_cash {
        // First snapshot with cash - consider it as initial deposit
        let cash_amount = to_cash.quantity.to_f64().unwrap_or(0.0);

        if cash_amount > 1.0 {
            let cash_flow = CreateCashFlow {
                flow_type: FlowType::Deposit,
                amount: BigDecimal::from_f64(cash_amount).unwrap_or_else(|| BigDecimal::from(0)),
                flow_date: to_date,
                description: Some(format!("Initial cash position: ${:.2}", cash_amount)),
            };

            cash_flow_queries::create(pool, account_id, cash_flow).await?;
            cash_flow_queries::update_account_totals(pool, account_id).await?;

            transactions_created += 1;
        }
    }

    // Detect new holdings (BUY) - skip cash entries
    for (ticker, to_holding) in &to_map {
        // Skip cash entries (empty ticker)
        if ticker.is_empty() {
            continue;
        }
        if let Some(from_holding) = from_map.get(ticker) {
            // Holding existed in both snapshots - check for quantity change
            let from_qty = from_holding.quantity.to_f64().unwrap_or(0.0);
            let to_qty = to_holding.quantity.to_f64().unwrap_or(0.0);

            if (to_qty - from_qty).abs() > 0.01 {
                // Quantity changed
                let qty_change = to_qty - from_qty;
                let transaction_type = if qty_change > 0.0 {
                    TransactionType::Buy
                } else {
                    TransactionType::Sell
                };

                let qty_change_bd = BigDecimal::from_f64(qty_change.abs()).unwrap_or_else(|| BigDecimal::from(0));
                let amount = &qty_change_bd * &to_holding.price;

                let transaction = CreateDetectedTransaction {
                    transaction_type,
                    ticker: ticker.clone(),
                    quantity: Some(qty_change_bd),
                    price: Some(to_holding.price.clone()),
                    amount: Some(amount),
                    from_snapshot_date: Some(from_date),
                    to_snapshot_date: Some(to_date),
                    description: Some(format!(
                        "Detected {} change: {:.2} -> {:.2}",
                        if qty_change > 0.0 { "buy" } else { "sell" },
                        from_qty,
                        to_qty
                    )),
                };

                detected_transaction_queries::create(pool, account_id, to_date, transaction).await?;
                transactions_created += 1;
            }
        } else {
            // New holding - BUY
            let amount = &to_holding.quantity * &to_holding.price;

            let transaction = CreateDetectedTransaction {
                transaction_type: TransactionType::Buy,
                ticker: ticker.clone(),
                quantity: Some(to_holding.quantity.clone()),
                price: Some(to_holding.price.clone()),
                amount: Some(amount),
                from_snapshot_date: Some(from_date),
                to_snapshot_date: Some(to_date),
                description: Some(format!("New position detected")),
            };

            detected_transaction_queries::create(pool, account_id, to_date, transaction).await?;
            transactions_created += 1;
        }
    }

    // Detect removed holdings (SELL) - skip cash entries
    for (ticker, from_holding) in &from_map {
        // Skip cash entries (empty ticker)
        if ticker.is_empty() {
            continue;
        }

        if !to_map.contains_key(ticker) {
            // Holding was removed - SELL
            let amount = &from_holding.quantity * &from_holding.price;

            let transaction = CreateDetectedTransaction {
                transaction_type: TransactionType::Sell,
                ticker: ticker.clone(),
                quantity: Some(from_holding.quantity.clone()),
                price: Some(from_holding.price.clone()),
                amount: Some(amount),
                from_snapshot_date: Some(from_date),
                to_snapshot_date: Some(to_date),
                description: Some(format!("Position closed")),
            };

            detected_transaction_queries::create(pool, account_id, to_date, transaction).await?;
            transactions_created += 1;
        }
    }

    Ok(transactions_created)
}

pub async fn detect_transactions_for_new_snapshot(
    pool: &PgPool,
    account_id: Uuid,
    new_snapshot_date: NaiveDate,
) -> Result<usize> {
    // Find the previous snapshot date
    let all_dates = sqlx::query_scalar::<_, NaiveDate>(
        "SELECT DISTINCT snapshot_date FROM holdings_snapshots
         WHERE account_id = $1 AND snapshot_date < $2
         ORDER BY snapshot_date DESC
         LIMIT 1"
    )
    .bind(account_id)
    .bind(new_snapshot_date)
    .fetch_optional(pool)
    .await?;

    if let Some(previous_date) = all_dates {
        detect_transactions_between_snapshots(pool, account_id, previous_date, new_snapshot_date).await
    } else {
        // No previous snapshot - this is the first import
        // Don't create any transactions, just starting point
        Ok(0)
    }
}
