/**
 * Smart asset type detector that corrects incorrect categorizations from CSV imports
 * by analyzing ticker symbols and holding names for patterns.
 */

export type DetectedAssetType =
  | 'Stock'
  | 'Mutual Fund'
  | 'ETF'
  | 'Bond'
  | 'Money Market'
  | 'Commodity'
  | 'Cash'
  | 'Unknown';

interface AssetDetectionInput {
  ticker: string;
  holdingName?: string | null;
  assetCategory?: string | null;
}

export function detectAssetType(input: AssetDetectionInput): DetectedAssetType {
  const { ticker, holdingName, assetCategory } = input;
  const name = (holdingName || '').toUpperCase();
  const tick = ticker.toUpperCase();
  const category = (assetCategory || '').toUpperCase();

  // Cash detection
  if (!ticker || ticker.trim() === '' || category.includes('CASH')) {
    return 'Cash';
  }

  // Mutual Fund detection (strongest patterns)
  // Pattern 1: Fidelity funds (FIDxxxx, FDLTY in name, -NL suffix)
  if (tick.match(/^FID\d+$/)) {
    return 'Mutual Fund';
  }

  // Pattern 2: Dynamic funds (DYNxxxx)
  if (tick.match(/^DYN\d+$/)) {
    return 'Mutual Fund';
  }

  // Pattern 3: AGF funds (AGFxxxx)
  if (tick.match(/^AGF\d+$/)) {
    return 'Mutual Fund';
  }

  // Pattern 4: Edge funds (EDGxxxx)
  if (tick.match(/^EDG\d+$/)) {
    return 'Mutual Fund';
  }

  // Pattern 5: RBF funds (RBFxxxx)
  if (tick.match(/^RBF\d+$/)) {
    return 'Mutual Fund';
  }

  // Pattern 6: Any fund with "FUND", "FD", "-NL" (No Load), "SR F" (Series F)
  if (name.includes(' FD ') || name.includes(' FUND') ||
      name.includes('-NL') || name.includes('SR F') ||
      name.includes('SERIES F') || name.includes('CLASS F')) {
    return 'Mutual Fund';
  }

  // Pattern 7: Fund families (FDLTY, DYNAMIC, etc.)
  if (name.includes('FDLTY') || name.includes('FIDELITY') ||
      name.includes('DYNAMIC') || name.includes('MACKENZIE') ||
      name.includes('INVESCO') || name.includes('CI ') ||
      name.includes('MANULIFE') || name.includes('TD MUTUAL')) {
    return 'Mutual Fund';
  }

  // ETF detection (before checking category)
  if (name.includes('ETF') || name.includes('EXCHANGE TRADED')) {
    // Further categorize ETFs
    if (name.includes('GOLD') || name.includes('SILVER') ||
        name.includes('COMMODITY') || name.includes('PRECIOUS')) {
      return 'Commodity';
    }
    if (name.includes('BOND') || name.includes('FIXED INCOME')) {
      return 'Bond';
    }
    return 'ETF';
  }

  // Bond/Fixed Income detection
  if (category.includes('FIXED INCOME') ||
      name.includes('BOND') || name.includes('DEBENTURE') ||
      name.includes('FIXED INCOME') || name.includes('TREASURY')) {
    return 'Bond';
  }

  // Money Market detection
  if (name.includes('MONEY MARKET') || name.includes('MONEY MKT') ||
      name.includes('CASH EQUIVALENT') || name.includes('SWEEP')) {
    return 'Money Market';
  }

  // Commodity detection
  if (category.includes('ALTERNATIVE') || category.includes('COMMODITY') ||
      name.includes('GOLD') || name.includes('SILVER') ||
      name.includes('COMMODITY') || name.includes('PRECIOUS METAL')) {
    return 'Commodity';
  }

  // Stock detection (typical stock tickers)
  // Pattern: 1-5 letters, possibly with .TO, .US, etc.
  if (tick.match(/^[A-Z]{1,5}(\.[A-Z]{1,3})?$/) && !tick.match(/\d/)) {
    // Additional checks for common stock patterns
    if (category.includes('EQUIT') || category.includes('STOCK') ||
        name.includes('COMMON') || name.includes('ORDINARY')) {
      return 'Stock';
    }
    // Default to stock for simple ticker symbols
    return 'Stock';
  }

  // If category says EQUITIES but we haven't matched stock patterns,
  // it's likely a mutual fund that was mislabeled
  if (category.includes('EQUIT') && tick.match(/\d/)) {
    return 'Mutual Fund';
  }

  // Default based on category if we couldn't detect otherwise
  if (category.includes('EQUIT') || category.includes('STOCK')) {
    return 'Stock';
  }

  return 'Unknown';
}

/**
 * Get a user-friendly display name for the detected asset type
 */
export function getAssetTypeDisplayName(assetType: DetectedAssetType): string {
  switch (assetType) {
    case 'Stock': return 'Stock';
    case 'Mutual Fund': return 'Mutual Fund';
    case 'ETF': return 'ETF';
    case 'Bond': return 'Bond';
    case 'Money Market': return 'Money Market';
    case 'Commodity': return 'Commodity';
    case 'Cash': return 'Cash';
    case 'Unknown': return 'Unknown';
  }
}
