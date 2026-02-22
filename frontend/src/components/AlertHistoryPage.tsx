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
    TextField,
    Select,
    MenuItem,
    FormControl,
    InputLabel,
    CircularProgress,
    Chip
} from '@mui/material';
import { Refresh } from '@mui/icons-material';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { format, subDays } from 'date-fns';
import { getAlertHistory } from '../lib/endpoints';
import AlertRuleTypeChip from './AlertRuleTypeChip';
import AlertSeverityChip from './AlertSeverityChip';
import type { AlertHistory, AlertSeverity, AlertRuleType } from '../types';

export default function AlertHistoryPage() {
    const queryClient = useQueryClient();
    const [dateFrom, setDateFrom] = useState(format(subDays(new Date(), 7), 'yyyy-MM-dd'));
    const [dateTo, setDateTo] = useState(format(new Date(), 'yyyy-MM-dd'));
    const [severityFilter, setSeverityFilter] = useState<string>('all');
    const [ruleTypeFilter, setRuleTypeFilter] = useState<string>('all');

    // Fetch alert history
    const alertHistoryQ = useQuery({
        queryKey: ['alertHistory'],
        queryFn: () => getAlertHistory()
    });

    // Filter history
    const history = alertHistoryQ.data || [];
    const filteredHistory = history.filter(alert => {
        const alertDate = new Date(alert.triggered_at);
        const from = new Date(dateFrom);
        const to = new Date(dateTo);
        to.setHours(23, 59, 59, 999); // End of day

        if (alertDate < from || alertDate > to) {
            return false;
        }

        if (severityFilter !== 'all' && alert.severity !== severityFilter) {
            return false;
        }

        if (ruleTypeFilter !== 'all' && alert.rule_type !== ruleTypeFilter) {
            return false;
        }

        return true;
    });

    const handleDateRangePreset = (days: number) => {
        setDateFrom(format(subDays(new Date(), days), 'yyyy-MM-dd'));
        setDateTo(format(new Date(), 'yyyy-MM-dd'));
    };

    return (
        <Box sx={{ p: 3 }}>
            <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
                <Typography variant="h4" component="h1">
                    Alert History
                </Typography>
                <Button
                    variant="outlined"
                    startIcon={<Refresh />}
                    onClick={() => queryClient.invalidateQueries({ queryKey: ['alertHistory'] })}
                    disabled={alertHistoryQ.isRefetching}
                >
                    Refresh
                </Button>
            </Box>

            {/* Filters */}
            <Paper sx={{ p: 2, mb: 3 }}>
                <Typography variant="subtitle2" gutterBottom>
                    Filters
                </Typography>
                <Box sx={{ display: 'flex', gap: 2, flexWrap: 'wrap', alignItems: 'flex-end' }}>
                    {/* Date Range */}
                    <TextField
                        label="From Date"
                        type="date"
                        value={dateFrom}
                        onChange={(e) => setDateFrom(e.target.value)}
                        InputLabelProps={{ shrink: true }}
                        size="small"
                    />
                    <TextField
                        label="To Date"
                        type="date"
                        value={dateTo}
                        onChange={(e) => setDateTo(e.target.value)}
                        InputLabelProps={{ shrink: true }}
                        size="small"
                    />

                    {/* Quick Presets */}
                    <Box sx={{ display: 'flex', gap: 1 }}>
                        <Button
                            size="small"
                            variant="outlined"
                            onClick={() => handleDateRangePreset(1)}
                        >
                            Today
                        </Button>
                        <Button
                            size="small"
                            variant="outlined"
                            onClick={() => handleDateRangePreset(7)}
                        >
                            Last 7 Days
                        </Button>
                        <Button
                            size="small"
                            variant="outlined"
                            onClick={() => handleDateRangePreset(30)}
                        >
                            Last 30 Days
                        </Button>
                    </Box>

                    {/* Severity Filter */}
                    <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>Severity</InputLabel>
                        <Select
                            value={severityFilter}
                            label="Severity"
                            onChange={(e) => setSeverityFilter(e.target.value)}
                        >
                            <MenuItem value="all">All Severities</MenuItem>
                            <MenuItem value="low">Low</MenuItem>
                            <MenuItem value="medium">Medium</MenuItem>
                            <MenuItem value="high">High</MenuItem>
                            <MenuItem value="critical">Critical</MenuItem>
                        </Select>
                    </FormControl>

                    {/* Rule Type Filter */}
                    <FormControl size="small" sx={{ minWidth: 150 }}>
                        <InputLabel>Rule Type</InputLabel>
                        <Select
                            value={ruleTypeFilter}
                            label="Rule Type"
                            onChange={(e) => setRuleTypeFilter(e.target.value)}
                        >
                            <MenuItem value="all">All Types</MenuItem>
                            <MenuItem value="price_change">Price Change</MenuItem>
                            <MenuItem value="volatility_spike">Volatility Spike</MenuItem>
                            <MenuItem value="drawdown_exceeded">Drawdown</MenuItem>
                            <MenuItem value="risk_threshold">Risk Threshold</MenuItem>
                            <MenuItem value="sentiment_change">Sentiment Change</MenuItem>
                            <MenuItem value="divergence">Divergence</MenuItem>
                        </Select>
                    </FormControl>

                    {/* Clear Filters */}
                    {(severityFilter !== 'all' || ruleTypeFilter !== 'all') && (
                        <Button
                            size="small"
                            onClick={() => {
                                setSeverityFilter('all');
                                setRuleTypeFilter('all');
                            }}
                        >
                            Clear Filters
                        </Button>
                    )}
                </Box>

                <Box sx={{ mt: 2 }}>
                    <Typography variant="body2" color="text.secondary">
                        Showing {filteredHistory.length} of {history.length} alerts
                    </Typography>
                </Box>
            </Paper>

            {alertHistoryQ.isLoading ? (
                <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
                    <CircularProgress />
                </Box>
            ) : filteredHistory.length === 0 ? (
                <Paper sx={{ p: 4, textAlign: 'center' }}>
                    <Typography variant="body1" color="text.secondary">
                        {history.length === 0
                            ? 'No alert history yet'
                            : 'No alerts match the current filters'}
                    </Typography>
                </Paper>
            ) : (
                <TableContainer component={Paper}>
                    <Table>
                        <TableHead>
                            <TableRow>
                                <TableCell>Triggered At</TableCell>
                                <TableCell>Rule Type</TableCell>
                                <TableCell>Scope</TableCell>
                                <TableCell>Severity</TableCell>
                                <TableCell>Message</TableCell>
                                <TableCell>Actual Value</TableCell>
                                <TableCell>Threshold</TableCell>
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            {filteredHistory.map((alert) => (
                                <TableRow key={alert.id}>
                                    <TableCell>
                                        <Typography variant="body2">
                                            {format(new Date(alert.triggered_at), 'MMM dd, yyyy')}
                                        </Typography>
                                        <Typography variant="caption" color="text.secondary">
                                            {format(new Date(alert.triggered_at), 'HH:mm:ss')}
                                        </Typography>
                                    </TableCell>
                                    <TableCell>
                                        <AlertRuleTypeChip ruleType={alert.rule_type} />
                                    </TableCell>
                                    <TableCell>
                                        {alert.portfolio_id ? (
                                            <Chip label="Portfolio" size="small" color="primary" />
                                        ) : (
                                            <Chip label={alert.ticker || 'N/A'} size="small" color="secondary" />
                                        )}
                                    </TableCell>
                                    <TableCell>
                                        <AlertSeverityChip severity={alert.severity} />
                                    </TableCell>
                                    <TableCell>
                                        <Typography variant="body2" sx={{ maxWidth: 300 }}>
                                            {alert.message}
                                        </Typography>
                                    </TableCell>
                                    <TableCell>
                                        <Typography variant="body2" fontWeight="bold">
                                            {alert.actual_value.toFixed(2)}
                                        </Typography>
                                    </TableCell>
                                    <TableCell>
                                        <Typography variant="body2">
                                            {alert.threshold.toFixed(2)}
                                        </Typography>
                                    </TableCell>
                                </TableRow>
                            ))}
                        </TableBody>
                    </Table>
                </TableContainer>
            )}
        </Box>
    );
}
