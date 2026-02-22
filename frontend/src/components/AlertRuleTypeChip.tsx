import { Chip } from '@mui/material';
import {
    TrendingDown,
    ShowChart,
    Warning,
    Shield,
    Psychology,
    Timeline
} from '@mui/icons-material';
import type { AlertRuleType } from '../types';

interface AlertRuleTypeChipProps {
    ruleType: string;
    size?: 'small' | 'medium';
}

const ruleTypeConfig: Record<AlertRuleType, { label: string; color: string; icon: JSX.Element }> = {
    price_change: {
        label: 'Price Change',
        color: '#2196f3',
        icon: <TrendingDown fontSize="small" />
    },
    volatility_spike: {
        label: 'Volatility Spike',
        color: '#ff9800',
        icon: <ShowChart fontSize="small" />
    },
    drawdown_exceeded: {
        label: 'Drawdown',
        color: '#f44336',
        icon: <Warning fontSize="small" />
    },
    risk_threshold: {
        label: 'Risk Threshold',
        color: '#9c27b0',
        icon: <Shield fontSize="small" />
    },
    sentiment_change: {
        label: 'Sentiment Change',
        color: '#00bcd4',
        icon: <Psychology fontSize="small" />
    },
    divergence: {
        label: 'Divergence',
        color: '#4caf50',
        icon: <Timeline fontSize="small" />
    }
};

export default function AlertRuleTypeChip({ ruleType, size = 'small' }: AlertRuleTypeChipProps) {
    const config = ruleTypeConfig[ruleType as AlertRuleType];

    if (!config) {
        return (
            <Chip
                label={ruleType}
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
