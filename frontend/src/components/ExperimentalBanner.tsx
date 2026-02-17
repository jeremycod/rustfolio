import { Alert, Box, Chip, Link, Typography } from '@mui/material';
import { Science } from '@mui/icons-material';

interface ExperimentalBannerProps {
    feature?: string;
}

export function ExperimentalBanner({ feature = "This feature" }: ExperimentalBannerProps) {
    return (
        <Alert
            severity="info"
            icon={<Science />}
            sx={{
                mb: 3,
                borderLeft: '4px solid #2196f3',
                backgroundColor: '#e3f2fd'
            }}
        >
            <Box>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 1 }}>
                    <Chip
                        label="EXPERIMENTAL"
                        size="small"
                        color="primary"
                        icon={<Science fontSize="small" />}
                    />
                    <Typography variant="subtitle2" fontWeight={600}>
                        {feature} is experimental
                    </Typography>
                </Box>

                <Typography variant="body2" color="text.secondary" sx={{ mb: 1 }}>
                    This sentiment analysis feature combines news sentiment with price momentum to provide
                    early warning indicators. Results should be used for educational purposes only.
                </Typography>

                <Typography variant="caption" color="text.secondary">
                    <strong>⚠️ Not Investment Advice:</strong> This tool is for research and learning purposes.
                    Always conduct your own analysis and consult with financial professionals before making investment decisions.
                    {' '}
                    <Link
                        href="#methodology"
                        underline="hover"
                        sx={{ fontWeight: 600 }}
                    >
                        Learn about methodology →
                    </Link>
                </Typography>
            </Box>
        </Alert>
    );
}
