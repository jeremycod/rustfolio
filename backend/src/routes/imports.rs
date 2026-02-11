use axum::extract::{Path, State};
use axum::{Json, Router};
use axum::routing::{get, post};
use serde::{Deserialize, Serialize};
use tracing::{info, error};
use uuid::Uuid;
use std::path::PathBuf;

use crate::errors::AppError;
use crate::services::{csv_import_service, activity_import_service};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/portfolios/:portfolio_id/import", post(import_csv))
        .route("/import/files", get(list_csv_files))
}

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub file_path: String,
}

#[derive(Debug, Serialize)]
pub struct ImportResponse {
    pub accounts_created: usize,
    pub holdings_created: usize,
    pub transactions_detected: usize,
    pub errors: Vec<String>,
    pub snapshot_date: String,
}

#[derive(Debug, Serialize)]
pub struct CsvFileInfo {
    pub name: String,
    pub path: String,
    pub date: Option<String>,
    pub file_type: String, // "holdings" or "activities"
}

pub async fn list_csv_files() -> Result<Json<Vec<CsvFileInfo>>, AppError> {
    info!("GET /import/files - Listing available CSV files");

    let data_dir = PathBuf::from("data");

    if !data_dir.exists() {
        error!("Data directory does not exist");
        return Ok(Json(vec![]));
    }

    let mut files = Vec::new();

    match std::fs::read_dir(&data_dir) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();

                    if path.is_file() {
                        if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                            if filename.ends_with(".csv") {
                                if filename.starts_with("AccountsHoldings") {
                                    // Extract date from filename (AccountsHoldings-YYYYMMDD.csv)
                                    let date = extract_date_from_filename(filename);

                                    files.push(CsvFileInfo {
                                        name: filename.to_string(),
                                        path: path.to_string_lossy().to_string(),
                                        date,
                                        file_type: "holdings".to_string(),
                                    });
                                } else if filename.starts_with("AccountActivities") {
                                    // Extract date from filename (AccountActivities-{account}-YYYYMMDD.csv)
                                    let date = extract_activity_date_from_filename(filename);

                                    files.push(CsvFileInfo {
                                        name: filename.to_string(),
                                        path: path.to_string_lossy().to_string(),
                                        date,
                                        file_type: "activities".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            error!("Failed to read data directory: {}", e);
            return Err(AppError::Validation(format!("Failed to read data directory: {}", e)));
        }
    }

    // Sort by date (newest first)
    files.sort_by(|a, b| b.date.cmp(&a.date));

    info!("Found {} CSV files", files.len());
    Ok(Json(files))
}

fn extract_date_from_filename(filename: &str) -> Option<String> {
    // Expected format: AccountsHoldings-YYYYMMDD.csv
    let parts: Vec<&str> = filename.split('-').collect();
    if parts.len() >= 2 {
        let date_part = parts[1].replace(".csv", "");
        if date_part.len() == 8 {
            // Format as YYYY-MM-DD
            let year = &date_part[0..4];
            let month = &date_part[4..6];
            let day = &date_part[6..8];
            return Some(format!("{}-{}-{}", year, month, day));
        }
    }
    None
}

fn extract_activity_date_from_filename(filename: &str) -> Option<String> {
    // Expected format: AccountActivities-{account_number}-YYYYMMDD.csv
    let parts: Vec<&str> = filename.split('-').collect();
    if parts.len() >= 3 {
        let date_part = parts[2].replace(".csv", "");
        if date_part.len() == 8 {
            // Format as YYYY-MM-DD
            let year = &date_part[0..4];
            let month = &date_part[4..6];
            let day = &date_part[6..8];
            return Some(format!("{}-{}-{}", year, month, day));
        }
    }
    None
}

pub async fn import_csv(
    State(state): State<AppState>,
    Path(portfolio_id): Path<Uuid>,
    Json(data): Json<ImportRequest>,
) -> Result<Json<ImportResponse>, AppError> {
    info!("POST /portfolios/{}/import - Importing CSV file: {}", portfolio_id, data.file_path);

    let file_path = PathBuf::from(&data.file_path);

    if !file_path.exists() {
        error!("File does not exist: {}", data.file_path);
        return Err(AppError::Validation("File does not exist".to_string()));
    }

    // Determine file type from filename
    let filename = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| AppError::Validation("Invalid filename".to_string()))?;

    if filename.starts_with("AccountActivities") {
        // Import activity/transaction file
        info!("Importing AccountActivities file");
        let result = activity_import_service::import_activities_file(&state.pool, portfolio_id, &file_path)
            .await
            .map_err(|e| {
                error!("Failed to import activities file: {}", e);
                AppError::Validation(format!("Failed to import activities: {}", e))
            })?;

        info!(
            "Activity import completed: {} transactions imported, {} errors",
            result.transactions_imported,
            result.errors.len()
        );

        Ok(Json(ImportResponse {
            accounts_created: 0,
            holdings_created: 0,
            transactions_detected: result.transactions_imported,
            errors: result.errors,
            snapshot_date: "N/A".to_string(), // Activities don't have a snapshot date
        }))
    } else {
        // Import holdings snapshot file
        info!("Importing AccountsHoldings file");
        let result = csv_import_service::import_csv_file(&state.pool, portfolio_id, &file_path)
            .await
            .map_err(|e| {
                error!("Failed to import CSV file: {}", e);
                AppError::Validation(format!("Failed to import CSV: {}", e))
            })?;

        info!(
            "Import completed: {} accounts created, {} holdings created, {} transactions detected, {} errors",
            result.accounts_created,
            result.holdings_created,
            result.transactions_detected,
            result.errors.len()
        );

        Ok(Json(ImportResponse {
            accounts_created: result.accounts_created,
            holdings_created: result.holdings_created,
            transactions_detected: result.transactions_detected,
            errors: result.errors,
            snapshot_date: result.snapshot_date.to_string(),
        }))
    }
}
