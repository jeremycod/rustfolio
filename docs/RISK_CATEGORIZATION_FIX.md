# Risk Categorization Fix - Industry Field Addition

**Date:** 2026-02-09
**Issue:** "Risk metrics not available for EQUITIES" error shown for mutual funds
**Status:** Fixed ✅

## Problem Description

Mutual fund tickers (FID7648, FID631, etc.) were showing the error message **"Risk metrics not available for EQUITIES"** instead of **"Risk metrics not available for mutual funds"**.

### Root Cause

1. **CSV Data Classification Issue**: Mutual funds were categorized as "EQUITIES" in the `asset_category` field, even though the `industry` field correctly identified them as "Mutual Funds"
2. **Missing Industry Field**: The `LatestAccountHolding` struct and database view didn't include the `industry` field
3. **Generic Error Messages**: The RiskBadge component only checked `asset_category` to generate error tooltips, not `industry`

### Example Data Issue

From CSV import:
```csv
Asset Category,Industry,Symbol,Holding
EQUITIES,Mutual Funds,FID7648,FDLTY GLOBAL EQUITY+ SR F -NL
EQUITIES,Mutual Funds,FID631,FDLTY CDN LRG CAP SR F -NL
```

These are mutual funds with proprietary tickers that don't exist in public price APIs (Alpha Vantage), but they're categorized as "EQUITIES" in the asset_category field.

## Solution Implemented

### 1. Backend Changes

#### Added Industry Field to Model
**File:** `backend/src/models/holding_snapshot.rs`

Added `industry: Option<String>` field to `LatestAccountHolding` struct:
```rust
pub struct LatestAccountHolding {
    pub id: uuid::Uuid,
    pub account_id: uuid::Uuid,
    pub account_nickname: String,
    pub account_number: String,
    pub ticker: String,
    pub holding_name: Option<String>,
    pub asset_category: Option<String>,
    pub industry: Option<String>,  // NEW FIELD
    pub quantity: BigDecimal,
    pub price: BigDecimal,
    pub market_value: BigDecimal,
    pub gain_loss: Option<BigDecimal>,
    pub gain_loss_pct: Option<BigDecimal>,
    pub snapshot_date: chrono::NaiveDate,
}
```

#### Updated Database View
**File:** `backend/migrations/20260209120000_add_industry_to_latest_holdings_view.sql`

Created migration to add `industry` field to the `latest_account_holdings` view:
```sql
DROP VIEW IF EXISTS latest_account_holdings;

CREATE VIEW latest_account_holdings AS
SELECT DISTINCT ON (h.account_id, h.ticker)
    h.id,
    h.account_id,
    a.account_nickname,
    a.account_number,
    h.ticker,
    h.holding_name,
    h.asset_category,
    h.industry,  -- NEW FIELD
    h.quantity,
    h.price,
    h.market_value,
    h.gain_loss,
    h.gain_loss_pct,
    h.snapshot_date
FROM holdings_snapshots h
JOIN accounts a ON h.account_id = a.id
ORDER BY h.account_id, h.ticker, h.snapshot_date DESC;
```

### 2. Frontend Changes

#### Updated TypeScript Type
**File:** `frontend/src/types.ts`

Added `industry` field to `LatestAccountHolding` type:
```typescript
export type LatestAccountHolding = {
    id: string;
    account_id: string;
    account_nickname: string;
    account_number: string;
    ticker: string;
    holding_name: string | null;
    asset_category: string | null;
    industry: string | null;  // NEW FIELD
    quantity: string;
    price: string;
    market_value: string;
    gain_loss: string | null;
    gain_loss_pct: string | null;
    snapshot_date: string;
};
```

#### Updated Portfolio Overview
**File:** `frontend/src/components/PortfolioOverview.tsx`

1. Added `industry` to aggregated holdings map
2. Pass `industry` prop to RiskBadge component:
```typescript
<RiskBadge
  ticker={holding.ticker}
  days={90}
  showLabel={false}
  onNavigate={onTickerNavigate}
  assetCategory={holding.asset_category}
  industry={holding.industry}  // NEW PROP
/>
```

#### Enhanced RiskBadge Component
**File:** `frontend/src/components/RiskBadge.tsx`

1. Added `industry` prop to interface
2. Updated error message logic to prioritize `industry` over `asset_category`:

```typescript
const getTooltipMessage = () => {
  // Prioritize industry field for better categorization
  if (industry) {
    const industryLower = industry.toLowerCase();
    if (industryLower.includes('mutual fund')) {
      return 'Risk metrics not available for mutual funds';
    }
    if (industryLower.includes('bond')) {
      return 'Risk metrics not available for bonds';
    }
    if (industryLower.includes('money market')) {
      return 'Risk metrics not available for money market funds';
    }
  }

  // Fall back to asset_category
  if (assetCategory) {
    const category = assetCategory.toLowerCase();
    if (category.includes('mutual fund') || category.includes('fund')) {
      return 'Risk metrics not available for mutual funds';
    }
    if (category.includes('bond') || category.includes('fixed')) {
      return 'Risk metrics not available for bonds';
    }
    return `Risk metrics not available for ${assetCategory}`;
  }

  return 'Risk metrics not available for this security type';
};
```

## Testing

1. **Backend Build:** ✅ Compiles successfully
2. **Frontend Build:** ✅ Compiles successfully
3. **Database Migration:** ✅ Applied successfully

## Results

Now when viewing the Portfolio Overview:
- **FID7648** (Mutual Fund): Shows tooltip **"Risk metrics not available for mutual funds"** ✅
- **FID631** (Mutual Fund): Shows tooltip **"Risk metrics not available for mutual funds"** ✅
- **Regular Equities** (GOOGL, AAPL, etc.): Show risk scores normally ✅

## Files Modified

### Backend
1. `backend/src/models/holding_snapshot.rs` - Added `industry` field
2. `backend/migrations/20260209120000_add_industry_to_latest_holdings_view.sql` - New migration

### Frontend
3. `frontend/src/types.ts` - Added `industry` field
4. `frontend/src/components/PortfolioOverview.tsx` - Pass industry to RiskBadge
5. `frontend/src/components/RiskBadge.tsx` - Prioritize industry for categorization

## Future Improvements

1. **Data Normalization**: Add logic to CSV import service to detect mutual fund tickers (FID*, RBF*, etc.) and automatically correct the asset_category field

2. **Ticker Classification Service**: Create a service that identifies proprietary mutual fund tickers to prevent unnecessary API calls:
   ```rust
   fn is_mutual_fund_ticker(ticker: &str) -> bool {
       ticker.starts_with("FID") ||
       ticker.starts_with("RBF") ||
       ticker.starts_with("LYZ")
       // etc.
   }
   ```

3. **Enhanced Logging**: Add debug logging when tickers are cached as failures to help diagnose future issues

4. **Pattern Detection**: Implement heuristics to detect proprietary ticker patterns and skip price API calls entirely for known mutual fund families

## Impact

- ✅ Clearer error messages for users
- ✅ Reduced confusion about why certain "equities" don't have risk data
- ✅ Better categorization using industry data
- ✅ No breaking changes to existing functionality
- ✅ Backward compatible (works with or without industry data)
