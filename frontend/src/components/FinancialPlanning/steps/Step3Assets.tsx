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
} from '@mui/material';
import { Add, Edit, Delete } from '@mui/icons-material';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import {
    createSurveyAsset,
    updateSurveyAsset,
    deleteSurveyAsset,
} from '../../../lib/endpoints';
import type { SurveyAsset, AssetType, CreateAssetRequest, UpdateAssetRequest } from '../../../types';

const ASSET_TYPE_OPTIONS: { value: AssetType; label: string }[] = [
    { value: 'liquid', label: 'Liquid (Cash, Savings)' },
    { value: 'investment', label: 'Investments (Stocks, Bonds)' },
    { value: 'retirement', label: 'Retirement (401k, IRA)' },
    { value: 'tfsa', label: 'TFSA (Tax-Free Savings Account)' },
    { value: 'rrsp', label: 'RRSP (Registered Retirement Savings Plan)' },
    { value: 'lira', label: 'LIRA (Locked-In Retirement Account)' },
    { value: 'resp', label: 'RESP (Registered Education Savings Plan)' },
    { value: 'rrif', label: 'RRIF (Registered Retirement Income Fund)' },
    { value: 'fhsa', label: 'FHSA (First Home Savings Account)' },
    { value: 'real_estate', label: 'Real Estate' },
    { value: 'other', label: 'Other' },
];

const ASSET_TYPE_COLORS: Record<string, 'success' | 'primary' | 'secondary' | 'warning' | 'default'> = {
    liquid: 'success',
    investment: 'primary',
    retirement: 'secondary',
    tfsa: 'primary',
    rrsp: 'secondary',
    lira: 'secondary',
    resp: 'primary',
    rrif: 'secondary',
    fhsa: 'primary',
    real_estate: 'warning',
    other: 'default',
};

