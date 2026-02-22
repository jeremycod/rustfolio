import { useState } from 'react';
import {
    Box,
    Typography,
    Paper,
    Button,
    ToggleButtonGroup,
    ToggleButton,
    Snackbar,
    Alert,
    CircularProgress
} from '@mui/material';
import { MarkEmailRead, Refresh } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
    getNotifications,
    markNotificationRead,
    markAllNotificationsRead,
    deleteNotification
} from '../lib/endpoints';
import NotificationCard from './NotificationCard';
import type { Notification } from '../types';

type FilterType = 'all' | 'unread' | 'read';

export default function NotificationsPage() {
    const queryClient = useQueryClient();
    const [filter, setFilter] = useState<FilterType>('all');
    const [snackbarOpen, setSnackbarOpen] = useState(false);
    const [snackbarMessage, setSnackbarMessage] = useState('');
    const [snackbarSeverity, setSnackbarSeverity] = useState<'success' | 'error'>('success');

    // Fetch notifications with auto-refresh every 30 seconds
    const notificationsQ = useQuery({
        queryKey: ['notifications'],
        queryFn: () => getNotifications(),
        refetchInterval: 30000
    });

    // Mark single notification as read
    const markReadM = useMutation({
        mutationFn: markNotificationRead,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notifications'] });
            queryClient.invalidateQueries({ queryKey: ['notificationCount'] });
            setSnackbarMessage('Notification marked as read');
            setSnackbarSeverity('success');
            setSnackbarOpen(true);
        },
        onError: (error: any) => {
            setSnackbarMessage(`Failed: ${error.response?.data || error.message}`);
            setSnackbarSeverity('error');
            setSnackbarOpen(true);
        }
    });

    // Mark all as read
    const markAllReadM = useMutation({
        mutationFn: markAllNotificationsRead,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notifications'] });
            queryClient.invalidateQueries({ queryKey: ['notificationCount'] });
            setSnackbarMessage('All notifications marked as read');
            setSnackbarSeverity('success');
            setSnackbarOpen(true);
        },
        onError: (error: any) => {
            setSnackbarMessage(`Failed: ${error.response?.data || error.message}`);
            setSnackbarSeverity('error');
            setSnackbarOpen(true);
        }
    });

    // Delete notification
    const deleteM = useMutation({
        mutationFn: deleteNotification,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['notifications'] });
            queryClient.invalidateQueries({ queryKey: ['notificationCount'] });
            setSnackbarMessage('Notification deleted');
            setSnackbarSeverity('success');
            setSnackbarOpen(true);
        },
        onError: (error: any) => {
            setSnackbarMessage(`Failed: ${error.response?.data || error.message}`);
            setSnackbarSeverity('error');
            setSnackbarOpen(true);
        }
    });

    const handleMarkUnread = (id: string) => {
        // Backend doesn't have mark unread endpoint, so we'll just show a message
        setSnackbarMessage('Mark unread functionality not yet implemented in backend');
        setSnackbarSeverity('error');
        setSnackbarOpen(true);
    };

    const filterNotifications = (notifications: Notification[]): Notification[] => {
        switch (filter) {
            case 'unread':
                return notifications.filter(n => !n.read);
            case 'read':
                return notifications.filter(n => n.read);
            default:
                return notifications;
        }
    };

    const notifications = notificationsQ.data || [];
    const filteredNotifications = filterNotifications(notifications);
    const unreadCount = notifications.filter(n => !n.read).length;

    return (
        <Box sx={{ p: 3 }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
                <Typography variant="h4" component="h1">
                    Notifications
                    {unreadCount > 0 && (
                        <Typography component="span" variant="h6" color="primary" sx={{ ml: 2 }}>
                            ({unreadCount} unread)
                        </Typography>
                    )}
                </Typography>
                <Box sx={{ display: 'flex', gap: 2 }}>
                    <Button
                        variant="outlined"
                        startIcon={<Refresh />}
                        onClick={() => queryClient.invalidateQueries({ queryKey: ['notifications'] })}
                        disabled={notificationsQ.isRefetching}
                    >
                        Refresh
                    </Button>
                    <Button
                        variant="contained"
                        startIcon={<MarkEmailRead />}
                        onClick={() => markAllReadM.mutate()}
                        disabled={unreadCount === 0 || markAllReadM.isPending}
                    >
                        Mark All Read
                    </Button>
                </Box>
            </Box>

            <Paper sx={{ p: 2, mb: 3 }}>
                <ToggleButtonGroup
                    value={filter}
                    exclusive
                    onChange={(_, newFilter) => newFilter && setFilter(newFilter)}
                    size="small"
                >
                    <ToggleButton value="all">
                        All ({notifications.length})
                    </ToggleButton>
                    <ToggleButton value="unread">
                        Unread ({unreadCount})
                    </ToggleButton>
                    <ToggleButton value="read">
                        Read ({notifications.length - unreadCount})
                    </ToggleButton>
                </ToggleButtonGroup>
            </Paper>

            {notificationsQ.isLoading ? (
                <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                    <CircularProgress />
                </Box>
            ) : filteredNotifications.length === 0 ? (
                <Paper sx={{ p: 4, textAlign: 'center' }}>
                    <Typography variant="body1" color="text.secondary">
                        {filter === 'all' ? 'No notifications yet' : `No ${filter} notifications`}
                    </Typography>
                </Paper>
            ) : (
                <Box>
                    {filteredNotifications.map(notification => (
                        <NotificationCard
                            key={notification.id}
                            notification={notification}
                            onMarkRead={(id) => markReadM.mutate(id)}
                            onMarkUnread={handleMarkUnread}
                            onDelete={(id) => deleteM.mutate(id)}
                        />
                    ))}
                </Box>
            )}

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
        </Box>
    );
}
