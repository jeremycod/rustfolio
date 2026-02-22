import { useState, useEffect } from 'react';
import {
    Box,
    Typography,
    Paper,
    FormControlLabel,
    Switch,
    TextField,
    Button,
    Select,
    MenuItem,
    FormControl,
    InputLabel,
    Snackbar,
    Alert,
    CircularProgress
} from '@mui/material';
import { Save } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { getNotificationPreferences, updateNotificationPreferences } from '../lib/endpoints';
import type { UpdateNotificationPreferences } from '../types';

export default function NotificationPreferencesSection() {
    const queryClient = useQueryClient();
    const [snackbarOpen, setSnackbarOpen] = useState(false);
    const [snackbarMessage, setSnackbarMessage] = useState('');
    const [snackbarSeverity, setSnackbarSeverity] = useState<'success' | 'error'>('success');

    // Form state
    const [emailEnabled, setEmailEnabled] = useState(true);
    const [inAppEnabled, setInAppEnabled] = useState(true);
    const [webhookEnabled, setWebhookEnabled] = useState(false);
    const [webhookUrl, setWebhookUrl] = useState('');
    const [quietHoursStart, setQuietHoursStart] = useState('');
    const [quietHoursEnd, setQuietHoursEnd] = useState('');
    const [timezone, setTimezone] = useState('America/New_York');
    const [maxDailyEmails, setMaxDailyEmails] = useState(10);

    // Fetch preferences
    const preferencesQ = useQuery({
        queryKey: ['notificationPreferences'],
        queryFn: getNotificationPreferences
    });

    // Load preferences into form state
    useEffect(() => {
        if (preferencesQ.data) {
            setEmailEnabled(preferencesQ.data.email_enabled);
            setInAppEnabled(preferencesQ.data.in_app_enabled);
            setWebhookEnabled(preferencesQ.data.webhook_enabled);
            setWebhookUrl(preferencesQ.data.webhook_url || '');
            setQuietHoursStart(preferencesQ.data.quiet_hours_start || '');
            setQuietHoursEnd(preferencesQ.data.quiet_hours_end || '');
            setTimezone(preferencesQ.data.timezone);
            setMaxDailyEmails(preferencesQ.data.max_daily_emails);
        }
    }, [preferencesQ.data]);

    // Update preferences mutation
    const updateM = useMutation({
        mutationFn: updateNotificationPreferences,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notificationPreferences'] });
            setSnackbarMessage('Notification preferences saved successfully');
            setSnackbarSeverity('success');
            setSnackbarOpen(true);
        },
        onError: (error: any) => {
            setSnackbarMessage(`Failed: ${error.response?.data || error.message}`);
            setSnackbarSeverity('error');
            setSnackbarOpen(true);
        }
    });

    const handleSave = () => {
        const data: UpdateNotificationPreferences = {
            email_enabled: emailEnabled,
            in_app_enabled: inAppEnabled,
            webhook_enabled: webhookEnabled,
            webhook_url: webhookUrl || undefined,
            quiet_hours_start: quietHoursStart || undefined,
            quiet_hours_end: quietHoursEnd || undefined,
            timezone,
            max_daily_emails: maxDailyEmails
        };
        updateM.mutate(data);
    };

    const timezones = [
        'America/New_York',
        'America/Chicago',
        'America/Denver',
        'America/Los_Angeles',
        'America/Anchorage',
        'Pacific/Honolulu',
        'Europe/London',
        'Europe/Paris',
        'Europe/Berlin',
        'Asia/Tokyo',
        'Asia/Shanghai',
        'Asia/Dubai',
        'Australia/Sydney',
        'UTC'
    ];

    if (preferencesQ.isLoading) {
        return (
            <Paper sx={{ p: 3, mb: 4 }}>
                <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                    <CircularProgress />
                </Box>
            </Paper>
        );
    }

    return (
        <>
            <Paper sx={{ p: 3, mb: 4 }}>
                <Typography variant="h6" gutterBottom>
                    Notification Preferences
                </Typography>
                <Typography variant="body2" color="text.secondary" sx={{ mb: 3 }}>
                    Configure how you receive alert notifications
                </Typography>

                <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3 }}>
                    {/* Notification Channels */}
                    <Box>
                        <Typography variant="subtitle2" gutterBottom>
                            Notification Channels
                        </Typography>
                        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1 }}>
                            <FormControlLabel
                                control={
                                    <Switch
                                        checked={emailEnabled}
                                        onChange={(e) => setEmailEnabled(e.target.checked)}
                                    />
                                }
                                label="Email Notifications"
                            />
                            <FormControlLabel
                                control={
                                    <Switch
                                        checked={inAppEnabled}
                                        onChange={(e) => setInAppEnabled(e.target.checked)}
                                    />
                                }
                                label="In-App Notifications"
                            />
                            <FormControlLabel
                                control={
                                    <Switch
                                        checked={webhookEnabled}
                                        onChange={(e) => setWebhookEnabled(e.target.checked)}
                                    />
                                }
                                label="Webhook Notifications"
                            />
                        </Box>
                    </Box>

                    {/* Webhook URL */}
                    {webhookEnabled && (
                        <TextField
                            label="Webhook URL"
                            value={webhookUrl}
                            onChange={(e) => setWebhookUrl(e.target.value)}
                            fullWidth
                            helperText="Enter the URL to receive webhook notifications"
                            placeholder="https://your-webhook-endpoint.com/alerts"
                        />
                    )}

                    {/* Quiet Hours */}
                    <Box>
                        <Typography variant="subtitle2" gutterBottom>
                            Quiet Hours
                        </Typography>
                        <Typography variant="caption" color="text.secondary" sx={{ mb: 1, display: 'block' }}>
                            No notifications will be sent during quiet hours (24-hour format)
                        </Typography>
                        <Box sx={{ display: 'flex', gap: 2 }}>
                            <TextField
                                label="Start Time"
                                type="time"
                                value={quietHoursStart}
                                onChange={(e) => setQuietHoursStart(e.target.value)}
                                InputLabelProps={{ shrink: true }}
                                sx={{ flex: 1 }}
                            />
                            <TextField
                                label="End Time"
                                type="time"
                                value={quietHoursEnd}
                                onChange={(e) => setQuietHoursEnd(e.target.value)}
                                InputLabelProps={{ shrink: true }}
                                sx={{ flex: 1 }}
                            />
                        </Box>
                    </Box>

                    {/* Timezone */}
                    <FormControl fullWidth>
                        <InputLabel>Timezone</InputLabel>
                        <Select
                            value={timezone}
                            label="Timezone"
                            onChange={(e) => setTimezone(e.target.value)}
                        >
                            {timezones.map((tz) => (
                                <MenuItem key={tz} value={tz}>
                                    {tz}
                                </MenuItem>
                            ))}
                        </Select>
                    </FormControl>

                    {/* Max Daily Emails */}
                    <TextField
                        label="Maximum Daily Emails"
                        type="number"
                        value={maxDailyEmails}
                        onChange={(e) => setMaxDailyEmails(Number(e.target.value))}
                        fullWidth
                        helperText="Maximum number of email notifications per day"
                        inputProps={{ min: 1, max: 100 }}
                    />

                    {/* Save Button */}
                    <Box>
                        <Button
                            variant="contained"
                            startIcon={<Save />}
                            onClick={handleSave}
                            disabled={updateM.isPending}
                        >
                            Save Preferences
                        </Button>
                    </Box>
                </Box>
            </Paper>

            <Snackbar
                open={snackbarOpen}
                autoHideDuration={6000}
                onClose={() => setSnackbarOpen(false)}
                anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}
            >
                <Alert onClose={() => setSnackbarOpen(false)} severity={snackbarSeverity}>
                    {snackbarMessage}
                </Alert>
            </Snackbar>
        </>
    );
}
