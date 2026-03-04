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
    ToggleButtonGroup,
    ToggleButton,
} from '@mui/material';
import { Add, Edit, Delete, People, Person } from '@mui/icons-material';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
    getHouseholdExpenses,
    createHouseholdExpense,
    updateHouseholdExpense,
    deleteHouseholdExpense,
} from '../../../lib/endpoints';
import type {
    SurveyHouseholdExpense,
    HouseholdExpenseType,
    CreateHouseholdExpenseRequest,
} from '../../../types';

const EXPENSE_CATEGORY_OPTIONS = [
    { value: 'housing', label: 'Mortgage & Rent' },
    { value: 'utilities', label: 'Bills & Utilities' },
    { value: 'groceries', label: 'Groceries' },
    { value: 'shopping', label: 'Shopping' },
    { value: 'transportation', label: 'Auto & Transport' },
    { value: 'travel', label: 'Travel' },
    { value: 'dining_out', label: 'Eating Out' },
    { value: 'entertainment', label: 'Entertainment' },
    { value: 'sports_fitness', label: 'Sports & Fitness' },
    { value: 'personal_care', label: 'Personal Care' },
    { value: 'clothing', label: 'Clothing' },
    { value: 'healthcare', label: 'Health & Medical' },
    { value: 'insurance', label: 'Insurance' },
    { value: 'education', label: 'Education' },
    { value: 'childcare', label: 'Childcare' },
    { value: 'home_maintenance', label: 'Home' },
    { value: 'pet_care', label: 'Pets' },
    { value: 'electronics_software', label: 'Electronics & Software' },
    { value: 'savings_investments', label: 'Investments' },
    { value: 'subscriptions', label: 'Subscriptions' },
    { value: 'gifts_donations', label: 'Gifts & Donations' },
    { value: 'business_services', label: 'Business Services' },
    { value: 'fees', label: 'Fees & Bank Charges' },
    { value: 'credit_card_payment', label: 'Credit Card Payment' },
    { value: 'transfer', label: 'Transfer' },
    { value: 'miscellaneous', label: 'Uncategorized / Miscellaneous' },
    { value: 'other', label: 'Other' },
];

const EXPENSE_TYPE_COLORS: Record<HouseholdExpenseType, 'primary' | 'secondary' | 'success'> = {
    shared: 'primary',
    mine: 'success',
    spouse: 'secondary',
};

const EXPENSE_TYPE_LABELS: Record<HouseholdExpenseType, string> = {
    shared: 'Shared',
    mine: 'Mine',
    spouse: 'Spouse',
};

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

interface Step4_5HouseholdExpensesProps {
    surveyId: string;
    spouseName: string | null;
}

