# Portfolio CSV Import Feature

## Overview

This feature allows you to import portfolio holdings from CSV files exported from your brokerage. The system tracks multiple accounts within a portfolio and maintains historical snapshots of holdings over time, enabling you to generate history graphs and analyze portfolio performance.

## Database Schema Changes

### New Tables

1. **accounts** - Tracks different accounts within a portfolio (e.g., RRSP, TFSA, Investment Account)
   - `id` - Unique identifier
   - `portfolio_id` - Reference to portfolio
   - `account_number` - Account number from broker
   - `account_nickname` - Human-readable account name
   - `client_id` - Optional client ID
   - `client_name` - Optional client name

2. **holdings_snapshots** - Historical holdings data
   - `id` - Unique identifier
   - `account_id` - Reference to account
   - `snapshot_date` - Date of the snapshot
   - `ticker` - Stock ticker symbol
   - `holding_name` - Full name of the holding
   - `asset_category` - Type of asset (EQUITIES, FIXED INCOME, etc.)
   - `industry` - Industry classification
   - `quantity` - Number of shares/units
   - `price` - Current price
   - `average_cost` - Average cost basis
   - `book_value` - Total cost basis
   - `market_value` - Current market value
   - `fund` - Fund type if applicable (USD, CAD, etc.)
   - `gain_loss` - Total gain/loss in dollars
   - `gain_loss_pct` - Gain/loss percentage
   - `percentage_of_assets` - Percentage of total portfolio

### Views

1. **latest_account_holdings** - Shows the most recent holdings for each ticker in each account
2. **account_value_history** - Aggregates total value, cost, and gain/loss by account and date

## CSV File Format

The CSV files should follow this structure (exported from your brokerage):

```csv
Client Name,Client Id,Account Nickname,Account Number,Asset Category,Industry,Symbol,Holding,Quantity,Price,Fund,Average Cost,Book Value,Market Value,Accrued Interest,G/L,G/L (%),Percentage of Assets
JOHN DOE,12345,RRSP,12345ABC,EQUITIES,Technology,AAPL,APPLE INC,100,$150.00,USD,$120.00,$12000.00,$15000.00,$0.00,$3000.00,25.00%,10.00%
```

### Expected Filename Format

Files should be named: `AccountsHoldings-YYYYMMDD.csv`

Example: `AccountsHoldings-20260207.csv`

The date in the filename determines the snapshot date for all holdings in that file.

## API Endpoints

### List Available CSV Files

```
GET /api/import/files
```

Returns a list of CSV files found in the backend/data directory:
```json
[
  {
    "name": "AccountsHoldings-20260207.csv",
    "path": "data/AccountsHoldings-20260207.csv",
    "date": "2026-02-07"
  },
  {
    "name": "AccountsHoldings-20260201.csv",
    "path": "data/AccountsHoldings-20260201.csv",
    "date": "2026-02-01"
  }
]
```

### Import CSV

```
POST /api/portfolios/:portfolio_id/import
Content-Type: application/json

{
  "file_path": "data/AccountsHoldings-20260207.csv"
}
```

Response:
```json
{
  "accounts_created": 3,
  "holdings_created": 25,
  "errors": [],
  "snapshot_date": "2026-02-07"
}
```

### List Accounts

```
GET /api/portfolios/:portfolio_id/accounts
```

Returns all accounts for a portfolio.

### Get Account Details

```
GET /api/accounts/:account_id
```

Returns details for a specific account.

### Get Latest Holdings

```
GET /api/accounts/:account_id/holdings
```

Returns the most recent holdings for an account.

### Get Account Value History

```
GET /api/accounts/:account_id/history
```

Returns historical value data for an account over time.

### Get Portfolio Value History

```
GET /api/portfolios/:portfolio_id/history
```

Returns aggregated historical value data for all accounts in a portfolio.

## Usage Example

1. **Create a portfolio** (if you don't have one):
   ```bash
   curl -X POST http://localhost:3000/api/portfolios \
     -H "Content-Type: application/json" \
     -d '{"name": "My Investment Portfolio"}'
   ```

2. **List available CSV files**:
   ```bash
   curl http://localhost:3000/api/import/files
   ```

3. **Import CSV file**:
   ```bash
   curl -X POST http://localhost:3000/api/portfolios/{portfolio_id}/import \
     -H "Content-Type: application/json" \
     -d '{"file_path": "data/AccountsHoldings-20260207.csv"}'
   ```

4. **View accounts**:
   ```bash
   curl http://localhost:3000/api/portfolios/{portfolio_id}/accounts
   ```

4. **View account history**:
   ```bash
   curl http://localhost:3000/api/accounts/{account_id}/history
   ```

6. **View portfolio history** (for graphing):
   ```bash
   curl http://localhost:3000/api/portfolios/{portfolio_id}/history
   ```

## Import Behavior

- **Accounts**: Created or updated based on account number. If an account with the same number exists, its nickname and client info are updated.
- **Holdings**: Upserted based on (account_id, snapshot_date, ticker). Multiple imports of the same date will update existing records.
- **Cash holdings**: Holdings without a ticker symbol (typically cash) are tracked at the account level but not as individual holdings.
- **Duplicate imports**: Safe to re-import the same file - it will update existing records rather than create duplicates.

## Next Steps

To integrate this with the frontend:

1. Create a UI for uploading CSV files or selecting from available files
2. Display account list with current values
3. Create charts using the history endpoints:
   - Line chart showing portfolio value over time
   - Stacked area chart showing account values
   - Pie chart showing current asset allocation
4. Add filters for date ranges and specific accounts

## Files Modified/Created

### Migrations
- `backend/migrations/20260207120000_create_accounts_and_history.sql`

### Models
- `backend/src/models/account.rs`
- `backend/src/models/holding_snapshot.rs`
- `backend/src/models/mod.rs`

### Database Queries
- `backend/src/db/account_queries.rs`
- `backend/src/db/holding_snapshot_queries.rs`
- `backend/src/db/mod.rs`

### Services
- `backend/src/services/csv_import_service.rs`
- `backend/src/services/mod.rs`

### Routes
- `backend/src/routes/accounts.rs`
- `backend/src/routes/imports.rs`
- `backend/src/routes/mod.rs`

### Configuration
- `backend/src/app.rs` - Added route registration
- `backend/Cargo.toml` - Added csv dependency

## Running Migrations

To apply the database schema changes:

```bash
cd backend
sqlx migrate run
```

Note: If you encounter migration issues, you may need to manually remove the incomplete migration record from the `_sqlx_migrations` table:

```sql
DELETE FROM _sqlx_migrations WHERE version = '20260207';
```

Then run `sqlx migrate run` again.
