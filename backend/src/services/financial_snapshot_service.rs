use bigdecimal::BigDecimal;
use std::str::FromStr;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::financial_planning_queries;
use crate::models::financial_planning::*;

// ==============================================================================
// Snapshot calculation constants
// ==============================================================================

const ASSUMED_RETURN_RATE: f64 = 0.06;
const WITHDRAWAL_RATE: f64 = 0.04;
const EXPENSE_RATIO: f64 = 0.70;

// ==============================================================================
// Snapshot detail structs (embedded in snapshot_data JSON)
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotDetail {
    pub net_worth_breakdown: NetWorthBreakdown,
    pub cash_flow: CashFlowProjection,
    pub retirement: Option<RetirementProjection>,
    pub goal_progress: Vec<GoalProgress>,
    pub recommendations: Vec<String>,
    pub household: Option<HouseholdCalculations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetWorthBreakdown {
    pub total_assets: f64,
    pub total_liabilities: f64,
    pub net_worth: f64,
    pub assets_by_type: Vec<CategoryAmount>,
    pub liabilities_by_type: Vec<CategoryAmount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryAmount {
    pub category: String,
    pub amount: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CashFlowProjection {
    pub monthly_gross_income: f64,
    pub monthly_taxes: f64,
    pub monthly_payroll_deductions: f64, // CPP, EI, benefit premiums etc.
    pub monthly_net_income: f64,
    pub estimated_monthly_expenses: f64,
    pub expenses_by_category: Vec<CategoryAmount>, // per-category breakdown
    pub monthly_cash_flow: f64,
    pub annual_cash_flow: f64,
    pub savings_rate: f64,
    /// true  = expenses come from actual survey entries
    /// false = 70% gross estimate was used
    pub using_actual_expenses: bool,
    /// true = housing excluded because mortgage is already in debt payments
    pub housing_excluded_from_expenses: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetirementProjection {
    pub current_retirement_savings: f64,
    pub annual_contribution: f64,
    pub years_to_retirement: i32,
    pub projected_total_at_retirement: f64,
    pub estimated_monthly_income: f64,
    pub assumed_return_rate: f64,
    pub assumed_withdrawal_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalProgress {
    pub goal_id: Uuid,
    pub goal_type: String,
    pub description: Option<String>,
    pub target_amount: f64,
    pub current_savings: f64,
    pub progress_percentage: f64,
    pub months_remaining: Option<i64>,
    pub monthly_contribution_needed: Option<f64>,
    /// Whether monthly_contribution_needed was calculated with compound growth.
    /// true → uses ASSUMED_RETURN_RATE (retirement), false → linear (all other goals).
    pub contribution_uses_growth: bool,
    pub status: String,
}

// ==============================================================================
// Household calculation structs
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseholdCalculations {
    // Individual - Primary
    pub primary_net_worth: f64,
    pub primary_total_assets: f64,
    pub primary_total_liabilities: f64,
    pub primary_monthly_income: f64,
    pub primary_monthly_expenses: f64,
    pub primary_monthly_cash_flow: f64,
    // Individual - Spouse
    pub spouse_net_worth: f64,
    pub spouse_total_assets: f64,
    pub spouse_total_liabilities: f64,
    pub spouse_monthly_income: f64,
    pub spouse_monthly_expenses: f64,
    pub spouse_monthly_cash_flow: f64,
    // Combined Household
    pub household_net_worth: f64,
    pub household_total_assets: f64,
    pub household_total_liabilities: f64,
    pub household_monthly_income: f64,
    pub household_monthly_expenses: f64,
    pub household_monthly_cash_flow: f64,
    pub household_annual_income: f64,
    // Expense breakdown
    pub shared_monthly_expenses: f64,
    pub primary_individual_expenses: f64,
    pub spouse_individual_expenses: f64,
}

// ==============================================================================
// Main snapshot generation
// ==============================================================================

pub async fn generate_snapshot(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<FinancialSnapshot, String> {
    // Fetch all survey data
    let assets = financial_planning_queries::get_assets(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch assets: {}", e))?;

    let liabilities = financial_planning_queries::get_liabilities(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch liabilities: {}", e))?;

    let income_info = financial_planning_queries::get_income_info(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch income info: {}", e))?;

    let additional_income = financial_planning_queries::get_additional_income(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch additional income: {}", e))?;

    let expenses = financial_planning_queries::get_expenses(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch expenses: {}", e))?;

    let household_expenses = financial_planning_queries::get_household_expenses(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch household expenses: {}", e))?;

    let personal_info = financial_planning_queries::get_personal_info(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch personal info: {}", e))?;

    let goals = financial_planning_queries::get_goals(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch goals: {}", e))?;

    // Determine if household mode is active
    let has_spouse = personal_info.as_ref().map(|p| p.has_spouse).unwrap_or(false);

    tracing::info!(
        survey_id = %survey_id,
        has_spouse = has_spouse,
        primary_salary = ?income_info.as_ref().and_then(|i| i.gross_annual_income.as_ref()).map(|v| v.to_string()),
        spouse_salary = ?income_info.as_ref().and_then(|i| i.spouse_gross_annual_income.as_ref()).map(|v| v.to_string()),
        additional_income_count = additional_income.len(),
        "snapshot: income inputs"
    );
    for ai in &additional_income {
        tracing::info!(
            income_type = %ai.income_type,
            owner = %ai.owner,
            monthly = %ai.monthly_amount,
            "snapshot: additional income item"
        );
    }

    // Calculate individual net worth (uses all assets/liabilities regardless of ownership for the primary person snapshot)
    let net_worth_breakdown = calculate_net_worth(&assets, &liabilities);

    // Calculate cash flow — household-combined when spouse is configured
    let cash_flow = estimate_monthly_cash_flow(&income_info, &additional_income, &expenses, &household_expenses, &liabilities, has_spouse);

    tracing::info!(
        monthly_gross = cash_flow.monthly_gross_income,
        monthly_taxes = cash_flow.monthly_taxes,
        monthly_payroll_deductions = cash_flow.monthly_payroll_deductions,
        monthly_net = cash_flow.monthly_net_income,
        monthly_cash_flow = cash_flow.monthly_cash_flow,
        "snapshot: cash flow result"
    );

    // Calculate retirement projection
    let retirement = project_retirement_income(
        &personal_info,
        &income_info,
        &assets,
        &goals,
    );

    // Calculate goal progress
    let goal_progress: Vec<GoalProgress> = goals
        .iter()
        .map(|g| calculate_goal_progress(g))
        .collect();

    // Calculate household data if spouse is present
    let household = if has_spouse {
        Some(calculate_household(
            &assets,
            &liabilities,
            &income_info,
            &additional_income,
            &household_expenses,
        ))
    } else {
        None
    };

    // Generate recommendations
    let recommendations = generate_recommendations(
        &net_worth_breakdown,
        &cash_flow,
        &retirement,
        &goal_progress,
        household.as_ref(),
    );

    let detail = SnapshotDetail {
        net_worth_breakdown: net_worth_breakdown.clone(),
        cash_flow: cash_flow.clone(),
        retirement: retirement.clone(),
        goal_progress,
        recommendations,
        household,
    };

    let snapshot_data = serde_json::to_value(&detail)
        .map_err(|e| format!("Failed to serialize snapshot: {}", e))?;

    let net_worth_bd = BigDecimal::from_str(&net_worth_breakdown.net_worth.to_string()).ok();
    let total_assets_bd = BigDecimal::from_str(&net_worth_breakdown.total_assets.to_string()).ok();
    let total_liabilities_bd = BigDecimal::from_str(&net_worth_breakdown.total_liabilities.to_string()).ok();
    let monthly_cash_flow_bd = BigDecimal::from_str(&cash_flow.monthly_cash_flow.to_string()).ok();
    let retirement_income_bd = retirement.as_ref()
        .map(|r| BigDecimal::from_str(&r.estimated_monthly_income.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    financial_planning_queries::create_snapshot(
        pool,
        survey_id,
        net_worth_bd,
        total_assets_bd,
        total_liabilities_bd,
        monthly_cash_flow_bd,
        retirement_income_bd,
        Some(snapshot_data),
    )
    .await
    .map_err(|e| format!("Failed to save snapshot: {}", e))
}

/// Generate and return just the household calculations (for the household endpoint).
/// Always re-computes from current data without persisting.
pub async fn generate_household_snapshot(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<serde_json::Value, String> {
    let personal_info = financial_planning_queries::get_personal_info(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch personal info: {}", e))?;

    let has_spouse = personal_info.as_ref().map(|p| p.has_spouse).unwrap_or(false);

    if !has_spouse {
        return Ok(serde_json::json!({
            "has_spouse": false,
            "message": "No spouse configured for this survey"
        }));
    }

    let assets = financial_planning_queries::get_assets(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch assets: {}", e))?;

    let liabilities = financial_planning_queries::get_liabilities(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch liabilities: {}", e))?;

    let income_info = financial_planning_queries::get_income_info(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch income info: {}", e))?;

    let household_expenses = financial_planning_queries::get_household_expenses(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch household expenses: {}", e))?;

    let additional_income = financial_planning_queries::get_additional_income(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch additional income: {}", e))?;

    let household = calculate_household(&assets, &liabilities, &income_info, &additional_income, &household_expenses);

    serde_json::to_value(&household)
        .map_err(|e| format!("Failed to serialize household snapshot: {}", e))
}

// ==============================================================================
// Household calculation functions
// ==============================================================================

fn calculate_household(
    assets: &[SurveyAsset],
    liabilities: &[SurveyLiability],
    income_info: &Option<SurveyIncomeInfo>,
    additional_income: &[SurveyAdditionalIncome],
    household_expenses: &[SurveyHouseholdExpense],
) -> HouseholdCalculations {
    // Attribute assets
    let (primary_assets, spouse_assets) = attribute_assets(assets);
    let (primary_liabilities, spouse_liabilities) = attribute_liabilities(liabilities);

    // Compute net worths
    let primary_net_worth = primary_assets - primary_liabilities;
    let spouse_net_worth = spouse_assets - spouse_liabilities;
    let household_net_worth = primary_net_worth + spouse_net_worth;

    // Income attribution — employment income
    let primary_employment = income_info.as_ref()
        .and_then(|i| i.gross_annual_income.as_ref())
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .map(|annual| annual / 12.0)
        .unwrap_or(0.0);

    let spouse_employment = income_info.as_ref()
        .and_then(|i| i.spouse_gross_annual_income.as_ref())
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .map(|annual| annual / 12.0)
        .unwrap_or(0.0);

    // Additional income attribution
    let primary_additional: f64 = additional_income.iter()
        .filter(|i| i.is_recurring.unwrap_or(true) && i.owner == "mine")
        .filter_map(|i| i.monthly_amount.to_string().parse::<f64>().ok())
        .sum();

    let spouse_additional: f64 = additional_income.iter()
        .filter(|i| i.is_recurring.unwrap_or(true) && i.owner == "spouse")
        .filter_map(|i| i.monthly_amount.to_string().parse::<f64>().ok())
        .sum();

    let primary_monthly_income = primary_employment + primary_additional;
    let spouse_monthly_income = spouse_employment + spouse_additional;

    let household_monthly_income = primary_monthly_income + spouse_monthly_income;

    // Expense attribution
    let mut shared_expenses = 0.0_f64;
    let mut primary_individual = 0.0_f64;
    let mut spouse_individual = 0.0_f64;

    for expense in household_expenses {
        let amount = expense.monthly_amount.to_string().parse::<f64>().unwrap_or(0.0);
        match expense.expense_type.as_str() {
            "shared" => shared_expenses += amount,
            "mine" => primary_individual += amount,
            "spouse" => spouse_individual += amount,
            _ => shared_expenses += amount,
        }
    }

    let primary_share_of_shared = shared_expenses / 2.0;
    let spouse_share_of_shared = shared_expenses / 2.0;

    let primary_monthly_expenses = primary_individual + primary_share_of_shared;
    let spouse_monthly_expenses = spouse_individual + spouse_share_of_shared;
    let household_monthly_expenses = shared_expenses + primary_individual + spouse_individual;

    let primary_monthly_cash_flow = primary_monthly_income - primary_monthly_expenses;
    let spouse_monthly_cash_flow = spouse_monthly_income - spouse_monthly_expenses;
    let household_monthly_cash_flow = household_monthly_income - household_monthly_expenses;

    HouseholdCalculations {
        primary_net_worth,
        primary_total_assets: primary_assets,
        primary_total_liabilities: primary_liabilities,
        primary_monthly_income,
        primary_monthly_expenses,
        primary_monthly_cash_flow,
        spouse_net_worth,
        spouse_total_assets: spouse_assets,
        spouse_total_liabilities: spouse_liabilities,
        spouse_monthly_income,
        spouse_monthly_expenses,
        spouse_monthly_cash_flow,
        household_net_worth,
        household_total_assets: primary_assets + spouse_assets,
        household_total_liabilities: primary_liabilities + spouse_liabilities,
        household_monthly_income,
        household_monthly_expenses,
        household_monthly_cash_flow,
        household_annual_income: household_monthly_income * 12.0,
        shared_monthly_expenses: shared_expenses,
        primary_individual_expenses: primary_individual,
        spouse_individual_expenses: spouse_individual,
    }
}


/// Returns (primary_total, spouse_total) attributed asset values.
fn attribute_assets(assets: &[SurveyAsset]) -> (f64, f64) {
    let mut primary = 0.0_f64;
    let mut spouse = 0.0_f64;

    for asset in assets {
        let value = asset.current_value.to_string().parse::<f64>().unwrap_or(0.0);
        match asset.ownership.as_str() {
            "mine" => primary += value,
            "spouse" => spouse += value,
            "joint" => {
                let split = asset.joint_split_percentage
                    .as_ref()
                    .and_then(|v| v.to_string().parse::<f64>().ok())
                    .unwrap_or(50.0);
                let primary_share = value * split / 100.0;
                primary += primary_share;
                spouse += value - primary_share;
            }
            _ => primary += value,
        }
    }

    (primary, spouse)
}

/// Returns (primary_total, spouse_total) attributed liability values.
fn attribute_liabilities(liabilities: &[SurveyLiability]) -> (f64, f64) {
    let mut primary = 0.0_f64;
    let mut spouse = 0.0_f64;

    for liability in liabilities {
        let value = liability.balance.to_string().parse::<f64>().unwrap_or(0.0);
        match liability.ownership.as_str() {
            "mine" => primary += value,
            "spouse" => spouse += value,
            "joint" => {
                let split = liability.joint_split_percentage
                    .as_ref()
                    .and_then(|v| v.to_string().parse::<f64>().ok())
                    .unwrap_or(50.0);
                let primary_share = value * split / 100.0;
                primary += primary_share;
                spouse += value - primary_share;
            }
            _ => primary += value,
        }
    }

    (primary, spouse)
}

// ==============================================================================
// Individual calculation functions
// ==============================================================================

fn calculate_net_worth(
    assets: &[SurveyAsset],
    liabilities: &[SurveyLiability],
) -> NetWorthBreakdown {
    let total_assets: f64 = assets
        .iter()
        .map(|a| a.current_value.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

    let total_liabilities: f64 = liabilities
        .iter()
        .map(|l| l.balance.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

    // Group assets by type
    let mut assets_by_type: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for asset in assets {
        let value = asset.current_value.to_string().parse::<f64>().unwrap_or(0.0);
        *assets_by_type.entry(asset.asset_type.clone()).or_insert(0.0) += value;
    }

    // Group liabilities by type
    let mut liabilities_by_type: std::collections::HashMap<String, f64> = std::collections::HashMap::new();
    for liability in liabilities {
        let value = liability.balance.to_string().parse::<f64>().unwrap_or(0.0);
        *liabilities_by_type.entry(liability.liability_type.clone()).or_insert(0.0) += value;
    }

    NetWorthBreakdown {
        total_assets,
        total_liabilities,
        net_worth: total_assets - total_liabilities,
        assets_by_type: assets_by_type
            .into_iter()
            .map(|(category, amount)| CategoryAmount { category, amount })
            .collect(),
        liabilities_by_type: liabilities_by_type
            .into_iter()
            .map(|(category, amount)| CategoryAmount { category, amount })
            .collect(),
    }
}

fn parse_rate(bd: &Option<bigdecimal::BigDecimal>) -> f64 {
    bd.as_ref()
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .unwrap_or(0.0) / 100.0
}

/// Returns true if the income type is taxed as investment income (dividends, interest).
fn is_investment_income(income_type: &str) -> bool {
    matches!(income_type, "dividends" | "interest")
}

fn estimate_monthly_cash_flow(
    income_info: &Option<SurveyIncomeInfo>,
    additional_income: &[SurveyAdditionalIncome],
    expenses: &[SurveyExpense],
    household_expenses: &[SurveyHouseholdExpense],
    liabilities: &[SurveyLiability],
    has_spouse: bool,
) -> CashFlowProjection {
    let income = match income_info {
        Some(info) => info,
        None => {
            return CashFlowProjection {
                monthly_gross_income: 0.0,
                monthly_taxes: 0.0,
                monthly_payroll_deductions: 0.0,
                monthly_net_income: 0.0,
                estimated_monthly_expenses: 0.0,
                expenses_by_category: vec![],
                monthly_cash_flow: 0.0,
                annual_cash_flow: 0.0,
                savings_rate: 0.0,
                using_actual_expenses: false,
                housing_excluded_from_expenses: false,
            };
        }
    };

    // Tax rates per person and per income type
    let primary_salary_rate   = parse_rate(&income.effective_tax_rate);
    let primary_invest_rate   = parse_rate(&income.investment_income_tax_rate);
    let spouse_salary_rate    = parse_rate(&income.spouse_effective_tax_rate);
    let spouse_invest_rate    = parse_rate(&income.spouse_investment_income_tax_rate);

    // Primary salary (always annual)
    let primary_salary = income.gross_annual_income
        .as_ref()
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .map(|a| a / 12.0)
        .unwrap_or(0.0);

    // Spouse salary (only when has_spouse)
    let spouse_salary = if has_spouse {
        income.spouse_gross_annual_income
            .as_ref()
            .and_then(|v| v.to_string().parse::<f64>().ok())
            .map(|a| a / 12.0)
            .unwrap_or(0.0)
    } else {
        0.0
    };

    // Additional income: include both owners when household mode, mine-only otherwise.
    // Apply investment_income_tax_rate to dividends/interest, salary rate to everything else.
    let (additional_gross, additional_taxes) = additional_income
        .iter()
        .filter(|i| {
            i.is_recurring.unwrap_or(true) &&
            (i.owner == "mine" || (has_spouse && i.owner == "spouse"))
        })
        .fold((0.0_f64, 0.0_f64), |(gross_acc, tax_acc), i| {
            let amount = i.monthly_amount.to_string().parse::<f64>().unwrap_or(0.0);
            let rate = if i.owner == "spouse" {
                if is_investment_income(&i.income_type) { spouse_invest_rate } else { spouse_salary_rate }
            } else {
                if is_investment_income(&i.income_type) { primary_invest_rate } else { primary_salary_rate }
            };
            (gross_acc + amount, tax_acc + amount * rate)
        });

    let primary_salary_taxes = primary_salary * primary_salary_rate;
    let spouse_salary_taxes  = spouse_salary  * spouse_salary_rate;

    // Payroll deductions (CPP, EI, benefit premiums etc.) — monthly dollar amounts
    let primary_deductions = income.monthly_deductions
        .as_ref()
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .unwrap_or(0.0);
    let spouse_deductions = if has_spouse {
        income.spouse_monthly_deductions
            .as_ref()
            .and_then(|v| v.to_string().parse::<f64>().ok())
            .unwrap_or(0.0)
    } else {
        0.0
    };

    let monthly_gross              = primary_salary + spouse_salary + additional_gross;
    let monthly_taxes              = primary_salary_taxes + spouse_salary_taxes + additional_taxes;
    let monthly_payroll_deductions = primary_deductions + spouse_deductions;
    let monthly_net                = monthly_gross - monthly_taxes - monthly_payroll_deductions;

    // Calculate total monthly debt payments from all liabilities
    let monthly_debt_payments: f64 = liabilities
        .iter()
        .filter_map(|liability| {
            let payment = liability
                .monthly_payment
                .as_ref()
                .and_then(|v| v.to_string().parse::<f64>().ok())
                .unwrap_or(0.0);

            if payment == 0.0 {
                return None;
            }

            // Convert payment to monthly based on payment frequency
            let monthly_payment = match liability.payment_frequency.as_deref() {
                Some("bi_weekly") => payment * 26.0 / 12.0,
                Some("weekly") => payment * 52.0 / 12.0,
                Some("monthly") | _ => payment,
            };

            Some(monthly_payment)
        })
        .sum();

    // Detect mortgage liabilities that have a monthly payment already captured in
    // the debt-payments line. When such a liability exists, exclude the 'housing'
    // expense category from living expenses to prevent double-counting.
    let has_mortgage_payment = liabilities.iter().any(|l| {
        l.liability_type == "mortgage" &&
        l.monthly_payment
            .as_ref()
            .and_then(|v| v.to_string().parse::<f64>().ok())
            .unwrap_or(0.0) > 0.0
    });

    // Build expense total and per-category breakdown.
    // When has_spouse=true, income is household-combined, so expenses must also be
    // the full household total (not just primary's half). Housing is excluded when
    // a mortgage liability with a monthly payment already covers it in debt payments.
    let mut expense_map: std::collections::HashMap<String, f64> = std::collections::HashMap::new();

    if !expenses.is_empty() {
        for e in expenses.iter().filter(|e| e.is_recurring.unwrap_or(true)) {
            if has_mortgage_payment && e.expense_category == "housing" { continue; }
            let amount = e.monthly_amount.to_string().parse::<f64>().unwrap_or(0.0);
            *expense_map.entry(e.expense_category.clone()).or_insert(0.0) += amount;
        }
    } else if !household_expenses.is_empty() {
        for e in household_expenses.iter() {
            if has_mortgage_payment && e.expense_category == "housing" { continue; }
            let amount = e.monthly_amount.to_string().parse::<f64>().unwrap_or(0.0);
            // When household mode: use full amounts (income is already household-combined).
            // When individual mode: primary's share only (mine + half shared).
            let counted = if has_spouse {
                amount
            } else {
                match e.expense_type.as_str() {
                    "mine" => amount,
                    "shared" => amount / 2.0,
                    _ => 0.0,
                }
            };
            if counted > 0.0 {
                *expense_map.entry(e.expense_category.clone()).or_insert(0.0) += counted;
            }
        }
    }

    let total_actual_expenses: f64 = expense_map.values().sum();
    let mut expenses_by_category: Vec<CategoryAmount> = expense_map
        .into_iter()
        .map(|(category, amount)| CategoryAmount { category, amount })
        .collect();
    expenses_by_category.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap_or(std::cmp::Ordering::Equal));

    let using_actual = total_actual_expenses > 0.0;
    let monthly_expenses = if using_actual {
        total_actual_expenses
    } else {
        monthly_gross * EXPENSE_RATIO // Fall back to 70% estimate if no expenses entered
    };

    // Cash flow is computed from net (post-tax) income
    let monthly_cash_flow = monthly_net - monthly_expenses - monthly_debt_payments;
    let savings_rate = if monthly_net > 0.0 {
        (monthly_cash_flow / monthly_net) * 100.0
    } else {
        0.0
    };

    CashFlowProjection {
        monthly_gross_income: monthly_gross,
        monthly_taxes,
        monthly_payroll_deductions,
        monthly_net_income: monthly_net,
        estimated_monthly_expenses: monthly_expenses,
        expenses_by_category,
        monthly_cash_flow,
        annual_cash_flow: monthly_cash_flow * 12.0,
        savings_rate,
        using_actual_expenses: using_actual,
        housing_excluded_from_expenses: has_mortgage_payment && using_actual,
    }
}

fn project_retirement_income(
    personal_info: &Option<SurveyPersonalInfo>,
    income_info: &Option<SurveyIncomeInfo>,
    assets: &[SurveyAsset],
    goals: &[SurveyGoal],
) -> Option<RetirementProjection> {
    let income = income_info.as_ref()?;
    let personal = personal_info.as_ref()?;

    let birth_year = personal.birth_year?;
    let current_year = Utc::now().year() as i32;
    let current_age = current_year - birth_year;

    let retirement_age = income.planned_retirement_age.unwrap_or(65);
    let years_to_retirement = retirement_age - current_age;

    if years_to_retirement <= 0 {
        return None;
    }

    let gross_annual = income.gross_annual_income
        .as_ref()
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .unwrap_or(0.0);

    let contribution_rate = income.retirement_contribution_rate
        .as_ref()
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .unwrap_or(0.0) / 100.0;

    let employer_match = income.employer_match_rate
        .as_ref()
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .unwrap_or(0.0) / 100.0;

    let annual_contribution = gross_annual * (contribution_rate + employer_match);

    // Prefer the retirement goal's current_savings if explicitly set by the user,
    // because that represents what the user considers their retirement savings.
    // Fall back to summing retirement-type assets if no goal is set.
    let retirement_goal_savings = goals.iter()
        .find(|g| g.goal_type == "retirement")
        .and_then(|g| g.current_savings.as_ref())
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .filter(|&v| v > 0.0);

    let current_retirement_savings: f64 = retirement_goal_savings.unwrap_or_else(|| {
        assets
            .iter()
            .filter(|a| {
                matches!(
                    a.asset_type.as_str(),
                    "retirement" | "rrsp" | "lira" | "rrif" | "tfsa"
                )
            })
            .map(|a| a.current_value.to_string().parse::<f64>().unwrap_or(0.0))
            .sum()
    });

    // Future value of current savings: FV = PV * (1 + r)^n
    let fv_current = current_retirement_savings
        * (1.0 + ASSUMED_RETURN_RATE).powi(years_to_retirement);

    // Future value of annual contributions (annuity): FV = PMT * [((1+r)^n - 1) / r]
    let fv_contributions = if ASSUMED_RETURN_RATE > 0.0 {
        annual_contribution
            * (((1.0 + ASSUMED_RETURN_RATE).powi(years_to_retirement) - 1.0) / ASSUMED_RETURN_RATE)
    } else {
        annual_contribution * years_to_retirement as f64
    };

    let total_at_retirement = fv_current + fv_contributions;

    // 4% withdrawal rule for monthly income
    let monthly_income = (total_at_retirement * WITHDRAWAL_RATE) / 12.0;

    Some(RetirementProjection {
        current_retirement_savings,
        annual_contribution,
        years_to_retirement,
        projected_total_at_retirement: total_at_retirement,
        estimated_monthly_income: monthly_income,
        assumed_return_rate: ASSUMED_RETURN_RATE,
        assumed_withdrawal_rate: WITHDRAWAL_RATE,
    })
}

fn calculate_goal_progress(goal: &SurveyGoal) -> GoalProgress {
    let target = goal.target_amount
        .as_ref()
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .unwrap_or(0.0);

    let current = goal.current_savings
        .as_ref()
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .unwrap_or(0.0);

    let progress_percentage = if target > 0.0 {
        ((current / target) * 100.0).min(100.0)
    } else {
        0.0
    };

    let months_remaining = goal.target_date.map(|target_date| {
        let today = Utc::now().date_naive();
        let days = (target_date - today).num_days();
        (days as f64 / 30.44).ceil() as i64
    });

    let monthly_contribution_needed = months_remaining.and_then(|months| {
        if months <= 0 || target <= current {
            return None;
        }
        let n = months as f64;

        if goal.goal_type == "retirement" {
            // Use compound-growth PMT formula — same assumption as the retirement projection
            // (6% annual = 0.5% monthly), so the two numbers stay consistent.
            let r = ASSUMED_RETURN_RATE / 12.0;
            // How much does current savings grow to by the target date?
            let fv_current = current * (1.0 + r).powf(n);
            let fv_still_needed = target - fv_current;
            if fv_still_needed <= 0.0 {
                // Existing savings + growth already reach the target — no contribution needed
                Some(0.0)
            } else {
                // PMT = FV_needed × r / ((1 + r)^n − 1)
                Some(fv_still_needed * r / ((1.0 + r).powf(n) - 1.0))
            }
        } else {
            // For non-retirement goals the money is typically in cash/savings —
            // use a conservative linear estimate (no growth assumed).
            Some((target - current) / n)
        }
    });

    let status = determine_goal_status(progress_percentage, months_remaining);

    GoalProgress {
        goal_id: goal.id,
        goal_type: goal.goal_type.clone(),
        description: goal.description.clone(),
        target_amount: target,
        current_savings: current,
        progress_percentage,
        months_remaining,
        monthly_contribution_needed,
        contribution_uses_growth: goal.goal_type == "retirement",
        status,
    }
}

fn determine_goal_status(progress: f64, months_remaining: Option<i64>) -> String {
    match months_remaining {
        Some(months) if months <= 0 => {
            if progress >= 100.0 {
                "achieved".to_string()
            } else {
                "overdue".to_string()
            }
        }
        Some(months) => {
            // Expected progress based on time elapsed (linear)
            let expected = 100.0 - (months as f64 / 12.0 * 10.0).min(100.0);
            if progress >= expected {
                "on_track".to_string()
            } else {
                "behind".to_string()
            }
        }
        None => {
            if progress >= 100.0 {
                "achieved".to_string()
            } else {
                "in_progress".to_string()
            }
        }
    }
}

fn generate_recommendations(
    net_worth: &NetWorthBreakdown,
    cash_flow: &CashFlowProjection,
    retirement: &Option<RetirementProjection>,
    goals: &[GoalProgress],
    household: Option<&HouseholdCalculations>,
) -> Vec<String> {
    let mut recommendations = Vec::new();

    // Check savings rate
    if cash_flow.savings_rate < 20.0 && cash_flow.monthly_gross_income > 0.0 {
        recommendations.push(
            "Consider increasing your savings rate. Financial experts recommend saving at least 20% of gross income.".to_string()
        );
    }

    // Check net worth
    if net_worth.net_worth < 0.0 {
        recommendations.push(
            "Your liabilities exceed your assets. Focus on paying down high-interest debt to improve your net worth.".to_string()
        );
    }

    // Check retirement
    if let Some(ret) = retirement {
        if ret.annual_contribution == 0.0 {
            recommendations.push(
                "You are not currently contributing to retirement savings. Consider starting even with a small amount to benefit from compound growth.".to_string()
            );
        }
    }

    // Check goals
    let overdue_goals: Vec<&GoalProgress> = goals.iter().filter(|g| g.status == "overdue").collect();
    if !overdue_goals.is_empty() {
        recommendations.push(format!(
            "You have {} overdue financial goal(s). Consider revising timelines or increasing contributions.",
            overdue_goals.len()
        ));
    }

    let behind_goals: Vec<&GoalProgress> = goals.iter().filter(|g| g.status == "behind").collect();
    if !behind_goals.is_empty() {
        recommendations.push(format!(
            "You are behind on {} goal(s). Review your monthly contributions to get back on track.",
            behind_goals.len()
        ));
    }

    // Emergency fund check
    let liquid_assets: f64 = net_worth
        .assets_by_type
        .iter()
        .filter(|a| a.category == "liquid")
        .map(|a| a.amount)
        .sum();

    if liquid_assets < cash_flow.estimated_monthly_expenses * 3.0
        && cash_flow.estimated_monthly_expenses > 0.0
    {
        recommendations.push(
            "Your liquid assets may not cover 3 months of expenses. Consider building an emergency fund.".to_string()
        );
    }

    // Household-specific recommendations
    if let Some(hh) = household {
        if hh.spouse_monthly_income > 0.0 && hh.household_monthly_cash_flow < 0.0 {
            recommendations.push(
                "Your combined household is spending more than it earns. Review shared and individual expenses to find savings opportunities.".to_string()
            );
        }

        let housing_pct = if hh.household_monthly_income > 0.0 {
            hh.shared_monthly_expenses / hh.household_monthly_income * 100.0
        } else {
            0.0
        };
        if housing_pct > 50.0 {
            recommendations.push(format!(
                "Shared household expenses represent {:.0}% of combined income. Consider ways to reduce joint costs.",
                housing_pct
            ));
        }
    }

    if recommendations.is_empty() {
        recommendations.push(
            "Your financial profile looks solid. Continue monitoring and adjusting as your situation evolves.".to_string()
        );
    }

    recommendations
}

// Expose chrono::Datelike for year()
use chrono::Datelike;
