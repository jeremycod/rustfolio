import { useState, useEffect } from 'react';
import {
    Box,
    Typography,
    Button,
    Table,
    TableBody,
    TableCell,
    TableContainer,
    TableHead,
    TableRow,
    Paper,
    IconButton,
    Dialog,
    DialogTitle,
    DialogContent,
    DialogActions,
    TextField,
    FormControl,
    InputLabel,
    Select,
    MenuItem,
    InputAdornment,
    Chip,
    ToggleButtonGroup,
    ToggleButton,
    Slider,
} from '@mui/material';
import { Add, Edit, Delete, Person, People } from '@mui/icons-material';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import {
    createSurveyLiability,
    updateSurveyLiability,
    deleteSurveyLiability,
} from '../../../lib/endpoints';
import type { SurveyLiability, LiabilityType, Ownership, PaymentFrequency, CreateLiabilityRequest, UpdateLiabilityRequest } from '../../../types';

const LIABILITY_TYPE_OPTIONS: { value: LiabilityType; label: string }[] = [
    { value: 'mortgage', label: 'Mortgage' },
    { value: 'student_loan', label: 'Student Loan' },
    { value: 'auto_loan', label: 'Auto Loan' },
    { value: 'credit_card', label: 'Credit Card' },
    { value: 'other', label: 'Other' },
];

const PAYMENT_FREQUENCY_OPTIONS: { value: PaymentFrequency; label: string }[] = [
    { value: 'monthly', label: 'Monthly' },
    { value: 'bi_weekly', label: 'Bi-Weekly (Every 2 weeks)' },
    { value: 'weekly', label: 'Weekly' },
];

const LIABILITY_TYPE_COLORS: Record<string, 'error' | 'warning' | 'secondary' | 'info' | 'default'> = {
    mortgage: 'error',
    student_loan: 'warning',
    auto_loan: 'secondary',
    credit_card: 'info',
    other: 'default',
};

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

interface Step4LiabilitiesProps {
    surveyId: string;
    liabilities: SurveyLiability[];
    hasSpouse?: boolean;
    spouseName?: string | null;
}

