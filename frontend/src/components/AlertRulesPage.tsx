import { useState } from 'react';
import {
    Box,
    Typography,
    Paper,
    Button,
    Table,
    TableBody,
    TableCell,
    TableContainer,
    TableHead,
    TableRow,
    IconButton,
    Switch,
    Snackbar,
    Alert,
    CircularProgress,
    Chip
} from '@mui/material';
import { Add, Edit, Delete, PlayArrow, Refresh } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { format } from 'date-fns';
import {
    listAlertRules,
    deleteAlertRule,
    enableAlertRule,
    disableAlertRule,
    createAlertRule,
    updateAlertRule,
    testAlertRule,
    evaluateAllAlerts
} from '../lib/endpoints';
import AlertRuleTypeChip from './AlertRuleTypeChip';
import AlertRuleFormDialog from './AlertRuleFormDialog';
import AlertRuleTestDialog from './AlertRuleTestDialog';
import type { AlertRule, CreateAlertRuleRequest, UpdateAlertRuleRequest, TestAlertResponse } from '../types';

export default function AlertRulesPage() {
    const queryClient = useQueryClient();
    const [snackbarOpen, setSnackbarOpen] = useState(false);
    const [snackbarMessage, setSnackbarMessage] = useState('');
    const [snackbarSeverity, setSnackbarSeverity] = useState<'success' | 'error'>('success');
    const [filterTicker, setFilterTicker] = useState('');
    const [filterRuleType, setFilterRuleType] = useState('');
    const [filterEnabled, setFilterEnabled] = useState<'all' | 'enabled' | 'disabled'>('all');
    const [formDialogOpen, setFormDialogOpen] = useState(false);
    const [editingRule, setEditingRule] = useState<AlertRule | null>(null);
    const [testDialogOpen, setTestDialogOpen] = useState(false);
    const [testResult, setTestResult] = useState<TestAlertResponse | null>(null);
    const [testError, setTestError] = useState<string | null>(null);
    const [testLoading, setTestLoading] = useState(false);

    // Fetch alert rules with auto-refresh every 60 seconds
    const alertRulesQ = useQuery({
        queryKey: ['alertRules'],
        queryFn: listAlertRules,
        refetchInterval: 60000
    });

    // Delete alert rule
    const deleteM = useMutation({
        mutationFn: deleteAlertRule,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['alertRules'] });
            setSnackbarMessage('Alert rule deleted successfully');
            setSnackbarSeverity('success');
            setSnackbarOpen(true);
        },
        onError: (error: any) => {
            setSnackbarMessage(`Failed: ${error.response?.data || error.message}`);
            setSnackbarSeverity('error');
            setSnackbarOpen(true);
        }
    });

    // Enable alert rule
    const enableM = useMutation({
        mutationFn: enableAlertRule,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['alertRules'] });
            setSnackbarMessage('Alert rule enabled');
            setSnackbarSeverity('success');
            setSnackbarOpen(true);
        },
        onError: (error: any) => {
            setSnackbarMessage(`Failed: ${error.response?.data || error.message}`);
            setSnackbarSeverity('error');
            setSnackbarOpen(true);
        }
    });

    // Disable alert rule
    const disableM = useMutation({
        mutationFn: disableAlertRule,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['alertRules'] });
            setSnackbarMessage('Alert rule disabled');
            setSnackbarSeverity('success');
            setSnackbarOpen(true);
        },
        onError: (error: any) => {
            setSnackbarMessage(`Failed: ${error.response?.data || error.message}`);
            setSnackbarSeverity('error');
            setSnackbarOpen(true);
        }
    });

    // Create alert rule
    const createM = useMutation({
        mutationFn: createAlertRule,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['alertRules'] });
            setSnackbarMessage('Alert rule created successfully');
            setSnackbarSeverity('success');
            setSnackbarOpen(true);
            setFormDialogOpen(false);
        },
        onError: (error: any) => {
            setSnackbarMessage(`Failed: ${error.response?.data || error.message}`);
            setSnackbarSeverity('error');
            setSnackbarOpen(true);
        }
    });

    // Update alert rule
    const updateM = useMutation({
        mutationFn: ({ ruleId, data }: { ruleId: string; data: UpdateAlertRuleRequest }) =>
            updateAlertRule(ruleId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['alertRules'] });
            setSnackbarMessage('Alert rule updated successfully');
            setSnackbarSeverity('success');
            setSnackbarOpen(true);
            setFormDialogOpen(false);
            setEditingRule(null);
        },
        onError: (error: any) => {
            setSnackbarMessage(`Failed: ${error.response?.data || error.message}`);
            setSnackbarSeverity('error');
            setSnackbarOpen(true);
        }
    });

    const handleToggleEnabled = (ruleId: string, currentlyEnabled: boolean) => {
        if (currentlyEnabled) {
            disableM.mutate(ruleId);
        } else {
            enableM.mutate(ruleId);
        }
    };

    const handleDelete = (ruleId: string) => {
        if (confirm('Are you sure you want to delete this alert rule?')) {
            deleteM.mutate(ruleId);
        }
    };

    const handleFormSubmit = (data: CreateAlertRuleRequest | UpdateAlertRuleRequest) => {
        if (editingRule) {
            updateM.mutate({ ruleId: editingRule.id, data: data as UpdateAlertRuleRequest });
        } else {
            createM.mutate(data as CreateAlertRuleRequest);
        }
    };

    const handleEdit = (rule: AlertRule) => {
        setEditingRule(rule);
        setFormDialogOpen(true);
    };

    const handleCloseDialog = () => {
        setFormDialogOpen(false);
        setEditingRule(null);
    };

    const handleTestAlert = async (ruleId: string) => {
        setTestLoading(true);
        setTestError(null);
        setTestDialogOpen(true);

        try {
            const result = await testAlertRule(ruleId);
            setTestResult(result);
        } catch (error: any) {
            setTestError(error.response?.data || error.message || 'Failed to test alert');
        } finally {
            setTestLoading(false);
        }
    };

    const handleCloseTestDialog = () => {
        setTestDialogOpen(false);
        setTestResult(null);
        setTestError(null);
    };

    // Evaluate all alerts mutation
    const evaluateAllM = useMutation({
        mutationFn: evaluateAllAlerts,
        onSuccess: (data) => {
            queryClient.invalidateQueries({ queryKey: ['alertRules'] });
            queryClient.invalidateQueries({ queryKey: ['notifications'] });
            queryClient.invalidateQueries({ queryKey: ['notificationCount'] });
            setSnackbarMessage(
                `Evaluated ${data.evaluated_rules} rules. ${data.triggered_alerts} alerts triggered!`
            );
            setSnackbarSeverity('success');
            setSnackbarOpen(true);
        },
        onError: (error: any) => {
            setSnackbarMessage(`Failed: ${error.response?.data || error.message}`);
            setSnackbarSeverity('error');
            setSnackbarOpen(true);
        }
    });

    // Filter rules
    const rules = alertRulesQ.data || [];
    const filteredRules = rules.filter(rule => {
        if (filterTicker && rule.ticker && !rule.ticker.toLowerCase().includes(filterTicker.toLowerCase())) {
            return false;
        }
        if (filterRuleType && rule.rule_type !== filterRuleType) {
            return false;
        }
        if (filterEnabled === 'enabled' && !rule.enabled) {
            return false;
        }
        if (filterEnabled === 'disabled' && rule.enabled) {
            return false;
        }
        return true;
    });

    return (
        <Box sx={{ p: 3 }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
                <Typography variant="h4" component="h1">
                    Alert Rules
                </Typography>
                <Box sx={{ display: 'flex', gap: 2 }}>
                    <Button
                        variant="outlined"
                        startIcon={<Refresh />}
                        onClick={() => queryClient.invalidateQueries({ queryKey: ['alertRules'] })}
                        disabled={alertRulesQ.isRefetching}
                    >
                        Refresh
                    </Button>
                    <Button
                        variant="outlined"
                        color="secondary"
                        startIcon={<PlayArrow />}
                        onClick={() => evaluateAllM.mutate()}
                        disabled={evaluateAllM.isPending}
                    >
                        Evaluate All Alerts
                    </Button>
                    <Button
                        variant="contained"
                        startIcon={<Add />}
                        onClick={() => setFormDialogOpen(true)}
                    >
                        Create Alert
                    </Button>
                </Box>
            </Box>

            {/* Filters */}
            <Paper sx={{ p: 2, mb: 3 }}>
                <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap', alignItems: 'center' }}>
                    <Typography variant="subtitle2">Filters:</Typography>
                    <Box>
                        <input
                            type="text"
                            placeholder="Filter by ticker..."
                            value={filterTicker}
                            onChange={(e) => setFilterTicker(e.target.value)}
                            style={{
                                padding: '8px 12px',
                                border: '1px solid #ccc',
                                borderRadius: '4px',
                                fontSize: '14px'
                            }}
                        />
                    </Box>
                    <Box>
                        <select
                            value={filterRuleType}
                            onChange={(e) => setFilterRuleType(e.target.value)}
                            style={{
                                padding: '8px 12px',
                                border: '1px solid #ccc',
                                borderRadius: '4px',
                                fontSize: '14px'
                            }}
                        >
                            <option value="">All Rule Types</option>
                            <option value="price_change">Price Change</option>
                            <option value="volatility_spike">Volatility Spike</option>
                            <option value="drawdown_exceeded">Drawdown</option>
                            <option value="risk_threshold">Risk Threshold</option>
                            <option value="sentiment_change">Sentiment Change</option>
                            <option value="divergence">Divergence</option>
                        </select>
                    </Box>
                    <Box>
                        <select
                            value={filterEnabled}
                            onChange={(e) => setFilterEnabled(e.target.value as 'all' | 'enabled' | 'disabled')}
                            style={{
                                padding: '8px 12px',
                                border: '1px solid #ccc',
                                borderRadius: '4px',
                                fontSize: '14px'
                            }}
                        >
                            <option value="all">All Statuses</option>
                            <option value="enabled">Enabled Only</option>
                            <option value="disabled">Disabled Only</option>
                        </select>
                    </Box>
                    {(filterTicker || filterRuleType || filterEnabled !== 'all') && (
                        <Button
                            size="small"
                            onClick={() => {
                                setFilterTicker('');
                                setFilterRuleType('');
                                setFilterEnabled('all');
                            }}
                        >
                            Clear Filters
                        </Button>
                    )}
                </Box>
            </Paper>

            {alertRulesQ.isLoading ? (
                <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                    <CircularProgress />
                </Box>
            ) : filteredRules.length === 0 ? (
                <Paper sx={{ p: 4, textAlign: 'center' }}>
                    <Typography variant="body1" color="text.secondary">
                        {rules.length === 0 ? 'No alert rules yet. Create your first alert!' : 'No rules match the current filters'}
                    </Typography>
                </Paper>
            ) : (
                <TableContainer component={Paper}>
                    <Table>
                        <TableHead>
                            <TableRow>
                                <TableCell>Name</TableCell>
                                <TableCell>Rule Type</TableCell>
                                <TableCell>Scope</TableCell>
                                <TableCell>Threshold</TableCell>
                                <TableCell>Channels</TableCell>
                                <TableCell>Last Triggered</TableCell>
                                <TableCell>Enabled</TableCell>
                                <TableCell align="right">Actions</TableCell>
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            {filteredRules.map((rule) => (
                                <TableRow key={rule.id}>
                                    <TableCell>
                                        <Typography variant="body2" fontWeight="bold">
                                            {rule.name}
                                        </Typography>
                                        {rule.description && (
                                            <Typography variant="caption" color="text.secondary">
                                                {rule.description}
                                            </Typography>
                                        )}
                                    </TableCell>
                                    <TableCell>
                                        <AlertRuleTypeChip ruleType={rule.rule_type} />
                                    </TableCell>
                                    <TableCell>
                                        {rule.portfolio_id ? (
                                            <Chip label="Portfolio" size="small" color="primary" />
                                        ) : (
                                            <Chip label={rule.ticker || 'N/A'} size="small" color="secondary" />
                                        )}
                                    </TableCell>
                                    <TableCell>
                                        {rule.comparison.toUpperCase()} {rule.threshold}
                                    </TableCell>
                                    <TableCell>
                                        <Box sx={{ display: 'flex', gap: 0.5, flexWrap: 'wrap' }}>
                                            {rule.notification_channels.map(channel => (
                                                <Chip key={channel} label={channel} size="small" />
                                            ))}
                                        </Box>
                                    </TableCell>
                                    <TableCell>
                                        {rule.last_triggered_at ? (
                                            <Typography variant="caption">
                                                {format(new Date(rule.last_triggered_at), 'MMM dd, HH:mm')}
                                            </Typography>
                                        ) : (
                                            <Typography variant="caption" color="text.secondary">
                                                Never
                                            </Typography>
                                        )}
                                    </TableCell>
                                    <TableCell>
                                        <Switch
                                            checked={rule.enabled}
                                            onChange={() => handleToggleEnabled(rule.id, rule.enabled)}
                                            size="small"
                                        />
                                    </TableCell>
                                    <TableCell align="right">
                                        <IconButton
                                            size="small"
                                            onClick={() => handleTestAlert(rule.id)}
                                            title="Test alert"
                                        >
                                            <PlayArrow fontSize="small" />
                                        </IconButton>
                                        <IconButton
                                            size="small"
                                            onClick={() => handleEdit(rule)}
                                            title="Edit alert"
                                        >
                                            <Edit fontSize="small" />
                                        </IconButton>
                                        <IconButton
                                            size="small"
                                            color="error"
                                            onClick={() => handleDelete(rule.id)}
                                            title="Delete alert"
                                        >
                                            <Delete fontSize="small" />
                                        </IconButton>
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </TableContainer>
            )}

            <AlertRuleFormDialog
                open={formDialogOpen}
                onClose={handleCloseDialog}
                onSubmit={handleFormSubmit}
                editRule={editingRule}
            />

            <AlertRuleTestDialog
                open={testDialogOpen}
                onClose={handleCloseTestDialog}
                testResult={testResult}
                isLoading={testLoading}
                error={testError}
            />

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
