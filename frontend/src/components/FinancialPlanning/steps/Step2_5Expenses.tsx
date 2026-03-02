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
    Alert,
} from '@mui/material';
import { Add, Edit, Delete } from '@mui/icons-material';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
    createExpense,
    getExpensesList,
    updateExpense,
    deleteExpense,
} from '../../../lib/endpoints';
import type { SurveyExpense, ExpenseCategory, CreateExpenseRequest, UpdateExpenseRequest } from '../../../types';

const EXPENSE_CATEGORY_OPTIONS: { value: ExpenseCategory; label: string }[] = [
    { value: 'housing', label: 'Housing (Rent/Mortgage)' },
    { value: 'utilities', label: 'Utilities' },
    { value: 'groceries', label: 'Groceries' },
    { value: 'transportation', label: 'Transportation' },
    { value: 'insurance', label: 'Insurance' },
    { value: 'healthcare', label: 'Healthcare' },
    { value: 'childcare', label: 'Childcare' },
    { value: 'education', label: 'Education' },
    { value: 'entertainment', label: 'Entertainment' },
    { value: 'dining_out', label: 'Dining Out' },
    { value: 'subscriptions', label: 'Subscriptions' },
    { value: 'personal_care', label: 'Personal Care' },
    { value: 'clothing', label: 'Clothing' },
    { value: 'gifts_donations', label: 'Gifts & Donations' },
    { value: 'pet_care', label: 'Pet Care' },
    { value: 'home_maintenance', label: 'Home Maintenance' },
    { value: 'savings_investments', label: 'Savings & Investments' },
    { value: 'miscellaneous', label: 'Miscellaneous' },
    { value: 'other', label: 'Other' },
];

const EXPENSE_CATEGORY_COLORS: Record<string, 'error' | 'warning' | 'info' | 'success' | 'secondary' | 'default'> = {
    housing: 'error',
    utilities: 'warning',
    groceries: 'success',
    transportation: 'info',
    insurance: 'secondary',
    healthcare: 'error',
    childcare: 'warning',
    education: 'info',
    entertainment: 'success',
    dining_out: 'success',
    subscriptions: 'info',
    personal_care: 'secondary',
    clothing: 'secondary',
    gifts_donations: 'success',
    pet_care: 'info',
    home_maintenance: 'warning',
    savings_investments: 'success',
    miscellaneous: 'default',
    other: 'default',
};

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

interface Step2_5ExpensesProps {
    surveyId: string;
}

