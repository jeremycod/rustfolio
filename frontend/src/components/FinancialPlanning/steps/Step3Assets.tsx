import { useState, useEffect } from 'react';
import {
    Box,
    Typography,
    Button,
    ButtonGroup,
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
    ToggleButtonGroup,
    ToggleButton,
    Slider,
    Alert,
    CircularProgress,
    Tooltip,
    List,
    ListItemButton,
    ListItemText,
    ListSubheader,
    Divider,
} from '@mui/material';
import { Add, Edit, Delete, Person, People, Link, LinkOff, Refresh, AccountBalance } from '@mui/icons-material';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
    createSurveyAsset,
    updateSurveyAsset,
    deleteSurveyAsset,
    getLinkableAccounts,
    refreshLinkedAsset,
    unlinkAssetAccount,
} from '../../../lib/endpoints';
import type { SurveyAsset, AssetType, Ownership, LinkableAccount, CreateAssetRequest, UpdateAssetRequest } from '../../../types';

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

function formatCurrency(value: number | null | undefined): string {
    if (value == null) return '—';
    return new Intl.NumberFormat('en-US', {
        style: 'currency',
        currency: 'USD',
        maximumFractionDigits: 0,
    }).format(value);
}

// Group linkable accounts by portfolio for display
function groupByPortfolio(accounts: LinkableAccount[]): Record<string, LinkableAccount[]> {
    return accounts.reduce((acc, a) => {
        if (!acc[a.portfolio_name]) acc[a.portfolio_name] = [];
        acc[a.portfolio_name].push(a);
        return acc;
    }, {} as Record<string, LinkableAccount[]>);
}

interface Step3AssetsProps {
    surveyId: string;
    assets: SurveyAsset[];
    hasSpouse?: boolean;
    spouseName?: string | null;
}

