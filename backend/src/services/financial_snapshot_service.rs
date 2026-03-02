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
    pub estimated_monthly_expenses: f64,
    pub monthly_cash_flow: f64,
    pub annual_cash_flow: f64,
    pub savings_rate: f64,
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
    pub status: String,
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

    let personal_info = financial_planning_queries::get_personal_info(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch personal info: {}", e))?;

    let goals = financial_planning_queries::get_goals(pool, survey_id)
        .await
        .map_err(|e| format!("Failed to fetch goals: {}", e))?;

    // Calculate net worth
    let net_worth_breakdown = calculate_net_worth(&assets, &liabilities);

    // Calculate cash flow
    let cash_flow = estimate_monthly_cash_flow(&income_info, &additional_income, &expenses, &liabilities);

    // Calculate retirement projection
    let retirement = project_retirement_income(
        &personal_info,
        &income_info,
        &assets,
    );

    // Calculate goal progress
    let goal_progress: Vec<GoalProgress> = goals
        .iter()
        .map(|g| calculate_goal_progress(g))
        .collect();

    // Generate recommendations
    let recommendations = generate_recommendations(
        &net_worth_breakdown,
        &cash_flow,
        &retirement,
        &goal_progress,
    );

    let detail = SnapshotDetail {
        net_worth_breakdown: net_worth_breakdown.clone(),
        cash_flow: cash_flow.clone(),
        retirement: retirement.clone(),
        goal_progress,
        recommendations,
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

// ==============================================================================
// Calculation functions
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

fn estimate_monthly_cash_flow(
    income_info: &Option<SurveyIncomeInfo>,
    additional_income: &[SurveyAdditionalIncome],
    expenses: &[SurveyExpense],
    liabilities: &[SurveyLiability],
) -> CashFlowProjection {
    let income = match income_info {
        Some(info) => info,
        None => {
            return CashFlowProjection {
                monthly_gross_income: 0.0,
                estimated_monthly_expenses: 0.0,
                monthly_cash_flow: 0.0,
                annual_cash_flow: 0.0,
                savings_rate: 0.0,
            };
        }
    };

    // gross_annual_income is ALWAYS annual, so divide by 12
    // pay_frequency just indicates how often they get paid, not the value's frequency
    let gross_annual = income.gross_annual_income
        .as_ref()
        .and_then(|v| v.to_string().parse::<f64>().ok())
        .unwrap_or(0.0);

    let employment_income_monthly = gross_annual / 12.0;

    // Add recurring additional income sources (dividends, rental, etc.)
    let additional_income_monthly: f64 = additional_income
        .iter()
        .filter(|i| i.is_recurring.unwrap_or(true))
        .filter_map(|i| i.monthly_amount.to_string().parse::<f64>().ok())
        .sum();

    let monthly_gross = employment_income_monthly + additional_income_monthly;

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

    // Calculate actual expenses from survey if available, otherwise use 70% estimate
    let total_actual_expenses: f64 = expenses
        .iter()
        .filter(|e| e.is_recurring.unwrap_or(true))
        .filter_map(|e| e.monthly_amount.to_string().parse::<f64>().ok())
        .sum();

    let monthly_expenses = if total_actual_expenses > 0.0 {
        total_actual_expenses
    } else {
        monthly_gross * EXPENSE_RATIO  // Fall back to 70% estimate if no expenses entered
    };

    let monthly_cash_flow = monthly_gross - monthly_expenses - monthly_debt_payments;
    let savings_rate = if monthly_gross > 0.0 {
        (monthly_cash_flow / monthly_gross) * 100.0
    } else {
        0.0
    };

    CashFlowProjection {
        monthly_gross_income: monthly_gross,
        estimated_monthly_expenses: monthly_expenses,
        monthly_cash_flow,
        annual_cash_flow: monthly_cash_flow * 12.0,
        savings_rate,
    }
}

fn project_retirement_income(
    personal_info: &Option<SurveyPersonalInfo>,
    income_info: &Option<SurveyIncomeInfo>,
    assets: &[SurveyAsset],
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

    // Sum retirement-type assets (including Canadian registered accounts)
    let current_retirement_savings: f64 = assets
        .iter()
        .filter(|a| {
            matches!(
                a.asset_type.as_str(),
                "retirement" | "rrsp" | "lira" | "rrif" | "tfsa"
            )
        })
        .map(|a| a.current_value.to_string().parse::<f64>().unwrap_or(0.0))
        .sum();

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
        if months > 0 && target > current {
            Some((target - current) / months as f64)
        } else {
            None
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

    if recommendations.is_empty() {
        recommendations.push(
            "Your financial profile looks solid. Continue monitoring and adjusting as your situation evolves.".to_string()
        );
    }

    recommendations
}

// Expose chrono::Datelike for year()
use chrono::Datelike;
