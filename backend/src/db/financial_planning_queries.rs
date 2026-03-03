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
            employment_status, dependents, contact_email,
            has_spouse, spouse_name, spouse_birth_year, spouse_employment_status
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        ON CONFLICT (survey_id)
        DO UPDATE SET
            full_name = COALESCE($2, survey_personal_info.full_name),
            birth_year = COALESCE($3, survey_personal_info.birth_year),
            marital_status = COALESCE($4, survey_personal_info.marital_status),
            employment_status = COALESCE($5, survey_personal_info.employment_status),
            dependents = COALESCE($6, survey_personal_info.dependents),
            contact_email = COALESCE($7, survey_personal_info.contact_email),
            has_spouse = COALESCE($8, survey_personal_info.has_spouse),
            spouse_name = COALESCE($9, survey_personal_info.spouse_name),
            spouse_birth_year = COALESCE($10, survey_personal_info.spouse_birth_year),
            spouse_employment_status = COALESCE($11, survey_personal_info.spouse_employment_status),
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
    .bind(req.has_spouse)
    .bind(&req.spouse_name)
    .bind(req.spouse_birth_year)
    .bind(&req.spouse_employment_status)
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

pub async fn delete_spouse_info(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE survey_personal_info SET
            has_spouse = FALSE,
            spouse_name = NULL,
            spouse_birth_year = NULL,
            spouse_employment_status = NULL,
            updated_at = NOW()
        WHERE survey_id = $1
        "#,
    )
    .bind(survey_id)
    .execute(pool)
    .await?;

    // Also clear spouse income
    sqlx::query(
        r#"
        UPDATE survey_income_info SET
            spouse_gross_annual_income = NULL,
            spouse_pay_frequency = NULL,
            spouse_retirement_contribution_rate = NULL,
            spouse_employer_match_rate = NULL,
            updated_at = NOW()
        WHERE survey_id = $1
        "#,
    )
    .bind(survey_id)
    .execute(pool)
    .await?;

    // Reset all assets to 'mine' ownership
    sqlx::query(
        "UPDATE survey_assets SET ownership = 'mine', joint_split_percentage = 50.00 WHERE survey_id = $1",
    )
    .bind(survey_id)
    .execute(pool)
    .await?;

    // Reset all liabilities to 'mine' ownership
    sqlx::query(
        "UPDATE survey_liabilities SET ownership = 'mine', joint_split_percentage = 50.00 WHERE survey_id = $1",
    )
    .bind(survey_id)
    .execute(pool)
    .await?;

    // Delete household expenses
    sqlx::query("DELETE FROM survey_household_expenses WHERE survey_id = $1")
        .bind(survey_id)
        .execute(pool)
        .await?;

    Ok(())
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
    let desired_annual_retirement_income = req.desired_annual_retirement_income
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));
    let spouse_gross_annual_income = req.spouse_gross_annual_income
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));
    let spouse_retirement_contribution_rate = req.spouse_retirement_contribution_rate
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));
    let spouse_employer_match_rate = req.spouse_employer_match_rate
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    sqlx::query_as::<_, SurveyIncomeInfo>(
        r#"
        INSERT INTO survey_income_info (
            survey_id, gross_annual_income, pay_frequency,
            retirement_contribution_rate, employer_match_rate,
            planned_retirement_age, desired_annual_retirement_income,
            retirement_income_needs_notes, currency, notes,
            spouse_gross_annual_income, spouse_pay_frequency,
            spouse_retirement_contribution_rate, spouse_employer_match_rate
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        ON CONFLICT (survey_id)
        DO UPDATE SET
            gross_annual_income = COALESCE($2, survey_income_info.gross_annual_income),
            pay_frequency = COALESCE($3, survey_income_info.pay_frequency),
            retirement_contribution_rate = COALESCE($4, survey_income_info.retirement_contribution_rate),
            employer_match_rate = COALESCE($5, survey_income_info.employer_match_rate),
            planned_retirement_age = COALESCE($6, survey_income_info.planned_retirement_age),
            desired_annual_retirement_income = COALESCE($7, survey_income_info.desired_annual_retirement_income),
            retirement_income_needs_notes = COALESCE($8, survey_income_info.retirement_income_needs_notes),
            currency = COALESCE($9, survey_income_info.currency),
            notes = COALESCE($10, survey_income_info.notes),
            spouse_gross_annual_income = COALESCE($11, survey_income_info.spouse_gross_annual_income),
            spouse_pay_frequency = COALESCE($12, survey_income_info.spouse_pay_frequency),
            spouse_retirement_contribution_rate = COALESCE($13, survey_income_info.spouse_retirement_contribution_rate),
            spouse_employer_match_rate = COALESCE($14, survey_income_info.spouse_employer_match_rate),
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
    .bind(desired_annual_retirement_income)
    .bind(&req.retirement_income_needs_notes)
    .bind(&req.currency)
    .bind(&req.notes)
    .bind(spouse_gross_annual_income)
    .bind(&req.spouse_pay_frequency)
    .bind(spouse_retirement_contribution_rate)
    .bind(spouse_employer_match_rate)
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
// Additional Income Operations
// ==============================================================================

pub async fn create_additional_income(
    pool: &PgPool,
    survey_id: Uuid,
    req: &CreateAdditionalIncomeRequest,
) -> Result<SurveyAdditionalIncome, sqlx::Error> {
    let monthly_amount = BigDecimal::from_str(&req.monthly_amount.to_string())
        .unwrap_or_else(|_| BigDecimal::from(0));
    let owner = req.owner.as_deref().unwrap_or("mine");

    sqlx::query_as::<_, SurveyAdditionalIncome>(
        r#"
        INSERT INTO survey_additional_income (
            survey_id, income_type, description, monthly_amount, is_recurring, currency, notes, owner
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(&req.income_type)
    .bind(&req.description)
    .bind(monthly_amount)
    .bind(req.is_recurring)
    .bind(&req.currency)
    .bind(&req.notes)
    .bind(owner)
    .fetch_one(pool)
    .await
}

pub async fn get_additional_income(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<Vec<SurveyAdditionalIncome>, sqlx::Error> {
    sqlx::query_as::<_, SurveyAdditionalIncome>(
        r#"
        SELECT * FROM survey_additional_income
        WHERE survey_id = $1
        ORDER BY income_type ASC, created_at ASC
        "#,
    )
    .bind(survey_id)
    .fetch_all(pool)
    .await
}

pub async fn update_additional_income(
    pool: &PgPool,
    income_id: Uuid,
    req: &UpdateAdditionalIncomeRequest,
) -> Result<SurveyAdditionalIncome, sqlx::Error> {
    let monthly_amount = req.monthly_amount
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    sqlx::query_as::<_, SurveyAdditionalIncome>(
        r#"
        UPDATE survey_additional_income SET
            income_type = COALESCE($2, income_type),
            description = COALESCE($3, description),
            monthly_amount = COALESCE($4, monthly_amount),
            is_recurring = COALESCE($5, is_recurring),
            currency = COALESCE($6, currency),
            notes = COALESCE($7, notes),
            owner = COALESCE($8, owner),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(income_id)
    .bind(&req.income_type)
    .bind(&req.description)
    .bind(monthly_amount)
    .bind(req.is_recurring)
    .bind(&req.currency)
    .bind(&req.notes)
    .bind(&req.owner)
    .fetch_one(pool)
    .await
}

pub async fn delete_additional_income(pool: &PgPool, income_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM survey_additional_income WHERE id = $1")
        .bind(income_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ==============================================================================
// Expense Operations
// ==============================================================================

pub async fn create_expense(
    pool: &PgPool,
    survey_id: Uuid,
    req: &CreateExpenseRequest,
) -> Result<SurveyExpense, sqlx::Error> {
    let monthly_amount = BigDecimal::from_str(&req.monthly_amount.to_string())
        .unwrap_or_else(|_| BigDecimal::from(0));

    sqlx::query_as::<_, SurveyExpense>(
        r#"
        INSERT INTO survey_expenses (
            survey_id, expense_category, description, monthly_amount, is_recurring, currency, notes
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(&req.expense_category)
    .bind(&req.description)
    .bind(monthly_amount)
    .bind(req.is_recurring)
    .bind(&req.currency)
    .bind(&req.notes)
    .fetch_one(pool)
    .await
}

pub async fn get_expenses(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<Vec<SurveyExpense>, sqlx::Error> {
    sqlx::query_as::<_, SurveyExpense>(
        r#"
        SELECT * FROM survey_expenses
        WHERE survey_id = $1
        ORDER BY expense_category ASC, created_at ASC
        "#,
    )
    .bind(survey_id)
    .fetch_all(pool)
    .await
}

pub async fn update_expense(
    pool: &PgPool,
    expense_id: Uuid,
    req: &UpdateExpenseRequest,
) -> Result<SurveyExpense, sqlx::Error> {
    let monthly_amount = req.monthly_amount
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    sqlx::query_as::<_, SurveyExpense>(
        r#"
        UPDATE survey_expenses SET
            expense_category = COALESCE($2, expense_category),
            description = COALESCE($3, description),
            monthly_amount = COALESCE($4, monthly_amount),
            is_recurring = COALESCE($5, is_recurring),
            currency = COALESCE($6, currency),
            notes = COALESCE($7, notes),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(expense_id)
    .bind(&req.expense_category)
    .bind(&req.description)
    .bind(monthly_amount)
    .bind(req.is_recurring)
    .bind(&req.currency)
    .bind(&req.notes)
    .fetch_one(pool)
    .await
}

pub async fn delete_expense(pool: &PgPool, expense_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM survey_expenses WHERE id = $1")
        .bind(expense_id)
        .execute(pool)
        .await?;
    Ok(())
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
    let ownership = req.ownership.as_deref().unwrap_or("mine");
    let joint_split_percentage = req.joint_split_percentage
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(50)));

    sqlx::query_as::<_, SurveyAsset>(
        r#"
        INSERT INTO survey_assets (
            survey_id, asset_type, description, current_value, currency, notes,
            ownership, joint_split_percentage, linked_account_id
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(&req.asset_type)
    .bind(&req.description)
    .bind(current_value)
    .bind(&req.currency)
    .bind(&req.notes)
    .bind(ownership)
    .bind(joint_split_percentage)
    .bind(req.linked_account_id)
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
    let joint_split_percentage = req.joint_split_percentage
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(50)));

    sqlx::query_as::<_, SurveyAsset>(
        r#"
        UPDATE survey_assets SET
            asset_type = COALESCE($2, asset_type),
            description = COALESCE($3, description),
            current_value = COALESCE($4, current_value),
            currency = COALESCE($5, currency),
            notes = COALESCE($6, notes),
            ownership = COALESCE($7, ownership),
            joint_split_percentage = COALESCE($8, joint_split_percentage),
            linked_account_id = COALESCE($9, linked_account_id)
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
    .bind(&req.ownership)
    .bind(joint_split_percentage)
    .bind(req.linked_account_id)
    .fetch_one(pool)
    .await
}

pub async fn unlink_asset_account(pool: &PgPool, asset_id: Uuid) -> Result<SurveyAsset, sqlx::Error> {
    sqlx::query_as::<_, SurveyAsset>(
        "UPDATE survey_assets SET linked_account_id = NULL, updated_at = NOW() WHERE id = $1 RETURNING *"
    )
    .bind(asset_id)
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
    let ownership = req.ownership.as_deref().unwrap_or("mine");
    let joint_split_percentage = req.joint_split_percentage
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(50)));

    sqlx::query_as::<_, SurveyLiability>(
        r#"
        INSERT INTO survey_liabilities (
            survey_id, liability_type, description, balance,
            interest_rate, monthly_payment, payment_frequency, linked_asset_id, currency, notes,
            ownership, joint_split_percentage
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
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
    .bind(ownership)
    .bind(joint_split_percentage)
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
    let joint_split_percentage = req.joint_split_percentage
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(50)));

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
            notes = COALESCE($10, notes),
            ownership = COALESCE($11, ownership),
            joint_split_percentage = COALESCE($12, joint_split_percentage)
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
    .bind(&req.ownership)
    .bind(joint_split_percentage)
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
    let owner = req.owner.as_deref().unwrap_or("mine");

    sqlx::query_as::<_, SurveyGoal>(
        r#"
        INSERT INTO survey_goals (
            survey_id, goal_type, description, target_amount,
            current_savings, target_date, priority, currency, notes, owner
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
    .bind(owner)
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
            notes = COALESCE($9, notes),
            owner = COALESCE($10, owner)
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
    .bind(&req.owner)
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
// Household Expense Operations
// ==============================================================================

pub async fn create_household_expense(
    pool: &PgPool,
    survey_id: Uuid,
    req: &CreateHouseholdExpenseRequest,
) -> Result<SurveyHouseholdExpense, sqlx::Error> {
    let monthly_amount = BigDecimal::from_str(&req.monthly_amount.to_string())
        .unwrap_or_else(|_| BigDecimal::from(0));
    let currency = req.currency.as_deref().unwrap_or("USD");

    sqlx::query_as::<_, SurveyHouseholdExpense>(
        r#"
        INSERT INTO survey_household_expenses (
            survey_id, expense_category, expense_type, monthly_amount, description, currency
        )
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING *
        "#,
    )
    .bind(survey_id)
    .bind(&req.expense_category)
    .bind(&req.expense_type)
    .bind(monthly_amount)
    .bind(&req.description)
    .bind(currency)
    .fetch_one(pool)
    .await
}

pub async fn get_household_expenses(
    pool: &PgPool,
    survey_id: Uuid,
) -> Result<Vec<SurveyHouseholdExpense>, sqlx::Error> {
    sqlx::query_as::<_, SurveyHouseholdExpense>(
        r#"
        SELECT * FROM survey_household_expenses
        WHERE survey_id = $1
        ORDER BY expense_type ASC, expense_category ASC, created_at ASC
        "#,
    )
    .bind(survey_id)
    .fetch_all(pool)
    .await
}

pub async fn update_household_expense(
    pool: &PgPool,
    expense_id: Uuid,
    req: &UpdateHouseholdExpenseRequest,
) -> Result<SurveyHouseholdExpense, sqlx::Error> {
    let monthly_amount = req.monthly_amount
        .map(|v| BigDecimal::from_str(&v.to_string()).unwrap_or_else(|_| BigDecimal::from(0)));

    sqlx::query_as::<_, SurveyHouseholdExpense>(
        r#"
        UPDATE survey_household_expenses SET
            expense_category = COALESCE($2, expense_category),
            expense_type = COALESCE($3, expense_type),
            monthly_amount = COALESCE($4, monthly_amount),
            description = COALESCE($5, description),
            currency = COALESCE($6, currency),
            updated_at = NOW()
        WHERE id = $1
        RETURNING *
        "#,
    )
    .bind(expense_id)
    .bind(&req.expense_category)
    .bind(&req.expense_type)
    .bind(monthly_amount)
    .bind(&req.description)
    .bind(&req.currency)
    .fetch_one(pool)
    .await
}

pub async fn delete_household_expense(pool: &PgPool, expense_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM survey_household_expenses WHERE id = $1")
        .bind(expense_id)
        .execute(pool)
        .await?;
    Ok(())
}

// ==============================================================================
// Linked Account Operations
// ==============================================================================

/// Returns all portfolio accounts for a user with their latest market value.
/// Used for the "Link Account" picker in the asset step.
pub async fn get_linkable_accounts(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<LinkableAccount>, sqlx::Error> {
    sqlx::query_as::<_, LinkableAccount>(
        r#"
        SELECT
            a.id,
            a.account_nickname,
            a.account_number,
            p.name AS portfolio_name,
            avh.total_value AS latest_value,
            avh.snapshot_date AS latest_snapshot_date
        FROM accounts a
        JOIN portfolios p ON a.portfolio_id = p.id
        LEFT JOIN LATERAL (
            SELECT total_value, snapshot_date
            FROM account_value_history
            WHERE account_id = a.id
            ORDER BY snapshot_date DESC
            LIMIT 1
        ) avh ON TRUE
        WHERE p.user_id = $1
        ORDER BY p.name, a.account_nickname
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
}

/// Fetches the latest total market value for the linked account and updates
/// the survey asset's current_value.
pub async fn refresh_asset_value(
    pool: &PgPool,
    asset_id: Uuid,
) -> Result<SurveyAsset, sqlx::Error> {
    // Get the linked account ID for this asset
    let linked_account_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT linked_account_id FROM survey_assets WHERE id = $1"
    )
    .bind(asset_id)
    .fetch_one(pool)
    .await?;

    if let Some(account_id) = linked_account_id {
        // Get latest value from account_value_history view
        let latest_value: Option<BigDecimal> = sqlx::query_scalar(
            "SELECT total_value FROM account_value_history WHERE account_id = $1 ORDER BY snapshot_date DESC LIMIT 1"
        )
        .bind(account_id)
        .fetch_optional(pool)
        .await?;

        if let Some(value) = latest_value {
            return sqlx::query_as::<_, SurveyAsset>(
                "UPDATE survey_assets SET current_value = $2, updated_at = NOW() WHERE id = $1 RETURNING *"
            )
            .bind(asset_id)
            .bind(value)
            .fetch_one(pool)
            .await;
        }
    }

    // No linked account or no history — return unchanged
    sqlx::query_as::<_, SurveyAsset>("SELECT * FROM survey_assets WHERE id = $1")
        .bind(asset_id)
        .fetch_one(pool)
        .await
}

/// Batch-fetches account nicknames for a set of account IDs.
/// Used to populate linked_account_nickname in asset responses.
pub async fn get_account_nicknames(
    pool: &PgPool,
    account_ids: &[Uuid],
) -> Result<std::collections::HashMap<Uuid, String>, sqlx::Error> {
    if account_ids.is_empty() {
        return Ok(std::collections::HashMap::new());
    }

    let rows: Vec<(Uuid, String)> = sqlx::query_as(
        "SELECT id, account_nickname FROM accounts WHERE id = ANY($1)"
    )
    .bind(account_ids)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().collect())
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