export function Step2_5Expenses({ surveyId }: Step2_5ExpensesProps) {
    const [dialogOpen, setDialogOpen] = useState(false);
    const [editingExpense, setEditingExpense] = useState<SurveyExpense | null>(null);
    const queryClient = useQueryClient();

    const { data: expenses = [] } = useQuery({
        queryKey: ['expenses', surveyId],
        queryFn: () => getExpensesList(surveyId),
        enabled: !!surveyId,
    });

    const createMutation = useMutation({
        mutationFn: (data: CreateExpenseRequest) => createExpense(surveyId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['expenses', surveyId] });
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
        },
    });

    const updateMutation = useMutation({
        mutationFn: ({ expenseId, data }: { expenseId: string; data: UpdateExpenseRequest }) =>
            updateExpense(surveyId, expenseId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['expenses', surveyId] });
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
            setEditingExpense(null);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: (expenseId: string) => deleteExpense(surveyId, expenseId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['expenses', surveyId] });
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    const handleAdd = () => {
        setEditingExpense(null);
        setDialogOpen(true);
    };

    const handleEdit = (expense: SurveyExpense) => {
        setEditingExpense(expense);
        setDialogOpen(true);
    };

    const handleDelete = (expenseId: string) => {
        deleteMutation.mutate(expenseId);
    };

    const handleSubmit = (data: CreateExpenseRequest) => {
        if (editingExpense) {
            updateMutation.mutate({ expenseId: editingExpense.id, data });
        } else {
            createMutation.mutate(data);
        }
    };

    const totalExpenses = expenses.reduce((sum, e) => e.is_recurring ? sum + e.monthly_amount : sum, 0);

    return (
        <Box>
            <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                <Box>
                    <Typography variant="h6">Monthly Expenses</Typography>
                    <Typography variant="body2" color="text.secondary">
                        Track your actual monthly expenses by category. This replaces the 70% income estimate.
                    </Typography>
                </Box>
                <Button variant="contained" startIcon={<Add />} onClick={handleAdd}>
                    Add Expense
                </Button>
            </Box>

            {expenses.length === 0 && (
                <Alert severity="info" sx={{ mb: 2 }}>
                    No expenses entered yet. The system will use a 70% of income estimate for cash flow calculations.
                    Add your actual expenses for more accurate projections.
                </Alert>
            )}

            {expenses.length > 0 ? (
                <TableContainer component={Paper} variant="outlined">
                    <Table size="small">
                        <TableHead>
                            <TableRow>
                                <TableCell>Category</TableCell>
                                <TableCell>Description</TableCell>
                                <TableCell align="right">Monthly Amount</TableCell>
                                <TableCell align="right">Actions</TableCell>
                            </TableRow>
                        </TableHead>
                        <TableBody>
                            {expenses.map((expense) => (
                                <TableRow key={expense.id}>
                                    <TableCell>
                                        <Chip
                                            label={
                                                EXPENSE_CATEGORY_OPTIONS.find((c) => c.value === expense.expense_category)?.label ||
                                                expense.expense_category
                                            }
                                            size="small"
                                            color={EXPENSE_CATEGORY_COLORS[expense.expense_category] || 'default'}
                                            variant="outlined"
                                        />
                                    </TableCell>
                                    <TableCell>{expense.description || '-'}</TableCell>
                                    <TableCell align="right">
                                        {formatCurrency(expense.monthly_amount)}
                                        {!expense.is_recurring && (
                                            <Typography variant="caption" display="block" color="text.secondary">
                                                one-time
                                            </Typography>
                                        )}
                                    </TableCell>
                                    <TableCell align="right">
                                        <IconButton size="small" onClick={() => handleEdit(expense)}>
                                            <Edit fontSize="small" />
                                        </IconButton>
                                        <IconButton
                                            size="small"
                                            color="error"
                                            onClick={() => handleDelete(expense.id)}
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
                                        Total Monthly Expenses
                                    </Typography>
                                </TableCell>
                                <TableCell align="right">
                                    <Typography variant="subtitle2" fontWeight="bold" color="error.main">
                                        {formatCurrency(totalExpenses)}
                                    </Typography>
                                    <Typography variant="caption" display="block" color="text.secondary">
                                        recurring only
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
                        No expenses added yet. Click "Add Expense" to get started.
                    </Typography>
                </Paper>
            )}

            <ExpenseFormDialog
                open={dialogOpen}
                onClose={() => {
                    setDialogOpen(false);
                    setEditingExpense(null);
                }}
                onSubmit={handleSubmit}
                editExpense={editingExpense}
                isSaving={createMutation.isPending || updateMutation.isPending}
            />
        </Box>
    );
}

function ExpenseFormDialog({
    open,
    onClose,
    onSubmit,
    editExpense,
    isSaving,
}: {
    open: boolean;
    onClose: () => void;
    onSubmit: (data: CreateExpenseRequest) => void;
    editExpense: SurveyExpense | null;
    isSaving: boolean;
}) {
    const [expenseCategory, setExpenseCategory] = useState<ExpenseCategory>(editExpense?.expense_category || 'housing');
    const [description, setDescription] = useState(editExpense?.description || '');
    const [monthlyAmount, setMonthlyAmount] = useState(editExpense?.monthly_amount?.toString() || '');
    const [isRecurring, setIsRecurring] = useState(editExpense?.is_recurring ?? true);
    const [notes, setNotes] = useState(editExpense?.notes || '');

    useEffect(() => {
        if (open) {
            setExpenseCategory(editExpense?.expense_category || 'housing');
            setDescription(editExpense?.description || '');
            setMonthlyAmount(editExpense?.monthly_amount?.toString() || '');
            setIsRecurring(editExpense?.is_recurring ?? true);
            setNotes(editExpense?.notes || '');
        }
    }, [open, editExpense]);

    const handleSave = () => {
        onSubmit({
            expense_category: expenseCategory,
            description: description || undefined,
            monthly_amount: parseFloat(monthlyAmount) || 0,
            is_recurring: isRecurring,
            notes: notes || undefined,
        });
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle>{editExpense ? 'Edit Expense' : 'Add Expense'}</DialogTitle>
            <DialogContent>
                <Box display="flex" flexDirection="column" gap={2} mt={1}>
                    <FormControl fullWidth>
                        <InputLabel>Expense Category</InputLabel>
                        <Select
                            value={expenseCategory}
                            label="Expense Category"
                            onChange={(e) => setExpenseCategory(e.target.value as ExpenseCategory)}
                        >
                            {EXPENSE_CATEGORY_OPTIONS.map((opt) => (
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
                        placeholder="e.g., Rent for apartment, Comcast internet"
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
                    {isSaving ? 'Saving...' : editExpense ? 'Update' : 'Add'}
                </Button>
            </DialogActions>
        </Dialog>
    );
}
