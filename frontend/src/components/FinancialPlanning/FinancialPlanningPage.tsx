import { useState } from 'react';
import {
    Box,
    Typography,
    Paper,
    Button,
    Grid,
    Card,
    CardContent,
    Chip,
    CircularProgress,
    Alert,
    IconButton,
} from '@mui/material';
import {
    Add,
    Assessment,
    Delete,
    Edit,
    Visibility,
} from '@mui/icons-material';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
    listFinancialSurveys,
    createFinancialSurvey,
    deleteFinancialSurvey,
} from '../../lib/endpoints';
import { SurveyWizard } from './SurveyWizard';
import { FinancialSnapshotView } from './FinancialSnapshot';

type PageView = 'list' | 'wizard' | 'snapshot';

export function FinancialPlanningPage() {
    const [view, setView] = useState<PageView>('list');
    const [selectedSurveyId, setSelectedSurveyId] = useState<string | null>(null);
    const queryClient = useQueryClient();

    const surveysQ = useQuery({
        queryKey: ['financial-surveys'],
        queryFn: listFinancialSurveys,
    });

    const createMutation = useMutation({
        mutationFn: createFinancialSurvey,
        onSuccess: (survey) => {
            queryClient.invalidateQueries({ queryKey: ['financial-surveys'] });
            setSelectedSurveyId(survey.id);
            setView('wizard');
        },
    });

    const deleteMutation = useMutation({
        mutationFn: deleteFinancialSurvey,
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-surveys'] });
        },
    });

    const handleNewSurvey = () => {
        createMutation.mutate();
    };

    const handleEditSurvey = (surveyId: string) => {
        setSelectedSurveyId(surveyId);
        setView('wizard');
    };

    const handleViewSnapshot = (surveyId: string) => {
        setSelectedSurveyId(surveyId);
        setView('snapshot');
    };

    const handleDeleteSurvey = (surveyId: string) => {
        if (window.confirm('Are you sure you want to delete this survey?')) {
            deleteMutation.mutate(surveyId);
        }
    };

    const handleSurveyComplete = () => {
        if (selectedSurveyId) {
            setView('snapshot');
        }
    };

    const handleBackToList = () => {
        setView('list');
        setSelectedSurveyId(null);
    };

    if (view === 'wizard' && selectedSurveyId) {
        return (
            <Box>
                <Box display="flex" alignItems="center" gap={2} mb={3}>
                    <Assessment sx={{ fontSize: 32, color: 'primary.main' }} />
                    <Typography variant="h4" fontWeight="bold">
                        Financial Planning Survey
                    </Typography>
                </Box>
                <SurveyWizard
                    surveyId={selectedSurveyId}
                    onComplete={handleSurveyComplete}
                    onBack={handleBackToList}
                />
            </Box>
        );
    }

    if (view === 'snapshot' && selectedSurveyId) {
        return (
            <Box>
                <Box display="flex" alignItems="center" justifyContent="space-between" mb={3}>
                    <Box display="flex" alignItems="center" gap={2}>
                        <Assessment sx={{ fontSize: 32, color: 'primary.main' }} />
                        <Typography variant="h4" fontWeight="bold">
                            Financial Snapshot
                        </Typography>
                    </Box>
                    <Button onClick={handleBackToList}>
                        Back to Surveys
                    </Button>
                </Box>
                <FinancialSnapshotView
                    surveyId={selectedSurveyId}
                    onEdit={() => handleEditSurvey(selectedSurveyId)}
                />
            </Box>
        );
    }

    // List view
    return (
        <Box>
            <Box display="flex" alignItems="center" justifyContent="space-between" mb={3}>
                <Box display="flex" alignItems="center" gap={2}>
                    <Assessment sx={{ fontSize: 32, color: 'primary.main' }} />
                    <Typography variant="h4" fontWeight="bold">
                        Financial Planning
                    </Typography>
                </Box>
                <Button
                    variant="contained"
                    startIcon={<Add />}
                    onClick={handleNewSurvey}
                    disabled={createMutation.isPending}
                >
                    {createMutation.isPending ? 'Creating...' : 'New Survey'}
                </Button>
            </Box>

            <Typography variant="body1" color="text.secondary" mb={3}>
                Complete a financial planning survey to generate a personalized financial snapshot
                with net worth breakdown, cash flow analysis, retirement projections, and goal tracking.
            </Typography>

            {createMutation.error && (
                <Alert severity="error" sx={{ mb: 2 }}>
                    Failed to create survey: {(createMutation.error as Error).message}
                </Alert>
            )}

            {surveysQ.isLoading && (
                <Box display="flex" justifyContent="center" py={6}>
                    <CircularProgress />
                </Box>
            )}

            {surveysQ.error && (
                <Alert severity="error">
                    Failed to load surveys: {(surveysQ.error as Error).message}
                </Alert>
            )}

            {surveysQ.data && surveysQ.data.length === 0 && (
                <Paper sx={{ p: 4, textAlign: 'center' }}>
                    <Assessment sx={{ fontSize: 64, color: 'text.disabled', mb: 2 }} />
                    <Typography variant="h6" gutterBottom>
                        No Financial Surveys Yet
                    </Typography>
                    <Typography variant="body2" color="text.secondary" mb={3}>
                        Create your first financial planning survey to get started with a comprehensive
                        financial health assessment.
                    </Typography>
                    <Button
                        variant="contained"
                        startIcon={<Add />}
                        onClick={handleNewSurvey}
                        disabled={createMutation.isPending}
                    >
                        Create Survey
                    </Button>
                </Paper>
            )}

            {surveysQ.data && surveysQ.data.length > 0 && (
                <Grid container spacing={2}>
                    {surveysQ.data.map((survey) => (
                        <Grid item xs={12} sm={6} md={4} key={survey.id}>
                            <Card variant="outlined">
                                <CardContent>
                                    <Box display="flex" justifyContent="space-between" alignItems="flex-start" mb={1}>
                                        <Box>
                                            <Typography variant="subtitle1" fontWeight="bold">
                                                Survey v{survey.version}
                                            </Typography>
                                            <Typography variant="caption" color="text.secondary">
                                                Created {new Date(survey.created_at).toLocaleDateString()}
                                            </Typography>
                                        </Box>
                                        <Chip
                                            label={survey.status}
                                            size="small"
                                            color={survey.status === 'completed' ? 'success' : 'default'}
                                            variant="outlined"
                                        />
                                    </Box>
                                    {survey.completed_at && (
                                        <Typography variant="caption" color="text.secondary" display="block" mb={1}>
                                            Completed {new Date(survey.completed_at).toLocaleDateString()}
                                        </Typography>
                                    )}
                                    <Box display="flex" gap={1} mt={2} flexWrap="wrap">
                                        {survey.status === 'completed' ? (
                                            <>
                                                <Button
                                                    size="small"
                                                    variant="outlined"
                                                    startIcon={<Visibility />}
                                                    onClick={() => handleViewSnapshot(survey.id)}
                                                >
                                                    View Snapshot
                                                </Button>
                                                <Button
                                                    size="small"
                                                    variant="outlined"
                                                    startIcon={<Edit />}
                                                    onClick={() => handleEditSurvey(survey.id)}
                                                >
                                                    Edit
                                                </Button>
                                            </>
                                        ) : (
                                            <Button
                                                size="small"
                                                variant="outlined"
                                                startIcon={<Edit />}
                                                onClick={() => handleEditSurvey(survey.id)}
                                            >
                                                Continue
                                            </Button>
                                        )}
                                        <IconButton
                                            size="small"
                                            color="error"
                                            onClick={() => handleDeleteSurvey(survey.id)}
                                            disabled={deleteMutation.isPending}
                                        >
                                            <Delete />
                                        </IconButton>
                                    </Box>
                                </CardContent>
                            </Card>
                        </Grid>
                    ))}
                </Grid>
            )}
        </Box>
    );
}
