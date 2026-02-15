import { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  Switch,
  FormControlLabel,
  TextField,
  Divider,
  Alert,
  CircularProgress,
  Tabs,
  Tab,
  Card,
  CardContent,
  Grid,
  Chip,
} from '@mui/material';
import {
  Settings as SettingsIcon,
  Psychology,
  BarChart,
  Close,
  Save,
  Info,
} from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getUserPreferences, updateUserPreferences, getLlmUsageStats } from '../lib/endpoints';
import type { UpdateUserPreferences } from '../types';

interface TabPanelProps {
  children?: React.ReactNode;
  value: number;
  index: number;
}

function TabPanel({ children, value, index }: TabPanelProps) {
  return (
    <div role="tabpanel" hidden={value !== index}>
      {value === index && <Box sx={{ py: 2 }}>{children}</Box>}
    </div>
  );
}

type Props = {
  open: boolean;
  onClose: () => void;
};

export default function UserSettingsDialog({ open, onClose }: Props) {
  const [activeTab, setActiveTab] = useState(0);
  const [llmEnabled, setLlmEnabled] = useState(false);
  const [narrativeCacheHours, setNarrativeCacheHours] = useState(24);
  const [hasChanges, setHasChanges] = useState(false);
  const queryClient = useQueryClient();

  // Use demo user ID for now
  const userId = '00000000-0000-0000-0000-000000000001';

  // Fetch user preferences
  const { data: preferences, isLoading: prefsLoading } = useQuery({
    queryKey: ['userPreferences', userId],
    queryFn: () => getUserPreferences(userId),
    enabled: open,
  });

  // Fetch LLM usage stats
  const { data: usageStats, isLoading: statsLoading } = useQuery({
    queryKey: ['llmUsageStats', userId],
    queryFn: () => getLlmUsageStats(userId),
    enabled: open,
  });

  // Update preferences mutation
  const updateMutation = useMutation({
    mutationFn: (data: UpdateUserPreferences) => updateUserPreferences(userId, data),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['userPreferences', userId] });
      setHasChanges(false);
    },
  });

  // Initialize form values when preferences load
  useEffect(() => {
    if (preferences) {
      setLlmEnabled(preferences.llm_enabled);
      setNarrativeCacheHours(preferences.narrative_cache_hours);
      setHasChanges(false);
    }
  }, [preferences]);

  const handleSave = () => {
    updateMutation.mutate({
      llm_enabled: llmEnabled,
      narrative_cache_hours: narrativeCacheHours,
    });
  };

  const handleLlmEnabledChange = (checked: boolean) => {
    setLlmEnabled(checked);
    setHasChanges(true);
  };

  const handleCacheHoursChange = (value: number) => {
    setNarrativeCacheHours(value);
    setHasChanges(true);
  };

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
      <DialogTitle>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
            <SettingsIcon />
            <Typography variant="h6">Settings</Typography>
          </Box>
          <Button onClick={onClose} color="inherit" size="small">
            <Close />
          </Button>
        </Box>
      </DialogTitle>

      <Divider />

      {prefsLoading ? (
        <DialogContent>
          <Box sx={{ display: 'flex', justifyContent: 'center', alignItems: 'center', py: 4 }}>
            <CircularProgress />
          </Box>
        </DialogContent>
      ) : (
        <>
          <Box sx={{ borderBottom: 1, borderColor: 'divider' }}>
            <Tabs value={activeTab} onChange={(_, val) => setActiveTab(val)} centered>
              <Tab icon={<Psychology />} label="AI Features" iconPosition="start" />
              <Tab icon={<BarChart />} label="Usage Stats" iconPosition="start" />
            </Tabs>
          </Box>

          <DialogContent>
            {/* Tab 1: AI Features */}
            <TabPanel value={activeTab} index={0}>
              <Alert severity="info" icon={<Info />} sx={{ mb: 3 }}>
                Configure AI-powered features including portfolio analysis, news sentiment, and the Q&A assistant.
              </Alert>

              {/* AI Features Toggle */}
              <Card variant="outlined" sx={{ mb: 3 }}>
                <CardContent>
                  <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', mb: 2 }}>
                    <Box>
                      <Typography variant="subtitle1" fontWeight="bold">
                        Enable AI Features
                      </Typography>
                      <Typography variant="body2" color="text.secondary">
                        Turn on AI-powered portfolio analysis, news sentiment, and Q&A assistant
                      </Typography>
                    </Box>
                    <FormControlLabel
                      control={
                        <Switch
                          checked={llmEnabled}
                          onChange={(e) => handleLlmEnabledChange(e.target.checked)}
                        />
                      }
                      label=""
                    />
                  </Box>

                  {llmEnabled && preferences?.consent_given_at && (
                    <Typography variant="caption" color="text.secondary">
                      Consent given: {new Date(preferences.consent_given_at).toLocaleDateString()}
                    </Typography>
                  )}
                </CardContent>
              </Card>

              {/* Cache Duration Setting */}
              <Card variant="outlined" sx={{ mb: 3 }}>
                <CardContent>
                  <Typography variant="subtitle1" fontWeight="bold" gutterBottom>
                    AI Analysis Cache Duration
                  </Typography>
                  <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
                    How long to cache AI-generated portfolio narratives before requiring a refresh
                  </Typography>

                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 2 }}>
                    <TextField
                      type="number"
                      value={narrativeCacheHours}
                      onChange={(e) => handleCacheHoursChange(parseInt(e.target.value) || 1)}
                      inputProps={{ min: 1, max: 168 }}
                      sx={{ width: 120 }}
                      size="small"
                      disabled={!llmEnabled}
                    />
                    <Typography variant="body2">hours</Typography>
                  </Box>

                  <Box sx={{ mt: 2, display: 'flex', gap: 1, flexWrap: 'wrap' }}>
                    <Chip
                      label="12 hours"
                      size="small"
                      onClick={() => handleCacheHoursChange(12)}
                      disabled={!llmEnabled}
                      variant={narrativeCacheHours === 12 ? 'filled' : 'outlined'}
                    />
                    <Chip
                      label="24 hours"
                      size="small"
                      onClick={() => handleCacheHoursChange(24)}
                      disabled={!llmEnabled}
                      variant={narrativeCacheHours === 24 ? 'filled' : 'outlined'}
                    />
                    <Chip
                      label="48 hours"
                      size="small"
                      onClick={() => handleCacheHoursChange(48)}
                      disabled={!llmEnabled}
                      variant={narrativeCacheHours === 48 ? 'filled' : 'outlined'}
                    />
                    <Chip
                      label="1 week"
                      size="small"
                      onClick={() => handleCacheHoursChange(168)}
                      disabled={!llmEnabled}
                      variant={narrativeCacheHours === 168 ? 'filled' : 'outlined'}
                    />
                  </Box>

                  <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 2 }}>
                    Shorter durations provide more up-to-date analysis but use more API calls. Longer durations reduce costs.
                  </Typography>
                </CardContent>
              </Card>

              {/* Information Box */}
              <Alert severity="warning" icon={<Info />}>
                <Typography variant="body2">
                  <strong>Note:</strong> AI features require API access and may incur costs. Cached data helps reduce expenses while maintaining accuracy.
                </Typography>
              </Alert>
            </TabPanel>

            {/* Tab 2: Usage Stats */}
            <TabPanel value={activeTab} index={1}>
              {statsLoading ? (
                <Box sx={{ display: 'flex', justifyContent: 'center', py: 4 }}>
                  <CircularProgress />
                </Box>
              ) : usageStats ? (
                <>
                  <Alert severity="info" icon={<Info />} sx={{ mb: 3 }}>
                    Track your AI usage and associated costs. Stats are updated in real-time.
                  </Alert>

                  <Grid container spacing={2}>
                    <Grid item xs={12} sm={6}>
                      <Card variant="outlined">
                        <CardContent>
                          <Typography variant="body2" color="text.secondary" gutterBottom>
                            Total Requests
                          </Typography>
                          <Typography variant="h4" fontWeight="bold">
                            {usageStats.total_requests.toLocaleString()}
                          </Typography>
                        </CardContent>
                      </Card>
                    </Grid>

                    <Grid item xs={12} sm={6}>
                      <Card variant="outlined">
                        <CardContent>
                          <Typography variant="body2" color="text.secondary" gutterBottom>
                            Total Cost
                          </Typography>
                          <Typography variant="h4" fontWeight="bold">
                            ${parseFloat(usageStats.total_cost).toFixed(4)}
                          </Typography>
                        </CardContent>
                      </Card>
                    </Grid>

                    <Grid item xs={12} sm={6}>
                      <Card variant="outlined">
                        <CardContent>
                          <Typography variant="body2" color="text.secondary" gutterBottom>
                            Prompt Tokens
                          </Typography>
                          <Typography variant="h5" fontWeight="bold">
                            {usageStats.total_prompt_tokens.toLocaleString()}
                          </Typography>
                        </CardContent>
                      </Card>
                    </Grid>

                    <Grid item xs={12} sm={6}>
                      <Card variant="outlined">
                        <CardContent>
                          <Typography variant="body2" color="text.secondary" gutterBottom>
                            Completion Tokens
                          </Typography>
                          <Typography variant="h5" fontWeight="bold">
                            {usageStats.total_completion_tokens.toLocaleString()}
                          </Typography>
                        </CardContent>
                      </Card>
                    </Grid>

                    <Grid item xs={12}>
                      <Card variant="outlined" sx={{ bgcolor: 'primary.light' }}>
                        <CardContent>
                          <Typography variant="body2" color="primary.contrastText" gutterBottom>
                            Current Month Cost
                          </Typography>
                          <Typography variant="h4" fontWeight="bold" color="primary.contrastText">
                            ${parseFloat(usageStats.current_month_cost).toFixed(4)}
                          </Typography>
                          <Typography variant="caption" color="primary.contrastText" sx={{ display: 'block', mt: 1 }}>
                            Resets at the beginning of each month
                          </Typography>
                        </CardContent>
                      </Card>
                    </Grid>
                  </Grid>
                </>
              ) : (
                <Alert severity="info">
                  No usage data available yet. AI features must be enabled to track usage.
                </Alert>
              )}
            </TabPanel>
          </DialogContent>

          <Divider />

          <DialogActions>
            <Button onClick={onClose} color="inherit">
              Cancel
            </Button>
            <Button
              onClick={handleSave}
              variant="contained"
              startIcon={<Save />}
              disabled={!hasChanges || updateMutation.isPending}
            >
              {updateMutation.isPending ? 'Saving...' : 'Save Changes'}
            </Button>
          </DialogActions>

          {updateMutation.isSuccess && (
            <Alert severity="success" sx={{ mx: 3, mb: 2 }}>
              Settings saved successfully!
            </Alert>
          )}

          {updateMutation.isError && (
            <Alert severity="error" sx={{ mx: 3, mb: 2 }}>
              Failed to save settings: {updateMutation.error instanceof Error ? updateMutation.error.message : 'Unknown error'}
            </Alert>
          )}
        </>
      )}
    </Dialog>
  );
}
