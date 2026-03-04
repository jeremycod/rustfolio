use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// ==============================================================================
// Financial Survey Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FinancialSurvey {
    pub id: Uuid,
    pub user_id: Uuid,
    pub version: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyResponse {
    pub id: Uuid,
    pub version: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<FinancialSurvey> for SurveyResponse {
    fn from(s: FinancialSurvey) -> Self {
        Self {
            id: s.id,
            version: s.version,
            status: s.status,
            created_at: s.created_at,
            updated_at: s.updated_at,
            completed_at: s.completed_at,
        }
    }
}

// ==============================================================================
// Personal Info Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SurveyPersonalInfo {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub full_name: Option<String>,
    pub birth_year: Option<i32>,
    pub marital_status: Option<String>,
    pub employment_status: Option<String>,
    pub dependents: Option<i32>,
    pub contact_email: Option<String>,
    // Spouse fields
    pub has_spouse: bool,
    pub spouse_name: Option<String>,
    pub spouse_birth_year: Option<i32>,
    pub spouse_employment_status: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertPersonalInfoRequest {
    pub full_name: Option<String>,
    pub birth_year: Option<i32>,
    pub marital_status: Option<String>,
    pub employment_status: Option<String>,
    pub dependents: Option<i32>,
    pub contact_email: Option<String>,
    // Spouse fields
    pub has_spouse: Option<bool>,
    pub spouse_name: Option<String>,
    pub spouse_birth_year: Option<i32>,
    pub spouse_employment_status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalInfoResponse {
    pub id: Uuid,
    pub full_name: Option<String>,
    pub birth_year: Option<i32>,
    pub marital_status: Option<String>,
    pub employment_status: Option<String>,
    pub dependents: Option<i32>,
    pub contact_email: Option<String>,
    // Spouse fields
    pub has_spouse: bool,
    pub spouse_name: Option<String>,
    pub spouse_birth_year: Option<i32>,
    pub spouse_employment_status: Option<String>,
}

impl From<SurveyPersonalInfo> for PersonalInfoResponse {
    fn from(p: SurveyPersonalInfo) -> Self {
        Self {
            id: p.id,
            full_name: p.full_name,
            birth_year: p.birth_year,
            marital_status: p.marital_status,
            employment_status: p.employment_status,
            dependents: p.dependents,
            contact_email: p.contact_email,
            has_spouse: p.has_spouse,
            spouse_name: p.spouse_name,
            spouse_birth_year: p.spouse_birth_year,
            spouse_employment_status: p.spouse_employment_status,
        }
    }
}

// ==============================================================================
// Income Info Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SurveyIncomeInfo {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub gross_annual_income: Option<BigDecimal>,
    pub pay_frequency: Option<String>,
    pub retirement_contribution_rate: Option<BigDecimal>,
    pub employer_match_rate: Option<BigDecimal>,
    pub planned_retirement_age: Option<i32>,
    pub desired_annual_retirement_income: Option<BigDecimal>,
    pub retirement_income_needs_notes: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Spouse income fields
    pub spouse_gross_annual_income: Option<BigDecimal>,
    pub spouse_pay_frequency: Option<String>,
    pub spouse_retirement_contribution_rate: Option<BigDecimal>,
    pub spouse_employer_match_rate: Option<BigDecimal>,
    // Tax rates (effective, not marginal)
    pub effective_tax_rate: Option<BigDecimal>,
    pub spouse_effective_tax_rate: Option<BigDecimal>,
    // Separate rate for dividends/interest (taxed preferentially in Canada etc.)
    pub investment_income_tax_rate: Option<BigDecimal>,
    pub spouse_investment_income_tax_rate: Option<BigDecimal>,
    // Payroll deductions beyond income tax (CPP, EI, benefit premiums etc.) — monthly amounts
    pub monthly_deductions: Option<BigDecimal>,
    pub spouse_monthly_deductions: Option<BigDecimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertIncomeInfoRequest {
    pub gross_annual_income: Option<f64>,
    pub pay_frequency: Option<String>,
    pub retirement_contribution_rate: Option<f64>,
    pub employer_match_rate: Option<f64>,
    pub planned_retirement_age: Option<i32>,
    pub desired_annual_retirement_income: Option<f64>,
    pub retirement_income_needs_notes: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Spouse income fields
    pub spouse_gross_annual_income: Option<f64>,
    pub spouse_pay_frequency: Option<String>,
    pub spouse_retirement_contribution_rate: Option<f64>,
    pub spouse_employer_match_rate: Option<f64>,
    // Tax rates
    pub effective_tax_rate: Option<f64>,
    pub spouse_effective_tax_rate: Option<f64>,
    pub investment_income_tax_rate: Option<f64>,
    pub spouse_investment_income_tax_rate: Option<f64>,
    pub monthly_deductions: Option<f64>,
    pub spouse_monthly_deductions: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeInfoResponse {
    pub id: Uuid,
    pub gross_annual_income: Option<f64>,
    pub pay_frequency: Option<String>,
    pub retirement_contribution_rate: Option<f64>,
    pub employer_match_rate: Option<f64>,
    pub planned_retirement_age: Option<i32>,
    pub desired_annual_retirement_income: Option<f64>,
    pub retirement_income_needs_notes: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Spouse income fields
    pub spouse_gross_annual_income: Option<f64>,
    pub spouse_pay_frequency: Option<String>,
    pub spouse_retirement_contribution_rate: Option<f64>,
    pub spouse_employer_match_rate: Option<f64>,
    // Tax rates
    pub effective_tax_rate: Option<f64>,
    pub spouse_effective_tax_rate: Option<f64>,
    pub investment_income_tax_rate: Option<f64>,
    pub spouse_investment_income_tax_rate: Option<f64>,
    pub monthly_deductions: Option<f64>,
    pub spouse_monthly_deductions: Option<f64>,
}

impl From<SurveyIncomeInfo> for IncomeInfoResponse {
    fn from(i: SurveyIncomeInfo) -> Self {
        Self {
            id: i.id,
            gross_annual_income: i.gross_annual_income.as_ref().and_then(|v| v.to_string().parse().ok()),
            pay_frequency: i.pay_frequency,
            retirement_contribution_rate: i.retirement_contribution_rate.as_ref().and_then(|v| v.to_string().parse().ok()),
            employer_match_rate: i.employer_match_rate.as_ref().and_then(|v| v.to_string().parse().ok()),
            planned_retirement_age: i.planned_retirement_age,
            desired_annual_retirement_income: i.desired_annual_retirement_income.as_ref().and_then(|v| v.to_string().parse().ok()),
            retirement_income_needs_notes: i.retirement_income_needs_notes,
            currency: i.currency,
            notes: i.notes,
            spouse_gross_annual_income: i.spouse_gross_annual_income.as_ref().and_then(|v| v.to_string().parse().ok()),
            spouse_pay_frequency: i.spouse_pay_frequency,
            spouse_retirement_contribution_rate: i.spouse_retirement_contribution_rate.as_ref().and_then(|v| v.to_string().parse().ok()),
            spouse_employer_match_rate: i.spouse_employer_match_rate.as_ref().and_then(|v| v.to_string().parse().ok()),
            effective_tax_rate: i.effective_tax_rate.as_ref().and_then(|v| v.to_string().parse().ok()),
            spouse_effective_tax_rate: i.spouse_effective_tax_rate.as_ref().and_then(|v| v.to_string().parse().ok()),
            investment_income_tax_rate: i.investment_income_tax_rate.as_ref().and_then(|v| v.to_string().parse().ok()),
            spouse_investment_income_tax_rate: i.spouse_investment_income_tax_rate.as_ref().and_then(|v| v.to_string().parse().ok()),
            monthly_deductions: i.monthly_deductions.as_ref().and_then(|v| v.to_string().parse().ok()),
            spouse_monthly_deductions: i.spouse_monthly_deductions.as_ref().and_then(|v| v.to_string().parse().ok()),
        }
    }
}

// ==============================================================================
// Asset Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SurveyAsset {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub asset_type: String,
    pub description: Option<String>,
    pub current_value: BigDecimal,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Ownership fields
    pub ownership: String,
    pub joint_split_percentage: Option<BigDecimal>,
    // Linked account (optional — for auto-refresh from portfolio)
    pub linked_account_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAssetRequest {
    pub asset_type: String,
    pub description: Option<String>,
    pub current_value: f64,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Ownership fields
    pub ownership: Option<String>,
    pub joint_split_percentage: Option<f64>,
    // Optional link to portfolio account
    pub linked_account_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAssetRequest {
    pub asset_type: Option<String>,
    pub description: Option<String>,
    pub current_value: Option<f64>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Ownership fields
    pub ownership: Option<String>,
    pub joint_split_percentage: Option<f64>,
    // Optional link to portfolio account
    pub linked_account_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetResponse {
    pub id: Uuid,
    pub asset_type: String,
    pub description: Option<String>,
    pub current_value: f64,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Ownership fields
    pub ownership: String,
    pub joint_split_percentage: Option<f64>,
    // Linked account info
    pub linked_account_id: Option<Uuid>,
    pub linked_account_nickname: Option<String>,
}

impl From<SurveyAsset> for AssetResponse {
    fn from(a: SurveyAsset) -> Self {
        Self {
            id: a.id,
            asset_type: a.asset_type,
            description: a.description,
            current_value: a.current_value.to_string().parse().unwrap_or(0.0),
            currency: a.currency,
            notes: a.notes,
            ownership: a.ownership,
            joint_split_percentage: a.joint_split_percentage.as_ref().and_then(|v| v.to_string().parse().ok()),
            linked_account_id: a.linked_account_id,
            linked_account_nickname: None, // populated separately via batch lookup in route handlers
        }
    }
}

// ==============================================================================
// Linkable Account (for listing portfolio accounts in the asset picker)
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LinkableAccount {
    pub id: Uuid,
    pub account_nickname: String,
    pub account_number: String,
    pub portfolio_name: String,
    pub latest_value: Option<BigDecimal>,
    pub latest_snapshot_date: Option<chrono::NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkableAccountResponse {
    pub id: Uuid,
    pub account_nickname: String,
    pub account_number: String,
    pub portfolio_name: String,
    pub latest_value: Option<f64>,
    pub latest_snapshot_date: Option<chrono::NaiveDate>,
}

impl From<LinkableAccount> for LinkableAccountResponse {
    fn from(a: LinkableAccount) -> Self {
        Self {
            id: a.id,
            account_nickname: a.account_nickname,
            account_number: a.account_number,
            portfolio_name: a.portfolio_name,
            latest_value: a.latest_value.as_ref().and_then(|v| v.to_string().parse().ok()),
            latest_snapshot_date: a.latest_snapshot_date,
        }
    }
}

// ==============================================================================
// Liability Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SurveyLiability {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub liability_type: String,
    pub description: Option<String>,
    pub balance: BigDecimal,
    pub interest_rate: Option<BigDecimal>,
    pub monthly_payment: Option<BigDecimal>,
    pub payment_frequency: Option<String>,
    pub linked_asset_id: Option<Uuid>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Ownership fields
    pub ownership: String,
    pub joint_split_percentage: Option<BigDecimal>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateLiabilityRequest {
    pub liability_type: String,
    pub description: Option<String>,
    pub balance: f64,
    pub interest_rate: Option<f64>,
    pub monthly_payment: Option<f64>,
    pub payment_frequency: Option<String>,
    pub linked_asset_id: Option<Uuid>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Ownership fields
    pub ownership: Option<String>,
    pub joint_split_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateLiabilityRequest {
    pub liability_type: Option<String>,
    pub description: Option<String>,
    pub balance: Option<f64>,
    pub interest_rate: Option<f64>,
    pub monthly_payment: Option<f64>,
    pub payment_frequency: Option<String>,
    pub linked_asset_id: Option<Uuid>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Ownership fields
    pub ownership: Option<String>,
    pub joint_split_percentage: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiabilityResponse {
    pub id: Uuid,
    pub liability_type: String,
    pub description: Option<String>,
    pub balance: f64,
    pub interest_rate: Option<f64>,
    pub monthly_payment: Option<f64>,
    pub payment_frequency: Option<String>,
    pub linked_asset_id: Option<Uuid>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    // Ownership fields
    pub ownership: String,
    pub joint_split_percentage: Option<f64>,
}

impl From<SurveyLiability> for LiabilityResponse {
    fn from(l: SurveyLiability) -> Self {
        Self {
            id: l.id,
            liability_type: l.liability_type,
            description: l.description,
            balance: l.balance.to_string().parse().unwrap_or(0.0),
            interest_rate: l.interest_rate.as_ref().and_then(|v| v.to_string().parse().ok()),
            monthly_payment: l.monthly_payment.as_ref().and_then(|v| v.to_string().parse().ok()),
            payment_frequency: l.payment_frequency,
            linked_asset_id: l.linked_asset_id,
            currency: l.currency,
            notes: l.notes,
            ownership: l.ownership,
            joint_split_percentage: l.joint_split_percentage.as_ref().and_then(|v| v.to_string().parse().ok()),
        }
    }
}

// ==============================================================================
// Goal Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SurveyGoal {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub goal_type: String,
    pub description: Option<String>,
    pub target_amount: Option<BigDecimal>,
    pub current_savings: Option<BigDecimal>,
    pub target_date: Option<NaiveDate>,
    pub priority: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub owner: String, // 'mine', 'spouse', 'joint'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateGoalRequest {
    pub goal_type: String,
    pub description: Option<String>,
    pub target_amount: Option<f64>,
    pub current_savings: Option<f64>,
    pub target_date: Option<NaiveDate>,
    pub priority: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateGoalRequest {
    pub goal_type: Option<String>,
    pub description: Option<String>,
    pub target_amount: Option<f64>,
    pub current_savings: Option<f64>,
    pub target_date: Option<NaiveDate>,
    pub priority: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalResponse {
    pub id: Uuid,
    pub goal_type: String,
    pub description: Option<String>,
    pub target_amount: Option<f64>,
    pub current_savings: Option<f64>,
    pub target_date: Option<NaiveDate>,
    pub priority: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub owner: String,
}

impl From<SurveyGoal> for GoalResponse {
    fn from(g: SurveyGoal) -> Self {
        Self {
            id: g.id,
            goal_type: g.goal_type,
            description: g.description,
            target_amount: g.target_amount.as_ref().and_then(|v| v.to_string().parse().ok()),
            current_savings: g.current_savings.as_ref().and_then(|v| v.to_string().parse().ok()),
            target_date: g.target_date,
            priority: g.priority,
            currency: g.currency,
            notes: g.notes,
            owner: g.owner,
        }
    }
}

// ==============================================================================
// Risk Profile Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SurveyRiskProfile {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub risk_tolerance: Option<String>,
    pub investment_experience: Option<String>,
    pub time_horizon_years: Option<i32>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpsertRiskProfileRequest {
    pub risk_tolerance: Option<String>,
    pub investment_experience: Option<String>,
    pub time_horizon_years: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfileResponse {
    pub id: Uuid,
    pub risk_tolerance: Option<String>,
    pub investment_experience: Option<String>,
    pub time_horizon_years: Option<i32>,
}

impl From<SurveyRiskProfile> for RiskProfileResponse {
    fn from(r: SurveyRiskProfile) -> Self {
        Self {
            id: r.id,
            risk_tolerance: r.risk_tolerance,
            investment_experience: r.investment_experience,
            time_horizon_years: r.time_horizon_years,
        }
    }
}

// ==============================================================================
// Financial Snapshot Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FinancialSnapshot {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub net_worth: Option<BigDecimal>,
    pub total_assets: Option<BigDecimal>,
    pub total_liabilities: Option<BigDecimal>,
    pub monthly_cash_flow: Option<BigDecimal>,
    pub projected_retirement_income: Option<BigDecimal>,
    pub snapshot_data: Option<serde_json::Value>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotResponse {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub net_worth: Option<f64>,
    pub total_assets: Option<f64>,
    pub total_liabilities: Option<f64>,
    pub monthly_cash_flow: Option<f64>,
    pub projected_retirement_income: Option<f64>,
    pub snapshot_data: Option<serde_json::Value>,
    pub generated_at: DateTime<Utc>,
}

impl From<FinancialSnapshot> for SnapshotResponse {
    fn from(s: FinancialSnapshot) -> Self {
        Self {
            id: s.id,
            survey_id: s.survey_id,
            net_worth: s.net_worth.as_ref().and_then(|v| v.to_string().parse().ok()),
            total_assets: s.total_assets.as_ref().and_then(|v| v.to_string().parse().ok()),
            total_liabilities: s.total_liabilities.as_ref().and_then(|v| v.to_string().parse().ok()),
            monthly_cash_flow: s.monthly_cash_flow.as_ref().and_then(|v| v.to_string().parse().ok()),
            projected_retirement_income: s.projected_retirement_income.as_ref().and_then(|v| v.to_string().parse().ok()),
            snapshot_data: s.snapshot_data,
            generated_at: s.generated_at,
        }
    }
}

// ==============================================================================
// Additional Income Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SurveyAdditionalIncome {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub income_type: String,
    pub description: Option<String>,
    pub monthly_amount: BigDecimal,
    pub is_recurring: Option<bool>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub owner: String, // 'mine' or 'spouse'
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateAdditionalIncomeRequest {
    pub income_type: String,
    pub description: Option<String>,
    pub monthly_amount: f64,
    pub is_recurring: Option<bool>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub owner: Option<String>, // 'mine' or 'spouse', defaults to 'mine'
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAdditionalIncomeRequest {
    pub income_type: Option<String>,
    pub description: Option<String>,
    pub monthly_amount: Option<f64>,
    pub is_recurring: Option<bool>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdditionalIncomeResponse {
    pub id: Uuid,
    pub income_type: String,
    pub description: Option<String>,
    pub monthly_amount: f64,
    pub is_recurring: bool,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub owner: String,
}

impl From<SurveyAdditionalIncome> for AdditionalIncomeResponse {
    fn from(i: SurveyAdditionalIncome) -> Self {
        Self {
            id: i.id,
            income_type: i.income_type,
            description: i.description,
            monthly_amount: i.monthly_amount.to_string().parse().unwrap_or(0.0),
            is_recurring: i.is_recurring.unwrap_or(true),
            currency: i.currency,
            notes: i.notes,
            owner: i.owner,
        }
    }
}

// ==============================================================================
// Expense Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SurveyExpense {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub expense_category: String,
    pub description: Option<String>,
    pub monthly_amount: BigDecimal,
    pub is_recurring: Option<bool>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateExpenseRequest {
    pub expense_category: String,
    pub description: Option<String>,
    pub monthly_amount: f64,
    pub is_recurring: Option<bool>,
    pub currency: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateExpenseRequest {
    pub expense_category: Option<String>,
    pub description: Option<String>,
    pub monthly_amount: Option<f64>,
    pub is_recurring: Option<bool>,
    pub currency: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseResponse {
    pub id: Uuid,
    pub expense_category: String,
    pub description: Option<String>,
    pub monthly_amount: f64,
    pub is_recurring: bool,
    pub currency: Option<String>,
    pub notes: Option<String>,
}

impl From<SurveyExpense> for ExpenseResponse {
    fn from(e: SurveyExpense) -> Self {
        Self {
            id: e.id,
            expense_category: e.expense_category,
            description: e.description,
            monthly_amount: e.monthly_amount.to_string().parse().unwrap_or(0.0),
            is_recurring: e.is_recurring.unwrap_or(true),
            currency: e.currency,
            notes: e.notes,
        }
    }
}

// ==============================================================================
// Household Expense Models
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SurveyHouseholdExpense {
    pub id: Uuid,
    pub survey_id: Uuid,
    pub expense_category: String,
    pub expense_type: String, // 'shared', 'mine', 'spouse'
    pub monthly_amount: BigDecimal,
    pub description: Option<String>,
    pub currency: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateHouseholdExpenseRequest {
    pub expense_category: String,
    pub expense_type: String,
    pub monthly_amount: f64,
    pub description: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateHouseholdExpenseRequest {
    pub expense_category: Option<String>,
    pub expense_type: Option<String>,
    pub monthly_amount: Option<f64>,
    pub description: Option<String>,
    pub currency: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HouseholdExpenseResponse {
    pub id: Uuid,
    pub expense_category: String,
    pub expense_type: String,
    pub monthly_amount: f64,
    pub description: Option<String>,
    pub currency: String,
}

impl From<SurveyHouseholdExpense> for HouseholdExpenseResponse {
    fn from(e: SurveyHouseholdExpense) -> Self {
        Self {
            id: e.id,
            expense_category: e.expense_category,
            expense_type: e.expense_type,
            monthly_amount: e.monthly_amount.to_string().parse().unwrap_or(0.0),
            description: e.description,
            currency: e.currency,
        }
    }
}

// ==============================================================================
// Full Survey Detail Response (combines all sections)
// ==============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurveyDetailResponse {
    pub id: Uuid,
    pub version: i32,
    pub status: String,
    pub personal_info: Option<PersonalInfoResponse>,
    pub income_info: Option<IncomeInfoResponse>,
    pub additional_income: Vec<AdditionalIncomeResponse>,
    pub expenses: Vec<ExpenseResponse>,
    pub household_expenses: Vec<HouseholdExpenseResponse>,
    pub assets: Vec<AssetResponse>,
    pub liabilities: Vec<LiabilityResponse>,
    pub goals: Vec<GoalResponse>,
    pub risk_profile: Option<RiskProfileResponse>,
    pub latest_snapshot: Option<SnapshotResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
