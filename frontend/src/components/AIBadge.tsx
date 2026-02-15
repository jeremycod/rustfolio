import { Chip } from '@mui/material';
import { Psychology } from '@mui/icons-material';

type AIBadgeProps = {
    variant?: 'experimental' | 'ai-generated';
    size?: 'small' | 'medium';
};

export default function AIBadge({ variant = 'ai-generated', size = 'small' }: AIBadgeProps) {
    const label = variant === 'experimental' ? 'Experimental' : 'AI Generated';
    const color = variant === 'experimental' ? 'warning' : 'info';

    return (
        <Chip
            icon={<Psychology />}
            label={label}
            color={color}
            size={size}
            sx={{ fontWeight: 500 }}
        />
    );
}
