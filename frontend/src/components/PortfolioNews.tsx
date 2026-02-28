import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Alert,
  CircularProgress,
  Tabs,
  Tab,
  Chip,
  IconButton,
  Tooltip,
} from '@mui/material';
import { Newspaper, Info, Refresh } from '@mui/icons-material';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { getPortfolioNews } from '../lib/endpoints';
import NewsThemeCard from './NewsThemeCard';
import SentimentBadge from './SentimentBadge';
import AIBadge from './AIBadge';
import AILoadingState from './AILoadingState';
import { TickerActionMenu } from './TickerActionMenu';

type Props = {
  portfolioId: string;
  onTickerNavigate?: (ticker: string, page?: string) => void;
};

export default function PortfolioNews({ portfolioId, onTickerNavigate }: Props) {
  const [days, setDays] = useState(7);
  const [activeTab, setActiveTab] = useState(0);
  const [isRefreshing, setIsRefreshing] = useState(false);
  const queryClient = useQueryClient();

  const { data: newsAnalysis, isLoading, error } = useQuery({
    queryKey: ['portfolioNews', portfolioId, days],
    queryFn: () => getPortfolioNews(portfolioId, days, false),
    staleTime: 24 * 60 * 60 * 1000, // Cache for 24 hours (matches backend cache)
    retry: 1,
  });

  const handleRefresh = async () => {
    setIsRefreshing(true);
    try {
      // Force fetch new data
      const freshNews = await getPortfolioNews(portfolioId, days, true);
      // Update the cache with fresh data
      queryClient.setQueryData(['portfolioNews', portfolioId, days], freshNews);
    } catch (err) {
      console.error('Failed to refresh news:', err);
    } finally {
      setIsRefreshing(false);
    }
  };

  if (isLoading) {
    return (
      <Paper sx={{ p: 3 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
          <Newspaper color="primary" />
          <Typography variant="h6">Portfolio News & Insights</Typography>
          <AIBadge />
        </Box>
        <AILoadingState message="Fetching and analyzing news..." variant="skeleton" />
      </Paper>
    );
  }

  if (error) {
    const errorMessage = error instanceof Error ? error.message : 'Failed to fetch news';
    const isNewsDisabled = errorMessage.includes('not enabled') || errorMessage.includes('News');

    return (
      <Paper sx={{ p: 3 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
          <Newspaper color="primary" />
          <Typography variant="h6">Portfolio News & Insights</Typography>
          <AIBadge />
        </Box>
        <Alert severity={isNewsDisabled ? 'info' : 'warning'} icon={<Info />}>
          {isNewsDisabled
            ? 'News aggregation is not enabled. Configure a news API key in the backend .env file to enable this feature.'
            : `Unable to fetch news: ${errorMessage}`}
        </Alert>
      </Paper>
    );
  }

  if (!newsAnalysis || newsAnalysis.themes.length === 0) {
    return (
      <Paper sx={{ p: 3 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
          <Newspaper color="primary" />
          <Typography variant="h6">Portfolio News & Insights</Typography>
          <AIBadge />
        </Box>
        <Alert severity="info">
          No recent news found for your portfolio positions in the last {days} days.
        </Alert>
      </Paper>
    );
  }

  // Get unique tickers with news
  const tickersWithNews = Object.keys(newsAnalysis.position_news).sort();

  return (
    <Paper sx={{ p: 3 }}>
      {/* Header */}
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 2 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Newspaper color="primary" />
          <Typography variant="h6">Portfolio News & Insights</Typography>
          <AIBadge />
        </Box>
        <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
          <FormControl size="small" sx={{ minWidth: 120 }}>
            <InputLabel>Time Period</InputLabel>
            <Select
              value={days}
              label="Time Period"
              onChange={(e) => setDays(Number(e.target.value))}
            >
              <MenuItem value={7}>7 Days</MenuItem>
              <MenuItem value={14}>14 Days</MenuItem>
              <MenuItem value={30}>30 Days</MenuItem>
            </Select>
          </FormControl>
          <Tooltip title="Refresh news (bypasses cache)">
            <IconButton
              onClick={handleRefresh}
              disabled={isRefreshing || isLoading}
              color="primary"
              size="small"
            >
              <Refresh
                sx={{
                  animation: isRefreshing ? 'spin 1s linear infinite' : 'none',
                  '@keyframes spin': {
                    '0%': { transform: 'rotate(0deg)' },
                    '100%': { transform: 'rotate(360deg)' },
                  },
                }}
              />
            </IconButton>
          </Tooltip>
        </Box>
      </Box>

      {/* Overall Sentiment */}
      <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', mb: 2, flexWrap: 'wrap' }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Typography variant="body2" color="text.secondary">
            Overall Sentiment:
          </Typography>
          <SentimentBadge sentiment={newsAnalysis.overall_sentiment} size="medium" />
        </Box>
        <Chip
          label={`${newsAnalysis.themes.length} theme${newsAnalysis.themes.length !== 1 ? 's' : ''} identified`}
          size="small"
          variant="outlined"
        />
        <Chip
          label={`${tickersWithNews.length} position${tickersWithNews.length !== 1 ? 's' : ''} with news`}
          size="small"
          variant="outlined"
        />
      </Box>

      <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 2 }}>
        Last updated: {new Date(newsAnalysis.fetched_at).toLocaleString()}
      </Typography>

      {/* Disclaimer */}
      <Alert severity="info" sx={{ mb: 3 }} icon={<Info />}>
        News themes are automatically generated by AI. This is for informational purposes only and does not constitute investment advice.
      </Alert>

      {/* Tabs */}
      <Box sx={{ borderBottom: 1, borderColor: 'divider', mb: 3 }}>
        <Tabs value={activeTab} onChange={(_, val) => setActiveTab(val)}>
          <Tab label="All Themes" />
          <Tab label="By Position" />
        </Tabs>
      </Box>

      {/* All Themes Tab */}
      {activeTab === 0 && (
        <Box>
          {newsAnalysis.themes.length === 0 ? (
            <Alert severity="info">No news themes identified in this time period.</Alert>
          ) : (
            newsAnalysis.themes.map((theme, idx) => (
              <NewsThemeCard key={idx} theme={theme} />
            ))
          )}
        </Box>
      )}

      {/* By Position Tab */}
      {activeTab === 1 && (
        <Box>
          {tickersWithNews.length === 0 ? (
            <Alert severity="info">No position-specific news found.</Alert>
          ) : (
            tickersWithNews.map((ticker) => {
              const themes = newsAnalysis.position_news[ticker];
              if (!themes || themes.length === 0) return null;

              return (
                <Box key={ticker} sx={{ mb: 4 }}>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                    {onTickerNavigate ? (
                      <TickerActionMenu
                        ticker={ticker}
                        variant="text"
                        onNavigate={onTickerNavigate}
                      />
                    ) : (
                      <Typography variant="h6" component="span">{ticker}</Typography>
                    )}
                    <Chip label={`${themes.length} theme${themes.length !== 1 ? 's' : ''}`} size="small" />
                  </Box>
                  {themes.map((theme, idx) => (
                    <NewsThemeCard key={idx} theme={theme} />
                  ))}
                </Box>
              );
            })
          )}
        </Box>
      )}
    </Paper>
  );
}
