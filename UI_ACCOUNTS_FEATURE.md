# Accounts UI Feature Documentation

## Overview

The frontend now includes a complete UI for managing accounts, importing CSV files, viewing holdings, and analyzing historical performance. The new "Accounts" section provides:

1. **Accounts Dashboard** - List all accounts with summary cards
2. **CSV Import** - Upload CSV files to populate account data
3. **Account Detail View** - Individual account view with tabs
4. **Holdings Tab** - Detailed holdings table per account
5. **History Tab** - Historical performance charts and data

## New Components

### 1. Accounts Component (`frontend/src/components/Accounts.tsx`)

The main accounts dashboard showing all accounts for the selected portfolio.

**Features:**
- Portfolio selector dropdown
- "Import CSV" button to upload holdings data
- Consolidated summary card showing total book value, market value, and gain/loss
- Account cards displaying:
  - Account nickname and number
  - Client name
  - Book value, market value, gain/loss, and G/L percentage
  - Visual progress bar
- Click on any account card to view details

**Usage:**
```tsx
<Accounts
  selectedPortfolioId={portfolioId}
  onPortfolioChange={setPortfolioId}
  onAccountSelect={handleAccountSelect}
/>
```

### 2. AccountDetail Component (`frontend/src/components/AccountDetail.tsx`)

Detailed view for a single account with tabs for holdings and history.

**Features:**
- Back button to return to accounts list
- Account header showing:
  - Account nickname and number
  - Client name
  - Summary: Book Value, Market Value, Gain/Loss, G/L %
- **Holdings Tab:**
  - Table showing all holdings with:
    - Symbol and holding name
    - Quantity, price, market value
    - Gain/loss in dollars and percentage
    - Asset category
- **History Tab:**
  - Line chart showing market value, book value, and gain/loss over time
  - Historical data table with all snapshots

**Usage:**
```tsx
<AccountDetail
  accountId={accountId}
  onBack={() => navigateBack()}
/>
```

### 3. CSV Import Modal

Built into the Accounts component. Allows users to import CSV files.

**How to use:**
1. Click "Import CSV" button
2. Enter the full path to the CSV file (e.g., `/Users/you/backend/data/AccountsHoldings-20260207.csv`)
3. Click "Import"
4. View results: accounts created, holdings created, any errors

**Expected CSV format:**
- Filename: `AccountsHoldings-YYYYMMDD.csv`
- Headers: Client Name, Client Id, Account Nickname, Account Number, Asset Category, Industry, Symbol, Holding, Quantity, Price, Fund, Average Cost, Book Value, Market Value, Accrued Interest, G/L, G/L (%), Percentage of Assets

## Navigation

The sidebar now includes a new "Accounts" menu item with an AccountBalanceWallet icon, positioned between "Dashboard" and "Holdings".

## Data Flow

```
User clicks "Import CSV"
  ↓
Enter file path
  ↓
POST /api/portfolios/{id}/import
  ↓
Backend parses CSV, creates/updates accounts and holdings
  ↓
Frontend refreshes account list
  ↓
User sees accounts dashboard
  ↓
User clicks on an account
  ↓
AccountDetail loads holdings and history
  ↓
User switches between Holdings and History tabs
```

## Type Definitions

New types added to `frontend/src/types.ts`:

```typescript
export type Account = {
    id: string;
    portfolio_id: string;
    account_number: string;
    account_nickname: string;
    client_id: string | null;
    client_name: string | null;
    created_at: string;
};

export type LatestAccountHolding = {
    id: string;
    account_id: string;
    account_nickname: string;
    account_number: string;
    ticker: string;
    holding_name: string | null;
    asset_category: string | null;
    quantity: string;
    price: string;
    market_value: string;
    gain_loss: string | null;
    gain_loss_pct: string | null;
    snapshot_date: string;
};

export type AccountValueHistory = {
    account_id: string;
    snapshot_date: string;
    total_value: string;
    total_cost: string;
    total_gain_loss: string;
    total_gain_loss_pct: string;
};

export type ImportResponse = {
    accounts_created: number;
    holdings_created: number;
    errors: string[];
    snapshot_date: string;
};
```

## API Endpoints Added

New endpoints in `frontend/src/lib/endpoints.ts`:

```typescript
// Account endpoints
export async function listAccounts(portfolioId: string): Promise<Account[]>
export async function getAccount(accountId: string): Promise<Account>
export async function getLatestHoldings(accountId: string): Promise<LatestAccountHolding[]>
export async function getAccountHistory(accountId: string): Promise<AccountValueHistory[]>
export async function getPortfolioHistory(portfolioId: string): Promise<AccountValueHistory[]>

// Import endpoints
export async function listCsvFiles(): Promise<CsvFileInfo[]>
export async function importCSV(portfolioId: string, filePath: string): Promise<ImportResponse>
```

Backend endpoints:
- `GET /api/import/files` - Lists available CSV files in backend/data directory
- `POST /api/portfolios/:id/import` - Imports a CSV file

## Testing the Feature

1. **Start the backend:**
   ```bash
   cd backend
   cargo run
   ```

2. **Start the frontend:**
   ```bash
   cd frontend
   npm run dev
   ```

3. **Import CSV data:**
   - Navigate to "Accounts" in the sidebar
   - Select a portfolio
   - Click "Import CSV"
   - Enter path: `/Users/zoranjeremic/Sources/RustLearning/rustfolio/backend/data/AccountsHoldings-20260207.csv`
   - Click Import
   - Wait for success message

4. **View accounts:**
   - See all accounts listed with summary cards
   - Click on an account to view details

5. **View holdings:**
   - In account detail, see Holdings tab with all positions
   - View quantity, prices, gain/loss for each holding

6. **View history:**
   - Switch to History tab
   - See line chart showing value over time
   - View historical data table below chart

7. **Import more snapshots:**
   - Import additional CSV files with different dates
   - History chart will update to show trends over time

## UI Design

The UI follows Material-UI design patterns:

- **Colors:**
  - Primary: Blue tones for headers and actions
  - Success: Green for positive gains
  - Error: Red for losses
  - Cards: White with subtle shadows

- **Typography:**
  - H4 for page titles
  - H6 for card titles
  - Body1/Body2 for content

- **Layout:**
  - Responsive grid for account cards
  - Full-width tables for holdings
  - Tabs for switching between views

## Future Enhancements

Potential improvements for the future:

1. **File Upload**: Add actual file upload instead of path input
2. **Real-time Data**: Fetch latest holdings values automatically
3. **Export**: Add export functionality for accounts and holdings
4. **Filters**: Filter by account type, date range, asset category
5. **Charts**: Add pie charts for asset allocation per account
6. **Comparison**: Compare multiple accounts side-by-side
7. **Alerts**: Set alerts for gain/loss thresholds
8. **Portfolio History Chart**: Add consolidated chart showing all accounts

## Files Modified/Created

### Frontend

**Created:**
- `frontend/src/components/Accounts.tsx` - Accounts dashboard component
- `frontend/src/components/AccountDetail.tsx` - Account detail with tabs

**Modified:**
- `frontend/src/types.ts` - Added Account, HoldingSnapshot, LatestAccountHolding, AccountValueHistory, ImportResponse types
- `frontend/src/lib/endpoints.ts` - Added account and import API endpoints
- `frontend/src/components/Layout.tsx` - Added "Accounts" menu item
- `frontend/src/App.tsx` - Added accounts routing and account detail navigation

## Troubleshooting

**Issue: No CSV files showing in the dropdown**
- Solution: Ensure CSV files are placed in the `backend/data` directory with the format `AccountsHoldings-YYYYMMDD.csv`
- Check that the backend server is running and can access the data directory

**Issue: Import fails with "File does not exist"**
- Solution: Ensure the CSV file exists in the backend/data directory
- Backend looks for files relative to its working directory

**Issue: No accounts showing after import**
- Solution: Ensure the CSV file is in the correct format and contains valid data
- Check backend logs for import errors

**Issue: History chart is empty**
- Solution: Import multiple CSV files with different dates to build historical data

**Issue: Holdings values are zero**
- Solution: The UI currently shows placeholder values. Holdings data comes from the latest snapshot. Ensure CSV was imported successfully.

## Summary

The Accounts UI feature provides a complete interface for:
- Importing CSV holdings data
- Viewing all accounts in a portfolio
- Drilling down into individual account details
- Analyzing holdings and performance
- Visualizing historical trends

The feature integrates seamlessly with the existing portfolio and holdings functionality, providing users with comprehensive account management capabilities.
