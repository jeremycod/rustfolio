import { useState, useEffect, useCallback } from 'react';
import {
    Box,
    Typography,
    TextField,
    Grid,
    FormControl,
    InputLabel,
    Select,
    MenuItem,
    InputAdornment,
    Slider,
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
    Chip,
    Divider,
} from '@mui/material';
import { Add, Edit, Delete } from '@mui/icons-material';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
    createAdditionalIncome,
    getAdditionalIncomeList,
    updateAdditionalIncome,
    deleteAdditionalIncome,
} from '../../../lib/endpoints';
import type {
    SurveyIncomeInfo,
    UpdateIncomeInfoRequest,
    PayFrequency,
    SurveyAdditionalIncome,
    AdditionalIncomeType,
    CreateAdditionalIncomeRequest,
    UpdateAdditionalIncomeRequest,
} from '../../../types';

const PAY_FREQUENCY_OPTIONS: { value: PayFrequency; label: string }[] = [
    { value: 'annual', label: 'Annual' },
    { value: 'monthly', label: 'Monthly' },
    { value: 'bi_weekly', label: 'Bi-Weekly' },
    { value: 'weekly', label: 'Weekly' },
];

const INCOME_TYPE_OPTIONS: { value: AdditionalIncomeType; label: string }[] = [
    { value: 'dividends', label: 'Dividends' },
    { value: 'interest', label: 'Interest' },
    { value: 'rental_income', label: 'Rental Income' },
    { value: 'side_business', label: 'Side Business' },
    { value: 'pension', label: 'Pension' },
    { value: 'social_security', label: 'Social Security' },
    { value: 'disability', label: 'Disability' },
    { value: 'child_support', label: 'Child Support' },
    { value: 'alimony', label: 'Alimony' },
    { value: 'other', label: 'Other' },
];

const INCOME_TYPE_COLORS: Record<string, 'primary' | 'secondary' | 'success' | 'warning' | 'info' | 'default'> = {
    dividends: 'primary',
    interest: 'primary',
    rental_income: 'success',
    side_business: 'secondary',
    pension: 'warning',
    social_security: 'warning',
    disability: 'info',
    child_support: 'info',
    alimony: 'info',
    other: 'default',
};

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

interface Step2IncomeRetirementProps {
    surveyId: string;
    data: SurveyIncomeInfo | null;
    onSave: (data: UpdateIncomeInfoRequest) => void;
    isSaving: boolean;
}

