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
} from '@mui/material';
import type { SurveyPersonalInfo, UpdatePersonalInfoRequest, MaritalStatus, EmploymentStatus } from '../../../types';

const MARITAL_OPTIONS: { value: MaritalStatus; label: string }[] = [
    { value: 'single', label: 'Single' },
    { value: 'married', label: 'Married' },
    { value: 'divorced', label: 'Divorced' },
    { value: 'widowed', label: 'Widowed' },
    { value: 'other', label: 'Other' },
];

const EMPLOYMENT_OPTIONS: { value: EmploymentStatus; label: string }[] = [
    { value: 'employed', label: 'Employed' },
    { value: 'self_employed', label: 'Self-Employed' },
    { value: 'retired', label: 'Retired' },
    { value: 'unemployed', label: 'Unemployed' },
    { value: 'student', label: 'Student' },
    { value: 'other', label: 'Other' },
];

interface Step1PersonalDetailsProps {
    data: SurveyPersonalInfo | null;
    onSave: (data: UpdatePersonalInfoRequest) => void;
    isSaving: boolean;
}

export function Step1PersonalDetails({ data, onSave, isSaving }: Step1PersonalDetailsProps) {
    const [fullName, setFullName] = useState(data?.full_name || '');
    const [birthYear, setBirthYear] = useState<string>(data?.birth_year?.toString() || '');
    const [maritalStatus, setMaritalStatus] = useState<MaritalStatus | ''>(data?.marital_status || '');
    const [employmentStatus, setEmploymentStatus] = useState<EmploymentStatus | ''>(data?.employment_status || '');
    const [dependents, setDependents] = useState<string>(data?.dependents?.toString() || '0');
    const [contactEmail, setContactEmail] = useState(data?.contact_email || '');

    // Update local state when data changes (initial load)
    useEffect(() => {
        if (data) {
            setFullName(data.full_name || '');
            setBirthYear(data.birth_year?.toString() || '');
            setMaritalStatus(data.marital_status || '');
            setEmploymentStatus(data.employment_status || '');
            setDependents(data.dependents?.toString() || '0');
            setContactEmail(data.contact_email || '');
        }
    }, [data]);

    // Debounced auto-save
    const saveData = useCallback(() => {
        const req: UpdatePersonalInfoRequest = {};
        if (fullName) req.full_name = fullName;
        if (birthYear) req.birth_year = parseInt(birthYear, 10);
        if (maritalStatus) req.marital_status = maritalStatus;
        if (employmentStatus) req.employment_status = employmentStatus;
        req.dependents = parseInt(dependents, 10) || 0;
        if (contactEmail) req.contact_email = contactEmail;
        onSave(req);
    }, [fullName, birthYear, maritalStatus, employmentStatus, dependents, contactEmail, onSave]);

    useEffect(() => {
        const timer = setTimeout(saveData, 1000);
        return () => clearTimeout(timer);
    }, [saveData]);

    const currentYear = new Date().getFullYear();

    return (
        <Box>
            <Typography variant="h6" gutterBottom>
                Personal Details
            </Typography>
            <Typography variant="body2" color="text.secondary" mb={3}>
                Provide your basic information to personalize your financial plan.
                All fields are optional.
            </Typography>
            <Grid container spacing={3}>
                <Grid item xs={12} sm={6}>
                    <TextField
                        fullWidth
                        label="Full Name"
                        value={fullName}
                        onChange={(e) => setFullName(e.target.value)}
                        helperText="Used to personalize your financial snapshot"
                    />
                </Grid>
                <Grid item xs={12} sm={6}>
                    <TextField
                        fullWidth
                        label="Birth Year"
                        type="number"
                        value={birthYear}
                        onChange={(e) => setBirthYear(e.target.value)}
                        inputProps={{ min: 1920, max: currentYear }}
                        helperText="Used for retirement projections"
                    />
                </Grid>
                <Grid item xs={12} sm={6}>
                    <FormControl fullWidth>
                        <InputLabel>Marital Status</InputLabel>
                        <Select
                            value={maritalStatus}
                            label="Marital Status"
                            onChange={(e) => setMaritalStatus(e.target.value as MaritalStatus)}
                        >
                            {MARITAL_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} value={opt.value}>
                                    {opt.label}
                                </MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                </Grid>
                <Grid item xs={12} sm={6}>
                    <FormControl fullWidth>
                        <InputLabel>Employment Status</InputLabel>
                        <Select
                            value={employmentStatus}
                            label="Employment Status"
                            onChange={(e) => setEmploymentStatus(e.target.value as EmploymentStatus)}
                        >
                            {EMPLOYMENT_OPTIONS.map((opt) => (
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
                        label="Number of Dependents"
                        type="number"
                        value={dependents}
                        onChange={(e) => setDependents(e.target.value)}
                        inputProps={{ min: 0, max: 20 }}
                    />
                </Grid>
                <Grid item xs={12} sm={6}>
                    <TextField
                        fullWidth
                        label="Contact Email"
                        type="email"
                        value={contactEmail}
                        onChange={(e) => setContactEmail(e.target.value)}
                        helperText="Optional - for financial plan export"
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
