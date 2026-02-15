import {
  Box,
  Paper,
  Typography,
  List,
  ListItem,
  ListItemText,
  Alert,
  Skeleton,
  Divider,
  Chip,
} from '@mui/material';
import { Info, Psychology } from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getPortfolioNarrative } from '../lib/endpoints';
import AIBadge from './AIBadge';
import AILoadingState from './AILoadingState';
import type { PortfolioNarrative as PortfolioNarrativeType } from '../types';

type Props = {
  portfolioId: string;
  timePeriod?: string;
};

export default function PortfolioNarrative({ portfolioId, timePeriod = '90d' }: Props) {
  const { data: narrative, isLoading, error } = useQuery({
    queryKey: ['portfolioNarrative', portfolioId, timePeriod],
    queryFn: () => getPortfolioNarrative(portfolioId, timePeriod),
    staleTime: 5 * 60 * 1000, // Cache for 5 minutes
    retry: 1,
  });

  if (isLoading) {
    return (
      <Paper sx={{ p: 3 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
          <Psychology color="primary" />
          <Typography variant="h6">AI Portfolio Analysis</Typography>
          <AIBadge />
        </Box>
        <AILoadingState message="Analyzing your portfolio..." variant="skeleton" />
      </Paper>
    );
  }

  if (error) {
    const errorMessage = error instanceof Error ? error.message : 'Failed to generate narrative';
    const isLlmDisabled = errorMessage.includes('disabled') || errorMessage.includes('LLM');

    return (
      <Paper sx={{ p: 3 }}>
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
          <Psychology color="primary" />
          <Typography variant="h6">AI Portfolio Analysis</Typography>
          <AIBadge />
        </Box>
        <Alert severity={isLlmDisabled ? 'info' : 'warning'} icon={<Info />}>
          {isLlmDisabled
            ? 'AI-powered analysis is not enabled. Enable AI features in Settings to get personalized portfolio insights.'
            : `Unable to generate analysis: ${errorMessage}`}
        </Alert>
      </Paper>
    );
  }

  if (!narrative) {
    return null;
  }

  return (
    <Paper sx={{ p: 3 }}>
      {/* Header */}
      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
        <Psychology color="primary" />
        <Typography variant="h6">AI Portfolio Analysis</Typography>
        <AIBadge />
      </Box>

      <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mb: 2 }}>
        Generated {new Date(narrative.generated_at).toLocaleString()} • {timePeriod} period
      </Typography>

      {/* Disclaimer */}
      <Alert severity="info" sx={{ mb: 3 }} icon={<Info />}>
        This AI-generated analysis is for educational purposes only and does not constitute investment advice.
      </Alert>

      {/* Summary Section */}
      <Box sx={{ mb: 3 }}>
        <Typography variant="subtitle1" fontWeight="bold" gutterBottom>
          Summary
        </Typography>
        <Typography variant="body1" color="text.secondary">
          {narrative.summary}
        </Typography>
      </Box>

      <Divider sx={{ my: 2 }} />

      {/* Performance Explanation */}
      <Box sx={{ mb: 3 }}>
        <Typography variant="subtitle1" fontWeight="bold" gutterBottom>
          Performance Explanation
        </Typography>
        <Typography variant="body1" color="text.secondary">
          {narrative.performance_explanation}
        </Typography>
      </Box>

      <Divider sx={{ my: 2 }} />

      {/* Risk Highlights */}
      <Box sx={{ mb: 3 }}>
        <Typography variant="subtitle1" fontWeight="bold" gutterBottom>
          Risk Highlights
        </Typography>
        <List dense>
          {narrative.risk_highlights.map((highlight, index) => (
            <ListItem key={index} sx={{ px: 0 }}>
              <ListItemText
                primary={`• ${highlight}`}
                primaryTypographyProps={{
                  variant: 'body2',
                  color: 'text.secondary',
                }}
              />
            </ListItem>
          ))}
        </List>
      </Box>

      <Divider sx={{ my: 2 }} />

      {/* Top Contributors */}
      <Box>
        <Typography variant="subtitle1" fontWeight="bold" gutterBottom>
          Top Contributors
        </Typography>
        <List dense>
          {narrative.top_contributors.map((contributor, index) => (
            <ListItem key={index} sx={{ px: 0 }}>
              <ListItemText
                primary={`• ${contributor}`}
                primaryTypographyProps={{
                  variant: 'body2',
                  color: 'text.secondary',
                }}
              />
            </ListItem>
          ))}
        </List>
      </Box>
    </Paper>
  );
}
