import { useState, useEffect } from 'react';
import {
    Box,
    Typography,
    Button,
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
    LinearProgress,
    ToggleButtonGroup,
    ToggleButton,
} from '@mui/material';
import { Add, Edit, Delete, Person, People } from '@mui/icons-material';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import {
    createSurveyGoal,
    updateSurveyGoal,
    deleteSurveyGoal,
} from '../../../lib/endpoints';
import type { SurveyGoal, GoalType, GoalPriority, GoalOwner, CreateGoalRequest, UpdateGoalRequest } from '../../../types';

const GOAL_TYPE_OPTIONS: { value: GoalType; label: string }[] = [
    { value: 'retirement', label: 'Retirement' },
    { value: 'home_purchase', label: 'Home Purchase' },
    { value: 'education', label: 'Education' },
    { value: 'travel', label: 'Travel' },
    { value: 'emergency_fund', label: 'Emergency Fund' },
    { value: 'other', label: 'Other' },
];

const PRIORITY_OPTIONS: { value: GoalPriority; label: string }[] = [
    { value: 'high', label: 'High' },
    { value: 'medium', label: 'Medium' },
    { value: 'low', label: 'Low' },
];

const PRIORITY_COLORS: Record<string, 'error' | 'warning' | 'info'> = {
    high: 'error',
    medium: 'warning',
    low: 'info',
};

const OWNER_COLORS: Record<GoalOwner, 'success' | 'secondary' | 'warning'> = {
    mine: 'success',
    spouse: 'secondary',
    joint: 'warning',
};

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

interface Step5GoalsProps {
    surveyId: string;
    goals: SurveyGoal[];
    hasSpouse?: boolean;
    spouseName?: string | null;
}