export function Step3Assets({ surveyId, assets, hasSpouse = false, spouseName }: Step3AssetsProps) {
    const [manualDialogOpen, setManualDialogOpen] = useState(false);
    const [accountPickerOpen, setAccountPickerOpen] = useState(false);
    const [editingAsset, setEditingAsset] = useState<SurveyAsset | null>(null);
    // When user picks an account, pre-fill the asset form with it
    const [selectedAccount, setSelectedAccount] = useState<LinkableAccount | null>(null);
    const queryClient = useQueryClient();

    const createMutation = useMutation({
        mutationFn: (data: CreateAssetRequest) => createSurveyAsset(surveyId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setManualDialogOpen(false);
            setSelectedAccount(null);
        },
    });

    const updateMutation = useMutation({
        mutationFn: ({ assetId, data }: { assetId: string; data: UpdateAssetRequest }) =>
            updateSurveyAsset(surveyId, assetId, data),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setManualDialogOpen(false);
            setEditingAsset(null);
        },
    });

    const deleteMutation = useMutation({
        mutationFn: (assetId: string) => deleteSurveyAsset(surveyId, assetId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    const refreshMutation = useMutation({
        mutationFn: (assetId: string) => refreshLinkedAsset(surveyId, assetId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
        },
    });

    const unlinkMutation = useMutation({
        mutationFn: (assetId: string) => unlinkAssetAccount(surveyId, assetId),
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['financial-survey', surveyId] });
            setManualDialogOpen(false);
            setEditingAsset(null);
        },
    });

    const handleEdit = (asset: SurveyAsset) => {
        setEditingAsset(asset);
        setSelectedAccount(null);
        setManualDialogOpen(true);
    };

    const handleAccountSelect = (account: LinkableAccount) => {
        setAccountPickerOpen(false);
        setEditingAsset(null);
        setSelectedAccount(account);
        setManualDialogOpen(true);
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
            <Box display="flex" justifyContent="space-between" alignItems="flex-start" mb={2}>
                <Box>
                    <Typography variant="h6">Assets</Typography>
                    <Typography variant="body2" color="text.secondary">
                        Add assets manually or link directly to a portfolio account for automatic value updates.
                    </Typography>
                </Box>
                <ButtonGroup variant="contained" size="small">
                    <Button startIcon={<Add />} onClick={() => { setEditingAsset(null); setSelectedAccount(null); setManualDialogOpen(true); }}>
                        Add Manually
                    </Button>
                    <Button startIcon={<AccountBalance />} onClick={() => setAccountPickerOpen(true)}>
                        Link Account
                    </Button>
                </ButtonGroup>
            </Box>

            {assets.length > 0 ? (
                <TableContainer component={Paper} variant="outlined">
                    <Table size="small">
                        <TableHead>
                            <TableRow>
                                <TableCell>Type</TableCell>
                                <TableCell>Description / Account</TableCell>
                                {hasSpouse && <TableCell>Owner</TableCell>}
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
                                    <TableCell>
                                        <Box>
                                            <Typography variant="body2">
                                                {asset.description || '-'}
                                            </Typography>
                                            {asset.linked_account_nickname && (
                                                <Chip
                                                    label={asset.linked_account_nickname}
                                                    size="small"
                                                    icon={<Link />}
                                                    color="info"
                                                    variant="outlined"
                                                    sx={{ mt: 0.5, fontSize: '0.7rem' }}
                                                />
                                            )}
                                        </Box>
                                    </TableCell>
                                    {hasSpouse && (
                                        <TableCell>
                                            <Chip
                                                label={asset.ownership === 'joint'
                                                    ? `Joint (${asset.joint_split_percentage ?? 50}%)`
                                                    : asset.ownership === 'spouse'
                                                    ? (spouseName ?? 'Spouse')
                                                    : 'Mine'}
                                                size="small"
                                                color={asset.ownership === 'joint' ? 'warning' : asset.ownership === 'spouse' ? 'secondary' : 'success'}
                                                icon={asset.ownership === 'joint' ? <People /> : <Person />}
                                                variant="outlined"
                                            />
                                        </TableCell>
                                    )}
                                    <TableCell align="right">
                                        <Typography variant="body2" fontWeight="medium">
                                            {formatCurrency(asset.current_value)}
                                        </Typography>
                                        {asset.linked_account_nickname && (
                                            <Typography variant="caption" color="text.secondary" display="block">
                                                live
                                            </Typography>
                                        )}
                                    </TableCell>
                                    <TableCell align="right">
                                        {asset.linked_account_id && (
                                            <Tooltip title="Refresh value from account">
                                                <IconButton
                                                    size="small"
                                                    color="info"
                                                    onClick={() => refreshMutation.mutate(asset.id)}
                                                    disabled={refreshMutation.isPending}
                                                >
                                                    {refreshMutation.isPending
                                                        ? <CircularProgress size={14} />
                                                        : <Refresh fontSize="small" />}
                                                </IconButton>
                                            </Tooltip>
                                        )}
                                        <IconButton size="small" onClick={() => handleEdit(asset)}>
                                            <Edit fontSize="small" />
                                        </IconButton>
                                        <IconButton
                                            size="small"
                                            color="error"
                                            onClick={() => deleteMutation.mutate(asset.id)}
                                            disabled={deleteMutation.isPending}
                                        >
                                            <Delete fontSize="small" />
                                        </IconButton>
                                    </TableCell>
                                </TableRow>
                            ))}
                            <TableRow>
                                <TableCell colSpan={hasSpouse ? 3 : 2}>
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
            ) : (
                <Paper variant="outlined" sx={{ p: 3, textAlign: 'center' }}>
                    <Typography variant="body2" color="text.secondary">
                        No assets added yet. Add manually or link an existing portfolio account.
                    </Typography>
                </Paper>
            )}

            {/* Account picker dialog */}
            <AccountPickerDialog
                open={accountPickerOpen}
                onClose={() => setAccountPickerOpen(false)}
                onSelect={handleAccountSelect}
            />

            {/* Add / edit asset dialog */}
            <AssetFormDialog
                open={manualDialogOpen}
                onClose={() => { setManualDialogOpen(false); setEditingAsset(null); setSelectedAccount(null); }}
                onSubmit={handleSubmit}
                onUnlink={editingAsset?.linked_account_id ? () => unlinkMutation.mutate(editingAsset.id) : undefined}
                editAsset={editingAsset}
                linkedAccount={selectedAccount}
                hasSpouse={hasSpouse}
                spouseName={spouseName}
                isSaving={createMutation.isPending || updateMutation.isPending || unlinkMutation.isPending}
            />
        </Box>
    );
}

