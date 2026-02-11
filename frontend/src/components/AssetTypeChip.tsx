import { Chip, Tooltip } from '@mui/material';
import {
  TrendingUp,
  AccountBalance,
  Savings,
  MonetizationOn,
  Category,
  CurrencyExchange,
  ShowChart,
} from '@mui/icons-material';
import { detectAssetType, getAssetTypeDisplayName, type DetectedAssetType } from '../lib/assetTypeDetector';

interface AssetTypeChipProps {
  ticker: string;
  holdingName?: string | null;
  assetCategory?: string | null;
  size?: 'small' | 'medium';
  showTooltip?: boolean;
}

interface AssetTypeInfo {
  color: 'default' | 'primary' | 'secondary' | 'info' | 'success' | 'warning' | 'error';
  icon: React.ReactElement;
  variant: 'filled' | 'outlined';
  description: string;
}

const getAssetTypeInfo = (assetType: DetectedAssetType): AssetTypeInfo => {
  switch (assetType) {
    case 'Stock':
      return {
        color: 'primary',
        icon: <TrendingUp fontSize="small" />,
        variant: 'filled',
        description: 'Individual stock/equity - Higher growth potential, higher volatility',
      };

    case 'Mutual Fund':
      return {
        color: 'secondary',
        icon: <Savings fontSize="small" />,
        variant: 'filled',
        description: 'Mutual Fund - Diversified portfolio, professionally managed investment',
      };

    case 'ETF':
      return {
        color: 'secondary',
        icon: <ShowChart fontSize="small" />,
        variant: 'filled',
        description: 'Exchange-Traded Fund - Diversified, trades like a stock',
      };

    case 'Bond':
      return {
        color: 'info',
        icon: <AccountBalance fontSize="small" />,
        variant: 'filled',
        description: 'Bond/Fixed Income - Lower risk, steady income',
      };

    case 'Money Market':
      return {
        color: 'success',
        icon: <MonetizationOn fontSize="small" />,
        variant: 'outlined',
        description: 'Money Market - Very low risk, highly liquid',
      };

    case 'Commodity':
      return {
        color: 'warning',
        icon: <CurrencyExchange fontSize="small" />,
        variant: 'outlined',
        description: 'Commodity/Alternative - Diversification asset, hedge against inflation',
      };

    case 'Cash':
      return {
        color: 'success',
        icon: <MonetizationOn fontSize="small" />,
        variant: 'outlined',
        description: 'Cash - Highly liquid, minimal risk',
      };

    case 'Unknown':
    default:
      return {
        color: 'default',
        icon: <Category fontSize="small" />,
        variant: 'outlined',
        description: 'Unknown asset type',
      };
  }
};

export function AssetTypeChip({
  ticker,
  holdingName,
  assetCategory,
  size = 'small',
  showTooltip = true
}: AssetTypeChipProps) {
  // Use smart detection to determine the actual asset type
  const detectedType = detectAssetType({ ticker, holdingName, assetCategory });
  const displayName = getAssetTypeDisplayName(detectedType);
  const info = getAssetTypeInfo(detectedType);

  const chip = (
    <Chip
      icon={info.icon}
      label={displayName}
      size={size}
      color={info.color}
      variant={info.variant}
      sx={{
        fontWeight: info.variant === 'filled' ? 600 : 400,
        '& .MuiChip-icon': {
          color: 'inherit',
        },
      }}
    />
  );

  if (showTooltip) {
    return (
      <Tooltip
        title={
          <span>
            {info.description}
            {assetCategory && assetCategory !== displayName && (
              <><br /><em style={{ fontSize: '0.85em', opacity: 0.8 }}>
                (CSV labeled as: {assetCategory})
              </em></>
            )}
          </span>
        }
        arrow
        placement="top"
      >
        {chip}
      </Tooltip>
    );
  }

  return chip;
}
