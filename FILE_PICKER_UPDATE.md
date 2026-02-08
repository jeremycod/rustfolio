# File Picker Update

## What Changed

Replaced the manual file path text input with an intelligent **file picker dropdown** that automatically lists available CSV files from your backend data directory.

## New User Experience

### Before
- User had to type the full absolute path: `/Users/zoranjeremic/Sources/RustLearning/rustfolio/backend/data/AccountsHoldings-20260207.csv`
- Error-prone (typos, wrong paths)
- Required knowledge of file system structure

### After
- Click "Import CSV" button
- **Select from a dropdown** showing all available files
- Each file shows:
  - 📅 Calendar icon
  - File name
  - Snapshot date (e.g., "Snapshot: 2026-02-07")
- Files sorted by date (newest first)
- One click to select, one click to import

## Technical Implementation

### Backend
Added new endpoint `GET /api/import/files` that:
- Scans the `backend/data` directory
- Filters for CSV files starting with "AccountsHoldings"
- Extracts date from filename (AccountsHoldings-YYYYMMDD.csv)
- Returns array of file info with name, path, and parsed date
- Sorts files by date (newest first)

**File:** `backend/src/routes/imports.rs`

### Frontend
Updated the import modal to:
- Fetch available CSV files when modal opens
- Display files in a Material-UI Select dropdown
- Show loading spinner while fetching files
- Display warning if no files found
- Show file details with calendar icon

**File:** `frontend/src/components/Accounts.tsx`

### New Types
```typescript
export type CsvFileInfo = {
    name: string;
    path: string;
    date: string | null;
};
```

## How It Works

1. User clicks "Import CSV" in the Accounts page
2. Modal opens and automatically fetches list of CSV files: `GET /api/import/files`
3. Backend scans `data/` directory and returns available files
4. Files appear in dropdown with formatted names and dates
5. User selects a file from the list
6. Clicks "Import" button
7. Backend processes the selected file

## Benefits

✅ **User-friendly** - No need to know file paths
✅ **Error-free** - No typos or invalid paths
✅ **Visual feedback** - See all available files at a glance
✅ **Date information** - Know which snapshot you're importing
✅ **Automatic** - Files discovered automatically
✅ **Sorted** - Newest files appear first

## File Format Requirements

CSV files must be in the backend/data directory and follow this naming convention:
- Format: `AccountsHoldings-YYYYMMDD.csv`
- Examples:
  - ✅ `AccountsHoldings-20260207.csv`
  - ✅ `AccountsHoldings-20260201.csv`
  - ❌ `holdings.csv` (won't be recognized)
  - ❌ `AccountsHoldings.csv` (missing date)

## Edge Cases Handled

1. **No files found**: Shows warning message suggesting correct file format
2. **Loading state**: Displays spinner while fetching files
3. **Invalid dates**: Files without valid dates show without date info
4. **Directory doesn't exist**: Returns empty list gracefully
5. **Permission errors**: Logs error and shows user-friendly message

## API Endpoints

### List CSV Files
```
GET /api/import/files
```

**Response:**
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

### Import CSV (unchanged)
```
POST /api/portfolios/:portfolio_id/import
```

**Request:**
```json
{
  "file_path": "data/AccountsHoldings-20260207.csv"
}
```

## Testing

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

3. **Test the file picker:**
   - Navigate to "Accounts"
   - Select a portfolio
   - Click "Import CSV"
   - You should see a dropdown with your CSV files
   - Select a file and click "Import"

## Files Modified

### Backend
- ✅ `backend/src/routes/imports.rs` - Added list_csv_files endpoint and extract_date_from_filename helper

### Frontend
- ✅ `frontend/src/types.ts` - Added CsvFileInfo type
- ✅ `frontend/src/lib/endpoints.ts` - Added listCsvFiles API call
- ✅ `frontend/src/components/Accounts.tsx` - Replaced TextField with Select dropdown

### Documentation
- ✅ `IMPORT_FEATURE.md` - Updated with new endpoint info
- ✅ `UI_ACCOUNTS_FEATURE.md` - Updated usage instructions

## Screenshots/UI Preview

**Import Modal with File Picker:**
```
┌─────────────────────────────────────┐
│ Import CSV File                  × │
├─────────────────────────────────────┤
│ ℹ️ Select a CSV file from the       │
│   backend/data directory to import  │
│   account holdings.                 │
│                                     │
│ CSV File ▼                          │
│ ┌─────────────────────────────────┐ │
│ │ 📅 AccountsHoldings-20260207.csv│ │
│ │    Snapshot: 2026-02-07         │ │
│ ├─────────────────────────────────┤ │
│ │ 📅 AccountsHoldings-20260201.csv│ │
│ │    Snapshot: 2026-02-01         │ │
│ └─────────────────────────────────┘ │
│                                     │
│              [Cancel]  [Import]     │
└─────────────────────────────────────┘
```

## Summary

The file picker makes importing CSV files much more user-friendly by automatically discovering available files and presenting them in an easy-to-use dropdown. No more typing long file paths or worrying about typos!