// ==============================================================================
// Account Picker Dialog
// ==============================================================================

function AccountPickerDialog({
    open,
    onClose,
    onSelect,
}: {
    open: boolean;
    onClose: () => void;
    onSelect: (account: LinkableAccount) => void;
}) {
    const { data: accounts = [], isLoading, error } = useQuery({
        queryKey: ['linkable-accounts'],
        queryFn: getLinkableAccounts,
        enabled: open,
    });

    const grouped = groupByPortfolio(accounts);

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle>
                <Box display="flex" alignItems="center" gap={1}>
                    <AccountBalance />
                    Link Portfolio Account
                </Box>
            </DialogTitle>
            <DialogContent sx={{ p: 0 }}>
                {isLoading && (
                    <Box display="flex" justifyContent="center" p={4}>
                        <CircularProgress />
                    </Box>
                )}
                {error && (
                    <Alert severity="error" sx={{ m: 2 }}>
                        Failed to load accounts
                    </Alert>
                )}
                {!isLoading && accounts.length === 0 && (
                    <Alert severity="info" sx={{ m: 2 }}>
                        No portfolio accounts found. Import account data first.
                    </Alert>
                )}
                {!isLoading && accounts.length > 0 && (
                    <List dense>
                        {Object.entries(grouped).map(([portfolio, accts], i) => (
                            <Box key={portfolio}>
                                {i > 0 && <Divider />}
                                <ListSubheader sx={{ bgcolor: 'background.paper' }}>
                                    {portfolio}
                                </ListSubheader>
                                {accts.map((account) => (
                                    <ListItemButton key={account.id} onClick={() => onSelect(account)}>
                                        <ListItemText
                                            primary={account.account_nickname}
                                            secondary={account.account_number}
                                        />
                                        <Box textAlign="right">
                                            <Typography variant="body2" fontWeight="medium">
                                                {formatCurrency(account.latest_value)}
                                            </Typography>
                                            {account.latest_snapshot_date && (
                                                <Typography variant="caption" color="text.secondary">
                                                    as of {new Date(account.latest_snapshot_date).toLocaleDateString()}
                                                </Typography>
                                            )}
                                        </Box>
                                    </ListItemButton>
                                ))}
                            </Box>
                        ))}
                    </List>
                )}
            </DialogContent>
            <DialogActions>
                <Button onClick={onClose}>Cancel</Button>
            </DialogActions>
        </Dialog>
    );
}

// ==============================================================================
// Asset Form Dialog (shared for manual + account-linked)
// ==============================================================================