export function Step2IncomeRetirement({ surveyId, data, onSave, isSaving }: Step2IncomeRetirementProps) {
    const [grossIncome, setGrossIncome] = useState<string>(data?.gross_annual_income?.toString() || '');
    const [payFrequency, setPayFrequency] = useState<PayFrequency | ''>(data?.pay_frequency || '');
    const [retirementRate, setRetirementRate] = useState<number>(data?.retirement_contribution_rate || 0);
    const [employerMatch, setEmployerMatch] = useState<string>(data?.employer_match_rate?.toString() || '');
    const [retirementAge, setRetirementAge] = useState<string>(data?.planned_retirement_age?.toString() || '65');
    const [desiredRetirementIncome, setDesiredRetirementIncome] = useState<string>(data?.desired_annual_retirement_income?.toString() || '');
    const [retirementIncomeNotes, setRetirementIncomeNotes] = useState(data?.retirement_income_needs_notes || '');
    const [notes, setNotes] = useState(data?.notes || '');
    const [dialogOpen, setDialogOpen] = useState(false);
    const [editingIncome, setEditingIncome] = useState<SurveyAdditionalIncome | null>(null);

    const queryClient = useQueryClient();

    // Fetch additional income sources
    const { data: additionalIncome = [] } = useQuery({
        queryKey: ['additional-income', surveyId],
        queryFn: () => getAdditionalIncomeList(surveyId),
        enabled: !!surveyId,
    });

    const createMutation = useMutation({
        mutationFn: (req: CreateAdditionalIncomeRequest) => createAdditionalIncome(surveyId, req),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['additional-income', surveyId] });
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
        },
    });

    const updateMutation = useMutation({
        mutationFn: ({ incomeId, req }: { incomeId: string; req: UpdateAdditionalIncomeRequest }) =>
            updateAdditionalIncome(surveyId, incomeId, req),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['additional-income', surveyId] });
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
            setEditingIncome(null);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: (incomeId: string) => deleteAdditionalIncome(surveyId, incomeId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['additional-income', surveyId] });
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    useEffect(() => {
        if (data) {
            setGrossIncome(data.gross_annual_income?.toString() || '');
            setPayFrequency(data.pay_frequency || '');
            setRetirementRate(data.retirement_contribution_rate || 0);
            setEmployerMatch(data.employer_match_rate?.toString() || '');
            setRetirementAge(data.planned_retirement_age?.toString() || '65');
            setDesiredRetirementIncome(data.desired_annual_retirement_income?.toString() || '');
            setRetirementIncomeNotes(data.retirement_income_needs_notes || '');
            setNotes(data.notes || '');
        }
    }, [data]);

    const saveData = useCallback(() => {
        const req: UpdateIncomeInfoRequest = {};
        if (grossIncome) req.gross_annual_income = parseFloat(grossIncome);
        if (payFrequency) req.pay_frequency = payFrequency;
        req.retirement_contribution_rate = retirementRate;
        if (employerMatch) req.employer_match_rate = parseFloat(employerMatch);
        if (retirementAge) req.planned_retirement_age = parseInt(retirementAge, 10);
        if (desiredRetirementIncome) req.desired_annual_retirement_income = parseFloat(desiredRetirementIncome);
        if (retirementIncomeNotes) req.retirement_income_needs_notes = retirementIncomeNotes;
        if (notes) req.notes = notes;
        onSave(req);
    }, [grossIncome, payFrequency, retirementRate, employerMatch, retirementAge, desiredRetirementIncome, retirementIncomeNotes, notes, onSave]);

    useEffect(() => {
        const timer = setTimeout(saveData, 1000);
        return () => clearTimeout(timer);
    }, [saveData]);

    const handleAddIncome = () => {
        setEditingIncome(null);
        setDialogOpen(true);
    };

    const handleEditIncome = (income: SurveyAdditionalIncome) => {
        setEditingIncome(income);
        setDialogOpen(true);
    };

    const handleDeleteIncome = (incomeId: string) => {
        deleteMutation.mutate(incomeId);
    };

    const handleSubmitIncome = (req: CreateAdditionalIncomeRequest) => {
        if (editingIncome) {
            updateMutation.mutate({ incomeId: editingIncome.id, req });
        } else {
            createMutation.mutate(req);
        }
    };

    // Calculate estimated employer match contribution
    const income = parseFloat(grossIncome) || 0;
    const matchRate = parseFloat(employerMatch) || 0;
    const annualContribution = income * (retirementRate / 100);
    const annualMatch = income * (matchRate / 100);

    // Calculate total additional income
    const totalAdditionalIncome = additionalIncome
        .filter((i) => i.is_recurring)
        .reduce((sum, i) => sum + i.monthly_amount, 0);

    return (
        <Box>
            <Typography variant="h6" gutterBottom>
                Income & Retirement
            </Typography>
            <Typography variant="body2" color="text.secondary" mb={3}>
                Enter your income details and retirement savings information.
            </Typography>

            {/* Employment Income Section */}
            <Typography variant="subtitle1" fontWeight="bold" mb={2}>
                Employment Income
            </Typography>
            <Grid container spacing={3} mb={4}>
                <Grid item xs={12} sm={6}>
                    <TextField
                        fullWidth
                        label="Gross Annual Income"
                        type="number"
                        value={grossIncome}
                        onChange={(e) => setGrossIncome(e.target.value)}
                        InputProps={{
                            startAdornment: <InputAdornment position="start">$</InputAdornment>,
                        }}
                        helperText="Total income before taxes and deductions"
                    />
                </Grid>
                <Grid item xs={12} sm={6}>
                    <FormControl fullWidth>
                        <InputLabel>Pay Frequency</InputLabel>
                        <Select
                            value={payFrequency}
                            label="Pay Frequency"
                            onChange={(e) => setPayFrequency(e.target.value as PayFrequency)}
                        >
                            {PAY_FREQUENCY_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} value={opt.value}>
                                    {opt.label}
                                </MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                </Grid>
            </Grid>

            <Divider sx={{ my: 3 }} />

            {/* Additional Income Section */}
            <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                <Box>
                    <Typography variant="subtitle1" fontWeight="bold">
                        Additional Income Sources
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                        Add other income sources like dividends, rental income, side business, etc.
                    </Typography>
                </Box>
                <Button variant="outlined" startIcon={<Add />} onClick={handleAddIncome} size="small">
                    Add Income
                </Button>
            </Box>

            {additionalIncome.length > 0 ? (
                <TableContainer component={Paper} variant="outlined" sx={{ mb: 3 }}>
                    <Table size="small">
                        <TableHead>
                            <TableRow>
                                <TableCell>Type</TableCell>
                                <TableCell>Description</TableCell>
                                <TableCell align="right">Monthly Amount</TableCell>
                                <TableCell align="right">Actions</TableCell>
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            {additionalIncome.map((income) => (
                                <TableRow key={income.id}>
                                    <TableCell>
                                        <Chip
                                            label={
                                                INCOME_TYPE_OPTIONS.find((t) => t.value === income.income_type)?.label ||
                                                income.income_type
                                            }
                                            size="small"
                                            color={INCOME_TYPE_COLORS[income.income_type] || 'default'}
                                            variant="outlined"
                                        />
                                    </TableCell>
                                    <TableCell>{income.description || '-'}</TableCell>
                                    <TableCell align="right">
                                        {formatCurrency(income.monthly_amount)}
                                        {!income.is_recurring && (
                                            <Typography variant="caption" display="block" color="text.secondary">
                                                one-time
                                            </Typography>
                                        )}
                                    </TableCell>
                                    <TableCell align="right">
                                        <IconButton size="small" onClick={() => handleEditIncome(income)}>
                                            <Edit fontSize="small" />
                                        </IconButton>
                                        <IconButton
                                            size="small"
                                            color="error"
                                            onClick={() => handleDeleteIncome(income.id)}
                                            disabled={deleteMutation.isPending}
                                        >
                                            <Delete fontSize="small" />
                                        </IconButton>
                                    </TableCell>
                                </TableRow>
                            ))}
                            <TableRow>
                                <TableCell colSpan={2}>
                                    <Typography variant="subtitle2" fontWeight="bold">
                                        Total Monthly Additional Income
                                    </Typography>
                                </TableCell>
                                <TableCell align="right">
                                    <Typography variant="subtitle2" fontWeight="bold" color="success.main">
                                        {formatCurrency(totalAdditionalIncome)}
                                    </Typography>
                                </TableCell>
                                <TableCell />
                            </TableRow>
                        </TableBody>
                    </Table>
                </TableContainer>
            ) : (
                <Paper variant="outlined" sx={{ p: 2, textAlign: 'center', mb: 3 }}>
                    <Typography variant="body2" color="text.secondary">
                        No additional income sources added yet.
                    </Typography>
                </Paper>
            )}

            <Divider sx={{ my: 3 }} />

            {/* Retirement Savings Section */}
            <Typography variant="subtitle1" fontWeight="bold" mb={2}>
                Retirement Savings
            </Typography>
            <Grid container spacing={3}>
                <Grid item xs={12}>
                    <Typography variant="subtitle2" gutterBottom>
                        Retirement Contribution Rate: {retirementRate}%
                    </Typography>
                    <Slider
                        value={retirementRate}
                        onChange={(_, v) => setRetirementRate(v as number)}
                        min={0}
                        max={50}
                        step={0.5}
                        marks={[
                            { value: 0, label: '0%' },
                            { value: 10, label: '10%' },
                            { value: 20, label: '20%' },
                            { value: 30, label: '30%' },
                            { value: 50, label: '50%' },
                        ]}
                        valueLabelDisplay="auto"
                        valueLabelFormat={(v) => `${v}%`}
                    />
                    {income > 0 && (
                        <Typography variant="caption" color="text.secondary">
                            Annual contribution: ${annualContribution.toLocaleString('en-US', { maximumFractionDigits: 0 })}
                        </Typography>
                    )}
                </Grid>
                <Grid item xs={12} sm={6}>
                    <TextField
                        fullWidth
                        label="Employer Match Rate"
                        type="number"
                        value={employerMatch}
                        onChange={(e) => setEmployerMatch(e.target.value)}
                        InputProps={{
                            endAdornment: <InputAdornment position="end">%</InputAdornment>,
                        }}
                        inputProps={{ min: 0, max: 100, step: 0.5 }}
                        helperText={
                            income > 0 && matchRate > 0
                                ? `Employer contributes ~$${annualMatch.toLocaleString('en-US', { maximumFractionDigits: 0 })}/year`
                                : 'Percentage of salary matched by employer'
                        }
                    />
                </Grid>
                <Grid item xs={12} sm={6}>
                    <TextField
                        fullWidth
                        label="Planned Retirement Age"
                        type="number"
                        value={retirementAge}
                        onChange={(e) => setRetirementAge(e.target.value)}
                        inputProps={{ min: 40, max: 80 }}
                        helperText="Used for retirement projections"
                    />
                </Grid>
                <Grid item xs={12} sm={6}>
                    <TextField
                        fullWidth
                        label="Desired Annual Retirement Income"
                        type="number"
                        value={desiredRetirementIncome}
                        onChange={(e) => setDesiredRetirementIncome(e.target.value)}
                        InputProps={{
                            startAdornment: <InputAdornment position="start">$</InputAdornment>,
                        }}
                        helperText={
                            desiredRetirementIncome
                                ? `Monthly: ${formatCurrency(parseFloat(desiredRetirementIncome) / 12)}`
                                : 'How much would you like to live on per year in retirement?'
                        }
                    />
                </Grid>
                <Grid item xs={12}>
                    <TextField
                        fullWidth
                        label="Retirement Income Needs"
                        multiline
                        rows={2}
                        value={retirementIncomeNotes}
                        onChange={(e) => setRetirementIncomeNotes(e.target.value)}
                        helperText="Describe your retirement lifestyle goals and income needs"
                    />
                </Grid>
                <Grid item xs={12}>
                    <TextField
                        fullWidth
                        label="Notes"
                        multiline
                        rows={2}
                        value={notes}
                        onChange={(e) => setNotes(e.target.value)}
                        helperText="Any additional notes about your income or retirement plans"
                    />
                </Grid>
            </Grid>
            {isSaving && (
                <Typography variant="caption" color="text.secondary" mt={2} display="block">
                    Saving...
                </Typography>
            )}

            <AdditionalIncomeFormDialog
                open={dialogOpen}
                onClose={() => {
                    setDialogOpen(false);
                    setEditingIncome(null);
                }}
                onSubmit={handleSubmitIncome}
                editIncome={editingIncome}
                isSaving={createMutation.isPending || updateMutation.isPending}
            />
        </Box>
    );
}

