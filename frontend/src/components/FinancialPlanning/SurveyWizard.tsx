import { useState, useCallback } from 'react';
import {
    Box,
    Paper,
    Stepper,
    Step,
    StepLabel,
    Button,
    CircularProgress,
    Alert,
} from '@mui/material';
import { ArrowForward, ArrowBack, CheckCircle } from '@mui/icons-material';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
    getFinancialSurvey,
    updatePersonalInfo,
    updateIncomeInfo,
    updateSurveyRiskProfile,
    completeSurvey,
} from '../../lib/endpoints';
import type {
    UpdatePersonalInfoRequest,
    UpdateIncomeInfoRequest,
    UpdateRiskProfileRequest,
} from '../../types';
import { Step1PersonalDetails } from './steps/Step1PersonalDetails';
import { Step2IncomeRetirement } from './steps/Step2IncomeRetirement';
import { Step3Assets } from './steps/Step3Assets';
import { Step4Liabilities } from './steps/Step4Liabilities';
import { Step5Goals } from './steps/Step5Goals';
import { Step6RiskProfile } from './steps/Step6RiskProfile';

const STEPS = [
    'Personal Details',
    'Income & Retirement',
    'Assets',
    'Liabilities',
    'Goals',
    'Risk Profile',
];

interface SurveyWizardProps {
    surveyId: string;
    onComplete: () => void;
    onBack: () => void;
}

export function SurveyWizard({ surveyId, onComplete, onBack }: SurveyWizardProps) {
    const [activeStep, setActiveStep] = useState(0);
    const queryClient = useQueryClient();

    const surveyQ = useQuery({
        queryKey: ['financial-survey', surveyId],
        queryFn: () => getFinancialSurvey(surveyId),
    });

    const personalInfoMutation = useMutation({
        mutationFn: (data: UpdatePersonalInfoRequest) => updatePersonalInfo(surveyId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    const incomeInfoMutation = useMutation({
        mutationFn: (data: UpdateIncomeInfoRequest) => updateIncomeInfo(surveyId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    const riskProfileMutation = useMutation({
        mutationFn: (data: UpdateRiskProfileRequest) => updateSurveyRiskProfile(surveyId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    const completeMutation = useMutation({
        mutationFn: () => completeSurvey(surveyId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-surveys'] });
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            onComplete();
        },
    });

    const handleSavePersonalInfo = useCallback((data: UpdatePersonalInfoRequest) => {
        personalInfoMutation.mutate(data);
    }, [personalInfoMutation]);

    const handleSaveIncomeInfo = useCallback((data: UpdateIncomeInfoRequest) => {
        incomeInfoMutation.mutate(data);
    }, [incomeInfoMutation]);

    const handleSaveRiskProfile = useCallback((data: UpdateRiskProfileRequest) => {
        riskProfileMutation.mutate(data);
    }, [riskProfileMutation]);

    const handleNext = () => {
        if (activeStep === STEPS.length - 1) {
            completeMutation.mutate();
        } else {
            setActiveStep((prev) => prev + 1);
        }
    };

    const handleBack = () => {
        if (activeStep === 0) {
            onBack();
        } else {
            setActiveStep((prev) => prev - 1);
        }
    };

    if (surveyQ.isLoading) {
        return (
            <Box display="flex" justifyContent="center" py={6}>
                <CircularProgress />
            </Box>
        );
    }

    if (surveyQ.error) {
        return (
            <Alert severity="error">
                Failed to load survey: {(surveyQ.error as Error).message}
            </Alert>
        );
    }

    const survey = surveyQ.data;
    if (!survey) return null;

    const renderStepContent = () => {
        switch (activeStep) {
            case 0:
                return (
                    <Step1PersonalDetails
                        data={survey.personal_info}
                        onSave={handleSavePersonalInfo}
                        isSaving={personalInfoMutation.isPending}
                    />
                );
            case 1:
                return (
                    <Step2IncomeRetirement
                        data={survey.income_info}
                        onSave={handleSaveIncomeInfo}
                        isSaving={incomeInfoMutation.isPending}
                    />
                );
            case 2:
                return (
                    <Step3Assets
                        surveyId={surveyId}
                        assets={survey.assets}
                    />
                );
            case 3:
                return (
                    <Step4Liabilities
                        surveyId={surveyId}
                        liabilities={survey.liabilities}
                    />
                );
            case 4:
                return (
                    <Step5Goals
                        surveyId={surveyId}
                        goals={survey.goals}
                    />
                );
            case 5:
                return (
                    <Step6RiskProfile
                        data={survey.risk_profile}
                        onSave={handleSaveRiskProfile}
                        isSaving={riskProfileMutation.isPending}
                    />
                );
            default:
                return null;
        }
    };

    return (
        <Paper sx={{ p: 3 }}>
            <Stepper activeStep={activeStep} sx={{ mb: 4 }} alternativeLabel>
                {STEPS.map((label) => (
                    <Step key={label}>
                        <StepLabel>{label}</StepLabel>
                    </Step>
                ))}
            </Stepper>

            {renderStepContent()}

            {completeMutation.error && (
                <Alert severity="error" sx={{ mt: 2 }}>
                    Failed to complete survey: {(completeMutation.error as Error).message}
                </Alert>
            )}

            <Box display="flex" justifyContent="space-between" mt={4}>
                <Button
                    onClick={handleBack}
                    startIcon={<ArrowBack />}
                >
                    {activeStep === 0 ? 'Back to Overview' : 'Back'}
                </Button>
                <Button
                    variant="contained"
                    onClick={handleNext}
                    disabled={completeMutation.isPending}
                    endIcon={activeStep === STEPS.length - 1 ? <CheckCircle /> : <ArrowForward />}
                >
                    {activeStep === STEPS.length - 1
                        ? (completeMutation.isPending ? 'Completing...' : 'Complete Survey')
                        : 'Next'}
                </Button>
            </Box>
        </Paper>
    );
}