export function Step5Goals({ surveyId, goals, hasSpouse = false, spouseName }: Step5GoalsProps) {
    const [dialogOpen, setDialogOpen] = useState(false);
    const [editingGoal, setEditingGoal] = useState<SurveyGoal | null>(null);
    const queryClient = useQueryClient();

    const createMutation = useMutation({
        mutationFn: (data: CreateGoalRequest) => createSurveyGoal(surveyId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
        },
    });

    const updateMutation = useMutation({
        mutationFn: ({ goalId, data }: { goalId: string; data: UpdateGoalRequest }) =>
            updateSurveyGoal(surveyId, goalId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
            setEditingGoal(null);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: (goalId: string) => deleteSurveyGoal(surveyId, goalId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    const handleAdd = () => {
        setEditingGoal(null);
        setDialogOpen(true);
    };

    const handleEdit = (goal: SurveyGoal) => {
        setEditingGoal(goal);
        setDialogOpen(true);
    };

    const handleSubmit = (data: CreateGoalRequest) => {
        if (editingGoal) {
            updateMutation.mutate({ goalId: editingGoal.id, data });
        } else {
            createMutation.mutate(data);
        }
    };

    const ownerLabel = (owner: GoalOwner) => {
        if (owner === 'spouse') return spouseName ?? 'Spouse';
        if (owner === 'joint') return 'Joint';
        return 'Mine';
    };

    return (
        <Box>
            <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                <Box>
                    <Typography variant="h6">Financial Goals</Typography>
                    <Typography variant="body2" color="text.secondary">
                        Define your financial goals and track progress.
                        {hasSpouse && ' Tag each goal as yours, your partner\'s, or a shared household goal.'}
                    </Typography>
                </Box>
                <Button variant="contained" startIcon={<Add />} onClick={handleAdd}>
                    Add Goal
                </Button>
            </Box>

            {goals.length > 0 ? (
                <Box display="flex" flexDirection="column" gap={2}>
                    {goals.map((goal) => {
                        const progress = goal.target_amount && goal.target_amount > 0
                            ? Math.min((goal.current_savings / goal.target_amount) * 100, 100)
                            : 0;
                        const progressColor = progress >= 75 ? 'success' :
                            progress >= 50 ? 'warning' : 'error';

                        return (
                            <Paper key={goal.id} variant="outlined" sx={{ p: 2 }}>
                                <Box display="flex" justifyContent="space-between" alignItems="flex-start" mb={1}>
                                    <Box display="flex" alignItems="center" gap={1} flexWrap="wrap">
                                        <Typography variant="subtitle1" fontWeight="bold">
                                            {goal.description || GOAL_TYPE_OPTIONS.find(t => t.value === goal.goal_type)?.label || goal.goal_type}
                                        </Typography>
                                        {goal.priority && (
                                            <Chip
                                                label={goal.priority}
                                                size="small"
                                                color={PRIORITY_COLORS[goal.priority] || 'default'}
                                                variant="outlined"
                                            />
                                        )}
                                        {hasSpouse && (
                                            <Chip
                                                label={ownerLabel(goal.owner)}
                                                size="small"
                                                color={OWNER_COLORS[goal.owner] ?? 'default'}
                                                icon={goal.owner === 'joint' ? <People /> : <Person />}
                                                variant="outlined"
                                            />
                                        )}
                                    </Box>
                                    <Box>
                                        <IconButton size="small" onClick={() => handleEdit(goal)}>
                                            <Edit fontSize="small" />
                                        </IconButton>
                                        <IconButton
                                            size="small"
                                            color="error"
                                            onClick={() => deleteMutation.mutate(goal.id)}
                                            disabled={deleteMutation.isPending}
                                        >
                                            <Delete fontSize="small" />
                                        </IconButton>
                                    </Box>
                                </Box>
                                <Box display="flex" justifyContent="space-between" mb={0.5}>
                                    <Typography variant="body2" color="text.secondary">
                                        {formatCurrency(goal.current_savings)} / {goal.target_amount ? formatCurrency(goal.target_amount) : 'No target'}
                                    </Typography>
                                    <Typography variant="body2" fontWeight="bold">
                                        {progress.toFixed(0)}%
                                    </Typography>
                                </Box>
                                <LinearProgress
                                    variant="determinate"
                                    value={progress}
                                    color={progressColor}
                                    sx={{ height: 8, borderRadius: 4 }}
                                />
                                {goal.target_date && (
                                    <Typography variant="caption" color="text.secondary" mt={0.5} display="block">
                                        Target date: {new Date(goal.target_date).toLocaleDateString()}
                                    </Typography>
                                )}
                            </Paper>
                        );
                    })}
                </Box>
            ) : (
                <Paper variant="outlined" sx={{ p: 3, textAlign: 'center' }}>
                    <Typography variant="body2" color="text.secondary">
                        No goals added yet. Click "Add Goal" to get started.
                    </Typography>
                </Paper>
            )}

            <GoalFormDialog
                open={dialogOpen}
                onClose={() => { setDialogOpen(false); setEditingGoal(null); }}
                onSubmit={handleSubmit}
                editGoal={editingGoal}
                hasSpouse={hasSpouse}
                spouseName={spouseName}
                isSaving={createMutation.isPending || updateMutation.isPending}
            />
        </Box>
    );
}

function GoalFormDialog({
    open,
    onClose,
    onSubmit,
    editGoal,
    hasSpouse,
    spouseName,
    isSaving,
}: {
    open: boolean;
    onClose: () => void;
    onSubmit: (data: CreateGoalRequest) => void;
    editGoal: SurveyGoal | null;
    hasSpouse: boolean;
    spouseName?: string | null;
    isSaving: boolean;
}) {
    const [goalType, setGoalType] = useState<GoalType>(editGoal?.goal_type || 'retirement');
    const [description, setDescription] = useState(editGoal?.description || '');
    const [targetAmount, setTargetAmount] = useState(editGoal?.target_amount?.toString() || '');
    const [currentSavings, setCurrentSavings] = useState(editGoal?.current_savings?.toString() || '0');
    const [targetDate, setTargetDate] = useState(editGoal?.target_date || '');
    const [priority, setPriority] = useState<GoalPriority>(editGoal?.priority || 'medium');
    const [notes, setNotes] = useState(editGoal?.notes || '');
    const [owner, setOwner] = useState<GoalOwner>(editGoal?.owner ?? 'mine');

    useEffect(() => {
        if (open) {
            setGoalType(editGoal?.goal_type || 'retirement');
            setDescription(editGoal?.description || '');
            setTargetAmount(editGoal?.target_amount?.toString() || '');
            setCurrentSavings(editGoal?.current_savings?.toString() || '0');
            setTargetDate(editGoal?.target_date || '');
            setPriority(editGoal?.priority || 'medium');
            setNotes(editGoal?.notes || '');
            setOwner(editGoal?.owner ?? 'mine');
        }
    }, [open, editGoal]);

    const handleSave = () => {
        onSubmit({
            goal_type: goalType,
            description: description || undefined,
            target_amount: targetAmount ? parseFloat(targetAmount) : undefined,
            current_savings: parseFloat(currentSavings) || 0,
            target_date: targetDate || undefined,
            priority,
            notes: notes || undefined,
            owner,
        });
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle>{editGoal ? 'Edit Goal' : 'Add Goal'}</DialogTitle>
            <DialogContent>
                <Box display="flex" flexDirection="column" gap={2} mt={1}>
                    <FormControl fullWidth>
                        <InputLabel>Goal Type</InputLabel>
                        <Select
                            value={goalType}
                            label="Goal Type"
                            onChange={(e) => setGoalType(e.target.value as GoalType)}
                        >
                            {GOAL_TYPE_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} value={opt.value}>{opt.label}</MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                    <TextField
                        fullWidth
                        label="Description"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        placeholder="e.g., Down payment for house"
                    />
                    <TextField
                        fullWidth
                        label="Target Amount"
                        type="number"
                        value={targetAmount}
                        onChange={(e) => setTargetAmount(e.target.value)}
                        InputProps={{
                            startAdornment: <InputAdornment position="start">$</InputAdornment>,
                        }}
                    />
                    <TextField
                        fullWidth
                        label="Current Savings"
                        type="number"
                        value={currentSavings}
                        onChange={(e) => setCurrentSavings(e.target.value)}
                        InputProps={{
                            startAdornment: <InputAdornment position="start">$</InputAdornment>,
                        }}
                        helperText={goalType === 'retirement' ? 'Used as your current retirement savings in the retirement projection' : undefined}
                    />
                    <TextField
                        fullWidth
                        label="Target Date"
                        type="date"
                        value={targetDate}
                        onChange={(e) => setTargetDate(e.target.value)}
                        InputLabelProps={{ shrink: true }}
                    />
                    <FormControl fullWidth>
                        <InputLabel>Priority</InputLabel>
                        <Select
                            value={priority}
                            label="Priority"
                            onChange={(e) => setPriority(e.target.value as GoalPriority)}
                        >
                            {PRIORITY_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} value={opt.value}>{opt.label}</MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                    {hasSpouse && (
                        <Box>
                            <Typography variant="body2" color="text.secondary" gutterBottom>
                                Who is this goal for?
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
                                <ToggleButton value="joint" color="warning">
                                    <People sx={{ mr: 0.5 }} fontSize="small" /> Joint
                                </ToggleButton>
                            </ToggleButtonGroup>
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
                    disabled={isSaving}
                >
                    {isSaving ? 'Saving...' : editGoal ? 'Update' : 'Add'}
                </Button>
            </DialogActions>
        </Dialog>
    );
}
