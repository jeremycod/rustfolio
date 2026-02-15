import { Chip } from '@mui/material';
import { TrendingUp, TrendingDown, Remove } from '@mui/icons-material';
import type { Sentiment } from '../types';

type Props = {
  sentiment: Sentiment;
  size?: 'small' | 'medium';
};

export default function SentimentBadge({ sentiment, size = 'small' }: Props) {
  const getColor = () => {
    switch (sentiment) {
      case 'positive':
        return 'success';
      case 'negative':
        return 'error';
      case 'neutral':
        return 'default';
    }
  };

  const getIcon = () => {
    switch (sentiment) {
      case 'positive':
        return <TrendingUp />;
      case 'negative':
        return <TrendingDown />;
      case 'neutral':
        return <Remove />;
    }
  };

  const getLabel = () => {
    return sentiment.charAt(0).toUpperCase() + sentiment.slice(1);
  };

  return (
    <Chip
      icon={getIcon()}
      label={getLabel()}
      color={getColor()}
      size={size}
      sx={{ fontWeight: 'bold' }}
    />
  );
}
