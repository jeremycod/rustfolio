import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  Card,
  CardContent,
  CardActions,
  Button,
  Chip,
  Grid,
  CircularProgress,
  Alert,
  Collapse,
  LinearProgress,
  Divider,
  Skeleton,
  IconButton,
  Tooltip,
} from '@mui/material';
import {
  TrendingUp,
  TrendingDown,
  TrendingFlat,
  ExpandMore,
  ExpandLess,
  Lightbulb,
  Warning,
  Info,
} from '@mui/icons-material';
import { useQuery } from '@tanstack/react-query';
import { getRecommendationExplanation } from '../lib/endpoints';
import type {
  ScreeningResult,
  RecommendationExplanation,
  FactorScore,
  RiskLevel,
} from '../types';

interface RecommendationCardProps {
  result: ScreeningResult;
  onAddToWatchlist?: (symbol: string) => void;
}

export function RecommendationCard({ result, onAddToWatchlist }: RecommendationCardProps) {
  const [showExplanation, setShowExplanation] = useState(false);

  const explanationQ = useQuery({
    queryKey: ['recommendation-explanation', result.symbol],
    queryFn: () => getRecommendationExplanation(result.symbol),
    enabled: showExplanation,
    staleTime: 1000 * 60 * 60, // Cache for 1 hour
  });

  const scoreColor = result.composite_score >= 70 ? 'success' :
    result.composite_score >= 40 ? 'warning' : 'error';

  const riskColor = result.risk_level === 'low' ? 'success' :
    result.risk_level === 'moderate' ? 'warning' : 'error';

  return (
    <Card
      elevation={2}
      sx={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        transition: 'box-shadow 0.2s',
        '&:hover': { boxShadow: 6 },
        borderTop: 3,
        borderColor: `${scoreColor}.main`,
      }}
    >
      <CardContent sx={{ flex: 1 }}>
        {/* Header */}
        <Box display="flex" justifyContent="space-between" alignItems="flex-start" mb={1.5}>
          <Box>
            <Typography variant="h6" fontWeight="bold" component="div">
              {result.symbol}
            </Typography>
            <Typography variant="body2" color="text.secondary" noWrap sx={{ maxWidth: 200 }}>
              {result.company_name}
            </Typography>
          </Box>
          <Box display="flex" flexDirection="column" alignItems="flex-end" gap={0.5}>
            <ScoreCircle score={result.composite_score} />
            <Chip
              label={result.recommendation_strength}
              size="small"
              color={result.recommendation_strength === 'Strong' ? 'success' :
                result.recommendation_strength === 'Moderate' ? 'warning' : 'error'}
            />
          </Box>
        </Box>

        {/* Key Info */}
        <Box display="flex" gap={1} mb={2} flexWrap="wrap">
          <Chip label={result.sector} size="small" variant="outlined" />
          <Chip
            label={`Risk: ${result.risk_level}`}
            size="small"
            color={riskColor as any}
            variant="outlined"
          />
          <Chip
            label={`$${result.price.toFixed(2)}`}
            size="small"
            variant="outlined"
          />
        </Box>

        {/* Factor Scores */}
        <Typography variant="subtitle2" gutterBottom>
          Factor Scores
        </Typography>
        {result.factor_scores.map(fs => (
          <FactorScoreBar key={fs.factor} factorScore={fs} />
        ))}

        {/* Explanation Section */}
        <Box mt={2}>
          <Button
            size="small"
            startIcon={showExplanation ? <ExpandLess /> : <Lightbulb />}
            onClick={() => setShowExplanation(!showExplanation)}
            color="primary"
          >
            {showExplanation ? 'Hide Explanation' : 'Why This Recommendation?'}
          </Button>
          <Collapse in={showExplanation}>
            <Box mt={1}>
              {explanationQ.isLoading && (
                <Box>
                  <Skeleton variant="text" width="100%" />
                  <Skeleton variant="text" width="90%" />
                  <Skeleton variant="text" width="95%" />
                </Box>
              )}
              {explanationQ.error && (
                <Alert severity="warning" variant="outlined" sx={{ mt: 1 }}>
                  Unable to generate explanation: {(explanationQ.error as Error).message}
                </Alert>
              )}
              {explanationQ.data && (
                <ExplanationContent explanation={explanationQ.data} />
              )}
            </Box>
          </Collapse>
        </Box>
      </CardContent>

      <CardActions sx={{ px: 2, pb: 2 }}>
        {onAddToWatchlist && (
          <Button
            size="small"
            variant="outlined"
            onClick={() => onAddToWatchlist(result.symbol)}
          >
            Add to Watchlist
          </Button>
        )}
      </CardActions>
    </Card>
  );
}