function AssetFormDialog({
    open,
    onClose,
    onSubmit,
    onUnlink,
    editAsset,
    linkedAccount,
    hasSpouse,
    spouseName,
    isSaving,
}: {
    open: boolean;
    onClose: () => void;
    onSubmit: (data: CreateAssetRequest) => void;
    onUnlink?: () => void;
    editAsset: SurveyAsset | null;
    linkedAccount: LinkableAccount | null;
    hasSpouse: boolean;
    spouseName?: string | null;
    isSaving: boolean;
}) {
    const isLinkedNew = !!linkedAccount;
    const isLinkedEdit = !!editAsset?.linked_account_id;

    const [assetType, setAssetType] = useState<AssetType>('investment');
    const [description, setDescription] = useState('');
    const [currentValue, setCurrentValue] = useState('');
    const [notes, setNotes] = useState('');
    const [ownership, setOwnership] = useState<Ownership>('mine');
    const [splitPct, setSplitPct] = useState(50);

    useEffect(() => {
        if (!open) return;
        if (editAsset) {
            setAssetType(editAsset.asset_type);
            setDescription(editAsset.description || '');
            setCurrentValue(editAsset.current_value?.toString() || '');
            setNotes(editAsset.notes || '');
            setOwnership(editAsset.ownership ?? 'mine');
            setSplitPct(editAsset.joint_split_percentage ?? 50);
        } else if (linkedAccount) {
            // Pre-fill from account — smart defaults for type
            setAssetType('investment');
            setDescription(linkedAccount.account_nickname);
            setCurrentValue(linkedAccount.latest_value?.toString() || '0');
            setNotes('');
            setOwnership('mine');
            setSplitPct(50);
        } else {
            setAssetType('liquid');
            setDescription('');
            setCurrentValue('');
            setNotes('');
            setOwnership('mine');
            setSplitPct(50);
        }
    }, [open, editAsset, linkedAccount]);

    const handleSave = () => {
        onSubmit({
            asset_type: assetType,
            description: description || undefined,
            current_value: parseFloat(currentValue) || 0,
            notes: notes || undefined,
            ownership: hasSpouse ? ownership : 'mine',
            joint_split_percentage: hasSpouse && ownership === 'joint' ? splitPct : undefined,
            linked_account_id: linkedAccount?.id ?? editAsset?.linked_account_id ?? undefined,
        });
    };

    const accountName = linkedAccount?.account_nickname ?? editAsset?.linked_account_nickname;

    return (
        <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
            <DialogTitle>
                {editAsset ? 'Edit Asset' : linkedAccount ? 'Link Account as Asset' : 'Add Asset'}
            </DialogTitle>
            <DialogContent>
                <Box display="flex" flexDirection="column" gap={2} mt={1}>
                    {/* Linked account badge */}
                    {(isLinkedNew || isLinkedEdit) && accountName && (
                        <Alert icon={<Link />} severity="info" variant="outlined">
                            Linked to <strong>{accountName}</strong>. Value can be refreshed from the account at any time.
                        </Alert>
                    )}

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
                        placeholder="e.g., RRSP at RBC"
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
                        helperText={isLinkedNew || isLinkedEdit
                            ? "Set from account's latest value — use Refresh to update automatically"
                            : undefined}
                        required
                    />

                    {hasSpouse && (
                        <Box>
                            <Typography variant="body2" color="text.secondary" gutterBottom>
                                Ownership
                            </Typography>
                            <ToggleButtonGroup
                                value={ownership}
                                exclusive
                                onChange={(_, val) => val && setOwnership(val)}
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
                            {ownership === 'joint' && (
                                <Box mt={1.5}>
                                    <Typography variant="body2" gutterBottom>
                                        My share: <strong>{splitPct}%</strong> &nbsp;/&nbsp; {spouseName ?? 'Spouse'}: <strong>{100 - splitPct}%</strong>
                                    </Typography>
                                    <Slider
                                        value={splitPct}
                                        min={1}
                                        max={99}
                                        onChange={(_, val) => setSplitPct(val as number)}
                                        valueLabelDisplay="auto"
                                        valueLabelFormat={(v) => `${v}%`}
                                    />
                                </Box>
                            )}
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

                    {/* Unlink option when editing a linked asset */}
                    {isLinkedEdit && onUnlink && (
                        <Alert
                            severity="warning"
                            variant="outlined"
                            icon={<LinkOff />}
                            action={
                                <Button size="small" color="warning" onClick={onUnlink}>
                                    Unlink
                                </Button>
                            }
                        >
                            Unlink this asset to manage its value manually.
                        </Alert>
                    )}
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
