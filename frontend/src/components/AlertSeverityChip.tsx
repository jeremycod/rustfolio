import { Chip } from '@mui/material';
import {
    CheckCircle,
    Warning,
    Error,
    ErrorOutline
} from '@mui/icons-material';
import type { AlertSeverity } from '../types';

interface AlertSeverityChipProps {
    severity: string;
    size?: 'small' | 'medium';
}

const severityConfig: Record<AlertSeverity, { label: string; color: string; icon: JSX.Element }> = {
    low: {
        label: 'Low',
        color: '#4caf50',
        icon: <CheckCircle fontSize="small" />
    },
    medium: {
        label: 'Medium',
        color: '#ff9800',
        icon: <Warning fontSize="small" />
    },
    high: {
        label: 'High',
        color: '#ff5722',
        icon: <ErrorOutline fontSize="small" />
    },
    critical: {
        label: 'Critical',
        color: '#f44336',
        icon: <Error fontSize="small" />
    }
};

export default function AlertSeverityChip({ severity, size = 'small' }: AlertSeverityChipProps) {
    const config = severityConfig[severity as AlertSeverity];

    if (!config) {
        return (
            <Chip
                label={severity}
                size={size}
                sx={{ bgcolor: '#757575', color: 'white' }}
            />
        );
    }

    return (
        <Chip
            label={config.label}
            icon={config.icon}
            size={size}
            sx={{
                bgcolor: config.color,
                color: 'white',
                '& .MuiChip-icon': {
                    color: 'white'
                }
            }}
        />
    );
}