export function Step4_5HouseholdExpenses({ surveyId, spouseName }: Step4_5HouseholdExpensesProps) {
    const [dialogOpen, setDialogOpen] = useState(false);
    const [editingExpense, setEditingExpense] = useState<SurveyHouseholdExpense | null>(null);
    const queryClient = useQueryClient();

    const expensesQ = useQuery({
        queryKey: ['household-expenses', surveyId],
        queryFn: () => getHouseholdExpenses(surveyId),
    });

    const createMutation = useMutation({
        mutationFn: (data: CreateHouseholdExpenseRequest) => createHouseholdExpense(surveyId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['household-expenses', surveyId] });
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
        },
    });

    const updateMutation = useMutation({
        mutationFn: ({ expenseId, data }: { expenseId: string; data: CreateHouseholdExpenseRequest }) =>
            updateHouseholdExpense(surveyId, expenseId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['household-expenses', surveyId] });
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
            setEditingExpense(null);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: (expenseId: string) => deleteHouseholdExpense(surveyId, expenseId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['household-expenses', surveyId] });
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    const expenses = expensesQ.data ?? [];

    const totals = expenses.reduce(
        (acc, e) => {
            acc[e.expense_type as HouseholdExpenseType] = (acc[e.expense_type as HouseholdExpenseType] ?? 0) + e.monthly_amount;
            acc.total += e.monthly_amount;
            return acc;
        },
        { shared: 0, mine: 0, spouse: 0, total: 0 }
    );

    const handleAdd = () => {
        setEditingExpense(null);
        setDialogOpen(true);
    };

    const handleEdit = (expense: SurveyHouseholdExpense) => {
        setEditingExpense(expense);
        setDialogOpen(true);
    };

    const handleSubmit = (data: CreateHouseholdExpenseRequest) => {
        if (editingExpense) {
            updateMutation.mutate({ expenseId: editingExpense.id, data });
        } else {
            createMutation.mutate(data);
        }
    };

    return (
        <Box>
            <Box display="flex" justifyContent="space-between" alignItems="flex-start" mb={2}>
                <Box>
                    <Typography variant="h6">Monthly Expenses</Typography>
                    <Typography variant="body2" color="text.secondary">
                        Tag each expense as shared (split 50/50), yours, or {spouseName ?? 'your partner'}'s.
                        This replaces the individual expenses step and powers both cash flow views.
                    </Typography>
                </Box>
                <Button variant="contained" startIcon={<Add />} onClick={handleAdd}>
                    Add Expense
                </Button>
            </Box>

            <Alert severity="info" sx={{ mb: 2 }} variant="outlined">
                <strong>Shared</strong> — both partners pay (split equally) &nbsp;·&nbsp;
                <strong>Mine</strong> — primary person's expense &nbsp;·&nbsp;
                <strong>{spouseName ?? 'Spouse'}</strong> — partner's expense
            </Alert>

            {expenses.length > 0 ? (
                <>
                    <TableContainer component={Paper} variant="outlined">
                        <Table size="small">
                            <TableHead>
                                <TableRow>
                                    <TableCell>Category</TableCell>
                                    <TableCell>Who Pays</TableCell>
                                    <TableCell>Description</TableCell>
                                    <TableCell align="right">Monthly</TableCell>
                                    <TableCell align="right">Actions</TableCell>
                                </TableRow>
                            </TableHead>
                            <TableBody>
                                {expenses.map((expense) => (
                                    <TableRow key={expense.id}>
                                        <TableCell>
                                            {EXPENSE_CATEGORY_OPTIONS.find(c => c.value === expense.expense_category)?.label ?? expense.expense_category}
                                        </TableCell>
                                        <TableCell>
                                            <Chip
                                                label={EXPENSE_TYPE_LABELS[expense.expense_type as HouseholdExpenseType] ?? expense.expense_type}
                                                size="small"
                                                color={EXPENSE_TYPE_COLORS[expense.expense_type as HouseholdExpenseType] ?? 'default'}
                                                icon={expense.expense_type === 'shared' ? <People /> : <Person />}
                                                variant="outlined"
                                            />
                                        </TableCell>
                                        <TableCell>{expense.description ?? '-'}</TableCell>
                                        <TableCell align="right">{formatCurrency(expense.monthly_amount)}</TableCell>
                                        <TableCell align="right">
                                            <IconButton size="small" onClick={() => handleEdit(expense)}>
                                                <Edit fontSize="small" />
                                            </IconButton>
                                            <IconButton
                                                size="small"
                                                color="error"
                                                onClick={() => deleteMutation.mutate(expense.id)}
                                                disabled={deleteMutation.isPending}
                                            >
                                                <Delete fontSize="small" />
                                            </IconButton>
                                        </TableCell>
                                    </TableRow>
                                ))}
                                <TableRow sx={{ bgcolor: 'action.hover' }}>
                                    <TableCell colSpan={3}>
                                        <Typography variant="subtitle2" fontWeight="bold">
                                            Total &nbsp;
                                            <Typography component="span" variant="caption" color="text.secondary">
                                                (Shared: {formatCurrency(totals.shared)} · Mine: {formatCurrency(totals.mine)} · {spouseName ?? 'Spouse'}: {formatCurrency(totals.spouse)})
                                            </Typography>
                                        </Typography>
                                    </TableCell>
                                    <TableCell align="right">
                                        <Typography variant="subtitle2" fontWeight="bold">
                                            {formatCurrency(totals.total)}
                                        </Typography>
                                    </TableCell>
                                    <TableCell />
                                </TableRow>
                            </TableBody>
                        </Table>
                    </TableContainer>
                </>
            ) : (
                <Paper variant="outlined" sx={{ p: 3, textAlign: 'center' }}>
                    <Typography variant="body2" color="text.secondary">
                        No household expenses added yet. Click "Add Expense" to start tracking.
                    </Typography>
                </Paper>
            )}

            <HouseholdExpenseDialog
                open={dialogOpen}
                onClose={() => { setDialogOpen(false); setEditingExpense(null); }}
                onSubmit={handleSubmit}
                editExpense={editingExpense}
                spouseName={spouseName}
                isSaving={createMutation.isPending || updateMutation.isPending}
            />
        </Box>
    );
}