export function Step4Liabilities({ surveyId, liabilities, hasSpouse = false, spouseName }: Step4LiabilitiesProps) {
    const [dialogOpen, setDialogOpen] = useState(false);
    const [editingLiability, setEditingLiability] = useState<SurveyLiability | null>(null);
    const queryClient = useQueryClient();

    const createMutation = useMutation({
        mutationFn: (data: CreateLiabilityRequest) => createSurveyLiability(surveyId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
        },
    });

    const updateMutation = useMutation({
        mutationFn: ({ liabilityId, data }: { liabilityId: string; data: UpdateLiabilityRequest }) =>
            updateSurveyLiability(surveyId, liabilityId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
            setEditingLiability(null);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: (liabilityId: string) => deleteSurveyLiability(surveyId, liabilityId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    const handleAdd = () => {
        setEditingLiability(null);
        setDialogOpen(true);
    };

    const handleEdit = (liability: SurveyLiability) => {
        setEditingLiability(liability);
        setDialogOpen(true);
    };

    const handleDelete = (liabilityId: string) => {
        deleteMutation.mutate(liabilityId);
    };

    const handleSubmit = (data: CreateLiabilityRequest) => {
        if (editingLiability) {
            updateMutation.mutate({ liabilityId: editingLiability.id, data });
        } else {
            createMutation.mutate(data);
        }
    };

    const totalLiabilities = liabilities.reduce((sum, l) => sum + l.balance, 0);
    const totalMonthlyPayments = liabilities.reduce((sum, l) => {
        const payment = l.monthly_payment || 0;
        // Convert to monthly equivalent based on frequency
        const monthlyEquivalent = l.payment_frequency === 'bi_weekly'
            ? payment * 26 / 12
            : l.payment_frequency === 'weekly'
            ? payment * 52 / 12
            : payment; // Default to monthly
        return sum + monthlyEquivalent;
    }, 0);

    return (
        <Box>
            <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                <Box>
                    <Typography variant="h6">Liabilities</Typography>
                    <Typography variant="body2" color="text.secondary">
                        Add all debts and obligations.
                    </Typography>
                </Box>
                <Button variant="contained" startIcon={<Add />} onClick={handleAdd}>
                    Add Liability
                </Button>
            </Box>

            {liabilities.length > 0 ? (
                <TableContainer component={Paper} variant="outlined">
                    <Table size="small">
                        <TableHead>
                            <TableRow>
                                <TableCell>Type</TableCell>
                                <TableCell>Description</TableCell>
                                {hasSpouse && <TableCell>Owner</TableCell>}
                                <TableCell align="right">Balance</TableCell>
                                <TableCell align="right">Rate</TableCell>
                                <TableCell align="right">Payment</TableCell>
                                <TableCell align="right">Actions</TableCell>
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            {liabilities.map((liability) => (
                                <TableRow key={liability.id}>
                                    <TableCell>
                                        <Chip
                                            label={LIABILITY_TYPE_OPTIONS.find(t => t.value === liability.liability_type)?.label || liability.liability_type}
                                            size="small"
                                            color={LIABILITY_TYPE_COLORS[liability.liability_type] || 'default'}
                                            variant="outlined"
                                        />
                                    </TableCell>
                                    <TableCell>{liability.description || '-'}</TableCell>
                                    {hasSpouse && (
                                        <TableCell>
                                            <Chip
                                                label={liability.ownership === 'joint'
                                                    ? `Joint (${liability.joint_split_percentage ?? 50}%)`
                                                    : liability.ownership === 'spouse'
                                                    ? (spouseName ?? 'Spouse')
                                                    : 'Mine'}
                                                size="small"
                                                color={liability.ownership === 'joint' ? 'warning' : liability.ownership === 'spouse' ? 'secondary' : 'success'}
                                                icon={liability.ownership === 'joint' ? <People /> : <Person />}
                                                variant="outlined"
                                            />
                                        </TableCell>
                                    )}
                                    <TableCell align="right">{formatCurrency(liability.balance)}</TableCell>
                                    <TableCell align="right">
                                        {liability.interest_rate != null ? `${liability.interest_rate}%` : '-'}
                                    </TableCell>
                                    <TableCell align="right">
                                        {liability.monthly_payment != null ? (
                                            <>
                                                {formatCurrency(liability.monthly_payment)}
                                                {liability.payment_frequency && (
                                                    <Typography variant="caption" display="block" color="text.secondary">
                                                        {liability.payment_frequency === 'bi_weekly' ? 'bi-weekly' : liability.payment_frequency}
                                                    </Typography>
                                                )}
                                            </>
                                        ) : '-'}
                                    </TableCell>
                                    <TableCell align="right">
                                        <IconButton size="small" onClick={() => handleEdit(liability)}>
                                            <Edit fontSize="small" />
                                        </IconButton>
                                        <IconButton
                                            size="small"
                                            color="error"
                                            onClick={() => handleDelete(liability.id)}
                                            disabled={deleteMutation.isPending}
                                        >
                                            <Delete fontSize="small" />
                                        </IconButton>
                                    </TableCell>
                                </TableRow>
                            ))}
                            <TableRow>
                                <TableCell colSpan={hasSpouse ? 3 : 2}>
                                    <Typography variant="subtitle2" fontWeight="bold">Total</Typography>
                                </TableCell>
                                <TableCell align="right">
                                    <Typography variant="subtitle2" fontWeight="bold" color="error.main">
                                        {formatCurrency(totalLiabilities)}
                                    </Typography>
                                </TableCell>
                                <TableCell />
                                <TableCell align="right">
                                    <Typography variant="subtitle2" fontWeight="bold">
                                        {formatCurrency(totalMonthlyPayments)}
                                    </Typography>
                                    <Typography variant="caption" display="block" color="text.secondary">
                                        monthly equiv.
                                    </Typography>
                                </TableCell>
                                <TableCell />
                            </TableRow>
                        </TableBody>
                    </Table>
                </TableContainer>
            ) : (
                <Paper variant="outlined" sx={{ p: 3, textAlign: 'center' }}>
                    <Typography variant="body2" color="text.secondary">
                        No liabilities added yet. Click "Add Liability" to get started.
                    </Typography>
                </Paper>
            )}

            <LiabilityFormDialog
                open={dialogOpen}
                onClose={() => { setDialogOpen(false); setEditingLiability(null); }}
                onSubmit={handleSubmit}
                editLiability={editingLiability}
                hasSpouse={hasSpouse}
                spouseName={spouseName}
                isSaving={createMutation.isPending || updateMutation.isPending}
            />
        </Box>
    );
}

function LiabilityFormDialog({
    open,
    onClose,
    onSubmit,
    editLiability,
    hasSpouse,
    spouseName,
    isSaving,
}: {
    open: boolean;
    onClose: () => void;
    onSubmit: (data: CreateLiabilityRequest) => void;
    editLiability: SurveyLiability | null;
    hasSpouse: boolean;
    spouseName?: string | null;
    isSaving: boolean;
}) {
    const [liabilityType, setLiabilityType] = useState<LiabilityType>(editLiability?.liability_type || 'mortgage');
    const [description, setDescription] = useState(editLiability?.description || '');
    const [balance, setBalance] = useState(editLiability?.balance?.toString() || '');
    const [interestRate, setInterestRate] = useState(editLiability?.interest_rate?.toString() || '');
    const [monthlyPayment, setMonthlyPayment] = useState(editLiability?.monthly_payment?.toString() || '');
    const [paymentFrequency, setPaymentFrequency] = useState<PaymentFrequency>(editLiability?.payment_frequency || 'monthly');
    const [notes, setNotes] = useState(editLiability?.notes || '');
    const [ownership, setOwnership] = useState<Ownership>(editLiability?.ownership ?? 'mine');
    const [splitPct, setSplitPct] = useState(editLiability?.joint_split_percentage ?? 50);

    // Update form when dialog opens with edit data
    useEffect(() => {
        if (open) {
            setLiabilityType(editLiability?.liability_type || 'mortgage');
            setDescription(editLiability?.description || '');
            setBalance(editLiability?.balance?.toString() || '');
            setInterestRate(editLiability?.interest_rate?.toString() || '');
            setMonthlyPayment(editLiability?.monthly_payment?.toString() || '');
            setPaymentFrequency(editLiability?.payment_frequency || 'monthly');
            setNotes(editLiability?.notes || '');
            setOwnership(editLiability?.ownership ?? 'mine');
            setSplitPct(editLiability?.joint_split_percentage ?? 50);
        }
    }, [open, editLiability]);

    const handleSave = () => {
        onSubmit({
            liability_type: liabilityType,
            description: description || undefined,
            balance: parseFloat(balance) || 0,
            interest_rate: interestRate ? parseFloat(interestRate) : undefined,
            monthly_payment: monthlyPayment ? parseFloat(monthlyPayment) : undefined,
            payment_frequency: monthlyPayment ? paymentFrequency : undefined,
            notes: notes || undefined,
            ownership: hasSpouse ? ownership : 'mine',
            joint_split_percentage: hasSpouse && ownership === 'joint' ? splitPct : undefined,
        });
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle>{editLiability ? 'Edit Liability' : 'Add Liability'}</DialogTitle>
            <DialogContent>
                <Box display="flex" flexDirection="column" gap={2} mt={1}>
                    <FormControl fullWidth>
                        <InputLabel>Liability Type</InputLabel>
                        <Select
                            value={liabilityType}
                            label="Liability Type"
                            onChange={(e) => setLiabilityType(e.target.value as LiabilityType)}
                        >
                            {LIABILITY_TYPE_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} value={opt.value}>{opt.label}</MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                    <TextField
                        fullWidth
                        label="Description"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        placeholder="e.g., Home Mortgage"
                    />
                    <TextField
                        fullWidth
                        label="Balance"
                        type="number"
                        value={balance}
                        onChange={(e) => setBalance(e.target.value)}
                        InputProps={{
                            startAdornment: <InputAdornment position="start">$</InputAdornment>,
                        }}
                        required
                    />
                    <TextField
                        fullWidth
                        label="Interest Rate"
                        type="number"
                        value={interestRate}
                        onChange={(e) => setInterestRate(e.target.value)}
                        InputProps={{
                            endAdornment: <InputAdornment position="end">%</InputAdornment>,
                        }}
                        inputProps={{ step: 0.1 }}
                    />
                    <TextField
                        fullWidth
                        label="Payment Amount"
                        type="number"
                        value={monthlyPayment}
                        onChange={(e) => setMonthlyPayment(e.target.value)}
                        InputProps={{
                            startAdornment: <InputAdornment position="start">$</InputAdornment>,
                        }}
                        helperText="Enter your payment amount (frequency selected below)"
                    />
                    <FormControl fullWidth>
                        <InputLabel>Payment Frequency</InputLabel>
                        <Select
                            value={paymentFrequency}
                            label="Payment Frequency"
                            onChange={(e) => setPaymentFrequency(e.target.value as PaymentFrequency)}
                        >
                            {PAYMENT_FREQUENCY_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} value={opt.value}>{opt.label}</MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                    {hasSpouse && (
                        <Box>
                            <Typography variant="body2" color="text.secondary" gutterBottom>
                                Ownership
                            </Typography>
                            <ToggleButtonGroup
                                value={ownership}
                                exclusive
                                onChange={(_, val) => val && setOwnership(val)}
                                fullWidth
                                size="small"
                            >
                                <ToggleButton value="mine" color="success">
                                    <Person sx={{ mr: 0.5 }} fontSize="small" /> Mine
                                </ToggleButton>
                                <ToggleButton value="spouse" color="secondary">
                                    <Person sx={{ mr: 0.5 }} fontSize="small" /> {spouseName ?? 'Spouse'}
                                </ToggleButton>
                                <ToggleButton value="joint" color="warning">
                                    <People sx={{ mr: 0.5 }} fontSize="small" /> Joint
                                </ToggleButton>
                            </ToggleButtonGroup>
                            {ownership === 'joint' && (
                                <Box mt={1.5}>
                                    <Typography variant="body2" gutterBottom>
                                        My share: <strong>{splitPct}%</strong> &nbsp;/&nbsp; {spouseName ?? 'Spouse'}: <strong>{100 - splitPct}%</strong>
                                    </Typography>
                                    <Slider
                                        value={splitPct}
                                        min={1}
                                        max={99}
                                        onChange={(_, val) => setSplitPct(val as number)}
                                        valueLabelDisplay="auto"
                                        valueLabelFormat={(v) => `${v}%`}
                                    />
                                </Box>
                            )}
                        </Box>
                    )}
                    <TextField
                        fullWidth
                        label="Notes"
                        multiline
                        rows={2}
                        value={notes}
                        onChange={(e) => setNotes(e.target.value)}
                    />
                </Box>
            </DialogContent>
            <DialogActions>
                <Button onClick={onClose}>Cancel</Button>
                <Button
                    variant="contained"
                    onClick={handleSave}
                    disabled={!balance || isSaving}
                >
                    {isSaving ? 'Saving...' : editLiability ? 'Update' : 'Add'}
                </Button>
            </DialogActions>
        </Dialog>
    );
}
