import {
    Dialog,
    DialogTitle,
    DialogContent,
    DialogActions,
    Button,
    Typography,
    Box,
    Checkbox,
    FormControlLabel,
    Alert,
} from '@mui/material';
import { Psychology } from '@mui/icons-material';
import { useState } from 'react';

type ConsentDialogProps = {
    open: boolean;
    onClose: () => void;
    onAccept: () => void;
    onDecline: () => void;
};

export default function ConsentDialog({
    open,
    onClose,
    onAccept,
    onDecline,
}: ConsentDialogProps) {
    const [consentChecked, setConsentChecked] = useState(false);

    const handleAccept = () => {
        if (consentChecked) {
            onAccept();
        }
    };

    const handleDecline = () => {
        setConsentChecked(false);
        onDecline();
    };

    return (
        <Dialog
            open={open}
            onClose={onClose}
            maxWidth="sm"
            fullWidth
        >
            <DialogTitle>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                    <Psychology color="primary" />
                    Enable AI-Powered Features?
                </Box>
            </DialogTitle>
            <DialogContent>
                <Typography variant="body1" gutterBottom>
                    Rustfolio can use AI to provide intelligent insights about your portfolio.
                </Typography>

                <Box sx={{ my: 2 }}>
                    <Typography variant="subtitle2" gutterBottom fontWeight="bold">
                        What data is shared:
                    </Typography>
                    <Typography variant="body2" component="ul" sx={{ pl: 2 }}>
                        <li>Your portfolio performance metrics</li>
                        <li>Risk analysis data</li>
                        <li>Position information (anonymized)</li>
                    </Typography>
                </Box>

                <Box sx={{ my: 2 }}>
                    <Typography variant="subtitle2" gutterBottom fontWeight="bold">
                        Important notes:
                    </Typography>
                    <Typography variant="body2" component="ul" sx={{ pl: 2 }}>
                        <li>AI-generated insights are educational, not investment advice</li>
                        <li>Data is sent to OpenAI's API for processing</li>
                        <li>You can disable AI features at any time</li>
                        <li>Limited to 50 requests per hour</li>
                    </Typography>
                </Box>

                <Alert severity="info" sx={{ my: 2 }}>
                    AI features are currently experimental and may occasionally produce inaccurate results.
                </Alert>

                <FormControlLabel
                    control={
                        <Checkbox
                            checked={consentChecked}
                            onChange={(e) => setConsentChecked(e.target.checked)}
                        />
                    }
                    label="I understand and consent to using AI-powered features"
                />
            </DialogContent>
            <DialogActions>
                <Button onClick={handleDecline} color="inherit">
                    Decline
                </Button>
                <Button
                    onClick={handleAccept}
                    variant="contained"
                    disabled={!consentChecked}
                >
                    Accept & Enable
                </Button>
            </DialogActions>
        </Dialog>
    );
}
