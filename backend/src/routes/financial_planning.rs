use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use sqlx::PgPool;
use tracing::info;
use uuid::Uuid;

use crate::db::{alert_queries, financial_planning_queries};
use crate::models::financial_planning::*;
use crate::services::financial_snapshot_service;
use crate::state::AppState;

// ==============================================================================
// Router
// ==============================================================================

pub fn router() -> Router<AppState> {
    Router::new()
        // Linkable accounts (for asset picker)
        .route("/linkable-accounts", get(get_linkable_accounts))
        // Survey management
        .route("/surveys", post(create_survey))
        .route("/surveys", get(list_surveys))
        .route("/surveys/:id", get(get_survey))
        .route("/surveys/:id", delete(delete_survey))
        .route("/surveys/:id/complete", post(complete_survey))
        // Personal info
        .route("/surveys/:id/personal-info", put(upsert_personal_info))
        .route("/surveys/:id/personal-info", get(get_personal_info))
        // Spouse info
        .route("/surveys/:id/spouse-info", delete(delete_spouse_info))
        // Income info
        .route("/surveys/:id/income-info", put(upsert_income_info))
        .route("/surveys/:id/income-info", get(get_income_info))
        // Additional income
        .route("/surveys/:id/additional-income", post(create_additional_income))
        .route("/surveys/:id/additional-income", get(get_additional_income_list))
        .route("/surveys/:survey_id/additional-income/:income_id", put(update_additional_income))
        .route("/surveys/:survey_id/additional-income/:income_id", delete(delete_additional_income))
        // Expenses
        .route("/surveys/:id/expenses", post(create_expense))
        .route("/surveys/:id/expenses", get(get_expenses_list))
        .route("/surveys/:survey_id/expenses/:expense_id", put(update_expense))
        .route("/surveys/:survey_id/expenses/:expense_id", delete(delete_expense))
        // Household expenses
        .route("/surveys/:id/household-expenses", post(create_household_expense))
        .route("/surveys/:id/household-expenses", get(get_household_expenses_list))
        .route("/surveys/:survey_id/household-expenses/:expense_id", put(update_household_expense))
        .route("/surveys/:survey_id/household-expenses/:expense_id", delete(delete_household_expense))
        // Assets
        .route("/surveys/:id/assets", post(create_asset))
        .route("/surveys/:id/assets", get(get_assets))
        .route("/surveys/:survey_id/assets/:asset_id", put(update_asset))
        .route("/surveys/:survey_id/assets/:asset_id", delete(delete_asset))
        .route("/surveys/:survey_id/assets/:asset_id/refresh", post(refresh_asset))
        .route("/surveys/:survey_id/assets/:asset_id/unlink", post(unlink_asset))
        // Liabilities
        .route("/surveys/:id/liabilities", post(create_liability))
        .route("/surveys/:id/liabilities", get(get_liabilities))
        .route("/surveys/:survey_id/liabilities/:liability_id", put(update_liability))
        .route("/surveys/:survey_id/liabilities/:liability_id", delete(delete_liability))
        // Goals
        .route("/surveys/:id/goals", post(create_goal))
        .route("/surveys/:id/goals", get(get_goals))
        .route("/surveys/:survey_id/goals/:goal_id", put(update_goal))
        .route("/surveys/:survey_id/goals/:goal_id", delete(delete_goal))
        // Risk profile
        .route("/surveys/:id/risk-profile", put(upsert_risk_profile))
        .route("/surveys/:id/risk-profile", get(get_risk_profile))
        // Snapshot
        .route("/surveys/:id/snapshot", get(get_snapshot))
        .route("/surveys/:id/snapshot/regenerate", post(regenerate_snapshot))
        .route("/surveys/:id/snapshot/household", get(get_household_snapshot))
}

// ==============================================================================
// Helper: get default user ID
// ==============================================================================

async fn get_default_user_id(pool: &PgPool) -> Result<Uuid, (StatusCode, String)> {
    let default_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000001")
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    alert_queries::get_user(pool, default_uuid)
        .await
        .map_err(|e| (StatusCode::UNAUTHORIZED, e.to_string()))?;

    Ok(default_uuid)
}

// ==============================================================================
// Survey Handlers
// ==============================================================================