function ScoreCircle({ score }: { score: number }) {
  const color = score >= 70 ? 'success.main' : score >= 40 ? 'warning.main' : 'error.main';

  return (
    <Tooltip title={`Composite Score: ${score.toFixed(1)}/100`}>
      <Box
        sx={{
          position: 'relative',
          display: 'inline-flex',
          width: 52,
          height: 52,
        }}
      >
        <CircularProgress
          variant="determinate"
          value={score}
          size={52}
          thickness={4}
          sx={{ color }}
        />
        <Box
          sx={{
            top: 0, left: 0, bottom: 0, right: 0,
            position: 'absolute',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          <Typography variant="body2" fontWeight="bold" sx={{ color }}>
            {score.toFixed(0)}
          </Typography>
        </Box>
      </Box>
    </Tooltip>
  );
}

function FactorScoreBar({ factorScore }: { factorScore: FactorScore }) {
  const color = factorScore.score >= 70 ? 'success' :
    factorScore.score >= 40 ? 'warning' : 'error';

  return (
    <Box mb={1}>
      <Box display="flex" justifyContent="space-between" alignItems="center">
        <Typography variant="caption" color="text.secondary">
          {factorScore.factor.charAt(0).toUpperCase() + factorScore.factor.slice(1)}
          {factorScore.weight > 0 && (
            <Typography component="span" variant="caption" color="text.disabled" sx={{ ml: 0.5 }}>
              ({(factorScore.weight * 100).toFixed(0)}%)
            </Typography>
          )}
        </Typography>
        <Typography variant="caption" fontWeight="bold">
          {factorScore.score.toFixed(1)}
        </Typography>
      </Box>
      <LinearProgress
        variant="determinate"
        value={Math.min(factorScore.score, 100)}
        color={color}
        sx={{ height: 6, borderRadius: 3 }}
      />
    </Box>
  );
}

function ExplanationContent({ explanation }: { explanation: RecommendationExplanation }) {
  return (
    <Paper variant="outlined" sx={{ p: 2, bgcolor: 'grey.50' }}>
      <Typography variant="body2" paragraph>
        {explanation.explanation}
      </Typography>

      {explanation.valuation_narrative && (
        <Box mb={1.5}>
          <Typography variant="subtitle2" color="primary.main" gutterBottom>
            Valuation
          </Typography>
          <Typography variant="body2">
            {explanation.valuation_narrative}
          </Typography>
        </Box>
      )}

      {explanation.risk_narrative && (
        <Box mb={1.5}>
          <Typography variant="subtitle2" color="warning.main" gutterBottom>
            Risk Assessment
          </Typography>
          <Typography variant="body2">
            {explanation.risk_narrative}
          </Typography>
        </Box>
      )}

      {explanation.factor_highlights && explanation.factor_highlights.length > 0 && (
        <Box mb={1.5}>
          <Typography variant="subtitle2" gutterBottom>
            Key Factors
          </Typography>
          {explanation.factor_highlights.map((fh, idx) => (
            <Box key={idx} display="flex" gap={1} mb={0.5}>
              <Chip label={fh.factor} size="small" variant="outlined" />
              <Typography variant="body2">{fh.narrative}</Typography>
            </Box>
          ))}
        </Box>
      )}

      {explanation.disclaimers && explanation.disclaimers.length > 0 && (
        <Box mt={1}>
          <Divider sx={{ mb: 1 }} />
          <Box display="flex" alignItems="flex-start" gap={0.5}>
            <Warning sx={{ fontSize: 16, color: 'text.disabled', mt: 0.3 }} />
            <Typography variant="caption" color="text.disabled">
              {explanation.disclaimers.join(' ')}
            </Typography>
          </Box>
        </Box>
      )}
    </Paper>
  );
}

// Grid display for multiple recommendation cards
interface RecommendationGridProps {
  results: ScreeningResult[];
  onAddToWatchlist?: (symbol: string) => void;
  loading?: boolean;
}

export function RecommendationGrid({ results, onAddToWatchlist, loading }: RecommendationGridProps) {
  if (loading) {
    return (
      <Grid container spacing={2}>
        {[1, 2, 3, 4, 5, 6].map(i => (
          <Grid item xs={12} sm={6} md={4} key={i}>
            <Card sx={{ height: 350 }}>
              <CardContent>
                <Skeleton variant="text" width="60%" height={32} />
                <Skeleton variant="text" width="80%" />
                <Skeleton variant="rectangular" height={120} sx={{ mt: 2, borderRadius: 1 }} />
                <Skeleton variant="text" width="40%" sx={{ mt: 2 }} />
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>
    );
  }

  if (results.length === 0) {
    return (
      <Alert severity="info">
        No recommendations available. Run the stock screener to generate recommendations.
      </Alert>
    );
  }

  return (
    <Grid container spacing={2}>
      {results.map(result => (
        <Grid item xs={12} sm={6} md={4} key={result.symbol}>
          <RecommendationCard
            result={result}
            onAddToWatchlist={onAddToWatchlist}
          />
        </Grid>
      ))}
    </Grid>
  );
}