function formatCurrency(value: number): string {
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

interface Step3AssetsProps {
    surveyId: string;
    assets: SurveyAsset[];
}

export function Step3Assets({ surveyId, assets }: Step3AssetsProps) {
    const [dialogOpen, setDialogOpen] = useState(false);
    const [editingAsset, setEditingAsset] = useState<SurveyAsset | null>(null);
    const queryClient = useQueryClient();

    const createMutation = useMutation({
        mutationFn: (data: CreateAssetRequest) => createSurveyAsset(surveyId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
        },
    });

    const updateMutation = useMutation({
        mutationFn: ({ assetId, data }: { assetId: string; data: UpdateAssetRequest }) =>
            updateSurveyAsset(surveyId, assetId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setDialogOpen(false);
            setEditingAsset(null);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: (assetId: string) => deleteSurveyAsset(surveyId, assetId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    const handleAdd = () => {
        setEditingAsset(null);
        setDialogOpen(true);
    };

    const handleEdit = (asset: SurveyAsset) => {
        setEditingAsset(asset);
        setDialogOpen(true);
    };

    const handleDelete = (assetId: string) => {
        deleteMutation.mutate(assetId);
    };

    const handleSubmit = (data: CreateAssetRequest) => {
        if (editingAsset) {
            updateMutation.mutate({ assetId: editingAsset.id, data });
        } else {
            createMutation.mutate(data);
        }
    };

    const totalAssets = assets.reduce((sum, a) => sum + a.current_value, 0);

    return (
        <Box>
            <Box display="flex" justifyContent="space-between" alignItems="center" mb={2}>
                <Box>
                    <Typography variant="h6">Assets</Typography>
                    <Typography variant="body2" color="text.secondary">
                        Add all your assets to calculate your net worth.
                    </Typography>
                </Box>
                <Button variant="contained" startIcon={<Add />} onClick={handleAdd}>
                    Add Asset
                </Button>
            </Box>

            {assets.length > 0 ? (
                <>
                    <TableContainer component={Paper} variant="outlined">
                        <Table size="small">
                            <TableHead>
                                <TableRow>
                                    <TableCell>Type</TableCell>
                                    <TableCell>Description</TableCell>
                                    <TableCell align="right">Value</TableCell>
                                    <TableCell align="right">Actions</TableCell>
                                </TableRow>
                            </TableHead>
                            <TableBody>
                                {assets.map((asset) => (
                                    <TableRow key={asset.id}>
                                        <TableCell>
                                            <Chip
                                                label={ASSET_TYPE_OPTIONS.find(t => t.value === asset.asset_type)?.label || asset.asset_type}
                                                size="small"
                                                color={ASSET_TYPE_COLORS[asset.asset_type] || 'default'}
                                                variant="outlined"
                                            />
                                        </TableCell>
                                        <TableCell>{asset.description || '-'}</TableCell>
                                        <TableCell align="right">{formatCurrency(asset.current_value)}</TableCell>
                                        <TableCell align="right">
                                            <IconButton size="small" onClick={() => handleEdit(asset)}>
                                                <Edit fontSize="small" />
                                            </IconButton>
                                            <IconButton
                                                size="small"
                                                color="error"
                                                onClick={() => handleDelete(asset.id)}
                                                disabled={deleteMutation.isPending}
                                            >
                                                <Delete fontSize="small" />
                                            </IconButton>
                                        </TableCell>
                                    </TableRow>
                                ))}
                                <TableRow>
                                    <TableCell colSpan={2}>
                                        <Typography variant="subtitle2" fontWeight="bold">Total</Typography>
                                    </TableCell>
                                    <TableCell align="right">
                                        <Typography variant="subtitle2" fontWeight="bold" color="success.main">
                                            {formatCurrency(totalAssets)}
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
                        No assets added yet. Click "Add Asset" to get started.
                    </Typography>
                </Paper>
            )}

            <AssetFormDialog
                open={dialogOpen}
                onClose={() => { setDialogOpen(false); setEditingAsset(null); }}
                onSubmit={handleSubmit}
                editAsset={editingAsset}
                isSaving={createMutation.isPending || updateMutation.isPending}
            />
        </Box>
    );
}

function AssetFormDialog({
    open,
    onClose,
    onSubmit,
    editAsset,
    isSaving,
}: {
    open: boolean;
    onClose: () => void;
    onSubmit: (data: CreateAssetRequest) => void;
    editAsset: SurveyAsset | null;
    isSaving: boolean;
}) {
    const [assetType, setAssetType] = useState<AssetType>(editAsset?.asset_type || 'liquid');
    const [description, setDescription] = useState(editAsset?.description || '');
    const [currentValue, setCurrentValue] = useState(editAsset?.current_value?.toString() || '');
    const [notes, setNotes] = useState(editAsset?.notes || '');

    // Reset form when dialog opens with different data
    useEffect(() => {
        if (open) {
            setAssetType(editAsset?.asset_type || 'liquid');
            setDescription(editAsset?.description || '');
            setCurrentValue(editAsset?.current_value?.toString() || '');
            setNotes(editAsset?.notes || '');
        }
    }, [open, editAsset]);

    const handleSave = () => {
        onSubmit({
            asset_type: assetType,
            description: description || undefined,
            current_value: parseFloat(currentValue) || 0,
            notes: notes || undefined,
        });
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle>{editAsset ? 'Edit Asset' : 'Add Asset'}</DialogTitle>
            <DialogContent>
                <Box display="flex" flexDirection="column" gap={2} mt={1}>
                    <FormControl fullWidth>
                        <InputLabel>Asset Type</InputLabel>
                        <Select
                            value={assetType}
                            label="Asset Type"
                            onChange={(e) => setAssetType(e.target.value as AssetType)}
                        >
                            {ASSET_TYPE_OPTIONS.map((opt) => (
                                <MenuItem key={opt.value} value={opt.value}>{opt.label}</MenuItem>
                            ))}
                        </Select>
                    </FormControl>
                    <TextField
                        fullWidth
                        label="Description"
                        value={description}
                        onChange={(e) => setDescription(e.target.value)}
                        placeholder="e.g., Savings Account at Chase"
                    />
                    <TextField
                        fullWidth
                        label="Current Value"
                        type="number"
                        value={currentValue}
                        onChange={(e) => setCurrentValue(e.target.value)}
                        InputProps={{
                            startAdornment: <InputAdornment position="start">$</InputAdornment>,
                        }}
                        required
                    />
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
                    disabled={!currentValue || isSaving}
                >
                    {isSaving ? 'Saving...' : editAsset ? 'Update' : 'Add'}
                </Button>
            </DialogActions>
        </Dialog>
    );
}
