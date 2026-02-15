import { Box, CircularProgress, Skeleton, Typography } from '@mui/material';
import { Psychology } from '@mui/icons-material';

type AILoadingStateProps = {
    message?: string;
    variant?: 'spinner' | 'skeleton';
};

export default function AILoadingState({
    message = 'Generating insights...',
    variant = 'spinner'
}: AILoadingStateProps) {
    if (variant === 'skeleton') {
        return (
            <Box>
                <Skeleton variant="text" width="80%" height={40} />
                <Skeleton variant="text" width="60%" height={30} />
                <Skeleton variant="rectangular" width="100%" height={100} sx={{ mt: 2 }} />
            </Box>
        );
    }

    return (
        <Box
            sx={{
                display: 'flex',
                flexDirection: 'column',
                alignItems: 'center',
                justifyContent: 'center',
                py: 4,
                gap: 2,
            }}
        >
            <Box sx={{ position: 'relative', display: 'inline-flex' }}>
                <CircularProgress size={50} />
                <Box
                    sx={{
                        top: 0,
                        left: 0,
                        bottom: 0,
                        right: 0,
                        position: 'absolute',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                    }}
                >
                    <Psychology color="primary" />
                </Box>
            </Box>
            <Typography variant="body2" color="text.secondary">
                {message}
            </Typography>
        </Box>
    );
}
