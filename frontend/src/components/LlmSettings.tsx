import {
    Box,
    Card,
    CardContent,
    Typography,
    Switch,
    FormControlLabel,
    Button,
    Alert,
    Divider,
    CircularProgress,
    TextField,
    Chip,
} from '@mui/material';
import { Psychology, TrendingUp, Save } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useState, useEffect } from 'react';
import {
    getUserPreferences,
    updateLlmConsent,
    getLlmUsageStats,
    updateUserPreferences,
} from '../lib/endpoints';
import ConsentDialog from './ConsentDialog';
import AIBadge from './AIBadge';
import type { UpdateUserPreferences } from '../types';

// Demo user ID - replace with actual auth when implemented
const DEMO_USER_ID = '00000000-0000-0000-0000-000000000001';

export default function LlmSettings() {
    const queryClient = useQueryClient();
    const [consentDialogOpen, setConsentDialogOpen] = useState(false);
    const [narrativeCacheHours, setNarrativeCacheHours] = useState(24);
    const [hasChanges, setHasChanges] = useState(false);

    // Fetch user preferences
    const { data: preferences, isLoading: prefsLoading } = useQuery({
        queryKey: ['userPreferences', DEMO_USER_ID],
        queryFn: () => getUserPreferences(DEMO_USER_ID),
    });

    // Fetch usage stats
    const { data: usageStats, isLoading: statsLoading } = useQuery({
        queryKey: ['llmUsageStats', DEMO_USER_ID],
        queryFn: () => getLlmUsageStats(DEMO_USER_ID),
        enabled: preferences?.llm_enabled === true,
    });

    // Initialize cache hours from preferences
    useEffect(() => {
        if (preferences) {
            setNarrativeCacheHours(preferences.narrative_cache_hours);
            setHasChanges(false);
        }
    }, [preferences]);

    // Mutation for updating consent
    const updateConsentMutation = useMutation({
        mutationFn: (consent: boolean) => updateLlmConsent(DEMO_USER_ID, consent),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['userPreferences', DEMO_USER_ID] });
            queryClient.invalidateQueries({ queryKey: ['llmUsageStats', DEMO_USER_ID] });
        },
    });

    // Mutation for updating preferences
    const updatePreferencesMutation = useMutation({
        mutationFn: (data: UpdateUserPreferences) => updateUserPreferences(DEMO_USER_ID, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['userPreferences', DEMO_USER_ID] });
            setHasChanges(false);
        },
    });

    const handleToggleChange = () => {
        if (!preferences?.llm_enabled) {
            // Show consent dialog when enabling
            setConsentDialogOpen(true);
        } else {
            // Disable directly
            updateConsentMutation.mutate(false);
        }
    };

    const handleAcceptConsent = () => {
        setConsentDialogOpen(false);
        updateConsentMutation.mutate(true);
    };

    const handleDeclineConsent = () => {
        setConsentDialogOpen(false);
    };

    const handleCacheHoursChange = (value: number) => {
        setNarrativeCacheHours(value);
        setHasChanges(true);
    };

    const handleSavePreferences = () => {
        if (preferences) {
            updatePreferencesMutation.mutate({
                llm_enabled: preferences.llm_enabled,
                narrative_cache_hours: narrativeCacheHours,
            });
        }
    };

    if (prefsLoading) {
        return (
            <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                <CircularProgress />
            </Box>
        );
    }

    const formatCost = (cost: string) => {
        const num = parseFloat(cost);
        return `$${num.toFixed(4)}`;
    };

    return (
        <Box>
            <Card>
                <CardContent>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 2 }}>
                        <Psychology color="primary" fontSize="large" />
                        <Typography variant="h6">AI-Powered Features</Typography>
                        <AIBadge variant="experimental" />
                    </Box>

                    <Typography variant="body2" color="text.secondary" paragraph>
                        Enable AI to get intelligent insights about your portfolio performance,
                        risk analysis, and investment patterns.
                    </Typography>

                    <FormControlLabel
                        control={
                            <Switch
                                checked={preferences?.llm_enabled || false}
                                onChange={handleToggleChange}
                                disabled={updateConsentMutation.isPending}
                            />
                        }
                        label="Enable AI-powered insights"
                    />

                    {preferences?.llm_enabled && (
                        <>
                            <Divider sx={{ my: 2 }} />

                            <Typography variant="subtitle2" gutterBottom fontWeight="bold">
                                Consent Information
                            </Typography>
                            <Typography variant="body2" color="text.secondary" paragraph>
                                Consent given: {preferences.consent_given_at ? new Date(preferences.consent_given_at).toLocaleString() : 'N/A'}
                            </Typography>

                            <Divider sx={{ my: 2 }} />

                            <Typography variant="subtitle2" gutterBottom fontWeight="bold">
                                AI Analysis Cache Duration
                            </Typography>
                            <Typography variant="body2" color="text.secondary" paragraph>
                                How long to cache AI-generated portfolio narratives before requiring a refresh
                            </Typography>

                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
                                <TextField
                                    type="number"
                                    value={narrativeCacheHours}
                                    onChange={(e) => handleCacheHoursChange(parseInt(e.target.value) || 1)}
                                    inputProps={{ min: 1, max: 168 }}
                                    sx={{ width: 120 }}
                                    size="small"
                                />
                                <Typography variant="body2">hours</Typography>
                            </Box>

                            <Box sx={{ display: 'flex', gap: 1, flexWrap: 'wrap', mb: 2 }}>
                                <Chip
                                    label="12 hours"
                                    size="small"
                                    onClick={() => handleCacheHoursChange(12)}
                                    variant={narrativeCacheHours === 12 ? 'filled' : 'outlined'}
                                />
                                <Chip
                                    label="24 hours"
                                    size="small"
                                    onClick={() => handleCacheHoursChange(24)}
                                    variant={narrativeCacheHours === 24 ? 'filled' : 'outlined'}
                                />
                                <Chip
                                    label="48 hours"
                                    size="small"
                                    onClick={() => handleCacheHoursChange(48)}
                                    variant={narrativeCacheHours === 48 ? 'filled' : 'outlined'}
                                />
                                <Chip
                                    label="1 week"
                                    size="small"
                                    onClick={() => handleCacheHoursChange(168)}
                                    variant={narrativeCacheHours === 168 ? 'filled' : 'outlined'}
                                />
                            </Box>

                            <Button
                                variant="contained"
                                startIcon={<Save />}
                                onClick={handleSavePreferences}
                                disabled={!hasChanges || updatePreferencesMutation.isPending}
                                size="small"
                            >
                                {updatePreferencesMutation.isPending ? 'Saving...' : 'Save Cache Settings'}
                            </Button>

                            {updatePreferencesMutation.isSuccess && (
                                <Alert severity="success" sx={{ mt: 2 }}>
                                    Cache settings saved successfully!
                                </Alert>
                            )}

                            {updatePreferencesMutation.isError && (
                                <Alert severity="error" sx={{ mt: 2 }}>
                                    Failed to save settings: {updatePreferencesMutation.error instanceof Error ? updatePreferencesMutation.error.message : 'Unknown error'}
                                </Alert>
                            )}

                            <Typography variant="caption" color="text.secondary" sx={{ display: 'block', mt: 2 }}>
                                Shorter durations provide more up-to-date analysis but use more API calls. Longer durations reduce costs.
                            </Typography>

                            {statsLoading ? (
                                <Box sx={{ display: 'flex', justifyContent: 'center', p: 2 }}>
                                    <CircularProgress size={24} />
                                </Box>
                            ) : usageStats && (
                                <>
                                    <Divider sx={{ my: 2 }} />

                                    <Typography variant="subtitle2" gutterBottom fontWeight="bold">
                                        <TrendingUp fontSize="small" sx={{ verticalAlign: 'middle', mr: 0.5 }} />
                                        Usage Statistics
                                    </Typography>

                                    <Box sx={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 2, mt: 1 }}>
                                        <Box>
                                            <Typography variant="caption" color="text.secondary">
                                                Total Requests
                                            </Typography>
                                            <Typography variant="h6">
                                                {usageStats.total_requests}
                                            </Typography>
                                        </Box>

                                        <Box>
                                            <Typography variant="caption" color="text.secondary">
                                                Total Cost
                                            </Typography>
                                            <Typography variant="h6">
                                                {formatCost(usageStats.total_cost)}
                                            </Typography>
                                        </Box>

                                        <Box>
                                            <Typography variant="caption" color="text.secondary">
                                                Current Month
                                            </Typography>
                                            <Typography variant="h6">
                                                {formatCost(usageStats.current_month_cost)}
                                            </Typography>
                                        </Box>

                                        <Box>
                                            <Typography variant="caption" color="text.secondary">
                                                Total Tokens
                                            </Typography>
                                            <Typography variant="h6">
                                                {(usageStats.total_prompt_tokens + usageStats.total_completion_tokens).toLocaleString()}
                                            </Typography>
                                        </Box>
                                    </Box>
                                </>
                            )}

                            <Alert severity="info" sx={{ mt: 2 }}>
                                Rate limit: 50 requests per hour. AI features use OpenAI's GPT-4o-mini model.
                            </Alert>

                            <Button
                                variant="outlined"
                                color="error"
                                size="small"
                                onClick={() => updateConsentMutation.mutate(false)}
                                disabled={updateConsentMutation.isPending}
                                sx={{ mt: 2 }}
                            >
                                Revoke Consent & Disable
                            </Button>
                        </>
                    )}

                    {!preferences?.llm_enabled && (
                        <Alert severity="info" sx={{ mt: 2 }}>
                            AI features are currently disabled. Enable them to get intelligent portfolio insights.
                        </Alert>
                    )}
                </CardContent>
            </Card>

            <ConsentDialog
                open={consentDialogOpen}
                onClose={() => setConsentDialogOpen(false)}
                onAccept={handleAcceptConsent}
                onDecline={handleDeclineConsent}
            />
        </Box>
    );
}
