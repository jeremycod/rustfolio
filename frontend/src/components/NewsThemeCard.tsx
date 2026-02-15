import {
  Card,
  CardContent,
  Typography,
  Box,
  Accordion,
  AccordionSummary,
  AccordionDetails,
  Link,
  Chip,
  Divider,
} from '@mui/material';
import { ExpandMore, Article, OpenInNew } from '@mui/icons-material';
import SentimentBadge from './SentimentBadge';
import type { NewsTheme } from '../types';

type Props = {
  theme: NewsTheme;
};

export default function NewsThemeCard({ theme }: Props) {
  const formatDate = (dateStr: string) => {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffHours / 24);

    if (diffHours < 24) {
      return `${diffHours}h ago`;
    } else if (diffDays < 7) {
      return `${diffDays}d ago`;
    } else {
      return date.toLocaleDateString();
    }
  };

  return (
    <Card sx={{ mb: 2 }}>
      <CardContent>
        {/* Header */}
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 2 }}>
          <Box sx={{ flex: 1 }}>
            <Typography variant="h6" gutterBottom>
              {theme.theme_name}
            </Typography>
            <Box sx={{ display: 'flex', gap: 1, alignItems: 'center', flexWrap: 'wrap' }}>
              <SentimentBadge sentiment={theme.sentiment} />
              <Chip
                icon={<Article />}
                label={`${theme.articles.length} article${theme.articles.length !== 1 ? 's' : ''}`}
                size="small"
                variant="outlined"
              />
              <Chip
                label={`Relevance: ${(theme.relevance_score * 100).toFixed(0)}%`}
                size="small"
                variant="outlined"
                color={theme.relevance_score > 0.7 ? 'primary' : 'default'}
              />
            </Box>
          </Box>
        </Box>

        {/* Summary */}
        <Typography variant="body1" color="text.secondary" sx={{ mb: 2 }}>
          {theme.summary}
        </Typography>

        <Divider sx={{ my: 2 }} />

        {/* Articles Accordion */}
        <Accordion elevation={0} sx={{ bgcolor: 'transparent', '&:before': { display: 'none' } }}>
          <AccordionSummary
            expandIcon={<ExpandMore />}
            sx={{ px: 0, minHeight: 'auto', '& .MuiAccordionSummary-content': { my: 1 } }}
          >
            <Typography variant="body2" fontWeight="bold">
              View Articles ({theme.articles.length})
            </Typography>
          </AccordionSummary>
          <AccordionDetails sx={{ px: 0, pt: 0 }}>
            <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1.5 }}>
              {theme.articles.map((article, idx) => (
                <Box
                  key={idx}
                  sx={{
                    p: 1.5,
                    bgcolor: 'background.default',
                    borderRadius: 1,
                    border: 1,
                    borderColor: 'divider',
                  }}
                >
                  <Link
                    href={article.url}
                    target="_blank"
                    rel="noopener noreferrer"
                    underline="hover"
                    sx={{
                      display: 'flex',
                      alignItems: 'flex-start',
                      gap: 0.5,
                      color: 'text.primary',
                      fontWeight: 'medium',
                      mb: 0.5,
                    }}
                  >
                    <Typography variant="body2" fontWeight="medium" sx={{ flex: 1 }}>
                      {article.title}
                    </Typography>
                    <OpenInNew fontSize="small" sx={{ color: 'action.active', flexShrink: 0 }} />
                  </Link>
                  <Typography variant="caption" color="text.secondary" display="block" sx={{ mb: 0.5 }}>
                    {article.snippet}
                  </Typography>
                  <Box sx={{ display: 'flex', gap: 1, alignItems: 'center' }}>
                    <Chip label={article.source} size="small" variant="outlined" />
                    <Typography variant="caption" color="text.secondary">
                      {formatDate(article.published_at)}
                    </Typography>
                  </Box>
                </Box>
              ))}
            </Box>
          </AccordionDetails>
        </Accordion>
      </CardContent>
    </Card>
  );
}
