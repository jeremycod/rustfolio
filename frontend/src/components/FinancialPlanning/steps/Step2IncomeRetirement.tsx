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
} from '@mui/material';
import type { SurveyIncomeInfo, UpdateIncomeInfoRequest, PayFrequency } from '../../../types';

const PAY_FREQUENCY_OPTIONS: { value: PayFrequency; label: string }[] = [
    { value: 'annual', label: 'Annual' },
    { value: 'monthly', label: 'Monthly' },
    { value: 'bi_weekly', label: 'Bi-Weekly' },
    { value: 'weekly', label: 'Weekly' },
];

interface Step2IncomeRetirementProps {
    data: SurveyIncomeInfo | null;
    onSave: (data: UpdateIncomeInfoRequest) => void;
    isSaving: boolean;
}

export function Step2IncomeRetirement({ data, onSave, isSaving }: Step2IncomeRetirementProps) {
    const [grossIncome, setGrossIncome] = useState<string>(data?.gross_annual_income?.toString() || '');
    const [payFrequency, setPayFrequency] = useState<PayFrequency | ''>(data?.pay_frequency || '');
    const [retirementRate, setRetirementRate] = useState<number>(data?.retirement_contribution_rate || 0);
    const [employerMatch, setEmployerMatch] = useState<string>(data?.employer_match_rate?.toString() || '');
    const [retirementAge, setRetirementAge] = useState<string>(data?.planned_retirement_age?.toString() || '65');
    const [notes, setNotes] = useState(data?.notes || '');

    useEffect(() => {
        if (data) {
            setGrossIncome(data.gross_annual_income?.toString() || '');
            setPayFrequency(data.pay_frequency || '');
            setRetirementRate(data.retirement_contribution_rate || 0);
            setEmployerMatch(data.employer_match_rate?.toString() || '');
            setRetirementAge(data.planned_retirement_age?.toString() || '65');
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
        if (notes) req.notes = notes;
        onSave(req);
    }, [grossIncome, payFrequency, retirementRate, employerMatch, retirementAge, notes, onSave]);

    useEffect(() => {
        const timer = setTimeout(saveData, 1000);
        return () => clearTimeout(timer);
    }, [saveData]);

    // Calculate estimated employer match contribution
    const income = parseFloat(grossIncome) || 0;
    const matchRate = parseFloat(employerMatch) || 0;
    const annualContribution = income * (retirementRate / 100);
    const annualMatch = income * (matchRate / 100);

    return (
        <Box>
            <Typography variant="h6" gutterBottom>
                Income & Retirement
            </Typography>
            <Typography variant="body2" color="text.secondary" mb={3}>
                Enter your income details and retirement savings information.
            </Typography>
            <Grid container spacing={3}>
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
                        helperText={income > 0 && matchRate > 0
                            ? `Employer contributes ~$${annualMatch.toLocaleString('en-US', { maximumFractionDigits: 0 })}/year`
                            : 'Percentage of salary matched by employer'}
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
                <Grid item xs={12}>
                    <TextField
                        fullWidth
                        label="Notes"
                        multiline
                        rows={2}
                        value={notes}
                        onChange={(e) => setNotes(e.target.value)}
                        helperText="Any additional income details (side income, bonuses, etc.)"
                    />
                </Grid>
            </Grid>
            {isSaving && (
                <Typography variant="caption" color="text.secondary" mt={2} display="block">
                    Saving...
                </Typography>
            )}
        </Box>
    );
}
