import { Card, CardContent, Typography, Box, IconButton, Chip } from '@mui/material';
import { Delete, MarkEmailRead, MarkEmailUnread } from '@mui/icons-material';
import { format } from 'date-fns';
import type { Notification } from '../types';
import AlertSeverityChip from './AlertSeverityChip';

interface NotificationCardProps {
    notification: Notification;
    onMarkRead: (id: string) => void;
    onMarkUnread: (id: string) => void;
    onDelete: (id: string) => void;
}

export default function NotificationCard({
    notification,
    onMarkRead,
    onMarkUnread,
    onDelete
}: NotificationCardProps) {
    const getSeverityFromType = (type: string): string => {
        if (type.includes('critical') || type.includes('error')) return 'critical';
        if (type.includes('warning') || type.includes('high')) return 'high';
        if (type.includes('medium')) return 'medium';
        return 'low';
    };

    const severity = getSeverityFromType(notification.notification_type);

    return (
        <Card
            sx={{
                mb: 2,
                border: notification.read ? '1px solid #e0e0e0' : '2px solid #2196f3',
                bgcolor: notification.read ? 'background.paper' : 'rgba(33, 150, 243, 0.05)'
            }}
        >
            <CardContent>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 1 }}>
                    <Box sx={{ flex: 1 }}>
                        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                            <Typography variant="h6" component="div">
                                {notification.title}
                            </Typography>
                            {!notification.read && (
                                <Chip
                                    label="New"
                                    size="small"
                                    color="primary"
                                    sx={{ height: 20 }}
                                />
                            )}
                            <AlertSeverityChip severity={severity} />
                        </Box>
                        <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
                            {notification.message}
                        </Typography>
                        <Typography variant="caption" color="text.secondary">
                            {format(new Date(notification.created_at), 'MMM dd, yyyy HH:mm')}
                        </Typography>
                    </Box>
                    <Box sx={{ display: 'flex', gap: 0.5 }}>
                        {notification.read ? (
                            <IconButton
                                size="small"
                                onClick={() => onMarkUnread(notification.id)}
                                title="Mark as unread"
                            >
                                <MarkEmailUnread fontSize="small" />
                            </IconButton>
                        ) : (
                            <IconButton
                                size="small"
                                onClick={() => onMarkRead(notification.id)}
                                title="Mark as read"
                            >
                                <MarkEmailRead fontSize="small" />
                            </IconButton>
                        )}
                        <IconButton
                            size="small"
                            color="error"
                            onClick={() => onDelete(notification.id)}
                            title="Delete notification"
                        >
                            <Delete fontSize="small" />
                        </IconButton>
                    </Box>
                </Box>
            </CardContent>
        </Card>
    );
}
