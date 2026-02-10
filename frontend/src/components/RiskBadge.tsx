import { Chip, CircularProgress, Tooltip } from '@mui/material';
import { Warning, CheckCircle, Error } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getPositionRisk } from '../lib/endpoints';
import type { RiskLevel } from '../types';

interface RiskBadgeProps {
  ticker: string;
  days?: number;
  benchmark?: string;
  assetCategory?: string | null;
  industry?: string | null;
  showLabel?: boolean;
  onNavigate?: (ticker: string) => void;
}

export function RiskBadge({ ticker, days = 90, benchmark = 'SPY', assetCategory, industry, showLabel = true, onNavigate }: RiskBadgeProps) {
  const { data: risk, isLoading, error } = useQuery({
    queryKey: ['risk', ticker, days, benchmark],
    queryFn: () => getPositionRisk(ticker, days, benchmark),
    staleTime: 1000 * 60 * 60, // 1 hour - risk doesn't change frequently
    retry: false, // Don't retry for mutual funds/bonds - they won't have data
    refetchOnWindowFocus: false, // Don't refetch on focus
  });

  if (isLoading) {
    return (
      <Chip
        size="small"
        label={<CircularProgress size={12} />}
        sx={{ minWidth: 70 }}
      />
    );
  }

  if (error || !risk) {
    // For mutual funds, bonds, and other securities without stock data,
    // we silently show N/A instead of cluttering the UI with errors
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

    return (
      <Tooltip title={getTooltipMessage()}>
        <Chip
          size="small"
          label={showLabel ? "N/A" : ""}
          color="default"
          variant="outlined"
          sx={{
            opacity: 0.6,
            cursor: onNavigate ? 'pointer' : 'default',
          }}
          onClick={onNavigate ? () => onNavigate(ticker) : undefined}
        />
      </Tooltip>
    );
  }

  const getRiskColor = (level: RiskLevel): 'success' | 'warning' | 'error' => {
    switch (level) {
      case 'low':
        return 'success';
      case 'moderate':
        return 'warning';
      case 'high':
        return 'error';
    }
  };

  const getRiskIcon = (level: RiskLevel) => {
    switch (level) {
      case 'low':
        return <CheckCircle sx={{ fontSize: 14 }} />;
      case 'moderate':
        return <Warning sx={{ fontSize: 14 }} />;
      case 'high':
        return <Error sx={{ fontSize: 14 }} />;
    }
  };

  const tooltipContent = `
    Risk Score: ${risk.risk_score.toFixed(1)}/100
    Volatility: ${risk.metrics.volatility.toFixed(2)}%
    Max Drawdown: ${risk.metrics.max_drawdown.toFixed(2)}%
    ${risk.metrics.beta !== null ? `Beta: ${risk.metrics.beta.toFixed(2)}` : ''}
  `.trim();

  return (
    <Tooltip title={<span style={{ whiteSpace: 'pre-line' }}>{tooltipContent}</span>}>
      <Chip
        size="small"
        label={showLabel ? risk.risk_level.toUpperCase() : ''}
        color={getRiskColor(risk.risk_level)}
        icon={getRiskIcon(risk.risk_level)}
        sx={{
          minWidth: showLabel ? 85 : 'auto',
          fontWeight: 'bold',
          cursor: onNavigate ? 'pointer' : 'default',
        }}
        onClick={onNavigate ? () => onNavigate(ticker) : undefined}
      />
    </Tooltip>
  );
}
