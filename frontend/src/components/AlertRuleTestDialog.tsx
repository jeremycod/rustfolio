import {
    Dialog,
    DialogTitle,
    DialogContent,
    DialogActions,
    Button,
    Box,
    Typography,
    CircularProgress,
    Alert,
    Chip,
    Divider
} from '@mui/material';
import { CheckCircle, Cancel } from '@mui/icons-material';
import type { TestAlertResponse } from '../types';
import AlertSeverityChip from './AlertSeverityChip';

interface AlertRuleTestDialogProps {
    open: boolean;
    onClose: () => void;
    testResult: TestAlertResponse | null;
    isLoading: boolean;
    error: string | null;
}

export default function AlertRuleTestDialog({
    open,
    onClose,
    testResult,
    isLoading,
    error
}: AlertRuleTestDialogProps) {
    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle>Alert Rule Test Results</DialogTitle>
            <DialogContent>
                {isLoading ? (
                    <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                        <CircularProgress />
                    </Box>
                ) : error ? (
                    <Alert severity="error" sx={{ mb: 2 }}>
                        {error}
                    </Alert>
                ) : testResult ? (
                    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, pt: 1 }}>
                        {/* Rule Info */}
                        <Box>
                            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                                Rule Name
                            </Typography>
                            <Typography variant="body1" fontWeight="bold">
                                {testResult.rule.name}
                            </Typography>
                        </Box>

                        <Divider />

                        {/* Would Trigger */}
                        <Box>
                            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                                Would Trigger Alert?
                            </Typography>
                            <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                                {testResult.would_trigger ? (
                                    <>
                                        <CheckCircle color="success" />
                                        <Chip label="Yes" color="success" />
                                    </>
                                ) : (
                                    <>
                                        <Cancel color="error" />
                                        <Chip label="No" color="default" />
                                    </>
                                )}
                            </Box>
                        </Box>

                        {testResult.evaluation && (
                            <>
                                <Divider />

                                {/* Evaluation Details */}
                                <Box>
                                    <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                                        Evaluation Details
                                    </Typography>

                                    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 1, mt: 1 }}>
                                        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                            <Typography variant="body2">Actual Value:</Typography>
                                            <Typography variant="body1" fontWeight="bold">
                                                {testResult.evaluation.actual_value.toFixed(2)}
                                            </Typography>
                                        </Box>

                                        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                            <Typography variant="body2">Threshold:</Typography>
                                            <Typography variant="body1" fontWeight="bold">
                                                {testResult.rule.comparison.toUpperCase()} {testResult.evaluation.threshold.toFixed(2)}
                                            </Typography>
                                        </Box>

                                        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                            <Typography variant="body2">Severity:</Typography>
                                            <AlertSeverityChip severity={testResult.evaluation.severity} />
                                        </Box>

                                        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                                            <Typography variant="body2">Triggered:</Typography>
                                            <Chip
                                                label={testResult.evaluation.triggered ? 'Yes' : 'No'}
                                                color={testResult.evaluation.triggered ? 'success' : 'default'}
                                                size="small"
                                            />
                                        </Box>
                                    </Box>
                                </Box>

                                <Divider />

                                {/* Message */}
                                <Box>
                                    <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                                        Message
                                    </Typography>
                                    <Typography variant="body2">
                                        {testResult.evaluation.message}
                                    </Typography>
                                </Box>

                                {/* Metadata */}
                                {testResult.evaluation.metadata && Object.keys(testResult.evaluation.metadata).length > 0 && (
                                    <>
                                        <Divider />
                                        <Box>
                                            <Typography variant="subtitle2" color="text.secondary" gutterBottom>
                                                Additional Details
                                            </Typography>
                                            <Box sx={{ bgcolor: 'grey.100', p: 1, borderRadius: 1 }}>
                                                <pre style={{ margin: 0, fontSize: '12px', overflow: 'auto' }}>
                                                    {JSON.stringify(testResult.evaluation.metadata, null, 2)}
                                                </pre>
                                            </Box>
                                        </Box>
                                    </>
                                )}
                            </>
                        )}
                    </Box>
                ) : (
                    <Alert severity="info">
                        No test results available
                    </Alert>
                )}
            </DialogContent>
            <DialogActions>
                <Button onClick={onClose}>Close</Button>
            </DialogActions>
        </Dialog>
    );
}
