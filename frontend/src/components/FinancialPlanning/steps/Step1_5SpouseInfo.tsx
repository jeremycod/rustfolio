import { useState, useEffect } from 'react';
import {
    Box,
    Typography,
    Switch,
    FormControlLabel,
    TextField,
    FormControl,
    InputLabel,
    Select,
    MenuItem,
    Paper,
    Collapse,
    Alert,
} from '@mui/material';
import { People, Person } from '@mui/icons-material';
import type { SurveyPersonalInfo, UpdatePersonalInfoRequest, EmploymentStatus } from '../../../types';

const EMPLOYMENT_STATUS_OPTIONS: { value: EmploymentStatus; label: string }[] = [
    { value: 'employed', label: 'Employed' },
    { value: 'self_employed', label: 'Self-Employed' },
    { value: 'retired', label: 'Retired' },
    { value: 'unemployed', label: 'Not Currently Employed' },
    { value: 'student', label: 'Student' },
    { value: 'other', label: 'Other' },
];

interface Step1_5SpouseInfoProps {
    data: SurveyPersonalInfo | null;
    onSave: (data: UpdatePersonalInfoRequest) => void;
    isSaving: boolean;
}

export function Step1_5SpouseInfo({ data, onSave, isSaving }: Step1_5SpouseInfoProps) {
    const [hasSpouse, setHasSpouse] = useState(data?.has_spouse ?? false);
    const [spouseName, setSpouseName] = useState(data?.spouse_name ?? '');
    const [spouseBirthYear, setSpouseBirthYear] = useState(data?.spouse_birth_year?.toString() ?? '');
    const [spouseEmployment, setSpouseEmployment] = useState<EmploymentStatus | ''>(
        (data?.spouse_employment_status as EmploymentStatus) ?? ''
    );

    // Sync from parent data when it changes (e.g. after save)
    useEffect(() => {
        setHasSpouse(data?.has_spouse ?? false);
        setSpouseName(data?.spouse_name ?? '');
        setSpouseBirthYear(data?.spouse_birth_year?.toString() ?? '');
        setSpouseEmployment((data?.spouse_employment_status as EmploymentStatus) ?? '');
    }, [data]);

    const handleToggle = (checked: boolean) => {
        setHasSpouse(checked);
        onSave({
            has_spouse: checked,
            // Clear spouse fields if toggling off
            spouse_name: checked ? spouseName || undefined : undefined,
            spouse_birth_year: checked ? (spouseBirthYear ? parseInt(spouseBirthYear) : undefined) : undefined,
            spouse_employment_status: checked ? (spouseEmployment || undefined) : undefined,
        });
    };

    const handleBlurSave = () => {
        if (!hasSpouse) return;
        onSave({
            has_spouse: true,
            spouse_name: spouseName || undefined,
            spouse_birth_year: spouseBirthYear ? parseInt(spouseBirthYear) : undefined,
            spouse_employment_status: spouseEmployment || undefined,
        });
    };

    const currentYear = new Date().getFullYear();

    return (
        <Box>
            <Box mb={3}>
                <Typography variant="h6" gutterBottom>
                    Spouse / Partner
                </Typography>
                <Typography variant="body2" color="text.secondary">
                    Adding a spouse or partner enables household-level financial tracking, ownership
                    attribution on assets and liabilities, and combined financial projections.
                </Typography>
            </Box>

            <Paper variant="outlined" sx={{ p: 2, mb: 2 }}>
                <FormControlLabel
                    control={
                        <Switch
                            checked={hasSpouse}
                            onChange={(e) => handleToggle(e.target.checked)}
                            disabled={isSaving}
                            color="primary"
                        />
                    }
                    label={
                        <Box display="flex" alignItems="center" gap={1}>
                            {hasSpouse ? <People /> : <Person />}
                            <Typography variant="body1" fontWeight="medium">
                                I have a spouse or partner to include
                            </Typography>
                        </Box>
                    }
                />
            </Paper>

            <Collapse in={hasSpouse}>
                <Paper variant="outlined" sx={{ p: 3 }}>
                    <Typography variant="subtitle1" fontWeight="medium" gutterBottom>
                        Spouse / Partner Details
                    </Typography>
                    <Alert severity="info" sx={{ mb: 2 }} variant="outlined">
                        This information is used for planning purposes only. All data is private to your account.
                    </Alert>
                    <Box display="flex" flexDirection="column" gap={2}>
                        <TextField
                            fullWidth
                            label="Name (optional)"
                            value={spouseName}
                            onChange={(e) => setSpouseName(e.target.value)}
                            onBlur={handleBlurSave}
                            placeholder="e.g., Alex"
                            helperText="Used for display in household views"
                        />
                        <TextField
                            fullWidth
                            label="Birth Year"
                            type="number"
                            value={spouseBirthYear}
                            onChange={(e) => setSpouseBirthYear(e.target.value)}
                            onBlur={handleBlurSave}
                            inputProps={{ min: currentYear - 100, max: currentYear - 18 }}
                            helperText="Used for retirement projection calculations"
                        />
                        <FormControl fullWidth>
                            <InputLabel>Employment Status</InputLabel>
                            <Select
                                value={spouseEmployment}
                                label="Employment Status"
                                onChange={(e) => {
                                    setSpouseEmployment(e.target.value as EmploymentStatus);
                                }}
                                onClose={handleBlurSave}
                            >
                                <MenuItem value=""><em>Not specified</em></MenuItem>
                                {EMPLOYMENT_STATUS_OPTIONS.map((opt) => (
                                    <MenuItem key={opt.value} value={opt.value}>{opt.label}</MenuItem>
                                ))}
                            </Select>
                        </FormControl>
                    </Box>
                </Paper>
            </Collapse>

            {!hasSpouse && (
                <Typography variant="body2" color="text.secondary" sx={{ mt: 2 }}>
                    You can add a spouse or partner at any time. Existing assets and liabilities will
                    default to "Mine" ownership until ownership is reassigned.
                </Typography>
            )}
        </Box>
    );
}
