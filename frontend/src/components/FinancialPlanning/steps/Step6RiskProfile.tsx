import { useState, useEffect, useCallback } from 'react';
import {
    Box,
    Typography,
    Paper,
    FormControl,
    RadioGroup,
    FormControlLabel,
    Radio,
    Slider,
} from '@mui/material';
import type { SurveyRiskProfile, UpdateRiskProfileRequest, SurveyRiskTolerance, InvestmentExperience } from '../../../types';

const RISK_TOLERANCE_OPTIONS: { value: SurveyRiskTolerance; label: string; description: string }[] = [
    {
        value: 'conservative',
        label: 'Conservative',
        description: 'Preserve capital with minimal volatility. Prefer bonds, CDs, and stable value funds.',
    },
    {
        value: 'moderate',
        label: 'Moderate',
        description: 'Balance between growth and stability. Mix of stocks, bonds, and diversified funds.',
    },
    {
        value: 'aggressive',
        label: 'Aggressive',
        description: 'Maximize growth potential with higher volatility. Primarily equities and growth-oriented investments.',
    },
];

const EXPERIENCE_OPTIONS: { value: InvestmentExperience; label: string; description: string }[] = [
    {
        value: 'none',
        label: 'None',
        description: 'No investing experience.',
    },
    {
        value: 'beginner',
        label: 'Beginner',
        description: 'Less than 2 years of investing experience.',
    },
    {
        value: 'intermediate',
        label: 'Intermediate',
        description: '2-5 years of investing experience.',
    },
    {
        value: 'advanced',
        label: 'Advanced',
        description: 'More than 5 years of active investing experience.',
    },
];

interface Step6RiskProfileProps {
    data: SurveyRiskProfile | null;
    onSave: (data: UpdateRiskProfileRequest) => void;
    isSaving: boolean;
}

export function Step6RiskProfile({ data, onSave, isSaving }: Step6RiskProfileProps) {
    const [riskTolerance, setRiskTolerance] = useState<SurveyRiskTolerance>(data?.risk_tolerance || 'moderate');
    const [investmentExperience, setInvestmentExperience] = useState<InvestmentExperience>(data?.investment_experience || 'beginner');
    const [timeHorizon, setTimeHorizon] = useState<number>(data?.time_horizon_years || 10);

    useEffect(() => {
        if (data) {
            setRiskTolerance(data.risk_tolerance || 'moderate');
            setInvestmentExperience(data.investment_experience || 'beginner');
            setTimeHorizon(data.time_horizon_years || 10);
        }
    }, [data]);

    const saveData = useCallback(() => {
        onSave({
            risk_tolerance: riskTolerance,
            investment_experience: investmentExperience,
            time_horizon_years: timeHorizon,
        });
    }, [riskTolerance, investmentExperience, timeHorizon, onSave]);

    useEffect(() => {
        const timer = setTimeout(saveData, 1000);
        return () => clearTimeout(timer);
    }, [saveData]);

    return (
        <Box>
            <Typography variant="h6" gutterBottom>
                Risk Profile
            </Typography>
            <Typography variant="body2" color="text.secondary" mb={3}>
                Your risk profile helps tailor recommendations and projections to your comfort level.
            </Typography>

            {/* Risk Tolerance */}
            <Typography variant="subtitle1" fontWeight="bold" gutterBottom>
                Risk Tolerance
            </Typography>
            <FormControl component="fieldset" sx={{ mb: 3, width: '100%' }}>
                <RadioGroup
                    value={riskTolerance}
                    onChange={(e) => setRiskTolerance(e.target.value as SurveyRiskTolerance)}
                >
                    {RISK_TOLERANCE_OPTIONS.map((opt) => (
                        <Paper
                            key={opt.value}
                            variant="outlined"
                            sx={{
                                p: 2,
                                mb: 1,
                                border: 2,
                                borderColor: riskTolerance === opt.value ? 'primary.main' : 'divider',
                                cursor: 'pointer',
                            }}
                            onClick={() => setRiskTolerance(opt.value)}
                        >
                            <FormControlLabel
                                value={opt.value}
                                control={<Radio />}
                                label={
                                    <Box>
                                        <Typography fontWeight="bold">{opt.label}</Typography>
                                        <Typography variant="body2" color="text.secondary">
                                            {opt.description}
                                        </Typography>
                                    </Box>
                                }
                            />
                        </Paper>
                    ))}
                </RadioGroup>
            </FormControl>

            {/* Investment Experience */}
            <Typography variant="subtitle1" fontWeight="bold" gutterBottom>
                Investment Experience
            </Typography>
            <FormControl component="fieldset" sx={{ mb: 3, width: '100%' }}>
                <RadioGroup
                    value={investmentExperience}
                    onChange={(e) => setInvestmentExperience(e.target.value as InvestmentExperience)}
                >
                    {EXPERIENCE_OPTIONS.map((opt) => (
                        <Paper
                            key={opt.value}
                            variant="outlined"
                            sx={{
                                p: 2,
                                mb: 1,
                                border: 2,
                                borderColor: investmentExperience === opt.value ? 'primary.main' : 'divider',
                                cursor: 'pointer',
                            }}
                            onClick={() => setInvestmentExperience(opt.value)}
                        >
                            <FormControlLabel
                                value={opt.value}
                                control={<Radio />}
                                label={
                                    <Box>
                                        <Typography fontWeight="bold">{opt.label}</Typography>
                                        <Typography variant="body2" color="text.secondary">
                                            {opt.description}
                                        </Typography>
                                    </Box>
                                }
                            />
                        </Paper>
                    ))}
                </RadioGroup>
            </FormControl>

            {/* Time Horizon */}
            <Typography variant="subtitle1" fontWeight="bold" gutterBottom>
                Investment Time Horizon: {timeHorizon} years
            </Typography>
            <Box sx={{ px: 2, pb: 2 }}>
                <Slider
                    value={timeHorizon}
                    onChange={(_, v) => setTimeHorizon(v as number)}
                    min={1}
                    max={40}
                    step={1}
                    marks={[
                        { value: 1, label: '1yr' },
                        { value: 5, label: '5yr' },
                        { value: 10, label: '10yr' },
                        { value: 20, label: '20yr' },
                        { value: 30, label: '30yr' },
                        { value: 40, label: '40yr' },
                    ]}
                    valueLabelDisplay="auto"
                    valueLabelFormat={(v) => `${v} years`}
                />
                <Typography variant="body2" color="text.secondary" textAlign="center">
                    {timeHorizon <= 3 ? 'Short-term (focus on preservation)' :
                        timeHorizon <= 10 ? 'Medium-term (balanced growth)' :
                            'Long-term (maximum growth potential)'}
                </Typography>
            </Box>

            {isSaving && (
                <Typography variant="caption" color="text.secondary" mt={2} display="block">
                    Saving...
                </Typography>
            )}
        </Box>
    );
}
