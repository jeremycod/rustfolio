use bigdecimal::BigDecimal;
use std::str::FromStr;
use crate::models::financial_planning::*;
use sqlx::PgPool;
use uuid::Uuid;

// ==============================================================================
// Survey CRUD Operations
// ==============================================================================

pub async fn create_survey(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<FinancialSurvey, sqlx::Error> {
    sqlx::query_as::<_, FinancialSurvey>(
        r#"
        INSERT INTO financial_surveys (user_id, status)
        VALUES ($1, 'draft')
        RETURNING *
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
}

pub async fn get_survey(pool: &PgPool, survey_id: Uuid) -> Result<FinancialSurvey, sqlx::Error> {
    sqlx::query_as::<_, FinancialSurvey>(
        "SELECT * FROM financial_surveys WHERE id = $1",
    )
    .bind(survey_id)
    .fetch_one(pool)
    .await
}

pub async fn get_surveys_for_user(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<FinancialSurvey>, sqlx::Error> {
    sqlx::query_as::<_, FinancialSurvey>(
        r#"
        SELECT * FROM financial_surveys
        WHERE user_id = $1
        ORDER BY updated_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

pub async fn update_survey_status(
    pool: &PgPool,
    survey_id: Uuid,
    status: &str,
) -> Result<FinancialSurvey, sqlx::Error> {
    let completed_at = if status == "completed" {
        Some(chrono::Utc::now())
    } else {
        None
    };

    sqlx::query_as::<_, FinancialSurvey>(
        r#"
        UPDATE financial_surveys
        SET status = $2, completed_at = COALESCE($3, completed_at)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(status)
    .bind(completed_at)
    .fetch_one(pool)
    .await
}

pub async fn delete_survey(pool: &PgPool, survey_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM financial_surveys WHERE id = $1")
        .bind(survey_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ==============================================================================
// Personal Info Operations
// ==============================================================================

pub async fn upsert_personal_info(
    pool: &PgPool,
    survey_id: Uuid,
    req: &UpsertPersonalInfoRequest,
) -> Result<SurveyPersonalInfo, sqlx::Error> {
    sqlx::query_as::<_, SurveyPersonalInfo>(
        r#"
        INSERT INTO survey_personal_info (
            survey_id, full_name, birth_year, marital_status,
            employment_status, dependents, contact_email
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (survey_id)
        DO UPDATE SET
            full_name = COALESCE($2, survey_personal_info.full_name),
            birth_year = COALESCE($3, survey_personal_info.birth_year),
            marital_status = COALESCE($4, survey_personal_info.marital_status),
            employment_status = COALESCE($5, survey_personal_info.employment_status),
            dependents = COALESCE($6, survey_personal_info.dependents),
            contact_email = COALESCE($7, survey_personal_info.contact_email),
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(&req.full_name)
    .bind(req.birth_year)
    .bind(&req.marital_status)
    .bind(&req.employment_status)
    .bind(req.dependents)
    .bind(&req.contact_email)
    .fetch_one(pool)
    .await
}

pub async fn get_personal_info(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<Option<SurveyPersonalInfo>, sqlx::Error> {
    sqlx::query_as::<_, SurveyPersonalInfo>(
        "SELECT * FROM survey_personal_info WHERE survey_id = $1",
    )
    .bind(survey_id)
    .fetch_optional(pool)
    .await
}

// ==============================================================================
// Income Info Operations
// ==============================================================================

pub async fn upsert_income_info(
    pool: &PgPool,
    survey_id: Uuid,
    req: &UpsertIncomeInfoRequest,
) -> Result<SurveyIncomeInfo, sqlx::Error> {
    let gross_annual_income = req.gross_annual_income
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));
    let retirement_contribution_rate = req.retirement_contribution_rate
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));
    let employer_match_rate = req.employer_match_rate
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    sqlx::query_as::<_, SurveyIncomeInfo>(
        r#"
        INSERT INTO survey_income_info (
            survey_id, gross_annual_income, pay_frequency,
            retirement_contribution_rate, employer_match_rate,
            planned_retirement_age, currency, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (survey_id)
        DO UPDATE SET
            gross_annual_income = COALESCE($2, survey_income_info.gross_annual_income),
            pay_frequency = COALESCE($3, survey_income_info.pay_frequency),
            retirement_contribution_rate = COALESCE($4, survey_income_info.retirement_contribution_rate),
            employer_match_rate = COALESCE($5, survey_income_info.employer_match_rate),
            planned_retirement_age = COALESCE($6, survey_income_info.planned_retirement_age),
            currency = COALESCE($7, survey_income_info.currency),
            notes = COALESCE($8, survey_income_info.notes),
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(gross_annual_income)
    .bind(&req.pay_frequency)
    .bind(retirement_contribution_rate)
    .bind(employer_match_rate)
    .bind(req.planned_retirement_age)
    .bind(&req.currency)
    .bind(&req.notes)
    .fetch_one(pool)
    .await
}

pub async fn get_income_info(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<Option<SurveyIncomeInfo>, sqlx::Error> {
    sqlx::query_as::<_, SurveyIncomeInfo>(
        "SELECT * FROM survey_income_info WHERE survey_id = $1",
    )
    .bind(survey_id)
    .fetch_optional(pool)
    .await
}

// ==============================================================================
// Asset Operations
// ==============================================================================

pub async fn create_asset(
    pool: &PgPool,
    survey_id: Uuid,
    req: &CreateAssetRequest,
) -> Result<SurveyAsset, sqlx::Error> {
    let current_value = BigDecimal::from_str(&req.current_value.to_string())
        .unwrap_or_else(|_| BigDecimal::from(0));

    sqlx::query_as::<_, SurveyAsset>(
        r#"
        INSERT INTO survey_assets (
            survey_id, asset_type, description, current_value, currency, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(&req.asset_type)
    .bind(&req.description)
    .bind(current_value)
    .bind(&req.currency)
    .bind(&req.notes)
    .fetch_one(pool)
    .await
}

pub async fn get_assets(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<Vec<SurveyAsset>, sqlx::Error> {
    sqlx::query_as::<_, SurveyAsset>(
        r#"
        SELECT * FROM survey_assets
        WHERE survey_id = $1
        ORDER BY asset_type ASC, created_at ASC
        "#,
    )
    .bind(survey_id)
    .fetch_all(pool)
    .await
}

pub async fn update_asset(
    pool: &PgPool,
    asset_id: Uuid,
    req: &UpdateAssetRequest,
) -> Result<SurveyAsset, sqlx::Error> {
    let current_value = req.current_value
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    sqlx::query_as::<_, SurveyAsset>(
        r#"
        UPDATE survey_assets SET
            asset_type = COALESCE($2, asset_type),
            description = COALESCE($3, description),
            current_value = COALESCE($4, current_value),
            currency = COALESCE($5, currency),
            notes = COALESCE($6, notes)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(asset_id)
    .bind(&req.asset_type)
    .bind(&req.description)
    .bind(current_value)
    .bind(&req.currency)
    .bind(&req.notes)
    .fetch_one(pool)
    .await
}

pub async fn delete_asset(pool: &PgPool, asset_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM survey_assets WHERE id = $1")
        .bind(asset_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ==============================================================================
// Liability Operations
// ==============================================================================

pub async fn create_liability(
    pool: &PgPool,
    survey_id: Uuid,
    req: &CreateLiabilityRequest,
) -> Result<SurveyLiability, sqlx::Error> {
    let balance = BigDecimal::from_str(&req.balance.to_string())
        .unwrap_or_else(|_| BigDecimal::from(0));
    let interest_rate = req.interest_rate
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));
    let monthly_payment = req.monthly_payment
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    sqlx::query_as::<_, SurveyLiability>(
        r#"
        INSERT INTO survey_liabilities (
            survey_id, liability_type, description, balance,
            interest_rate, monthly_payment, payment_frequency, linked_asset_id, currency, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(&req.liability_type)
    .bind(&req.description)
    .bind(balance)
    .bind(interest_rate)
    .bind(monthly_payment)
    .bind(&req.payment_frequency)
    .bind(req.linked_asset_id)
    .bind(&req.currency)
    .bind(&req.notes)
    .fetch_one(pool)
    .await
}

pub async fn get_liabilities(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<Vec<SurveyLiability>, sqlx::Error> {
    sqlx::query_as::<_, SurveyLiability>(
        r#"
        SELECT * FROM survey_liabilities
        WHERE survey_id = $1
        ORDER BY liability_type ASC, created_at ASC
        "#,
    )
    .bind(survey_id)
    .fetch_all(pool)
    .await
}

pub async fn update_liability(
    pool: &PgPool,
    liability_id: Uuid,
    req: &UpdateLiabilityRequest,
) -> Result<SurveyLiability, sqlx::Error> {
    let balance = req.balance
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));
    let interest_rate = req.interest_rate
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));
    let monthly_payment = req.monthly_payment
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    sqlx::query_as::<_, SurveyLiability>(
        r#"
        UPDATE survey_liabilities SET
            liability_type = COALESCE($2, liability_type),
            description = COALESCE($3, description),
            balance = COALESCE($4, balance),
            interest_rate = COALESCE($5, interest_rate),
            monthly_payment = COALESCE($6, monthly_payment),
            payment_frequency = COALESCE($7, payment_frequency),
            linked_asset_id = COALESCE($8, linked_asset_id),
            currency = COALESCE($9, currency),
            notes = COALESCE($10, notes)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(liability_id)
    .bind(&req.liability_type)
    .bind(&req.description)
    .bind(balance)
    .bind(interest_rate)
    .bind(monthly_payment)
    .bind(&req.payment_frequency)
    .bind(req.linked_asset_id)
    .bind(&req.currency)
    .bind(&req.notes)
    .fetch_one(pool)
    .await
}

pub async fn delete_liability(pool: &PgPool, liability_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM survey_liabilities WHERE id = $1")
        .bind(liability_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ==============================================================================
// Goal Operations
// ==============================================================================

pub async fn create_goal(
    pool: &PgPool,
    survey_id: Uuid,
    req: &CreateGoalRequest,
) -> Result<SurveyGoal, sqlx::Error> {
    let target_amount = req.target_amount
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));
    let current_savings = req.current_savings
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    sqlx::query_as::<_, SurveyGoal>(
        r#"
        INSERT INTO survey_goals (
            survey_id, goal_type, description, target_amount,
            current_savings, target_date, priority, currency, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(&req.goal_type)
    .bind(&req.description)
    .bind(target_amount)
    .bind(current_savings)
    .bind(req.target_date)
    .bind(&req.priority)
    .bind(&req.currency)
    .bind(&req.notes)
    .fetch_one(pool)
    .await
}

pub async fn get_goals(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<Vec<SurveyGoal>, sqlx::Error> {
    sqlx::query_as::<_, SurveyGoal>(
        r#"
        SELECT * FROM survey_goals
        WHERE survey_id = $1
        ORDER BY priority ASC, created_at ASC
        "#,
    )
    .bind(survey_id)
    .fetch_all(pool)
    .await
}

pub async fn update_goal(
    pool: &PgPool,
    goal_id: Uuid,
    req: &UpdateGoalRequest,
) -> Result<SurveyGoal, sqlx::Error> {
    let target_amount = req.target_amount
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));
    let current_savings = req.current_savings
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    sqlx::query_as::<_, SurveyGoal>(
        r#"
        UPDATE survey_goals SET
            goal_type = COALESCE($2, goal_type),
            description = COALESCE($3, description),
            target_amount = COALESCE($4, target_amount),
            current_savings = COALESCE($5, current_savings),
            target_date = COALESCE($6, target_date),
            priority = COALESCE($7, priority),
            currency = COALESCE($8, currency),
            notes = COALESCE($9, notes)
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(goal_id)
    .bind(&req.goal_type)
    .bind(&req.description)
    .bind(target_amount)
    .bind(current_savings)
    .bind(req.target_date)
    .bind(&req.priority)
    .bind(&req.currency)
    .bind(&req.notes)
    .fetch_one(pool)
    .await
}

pub async fn delete_goal(pool: &PgPool, goal_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM survey_goals WHERE id = $1")
        .bind(goal_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ==============================================================================
// Risk Profile Operations
// ==============================================================================

pub async fn upsert_risk_profile(
    pool: &PgPool,
    survey_id: Uuid,
    req: &UpsertRiskProfileRequest,
) -> Result<SurveyRiskProfile, sqlx::Error> {
    sqlx::query_as::<_, SurveyRiskProfile>(
        r#"
        INSERT INTO survey_risk_profile (
            survey_id, risk_tolerance, investment_experience, time_horizon_years
        )
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (survey_id)
        DO UPDATE SET
            risk_tolerance = COALESCE($2, survey_risk_profile.risk_tolerance),
            investment_experience = COALESCE($3, survey_risk_profile.investment_experience),
            time_horizon_years = COALESCE($4, survey_risk_profile.time_horizon_years),
            updated_at = NOW()
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(&req.risk_tolerance)
    .bind(&req.investment_experience)
    .bind(req.time_horizon_years)
    .fetch_one(pool)
    .await
}

pub async fn get_risk_profile(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<Option<SurveyRiskProfile>, sqlx::Error> {
    sqlx::query_as::<_, SurveyRiskProfile>(
        "SELECT * FROM survey_risk_profile WHERE survey_id = $1",
    )
    .bind(survey_id)
    .fetch_optional(pool)
    .await
}

// ==============================================================================
// Snapshot Operations
// ==============================================================================

pub async fn create_snapshot(
    pool: &PgPool,
    survey_id: Uuid,
    net_worth: Option<BigDecimal>,
    total_assets: Option<BigDecimal>,
    total_liabilities: Option<BigDecimal>,
    monthly_cash_flow: Option<BigDecimal>,
    projected_retirement_income: Option<BigDecimal>,
    snapshot_data: Option<serde_json::Value>,
) -> Result<FinancialSnapshot, sqlx::Error> {
    sqlx::query_as::<_, FinancialSnapshot>(
        r#"
        INSERT INTO financial_snapshots (
            survey_id, net_worth, total_assets, total_liabilities,
            monthly_cash_flow, projected_retirement_income, snapshot_data
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(net_worth)
    .bind(total_assets)
    .bind(total_liabilities)
    .bind(monthly_cash_flow)
    .bind(projected_retirement_income)
    .bind(snapshot_data)
    .fetch_one(pool)
    .await
}

pub async fn get_latest_snapshot(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<Option<FinancialSnapshot>, sqlx::Error> {
    sqlx::query_as::<_, FinancialSnapshot>(
        r#"
        SELECT * FROM financial_snapshots
        WHERE survey_id = $1
        ORDER BY generated_at DESC
        LIMIT 1
        "#,
    )
    .bind(survey_id)
    .fetch_optional(pool)
    .await
}