function AdditionalIncomeFormDialog({
    open,
    onClose,
    onSubmit,
    editIncome,
    isSaving,
}: {
    open: boolean;
    onClose: () => void;
    onSubmit: (data: CreateAdditionalIncomeRequest) => void;
    editIncome: SurveyAdditionalIncome | null;
    isSaving: boolean;
}) {
    const [incomeType, setIncomeType] = useState<AdditionalIncomeType>(editIncome?.income_type || 'dividends');
    const [description, setDescription] = useState(editIncome?.description || '');
    const [monthlyAmount, setMonthlyAmount] = useState(editIncome?.monthly_amount?.toString() || '');
    const [isRecurring, setIsRecurring] = useState(editIncome?.is_recurring ?? true);
    const [notes, setNotes] = useState(editIncome?.notes || '');

    useEffect(() => {
        if (open) {
            setIncomeType(editIncome?.income_type || 'dividends');
            setDescription(editIncome?.description || '');
            setMonthlyAmount(editIncome?.monthly_amount?.toString() || '');
            setIsRecurring(editIncome?.is_recurring ?? true);
            setNotes(editIncome?.notes || '');
        }
    }, [open, editIncome]);

    const handleSave = () => {
        onSubmit({
            income_type: incomeType,
            description: description || undefined,
            monthly_amount: parseFloat(monthlyAmount) || 0,
            is_recurring: isRecurring,
            notes: notes || undefined,
        });
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle>{editIncome ? 'Edit Additional Income' : 'Add Additional Income'}</DialogTitle>
            <DialogContent>
                <Box display="flex" flexDirection="column" gap={2} mt={1}>
                    <FormControl fullWidth>
                        <InputLabel>Income Type</InputLabel>
                        <Select
                            value={incomeType}
                            label="Income Type"
                            onChange={(e) => setIncomeType(e.target.value as AdditionalIncomeType)}
                        >
                            {INCOME_TYPE_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} value={opt.value}>
                                    {opt.label}
                                </MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                    <TextField
                        fullWidth
                        label="Description"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        placeholder="e.g., Tesla dividends, Rental property on Main St"
                    />
                    <TextField
                        fullWidth
                        label="Monthly Amount"
                        type="number"
                        value={monthlyAmount}
                        onChange={(e) => setMonthlyAmount(e.target.value)}
                        InputProps={{
                            startAdornment: <InputAdornment position="start">$</InputAdornment>,
                        }}
                        required
                        helperText="Enter as monthly amount"
                    />
                    <FormControl fullWidth>
                        <InputLabel>Frequency</InputLabel>
                        <Select
                            value={isRecurring ? 'recurring' : 'one-time'}
                            label="Frequency"
                            onChange={(e) => setIsRecurring(e.target.value === 'recurring')}
                        >
                            <MenuItem value="recurring">Recurring (monthly)</MenuItem>
                            <MenuItem value="one-time">One-time</MenuItem>
                        </Select>
                    </FormControl>
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
                <Button variant="contained" onClick={handleSave} disabled={!monthlyAmount || isSaving}>
                    {isSaving ? 'Saving...' : editIncome ? 'Update' : 'Add'}
                </Button>
            </DialogActions>
        </Dialog>
    );
}
