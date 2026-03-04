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
    ToggleButtonGroup,
    ToggleButton,
} from '@mui/material';
import { Add, Edit, Delete, Person } from '@mui/icons-material';
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
    IncomeOwner,
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
    hasSpouse?: boolean;
    spouseName?: string | null;
}

export function Step2IncomeRetirement({ surveyId, data, onSave, isSaving, hasSpouse = false, spouseName }: Step2IncomeRetirementProps) {
    const [grossIncome, setGrossIncome] = useState<string>(data?.gross_annual_income?.toString() || '');
    const [payFrequency, setPayFrequency] = useState<PayFrequency | ''>(data?.pay_frequency || '');
    const [retirementRate, setRetirementRate] = useState<number>(data?.retirement_contribution_rate || 0);
    const [employerMatch, setEmployerMatch] = useState<string>(data?.employer_match_rate?.toString() || '');
    const [retirementAge, setRetirementAge] = useState<string>(data?.planned_retirement_age?.toString() || '65');
    const [desiredRetirementIncome, setDesiredRetirementIncome] = useState<string>(data?.desired_annual_retirement_income?.toString() || '');
    const [retirementIncomeNotes, setRetirementIncomeNotes] = useState(data?.retirement_income_needs_notes || '');
    const [notes, setNotes] = useState(data?.notes || '');
    // Spouse income fields
    const [effectiveTaxRate, setEffectiveTaxRate] = useState<string>(data?.effective_tax_rate?.toString() || '');
    const [investmentTaxRate, setInvestmentTaxRate] = useState<string>(data?.investment_income_tax_rate?.toString() || '');
    const [monthlyDeductions, setMonthlyDeductions] = useState<string>(data?.monthly_deductions?.toString() || '');
    const [spouseGrossIncome, setSpouseGrossIncome] = useState<string>(data?.spouse_gross_annual_income?.toString() || '');
    const [spousePayFrequency, setSpousePayFrequency] = useState<PayFrequency | ''>(data?.spouse_pay_frequency || '');
    const [spouseRetirementRate, setSpouseRetirementRate] = useState<number>(data?.spouse_retirement_contribution_rate || 0);
    const [spouseEmployerMatch, setSpouseEmployerMatch] = useState<string>(data?.spouse_employer_match_rate?.toString() || '');
    const [spouseEffectiveTaxRate, setSpouseEffectiveTaxRate] = useState<string>(data?.spouse_effective_tax_rate?.toString() || '');
    const [spouseInvestmentTaxRate, setSpouseInvestmentTaxRate] = useState<string>(data?.spouse_investment_income_tax_rate?.toString() || '');
    const [spouseMonthlyDeductions, setSpouseMonthlyDeductions] = useState<string>(data?.spouse_monthly_deductions?.toString() || '');
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
            setEffectiveTaxRate(data.effective_tax_rate?.toString() || '');
            setInvestmentTaxRate(data.investment_income_tax_rate?.toString() || '');
            setMonthlyDeductions(data.monthly_deductions?.toString() || '');
            setSpouseGrossIncome(data.spouse_gross_annual_income?.toString() || '');
            setSpousePayFrequency(data.spouse_pay_frequency || '');
            setSpouseRetirementRate(data.spouse_retirement_contribution_rate || 0);
            setSpouseEmployerMatch(data.spouse_employer_match_rate?.toString() || '');
            setSpouseEffectiveTaxRate(data.spouse_effective_tax_rate?.toString() || '');
            setSpouseInvestmentTaxRate(data.spouse_investment_income_tax_rate?.toString() || '');
            setSpouseMonthlyDeductions(data.spouse_monthly_deductions?.toString() || '');
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
        // Spouse income
        if (effectiveTaxRate) req.effective_tax_rate = parseFloat(effectiveTaxRate);
        if (investmentTaxRate) req.investment_income_tax_rate = parseFloat(investmentTaxRate);
        if (monthlyDeductions) req.monthly_deductions = parseFloat(monthlyDeductions);
        if (hasSpouse && spouseGrossIncome) req.spouse_gross_annual_income = parseFloat(spouseGrossIncome);
        if (hasSpouse && spousePayFrequency) req.spouse_pay_frequency = spousePayFrequency;
        if (hasSpouse) req.spouse_retirement_contribution_rate = spouseRetirementRate;
        if (hasSpouse && spouseEmployerMatch) req.spouse_employer_match_rate = parseFloat(spouseEmployerMatch);
        if (hasSpouse && spouseEffectiveTaxRate) req.spouse_effective_tax_rate = parseFloat(spouseEffectiveTaxRate);
        if (hasSpouse && spouseInvestmentTaxRate) req.spouse_investment_income_tax_rate = parseFloat(spouseInvestmentTaxRate);
        if (hasSpouse && spouseMonthlyDeductions) req.spouse_monthly_deductions = parseFloat(spouseMonthlyDeductions);
        onSave(req);
    }, [grossIncome, payFrequency, effectiveTaxRate, investmentTaxRate, monthlyDeductions, retirementRate, employerMatch, retirementAge, desiredRetirementIncome, retirementIncomeNotes, notes, hasSpouse, spouseGrossIncome, spousePayFrequency, spouseRetirementRate, spouseEmployerMatch, spouseEffectiveTaxRate, spouseInvestmentTaxRate, spouseMonthlyDeductions, onSave]);

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
                <Grid item xs={12} sm={6}>
                    <TextField
                        fullWidth
                        label="Salary Tax Rate (effective)"
                        type="number"
                        value={effectiveTaxRate}
                        onChange={(e) => setEffectiveTaxRate(e.target.value)}
                        InputProps={{
                            endAdornment: <InputAdornment position="end">%</InputAdornment>,
                        }}
                        inputProps={{ min: 0, max: 70, step: 0.5 }}
                        helperText="Effective (average) rate on employment income. Leave blank to skip."
                    />
                </Grid>
                <Grid item xs={12} sm={6}>
                    <TextField
                        fullWidth
                        label="Investment Income Tax Rate"
                        type="number"
                        value={investmentTaxRate}
                        onChange={(e) => setInvestmentTaxRate(e.target.value)}
                        InputProps={{
                            endAdornment: <InputAdornment position="end">%</InputAdornment>,
                        }}
                        inputProps={{ min: 0, max: 70, step: 0.5 }}
                        helperText="Rate applied to dividends & interest. Often lower than salary rate (e.g. ~18% in Canada for eligible dividends)."
                    />
                </Grid>
                <Grid item xs={12} sm={6}>
                    <TextField
                        fullWidth
                        label="Monthly Payroll Deductions"
                        type="number"
                        value={monthlyDeductions}
                        onChange={(e) => setMonthlyDeductions(e.target.value)}
                        InputProps={{
                            startAdornment: <InputAdornment position="start">$</InputAdornment>,
                            endAdornment: <InputAdornment position="end">/mo</InputAdornment>,
                        }}
                        inputProps={{ min: 0 }}
                        helperText="CPP, EI, benefit premiums, union dues etc. (e.g. CPP ~$322/mo + EI ~$87/mo in Canada)"
                    />
                </Grid>
            </Grid>

            {hasSpouse && (
                <>
                    <Divider sx={{ my: 3 }} />
                    <Box display="flex" alignItems="center" gap={1} mb={2}>
                        <Person color="secondary" />
                        <Typography variant="subtitle1" fontWeight="bold">
                            {spouseName ?? 'Spouse'} Income
                        </Typography>
                    </Box>
                    <Grid container spacing={3} mb={2}>
                        <Grid item xs={12} sm={6}>
                            <TextField
                                fullWidth
                                label="Gross Annual Income"
                                type="number"
                                value={spouseGrossIncome}
                                onChange={(e) => setSpouseGrossIncome(e.target.value)}
                                InputProps={{
                                    startAdornment: <InputAdornment position="start">$</InputAdornment>,
                                }}
                                helperText="Spouse total annual income before taxes"
                            />
                        </Grid>
                        <Grid item xs={12} sm={6}>
                            <FormControl fullWidth>
                                <InputLabel>Pay Frequency</InputLabel>
                                <Select
                                    value={spousePayFrequency}
                                    label="Pay Frequency"
                                    onChange={(e) => setSpousePayFrequency(e.target.value as PayFrequency)}
                                >
                                    {PAY_FREQUENCY_OPTIONS.map((opt) => (
                                        <MenuItem key={opt.value} value={opt.value}>
                                            {opt.label}
                                        </MenuItem>
                                    ))}
                                </Select>
                            </FormControl>
                        </Grid>
                        <Grid item xs={12} sm={6}>
                            <TextField
                                fullWidth
                                label="Salary Tax Rate (effective)"
                                type="number"
                                value={spouseEffectiveTaxRate}
                                onChange={(e) => setSpouseEffectiveTaxRate(e.target.value)}
                                InputProps={{
                                    endAdornment: <InputAdornment position="end">%</InputAdornment>,
                                }}
                                inputProps={{ min: 0, max: 70, step: 0.5 }}
                                helperText="Leave blank to skip"
                            />
                        </Grid>
                        <Grid item xs={12} sm={6}>
                            <TextField
                                fullWidth
                                label="Investment Income Tax Rate"
                                type="number"
                                value={spouseInvestmentTaxRate}
                                onChange={(e) => setSpouseInvestmentTaxRate(e.target.value)}
                                InputProps={{
                                    endAdornment: <InputAdornment position="end">%</InputAdornment>,
                                }}
                                inputProps={{ min: 0, max: 70, step: 0.5 }}
                                helperText="For dividends & interest. Leave blank to skip"
                            />
                        </Grid>
                        <Grid item xs={12} sm={6}>
                            <TextField
                                fullWidth
                                label="Monthly Payroll Deductions"
                                type="number"
                                value={spouseMonthlyDeductions}
                                onChange={(e) => setSpouseMonthlyDeductions(e.target.value)}
                                InputProps={{
                                    startAdornment: <InputAdornment position="start">$</InputAdornment>,
                                    endAdornment: <InputAdornment position="end">/mo</InputAdornment>,
                                }}
                                inputProps={{ min: 0 }}
                                helperText="CPP, EI, benefit premiums etc."
                            />
                        </Grid>
                    </Grid>
                </>
            )}

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
                                {hasSpouse && <TableCell>Who</TableCell>}
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
                                    {hasSpouse && (
                                        <TableCell>
                                            <Chip
                                                label={income.owner === 'spouse' ? (spouseName ?? 'Spouse') : 'Mine'}
                                                size="small"
                                                color={income.owner === 'spouse' ? 'secondary' : 'success'}
                                                icon={<Person />}
                                                variant="outlined"
                                            />
                                        </TableCell>
                                    )}
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
                                <TableCell colSpan={hasSpouse ? 3 : 2}>
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

                {/* Spouse retirement savings — only shown when spouse is configured */}
                {hasSpouse && (
                    <>
                        <Grid item xs={12}>
                            <Divider />
                            <Box display="flex" alignItems="center" gap={1} mt={2}>
                                <Person color="secondary" fontSize="small" />
                                <Typography variant="subtitle2" fontWeight="bold">
                                    {spouseName ?? 'Spouse'} Retirement Savings
                                </Typography>
                            </Box>
                        </Grid>
                        <Grid item xs={12}>
                            <Typography variant="subtitle2" gutterBottom>
                                Retirement Contribution Rate: {spouseRetirementRate}%
                            </Typography>
                            <Slider
                                value={spouseRetirementRate}
                                onChange={(_, v) => setSpouseRetirementRate(v as number)}
                                min={0}
                                max={50}
                                step={0.5}
                                marks={[
                                    { value: 0, label: '0%' },
                                    { value: 10, label: '10%' },
                                    { value: 20, label: '20%' },
                                    { value: 50, label: '50%' },
                                ]}
                                valueLabelDisplay="auto"
                                valueLabelFormat={(v) => `${v}%`}
                            />
                        </Grid>
                        <Grid item xs={12} sm={6}>
                            <TextField
                                fullWidth
                                label="Employer Match Rate"
                                type="number"
                                value={spouseEmployerMatch}
                                onChange={(e) => setSpouseEmployerMatch(e.target.value)}
                                InputProps={{
                                    endAdornment: <InputAdornment position="end">%</InputAdornment>,
                                }}
                                inputProps={{ min: 0, max: 100, step: 0.5 }}
                                helperText="Percentage of spouse's salary matched by their employer"
                            />
                        </Grid>
                    </>
                )}
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
                hasSpouse={hasSpouse}
                spouseName={spouseName}
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
    hasSpouse,
    spouseName,
    isSaving,
}: {
    open: boolean;
    onClose: () => void;
    onSubmit: (data: CreateAdditionalIncomeRequest) => void;
    editIncome: SurveyAdditionalIncome | null;
    hasSpouse: boolean;
    spouseName?: string | null;
    isSaving: boolean;
}) {
    const [incomeType, setIncomeType] = useState<AdditionalIncomeType>(editIncome?.income_type || 'dividends');
    const [description, setDescription] = useState(editIncome?.description || '');
    const [monthlyAmount, setMonthlyAmount] = useState(editIncome?.monthly_amount?.toString() || '');
    const [isRecurring, setIsRecurring] = useState(editIncome?.is_recurring ?? true);
    const [notes, setNotes] = useState(editIncome?.notes || '');
    const [owner, setOwner] = useState<IncomeOwner>(editIncome?.owner ?? 'mine');

    useEffect(() => {
        if (open) {
            setIncomeType(editIncome?.income_type || 'dividends');
            setDescription(editIncome?.description || '');
            setMonthlyAmount(editIncome?.monthly_amount?.toString() || '');
            setIsRecurring(editIncome?.is_recurring ?? true);
            setNotes(editIncome?.notes || '');
            setOwner(editIncome?.owner ?? 'mine');
        }
    }, [open, editIncome]);

    const handleSave = () => {
        onSubmit({
            income_type: incomeType,
            description: description || undefined,
            monthly_amount: parseFloat(monthlyAmount) || 0,
            is_recurring: isRecurring,
            notes: notes || undefined,
            owner,
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
                    {hasSpouse && (
                        <Box>
                            <Typography variant="body2" color="text.secondary" gutterBottom>
                                Whose income is this?
                            </Typography>
                            <ToggleButtonGroup
                                value={owner}
                                exclusive
                                onChange={(_, val) => val && setOwner(val)}
                                fullWidth
                                size="small"
                            >
                                <ToggleButton value="mine" color="success">
                                    <Person sx={{ mr: 0.5 }} fontSize="small" /> Mine
                                </ToggleButton>
                                <ToggleButton value="spouse" color="secondary">
                                    <Person sx={{ mr: 0.5 }} fontSize="small" /> {spouseName ?? 'Spouse'}
                                </ToggleButton>
                            </ToggleButtonGroup>
                        </Box>
                    )}
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