function HouseholdExpenseDialog({
    open,
    onClose,
    onSubmit,
    editExpense,
    spouseName,
    isSaving,
}: {
    open: boolean;
    onClose: () => void;
    onSubmit: (data: CreateHouseholdExpenseRequest) => void;
    editExpense: SurveyHouseholdExpense | null;
    spouseName: string | null;
    isSaving: boolean;
}) {
    const [category, setCategory] = useState(editExpense?.expense_category ?? 'housing');
    const [expenseType, setExpenseType] = useState<HouseholdExpenseType>(
        (editExpense?.expense_type as HouseholdExpenseType) ?? 'shared'
    );
    const [amount, setAmount] = useState(editExpense?.monthly_amount?.toString() ?? '');
    const [description, setDescription] = useState(editExpense?.description ?? '');

    useEffect(() => {
        if (open) {
            setCategory(editExpense?.expense_category ?? 'housing');
            setExpenseType((editExpense?.expense_type as HouseholdExpenseType) ?? 'shared');
            setAmount(editExpense?.monthly_amount?.toString() ?? '');
            setDescription(editExpense?.description ?? '');
        }
    }, [open, editExpense]);

    const handleSave = () => {
        onSubmit({
            expense_category: category,
            expense_type: expenseType,
            monthly_amount: parseFloat(amount) || 0,
            description: description || undefined,
        });
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle>{editExpense ? 'Edit Expense' : 'Add Household Expense'}</DialogTitle>
            <DialogContent>
                <Box display="flex" flexDirection="column" gap={2} mt={1}>
                    <FormControl fullWidth>
                        <InputLabel>Category</InputLabel>
                        <Select
                            value={category}
                            label="Category"
                            onChange={(e) => setCategory(e.target.value)}
                        >
                            {EXPENSE_CATEGORY_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} value={opt.value}>{opt.label}</MenuItem>
                            ))}
                        </Select>
                    </FormControl>

                    <Box>
                        <Typography variant="body2" color="text.secondary" gutterBottom>
                            Who pays this expense?
                        </Typography>
                        <ToggleButtonGroup
                            value={expenseType}
                            exclusive
                            onChange={(_, val) => val && setExpenseType(val)}
                            fullWidth
                            size="small"
                        >
                            <ToggleButton value="shared" color="primary">
                                <People sx={{ mr: 0.5 }} fontSize="small" />
                                Shared
                            </ToggleButton>
                            <ToggleButton value="mine" color="success">
                                <Person sx={{ mr: 0.5 }} fontSize="small" />
                                Mine
                            </ToggleButton>
                            <ToggleButton value="spouse" color="secondary">
                                <Person sx={{ mr: 0.5 }} fontSize="small" />
                                {spouseName ?? 'Spouse'}
                            </ToggleButton>
                        </ToggleButtonGroup>
                    </Box>

                    <TextField
                        fullWidth
                        label="Monthly Amount"
                        type="number"
                        value={amount}
                        onChange={(e) => setAmount(e.target.value)}
                        InputProps={{
                            startAdornment: <InputAdornment position="start">$</InputAdornment>,
                        }}
                        required
                    />
                    <TextField
                        fullWidth
                        label="Description (optional)"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        placeholder="e.g., Monthly rent"
                    />
                </Box>
            </DialogContent>
            <DialogActions>
                <Button onClick={onClose}>Cancel</Button>
                <Button
                    variant="contained"
                    onClick={handleSave}
                    disabled={!amount || isSaving}
                >
                    {isSaving ? 'Saving...' : editExpense ? 'Update' : 'Add'}
                </Button>
            </DialogActions>
        </Dialog>
    );
}
