import { Chip } from '@mui/material';
import { TrendingUp } from '@mui/icons-material';

interface TickerChipProps {
  ticker: string;
  onNavigate: (ticker: string) => void;
  size?: 'small' | 'medium';
  variant?: 'filled' | 'outlined';
}

export function TickerChip({ ticker, onNavigate, size = 'small', variant = 'outlined' }: TickerChipProps) {
  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation();
    onNavigate(ticker);
  };

  return (
    <Chip
      label={ticker}
      size={size}
      variant={variant}
      onClick={handleClick}
      icon={<TrendingUp fontSize="small" />}
      sx={{
        cursor: 'pointer',
        '&:hover': {
          bgcolor: 'primary.light',
          color: 'primary.contrastText',
        },
      }}
    />
  );
}