async fn create_survey(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    info!("Creating new financial survey for user {}", user_id);

    let survey = financial_planning_queries::create_survey(pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(SurveyResponse::from(survey))))
}

async fn list_surveys(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let surveys = financial_planning_queries::get_surveys_for_user(pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<SurveyResponse> = surveys.into_iter().map(SurveyResponse::from).collect();
    Ok(Json(responses))
}

async fn get_survey(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let survey = financial_planning_queries::get_survey(pool, id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Survey not found: {}", e)))?;

    let personal_info = financial_planning_queries::get_personal_info(pool, id)
        .await
        .unwrap_or(None)
        .map(PersonalInfoResponse::from);

    let income_info = financial_planning_queries::get_income_info(pool, id)
        .await
        .unwrap_or(None)
        .map(IncomeInfoResponse::from);

    let additional_income: Vec<AdditionalIncomeResponse> = financial_planning_queries::get_additional_income(pool, id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(AdditionalIncomeResponse::from)
        .collect();

    let expenses: Vec<ExpenseResponse> = financial_planning_queries::get_expenses(pool, id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(ExpenseResponse::from)
        .collect();

    let household_expenses: Vec<HouseholdExpenseResponse> = financial_planning_queries::get_household_expenses(pool, id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(HouseholdExpenseResponse::from)
        .collect();

    let raw_assets = financial_planning_queries::get_assets(pool, id)
        .await
        .unwrap_or_default();

    // Batch-fetch account nicknames for any linked assets
    let linked_ids: Vec<uuid::Uuid> = raw_assets.iter()
        .filter_map(|a| a.linked_account_id)
        .collect();
    let nickname_map = financial_planning_queries::get_account_nicknames(pool, &linked_ids)
        .await
        .unwrap_or_default();

    let assets: Vec<AssetResponse> = raw_assets.into_iter().map(|a| {
        let mut resp = AssetResponse::from(a);
        if let Some(aid) = resp.linked_account_id {
            resp.linked_account_nickname = nickname_map.get(&aid).cloned();
        }
        resp
    }).collect();

    let liabilities: Vec<LiabilityResponse> = financial_planning_queries::get_liabilities(pool, id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(LiabilityResponse::from)
        .collect();

    let goals: Vec<GoalResponse> = financial_planning_queries::get_goals(pool, id)
        .await
        .unwrap_or_default()
        .into_iter()
        .map(GoalResponse::from)
        .collect();

    let risk_profile = financial_planning_queries::get_risk_profile(pool, id)
        .await
        .unwrap_or(None)
        .map(RiskProfileResponse::from);

    let latest_snapshot = financial_planning_queries::get_latest_snapshot(pool, id)
        .await
        .unwrap_or(None)
        .map(SnapshotResponse::from);

    let response = SurveyDetailResponse {
        id: survey.id,
        version: survey.version,
        status: survey.status,
        personal_info,
        income_info,
        additional_income,
        expenses,
        household_expenses,
        assets,
        liabilities,
        goals,
        risk_profile,
        latest_snapshot,
        created_at: survey.created_at,
        updated_at: survey.updated_at,
        completed_at: survey.completed_at,
    };

    Ok(Json(response))
}

async fn delete_survey(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    financial_planning_queries::delete_survey(pool, id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn complete_survey(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    info!("Marking survey {} as completed", id);

    let survey = financial_planning_queries::update_survey_status(pool, id, "completed")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Auto-generate snapshot on completion
    let _ = financial_snapshot_service::generate_snapshot(pool, id).await;

    Ok(Json(SurveyResponse::from(survey)))
}

// ==============================================================================
// Personal Info Handlers
// ==============================================================================

async fn upsert_personal_info(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
    Json(req): Json<UpsertPersonalInfoRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let info = financial_planning_queries::upsert_personal_info(pool, survey_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(PersonalInfoResponse::from(info)))
}

async fn get_personal_info(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let info = financial_planning_queries::get_personal_info(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match info {
        Some(i) => Ok(Json(serde_json::to_value(PersonalInfoResponse::from(i)).unwrap())),
        None => Ok(Json(serde_json::Value::Null)),
    }
}

// ==============================================================================
// Spouse Info Handlers
// ==============================================================================

async fn delete_spouse_info(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    info!("Removing spouse info for survey {}", survey_id);

    financial_planning_queries::delete_spouse_info(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==============================================================================
// Income Info Handlers
// ==============================================================================

async fn upsert_income_info(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
    Json(req): Json<UpsertIncomeInfoRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let info = financial_planning_queries::upsert_income_info(pool, survey_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(IncomeInfoResponse::from(info)))
}

async fn get_income_info(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let info = financial_planning_queries::get_income_info(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match info {
        Some(i) => Ok(Json(serde_json::to_value(IncomeInfoResponse::from(i)).unwrap())),
        None => Ok(Json(serde_json::Value::Null)),
    }
}

// ==============================================================================
// Additional Income Handlers
// ==============================================================================

async fn create_additional_income(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
    Json(req): Json<CreateAdditionalIncomeRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let income = financial_planning_queries::create_additional_income(pool, survey_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(AdditionalIncomeResponse::from(income))))
}

async fn get_additional_income_list(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let income_list = financial_planning_queries::get_additional_income(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<AdditionalIncomeResponse> = income_list.into_iter().map(AdditionalIncomeResponse::from).collect();
    Ok(Json(responses))
}

async fn update_additional_income(
    State(state): State<AppState>,
    Path((_survey_id, income_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateAdditionalIncomeRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let income = financial_planning_queries::update_additional_income(pool, income_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AdditionalIncomeResponse::from(income)))
}

async fn delete_additional_income(
    State(state): State<AppState>,
    Path((_survey_id, income_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    financial_planning_queries::delete_additional_income(pool, income_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==============================================================================
// Expense Handlers
// ==============================================================================

async fn create_expense(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
    Json(req): Json<CreateExpenseRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let expense = financial_planning_queries::create_expense(pool, survey_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(ExpenseResponse::from(expense))))
}

async fn get_expenses_list(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let expenses = financial_planning_queries::get_expenses(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<ExpenseResponse> = expenses.into_iter().map(ExpenseResponse::from).collect();
    Ok(Json(responses))
}

async fn update_expense(
    State(state): State<AppState>,
    Path((_survey_id, expense_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateExpenseRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let expense = financial_planning_queries::update_expense(pool, expense_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ExpenseResponse::from(expense)))
}

async fn delete_expense(
    State(state): State<AppState>,
    Path((_survey_id, expense_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    financial_planning_queries::delete_expense(pool, expense_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==============================================================================
// Household Expense Handlers
// ==============================================================================

async fn create_household_expense(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
    Json(req): Json<CreateHouseholdExpenseRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let expense = financial_planning_queries::create_household_expense(pool, survey_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(HouseholdExpenseResponse::from(expense))))
}

async fn get_household_expenses_list(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let expenses = financial_planning_queries::get_household_expenses(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<HouseholdExpenseResponse> = expenses.into_iter().map(HouseholdExpenseResponse::from).collect();
    Ok(Json(responses))
}

async fn update_household_expense(
    State(state): State<AppState>,
    Path((_survey_id, expense_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateHouseholdExpenseRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let expense = financial_planning_queries::update_household_expense(pool, expense_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(HouseholdExpenseResponse::from(expense)))
}

async fn delete_household_expense(
    State(state): State<AppState>,
    Path((_survey_id, expense_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    financial_planning_queries::delete_household_expense(pool, expense_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==============================================================================
// Asset Handlers
// ==============================================================================

async fn create_asset(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
    Json(req): Json<CreateAssetRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let asset = financial_planning_queries::create_asset(pool, survey_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(AssetResponse::from(asset))))
}

async fn get_assets(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let assets = financial_planning_queries::get_assets(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<AssetResponse> = assets.into_iter().map(AssetResponse::from).collect();
    Ok(Json(responses))
}

async fn update_asset(
    State(state): State<AppState>,
    Path((_survey_id, asset_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateAssetRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let asset = financial_planning_queries::update_asset(pool, asset_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AssetResponse::from(asset)))
}

async fn delete_asset(
    State(state): State<AppState>,
    Path((_survey_id, asset_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    financial_planning_queries::delete_asset(pool, asset_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn refresh_asset(
    State(state): State<AppState>,
    Path((_survey_id, asset_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let asset = financial_planning_queries::refresh_asset_value(pool, asset_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Fetch nickname for the response if linked
    let mut resp = AssetResponse::from(asset);
    if let Some(account_id) = resp.linked_account_id {
        let nickname_map = financial_planning_queries::get_account_nicknames(pool, &[account_id])
            .await
            .unwrap_or_default();
        resp.linked_account_nickname = nickname_map.get(&account_id).cloned();
    }

    Ok(Json(resp))
}

async fn unlink_asset(
    State(state): State<AppState>,
    Path((_survey_id, asset_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let asset = financial_planning_queries::unlink_asset_account(pool, asset_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(AssetResponse::from(asset)))
}

async fn get_linkable_accounts(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;
    let user_id = get_default_user_id(pool).await?;

    let accounts = financial_planning_queries::get_linkable_accounts(pool, user_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<LinkableAccountResponse> = accounts
        .into_iter()
        .map(LinkableAccountResponse::from)
        .collect();

    Ok(Json(responses))
}

// ==============================================================================
// Liability Handlers
// ==============================================================================

async fn create_liability(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
    Json(req): Json<CreateLiabilityRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let liability = financial_planning_queries::create_liability(pool, survey_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(LiabilityResponse::from(liability))))
}

async fn get_liabilities(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let liabilities = financial_planning_queries::get_liabilities(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<LiabilityResponse> = liabilities.into_iter().map(LiabilityResponse::from).collect();
    Ok(Json(responses))
}

async fn update_liability(
    State(state): State<AppState>,
    Path((_survey_id, liability_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateLiabilityRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let liability = financial_planning_queries::update_liability(pool, liability_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(LiabilityResponse::from(liability)))
}

async fn delete_liability(
    State(state): State<AppState>,
    Path((_survey_id, liability_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    financial_planning_queries::delete_liability(pool, liability_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==============================================================================
// Goal Handlers
// ==============================================================================

async fn create_goal(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
    Json(req): Json<CreateGoalRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let goal = financial_planning_queries::create_goal(pool, survey_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok((StatusCode::CREATED, Json(GoalResponse::from(goal))))
}

async fn get_goals(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let goals = financial_planning_queries::get_goals(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let responses: Vec<GoalResponse> = goals.into_iter().map(GoalResponse::from).collect();
    Ok(Json(responses))
}

async fn update_goal(
    State(state): State<AppState>,
    Path((_survey_id, goal_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateGoalRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let goal = financial_planning_queries::update_goal(pool, goal_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(GoalResponse::from(goal)))
}

async fn delete_goal(
    State(state): State<AppState>,
    Path((_survey_id, goal_id)): Path<(Uuid, Uuid)>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    financial_planning_queries::delete_goal(pool, goal_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}

// ==============================================================================
// Risk Profile Handlers
// ==============================================================================

async fn upsert_risk_profile(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
    Json(req): Json<UpsertRiskProfileRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let profile = financial_planning_queries::upsert_risk_profile(pool, survey_id, &req)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(RiskProfileResponse::from(profile)))
}

async fn get_risk_profile(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let profile = financial_planning_queries::get_risk_profile(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match profile {
        Some(p) => Ok(Json(serde_json::to_value(RiskProfileResponse::from(p)).unwrap())),
        None => Ok(Json(serde_json::Value::Null)),
    }
}

// ==============================================================================
// Snapshot Handlers
// ==============================================================================

async fn get_snapshot(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let snapshot = financial_planning_queries::get_latest_snapshot(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match snapshot {
        Some(s) => Ok(Json(serde_json::to_value(SnapshotResponse::from(s)).unwrap())),
        None => {
            info!("No snapshot found for survey {}, generating new one", survey_id);
            let new_snapshot = financial_snapshot_service::generate_snapshot(pool, survey_id)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
            Ok(Json(serde_json::to_value(SnapshotResponse::from(new_snapshot)).unwrap()))
        }
    }
}

async fn regenerate_snapshot(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    info!("Regenerating snapshot for survey {}", survey_id);

    let snapshot = financial_snapshot_service::generate_snapshot(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok((StatusCode::CREATED, Json(SnapshotResponse::from(snapshot))))
}

async fn get_household_snapshot(
    State(state): State<AppState>,
    Path(survey_id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let pool = &state.pool;

    let household = financial_snapshot_service::generate_household_snapshot(pool, survey_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(household))
}
