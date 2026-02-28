import { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Paper,
  Button,
  Grid,
  Card,
  CardContent,
  CardActions,
  TextField,
  IconButton,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Chip,
  CircularProgress,
  Alert,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Tabs,
  Tab,
  Tooltip,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Badge,
  Divider,
  InputAdornment,
  Snackbar,
} from '@mui/material';
import { TickerActionMenu } from './TickerActionMenu';
import {
  Visibility,
  Add,
  Delete,
  Edit,
  MoreVert,
  Search,
  NotificationsActive,
  TrendingUp,
  TrendingDown,
  Settings,
  PlaylistAdd,
  CheckCircle,
  Warning,
  Close,
  Refresh,
  FilterList,
} from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  listWatchlists,
  createWatchlist,
  updateWatchlist,
  deleteWatchlist,
  getWatchlistItems,
  addWatchlistItem,
  removeWatchlistItem,
  updateWatchlistThresholds,
  getWatchlistAlerts,
  searchTickers,
  refreshWatchlistPrices,
  listIndexTemplates,
  getIndexTemplate,
  createWatchlistFromTemplate,
} from '../lib/endpoints';
import type {
  Watchlist,
  WatchlistItem,
  WatchlistAlert,
  WatchlistThresholds,
  CreateWatchlistRequest,
  IndexTemplateListItem,
  IndexTemplate,
  CreateWatchlistFromTemplateRequest,
} from '../types';

export function WatchlistPage({ onTickerNavigate }: { onTickerNavigate?: (ticker: string, page?: string) => void }) {
  const [selectedWatchlistId, setSelectedWatchlistId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState(0);
  const [createDialogOpen, setCreateDialogOpen] = useState(false);
  const [templateDialogOpen, setTemplateDialogOpen] = useState(false);
  const [editDialogOpen, setEditDialogOpen] = useState(false);
  const [addStockDialogOpen, setAddStockDialogOpen] = useState(false);
  const [thresholdDialogOpen, setThresholdDialogOpen] = useState(false);
  const [selectedItem, setSelectedItem] = useState<WatchlistItem | null>(null);
  const [deleteConfirmOpen, setDeleteConfirmOpen] = useState(false);
  const [refreshResultOpen, setRefreshResultOpen] = useState(false);
  const [refreshResultMessage, setRefreshResultMessage] = useState('');

  const queryClient = useQueryClient();

  const watchlistsQ = useQuery({
    queryKey: ['watchlists'],
    queryFn: listWatchlists,
  });

  const selectedWatchlist = watchlistsQ.data?.find(w => w.id === selectedWatchlistId) || null;

  const itemsQ = useQuery({
    queryKey: ['watchlist-items', selectedWatchlistId],
    queryFn: () => getWatchlistItems(selectedWatchlistId!),
    enabled: !!selectedWatchlistId,
  });

  const alertsQ = useQuery({
    queryKey: ['watchlist-alerts', selectedWatchlistId],
    queryFn: () => getWatchlistAlerts(selectedWatchlistId!),
    enabled: !!selectedWatchlistId && activeTab === 1,
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => deleteWatchlist(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['watchlists'] });
      setSelectedWatchlistId(null);
      setDeleteConfirmOpen(false);
    },
  });

  const removeItemMutation = useMutation({
    mutationFn: ({ watchlistId, itemId }: { watchlistId: string; itemId: string }) =>
      removeWatchlistItem(watchlistId, itemId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['watchlist-items', selectedWatchlistId] });
      queryClient.invalidateQueries({ queryKey: ['watchlists'] });
    },
  });

  const refreshPricesMutation = useMutation({
    mutationFn: (watchlistId: string) => refreshWatchlistPrices(watchlistId),
    onSuccess: async (data) => {
      // Refetch the items immediately to show updated prices
      await itemsQ.refetch();

      const { refreshed, skipped, failed } = data;
      let message = '';
      if (refreshed > 0) {
        message = `✅ Refreshed ${refreshed} price${refreshed > 1 ? 's' : ''}`;
      }
      if (skipped > 0) {
        message += message ? `, ${skipped} already had data` : `${skipped} already had data`;
      }
      if (failed > 0) {
        message += message ? `, ${failed} failed` : `❌ ${failed} failed`;
      }
      if (!message) {
        message = 'No prices to refresh';
      }
      setRefreshResultMessage(message);
      setRefreshResultOpen(true);
    },
  });

  const unacknowledgedAlerts = alertsQ.data?.filter(a => !a.acknowledged).length || 0;

  return (
    <Box>
      <Box display="flex" alignItems="center" justifyContent="space-between" mb={3}>
        <Box display="flex" alignItems="center" gap={2}>
          <Visibility sx={{ fontSize: 32, color: 'primary.main' }} />
          <Typography variant="h4" fontWeight="bold">
            Watchlists
          </Typography>
        </Box>
        <Box display="flex" gap={1}>
          <Button
            variant="outlined"
            startIcon={<FilterList />}
            onClick={() => setTemplateDialogOpen(true)}
          >
            From Template
          </Button>
          <Button
            variant="contained"
            startIcon={<Add />}
            onClick={() => setCreateDialogOpen(true)}
          >
            New Watchlist
          </Button>
        </Box>
      </Box>

      {watchlistsQ.isLoading && (
        <Box display="flex" justifyContent="center" py={6}>
          <CircularProgress />
        </Box>
      )}

      {watchlistsQ.error && (
        <Alert severity="error">
          Failed to load watchlists: {(watchlistsQ.error as Error).message}
        </Alert>
      )}

      {watchlistsQ.data && !selectedWatchlistId && (
        <WatchlistGrid
          watchlists={watchlistsQ.data}
          onSelect={setSelectedWatchlistId}
          onCreate={() => setCreateDialogOpen(true)}
        />
      )}

      {selectedWatchlist && (
        <Box>
          {/* Watchlist Header */}
          <Paper sx={{ p: 2, mb: 2 }}>
            <Box display="flex" justifyContent="space-between" alignItems="center">
              <Box>
                <Box display="flex" alignItems="center" gap={1}>
                  <Button
                    size="small"
                    onClick={() => setSelectedWatchlistId(null)}
                  >
                    All Watchlists
                  </Button>
                  <Typography variant="body2" color="text.secondary">/</Typography>
                  <Typography variant="h6" fontWeight="bold">
                    {selectedWatchlist.name}
                  </Typography>
                  {selectedWatchlist.is_default && (
                    <Chip label="Default" size="small" color="primary" variant="outlined" />
                  )}
                </Box>
                {selectedWatchlist.description && (
                  <Typography variant="body2" color="text.secondary" mt={0.5}>
                    {selectedWatchlist.description}
                  </Typography>
                )}
              </Box>
              <Box display="flex" gap={1}>
                <Button
                  size="small"
                  startIcon={<Add />}
                  variant="contained"
                  onClick={() => setAddStockDialogOpen(true)}
                >
                  Add Stock
                </Button>
                <Tooltip title="Refresh prices for items with missing data">
                  <IconButton
                    size="small"
                    onClick={() => selectedWatchlistId && refreshPricesMutation.mutate(selectedWatchlistId)}
                    disabled={refreshPricesMutation.isPending}
                    color="primary"
                  >
                    <Refresh fontSize="small" />
                  </IconButton>
                </Tooltip>
                <IconButton
                  size="small"
                  onClick={() => setEditDialogOpen(true)}
                >
                  <Edit fontSize="small" />
                </IconButton>
                <IconButton
                  size="small"
                  color="error"
                  onClick={() => setDeleteConfirmOpen(true)}
                >
                  <Delete fontSize="small" />
                </IconButton>
              </Box>
            </Box>
          </Paper>

          {/* Tabs */}
          <Paper sx={{ mb: 2 }}>
            <Tabs value={activeTab} onChange={(_, v) => setActiveTab(v)}>
              <Tab label={`Stocks (${itemsQ.data?.length || 0})`} />
              <Tab
                label={
                  <Badge badgeContent={unacknowledgedAlerts} color="error">
                    <Box sx={{ pr: unacknowledgedAlerts > 0 ? 1 : 0 }}>Alerts</Box>
                  </Badge>
                }
              />
            </Tabs>
          </Paper>

          {/* Stocks Tab */}
          {activeTab === 0 && (
            <WatchlistItemsTable
              items={itemsQ.data || []}
              loading={itemsQ.isLoading}
              watchlistId={selectedWatchlistId!}
              onRemove={(itemId) => removeItemMutation.mutate({ watchlistId: selectedWatchlistId!, itemId })}
              onConfigureThresholds={(item) => {
                setSelectedItem(item);
                setThresholdDialogOpen(true);
              }}
              onTickerNavigate={onTickerNavigate}
            />
          )}

          {/* Alerts Tab */}
          {activeTab === 1 && (
            <WatchlistAlertsView
              alerts={alertsQ.data || []}
              loading={alertsQ.isLoading}
            />
          )}
        </Box>
      )}

      {/* Dialogs */}
      <CreateWatchlistDialog
        open={createDialogOpen}
        onClose={() => setCreateDialogOpen(false)}
        onCreated={(watchlist) => {
          queryClient.invalidateQueries({ queryKey: ['watchlists'] });
          setSelectedWatchlistId(watchlist.id);
          setCreateDialogOpen(false);
        }}
      />

      <TemplateSelectionDialog
        open={templateDialogOpen}
        onClose={() => setTemplateDialogOpen(false)}
        onCreated={(watchlistId) => {
          queryClient.invalidateQueries({ queryKey: ['watchlists'] });
          setSelectedWatchlistId(watchlistId);
          setTemplateDialogOpen(false);
        }}
      />

      {selectedWatchlist && (
        <EditWatchlistDialog
          open={editDialogOpen}
          onClose={() => setEditDialogOpen(false)}
          watchlist={selectedWatchlist}
          onUpdated={() => {
            queryClient.invalidateQueries({ queryKey: ['watchlists'] });
            setEditDialogOpen(false);
          }}
        />
      )}

      <AddStockDialog
        open={addStockDialogOpen}
        onClose={() => setAddStockDialogOpen(false)}
        watchlistId={selectedWatchlistId || ''}
        onAdded={() => {
          queryClient.invalidateQueries({ queryKey: ['watchlist-items', selectedWatchlistId] });
          queryClient.invalidateQueries({ queryKey: ['watchlists'] });
          setAddStockDialogOpen(false);
        }}
      />

      {selectedItem && (
        <ThresholdDialog
          open={thresholdDialogOpen}
          onClose={() => { setThresholdDialogOpen(false); setSelectedItem(null); }}
          watchlistId={selectedWatchlistId || ''}
          item={selectedItem}
          onUpdated={() => {
            queryClient.invalidateQueries({ queryKey: ['watchlist-items', selectedWatchlistId] });
            setThresholdDialogOpen(false);
            setSelectedItem(null);
          }}
        />
      )}

      {/* Delete Confirmation */}
      <Dialog open={deleteConfirmOpen} onClose={() => setDeleteConfirmOpen(false)}>
        <DialogTitle>Delete Watchlist</DialogTitle>
        <DialogContent>
          <Typography>
            Are you sure you want to delete "{selectedWatchlist?.name}"? This action cannot be undone.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setDeleteConfirmOpen(false)}>Cancel</Button>
          <Button
            color="error"
            variant="contained"
            onClick={() => selectedWatchlistId && deleteMutation.mutate(selectedWatchlistId)}
            disabled={deleteMutation.isPending}
          >
            {deleteMutation.isPending ? 'Deleting...' : 'Delete'}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Refresh Result Notification */}
      <Snackbar
        open={refreshResultOpen}
        autoHideDuration={4000}
        onClose={() => setRefreshResultOpen(false)}
        message={refreshResultMessage}
      />
    </Box>
  );
}

// Watchlist Grid for overview
function WatchlistGrid({
  watchlists,
  onSelect,
  onCreate,
}: {
  watchlists: Watchlist[];
  onSelect: (id: string) => void;
  onCreate: () => void;
}) {
  if (watchlists.length === 0) {
    return (
      <Paper sx={{ p: 6, textAlign: 'center' }}>
        <Visibility sx={{ fontSize: 64, color: 'text.disabled', mb: 2 }} />
        <Typography variant="h6" gutterBottom>
          No Watchlists Yet
        </Typography>
        <Typography variant="body2" color="text.secondary" mb={3}>
          Create a watchlist to monitor your favorite stocks, set custom thresholds, and receive alerts.
        </Typography>
        <Button variant="contained" startIcon={<Add />} onClick={onCreate}>
          Create Your First Watchlist
        </Button>
      </Paper>
    );
  }

  return (
    <Grid container spacing={2}>
      {watchlists.map(wl => (
        <Grid item xs={12} sm={6} md={4} key={wl.id}>
          <Card
            sx={{
              cursor: 'pointer',
              transition: 'box-shadow 0.2s',
              '&:hover': { boxShadow: 4 },
            }}
            onClick={() => onSelect(wl.id)}
          >
            <CardContent>
              <Box display="flex" justifyContent="space-between" alignItems="flex-start">
                <Typography variant="h6" fontWeight="bold">
                  {wl.name}
                </Typography>
                {wl.is_default && (
                  <Chip label="Default" size="small" color="primary" variant="outlined" />
                )}
              </Box>
              {wl.description && (
                <Typography variant="body2" color="text.secondary" mt={0.5}>
                  {wl.description}
                </Typography>
              )}
              <Box display="flex" gap={1} mt={2}>
                <Chip
                  label={`${wl.item_count || 0} stocks`}
                  size="small"
                  variant="outlined"
                />
                <Typography variant="caption" color="text.disabled">
                  Updated {new Date(wl.updated_at).toLocaleDateString()}
                </Typography>
              </Box>
            </CardContent>
          </Card>
        </Grid>
      ))}
      <Grid item xs={12} sm={6} md={4}>
        <Card
          sx={{
            cursor: 'pointer',
            height: '100%',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            border: '2px dashed',
            borderColor: 'divider',
            bgcolor: 'transparent',
            transition: 'border-color 0.2s',
            '&:hover': { borderColor: 'primary.main' },
          }}
          onClick={onCreate}
        >
          <CardContent sx={{ textAlign: 'center' }}>
            <Add sx={{ fontSize: 40, color: 'text.disabled' }} />
            <Typography color="text.secondary">New Watchlist</Typography>
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );
}

// Watchlist Items Table
function WatchlistItemsTable({
  items,
  loading,
  watchlistId,
  onRemove,
  onConfigureThresholds,
  onTickerNavigate,
}: {
  items: WatchlistItem[];
  loading: boolean;
  watchlistId: string;
  onRemove: (itemId: string) => void;
  onConfigureThresholds: (item: WatchlistItem) => void;
  onTickerNavigate?: (ticker: string, page?: string) => void;
}) {
  const [menuAnchor, setMenuAnchor] = useState<{ el: HTMLElement; item: WatchlistItem } | null>(null);

  if (loading) {
    return (
      <Box display="flex" justifyContent="center" py={4}>
        <CircularProgress />
      </Box>
    );
  }

  if (items.length === 0) {
    return (
      <Paper sx={{ p: 4, textAlign: 'center' }}>
        <PlaylistAdd sx={{ fontSize: 48, color: 'text.disabled', mb: 1 }} />
        <Typography color="text.secondary">
          No stocks in this watchlist yet. Click "Add Stock" to start monitoring.
        </Typography>
      </Paper>
    );
  }

  return (
    <TableContainer component={Paper}>
      <Table size="small">
        <TableHead>
          <TableRow sx={{ bgcolor: 'grey.50' }}>
            <TableCell>Symbol</TableCell>
            <TableCell>Company</TableCell>
            <TableCell align="right">Price</TableCell>
            <TableCell align="right">Change</TableCell>
            <TableCell align="center">Risk</TableCell>
            <TableCell>Thresholds</TableCell>
            <TableCell>Notes</TableCell>
            <TableCell align="right">Added</TableCell>
            <TableCell align="center" width={60}>Actions</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {items.map(item => (
            <TableRow key={item.symbol} hover>
              <TableCell>
                {onTickerNavigate ? (
                  <TickerActionMenu
                    ticker={item.symbol}
                    variant="text"
                    onNavigate={onTickerNavigate}
                  />
                ) : (
                  <Typography fontWeight="bold">{item.symbol}</Typography>
                )}
              </TableCell>
              <TableCell>
                <Typography variant="body2" noWrap sx={{ maxWidth: 180 }}>
                  {item.company_name || '--'}
                </Typography>
              </TableCell>
              <TableCell align="right">
                {item.current_price ? `$${item.current_price.toFixed(2)}` : '--'}
              </TableCell>
              <TableCell align="right">
                {item.price_change_pct != null ? (
                  <Box display="flex" alignItems="center" justifyContent="flex-end" gap={0.5}>
                    {item.price_change_pct >= 0 ? (
                      <TrendingUp sx={{ fontSize: 16, color: 'success.main' }} />
                    ) : (
                      <TrendingDown sx={{ fontSize: 16, color: 'error.main' }} />
                    )}
                    <Typography
                      variant="body2"
                      color={item.price_change_pct >= 0 ? 'success.main' : 'error.main'}
                    >
                      {item.price_change_pct >= 0 ? '+' : ''}{item.price_change_pct.toFixed(2)}%
                    </Typography>
                  </Box>
                ) : '--'}
              </TableCell>
              <TableCell align="center">
                {item.risk_level ? (
                  <Chip
                    label={item.risk_level}
                    size="small"
                    color={item.risk_level === 'low' ? 'success' :
                      item.risk_level === 'moderate' ? 'warning' : 'error'}
                    variant="outlined"
                  />
                ) : '--'}
              </TableCell>
              <TableCell>
                {item.thresholds && item.thresholds.length > 0 ? (
                  <Box display="flex" gap={0.5} flexWrap="wrap">
                    {item.thresholds.map((t) => {
                      const label = (() => {
                        switch (t.threshold_type) {
                          case 'price_above':
                            return `High: $${t.value.toFixed(2)}`;
                          case 'price_below':
                            return `Low: $${t.value.toFixed(2)}`;
                          case 'volatility':
                            return `Vol: ${t.value.toFixed(1)}%`;
                          case 'volume_spike':
                            return `Vol Spike: ${t.value.toFixed(1)}x`;
                          case 'rsi_overbought':
                            return `RSI>: ${t.value.toFixed(0)}`;
                          case 'rsi_oversold':
                            return `RSI<: ${t.value.toFixed(0)}`;
                          default:
                            return `${t.threshold_type}: ${t.value}`;
                        }
                      })();
                      return (
                        <Chip
                          key={t.id}
                          label={label}
                          size="small"
                          variant="outlined"
                          color={t.enabled ? 'primary' : 'default'}
                        />
                      );
                    })}
                  </Box>
                ) : (
                  <Typography variant="caption" color="text.disabled">None</Typography>
                )}
              </TableCell>
              <TableCell>
                <Typography variant="body2" noWrap sx={{ maxWidth: 150 }}>
                  {item.notes || '--'}
                </Typography>
              </TableCell>
              <TableCell align="right">
                <Typography variant="caption" color="text.secondary">
                  {new Date(item.added_at).toLocaleDateString()}
                </Typography>
              </TableCell>
              <TableCell align="center">
                <IconButton
                  size="small"
                  onClick={(e) => setMenuAnchor({ el: e.currentTarget, item })}
                >
                  <MoreVert fontSize="small" />
                </IconButton>
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>

      <Menu
        anchorEl={menuAnchor?.el}
        open={!!menuAnchor}
        onClose={() => setMenuAnchor(null)}
      >
        <MenuItem onClick={() => {
          if (menuAnchor) onConfigureThresholds(menuAnchor.item);
          setMenuAnchor(null);
        }}>
          <ListItemIcon><Settings fontSize="small" /></ListItemIcon>
          <ListItemText>Configure Thresholds</ListItemText>
        </MenuItem>
        <MenuItem
          onClick={() => {
            if (menuAnchor) onRemove(menuAnchor.item.id);
            setMenuAnchor(null);
          }}
          sx={{ color: 'error.main' }}
        >
          <ListItemIcon><Delete fontSize="small" color="error" /></ListItemIcon>
          <ListItemText>Remove</ListItemText>
        </MenuItem>
      </Menu>
    </TableContainer>
  );
}

// Alerts View
function WatchlistAlertsView({
  alerts,
  loading,
}: {
  alerts: WatchlistAlert[];
  loading: boolean;
}) {
  if (loading) {
    return (
      <Box display="flex" justifyContent="center" py={4}>
        <CircularProgress />
      </Box>
    );
  }

  if (alerts.length === 0) {
    return (
      <Paper sx={{ p: 4, textAlign: 'center' }}>
        <CheckCircle sx={{ fontSize: 48, color: 'success.main', mb: 1 }} />
        <Typography color="text.secondary">
          No alerts triggered. Your watchlist items are within configured thresholds.
        </Typography>
      </Paper>
    );
  }

  return (
    <TableContainer component={Paper}>
      <Table size="small">
        <TableHead>
          <TableRow sx={{ bgcolor: 'grey.50' }}>
            <TableCell>Symbol</TableCell>
            <TableCell>Alert Type</TableCell>
            <TableCell>Message</TableCell>
            <TableCell align="right">Threshold</TableCell>
            <TableCell align="right">Actual</TableCell>
            <TableCell>Triggered At</TableCell>
            <TableCell align="center">Status</TableCell>
          </TableRow>
        </TableHead>
        <TableBody>
          {alerts.map(alert => (
            <TableRow key={alert.id} hover>
              <TableCell>
                <Typography fontWeight="bold">{alert.symbol}</Typography>
              </TableCell>
              <TableCell>
                <Chip
                  label={alert.alert_type.replace('_', ' ')}
                  size="small"
                  color={
                    alert.alert_type === 'price_target' ? 'primary' :
                    alert.alert_type === 'volatility' ? 'warning' :
                    alert.alert_type === 'sentiment' ? 'info' : 'default'
                  }
                  variant="outlined"
                />
              </TableCell>
              <TableCell>
                <Typography variant="body2">{alert.message}</Typography>
              </TableCell>
              <TableCell align="right">
                {alert.threshold_value?.toFixed(2) ?? '--'}
              </TableCell>
              <TableCell align="right">
                {alert.actual_value?.toFixed(2) ?? '--'}
              </TableCell>
              <TableCell>
                <Typography variant="caption">
                  {new Date(alert.triggered_at).toLocaleString()}
                </Typography>
              </TableCell>
              <TableCell align="center">
                {alert.acknowledged ? (
                  <Chip label="Acknowledged" size="small" color="success" variant="outlined" />
                ) : (
                  <Chip label="New" size="small" color="error" />
                )}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </TableContainer>
  );
}

// Create Watchlist Dialog
function CreateWatchlistDialog({
  open,
  onClose,
  onCreated,
}: {
  open: boolean;
  onClose: () => void;
  onCreated: (watchlist: Watchlist) => void;
}) {
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');

  const mutation = useMutation({
    mutationFn: (data: CreateWatchlistRequest) => createWatchlist(data),
    onSuccess: (watchlist) => {
      setName('');
      setDescription('');
      onCreated(watchlist);
    },
  });

  return (
    <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
      <DialogTitle>Create Watchlist</DialogTitle>
      <DialogContent>
        <TextField
          fullWidth
          label="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          sx={{ mt: 1 }}
          autoFocus
        />
        <TextField
          fullWidth
          label="Description (optional)"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          sx={{ mt: 2 }}
          multiline
          rows={2}
        />
        {mutation.error && (
          <Alert severity="error" sx={{ mt: 2 }}>
            {(mutation.error as Error).message}
          </Alert>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button
          variant="contained"
          onClick={() => mutation.mutate({ name, description: description || undefined })}
          disabled={!name.trim() || mutation.isPending}
        >
          {mutation.isPending ? 'Creating...' : 'Create'}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

// Edit Watchlist Dialog
function EditWatchlistDialog({
  open,
  onClose,
  watchlist,
  onUpdated,
}: {
  open: boolean;
  onClose: () => void;
  watchlist: Watchlist;
  onUpdated: () => void;
}) {
  const [name, setName] = useState(watchlist.name);
  const [description, setDescription] = useState(watchlist.description || '');

  const mutation = useMutation({
    mutationFn: () => updateWatchlist(watchlist.id, {
      name: name !== watchlist.name ? name : undefined,
      description: description !== (watchlist.description || '') ? description : undefined,
    }),
    onSuccess: () => onUpdated(),
  });

  return (
    <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
      <DialogTitle>Edit Watchlist</DialogTitle>
      <DialogContent>
        <TextField
          fullWidth
          label="Name"
          value={name}
          onChange={(e) => setName(e.target.value)}
          sx={{ mt: 1 }}
        />
        <TextField
          fullWidth
          label="Description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          sx={{ mt: 2 }}
          multiline
          rows={2}
        />
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button
          variant="contained"
          onClick={() => mutation.mutate()}
          disabled={!name.trim() || mutation.isPending}
        >
          {mutation.isPending ? 'Saving...' : 'Save'}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

// Add Stock Dialog
function AddStockDialog({
  open,
  onClose,
  watchlistId,
  onAdded,
}: {
  open: boolean;
  onClose: () => void;
  watchlistId: string;
  onAdded: () => void;
}) {
  const [searchTerm, setSearchTerm] = useState('');
  const [notes, setNotes] = useState('');

  const searchQ = useQuery({
    queryKey: ['ticker-search-watchlist', searchTerm],
    queryFn: () => searchTickers(searchTerm),
    enabled: searchTerm.length >= 1,
    staleTime: 1000 * 60 * 5,
  });

  const mutation = useMutation({
    mutationFn: (ticker: string) => addWatchlistItem(watchlistId, {
      ticker,
      notes: notes || undefined,
    }),
    onSuccess: () => {
      setSearchTerm('');
      setNotes('');
      onAdded();
    },
  });

  return (
    <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
      <DialogTitle>Add Stock to Watchlist</DialogTitle>
      <DialogContent>
        <TextField
          fullWidth
          label="Search by ticker or company name"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          sx={{ mt: 1 }}
          autoFocus
          InputProps={{
            startAdornment: (
              <InputAdornment position="start">
                <Search />
              </InputAdornment>
            ),
          }}
        />

        {searchQ.data && searchQ.data.length > 0 && (
          <Paper variant="outlined" sx={{ mt: 1, maxHeight: 200, overflow: 'auto' }}>
            {searchQ.data.slice(0, 10).map((match, idx) => (
              <Box
                key={`${match.symbol}-${idx}`}
                sx={{
                  p: 1.5,
                  cursor: 'pointer',
                  '&:hover': { bgcolor: 'action.hover' },
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  borderBottom: '1px solid',
                  borderColor: 'divider',
                }}
                onClick={() => mutation.mutate(match.symbol)}
              >
                <Box>
                  <Typography variant="body2" fontWeight="bold">
                    {match.symbol}
                  </Typography>
                  <Typography variant="caption" color="text.secondary">
                    {match.name}
                  </Typography>
                </Box>
                <IconButton size="small" color="primary">
                  <Add fontSize="small" />
                </IconButton>
              </Box>
            ))}
          </Paper>
        )}

        <TextField
          fullWidth
          label="Notes (optional)"
          value={notes}
          onChange={(e) => setNotes(e.target.value)}
          sx={{ mt: 2 }}
          multiline
          rows={2}
          placeholder="Add personal notes about this stock..."
        />

        {mutation.error && (
          <Alert severity="error" sx={{ mt: 2 }}>
            {(mutation.error as Error).message}
          </Alert>
        )}
        {mutation.isPending && (
          <Box display="flex" justifyContent="center" mt={2}>
            <CircularProgress size={24} />
          </Box>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Close</Button>
      </DialogActions>
    </Dialog>
  );
}

// Threshold Configuration Dialog
function ThresholdDialog({
  open,
  onClose,
  watchlistId,
  item,
  onUpdated,
}: {
  open: boolean;
  onClose: () => void;
  watchlistId: string;
  item: WatchlistItem;
  onUpdated: () => void;
}) {
  // Convert thresholds array to WatchlistThresholds object
  const existingThresholds = item.thresholds || [];
  const existing: WatchlistThresholds = {};
  existingThresholds.forEach((t) => {
    switch (t.threshold_type) {
      case 'price_above':
        existing.price_target_high = t.value;
        break;
      case 'price_below':
        existing.price_target_low = t.value;
        break;
      case 'volatility':
        existing.volatility_threshold = t.value;
        break;
      case 'volume_spike':
        existing.volume_anomaly_threshold = t.value;
        break;
      case 'rsi_overbought':
        existing.rsi_overbought = t.value;
        break;
      case 'rsi_oversold':
        existing.rsi_oversold = t.value;
        break;
    }
  });

  const [thresholds, setThresholds] = useState<WatchlistThresholds>({
    price_target_high: existing.price_target_high,
    price_target_low: existing.price_target_low,
    volatility_threshold: existing.volatility_threshold,
    volume_anomaly_threshold: existing.volume_anomaly_threshold,
    rsi_overbought: existing.rsi_overbought ?? 70,
    rsi_oversold: existing.rsi_oversold ?? 30,
    sentiment_threshold: existing.sentiment_threshold,
  });

  const mutation = useMutation({
    mutationFn: () => updateWatchlistThresholds(item.id, thresholds),
    onSuccess: () => onUpdated(),
  });

  const updateField = (field: keyof WatchlistThresholds, value: string) => {
    setThresholds(prev => ({
      ...prev,
      [field]: value ? Number(value) : undefined,
    }));
  };

  return (
    <Dialog open={open} onClose={onClose} maxWidth="sm" fullWidth>
      <DialogTitle>
        Configure Thresholds for {item.symbol}
      </DialogTitle>
      <DialogContent>
        <Typography variant="body2" color="text.secondary" mb={2}>
          Set custom alert thresholds. You will be notified when these thresholds are breached.
        </Typography>

        <Grid container spacing={2}>
          <Grid item xs={6}>
            <TextField
              fullWidth
              size="small"
              label="Price Target (High)"
              type="number"
              value={thresholds.price_target_high ?? ''}
              onChange={(e) => updateField('price_target_high', e.target.value)}
              InputProps={{ startAdornment: <InputAdornment position="start">$</InputAdornment> }}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              fullWidth
              size="small"
              label="Price Target (Low)"
              type="number"
              value={thresholds.price_target_low ?? ''}
              onChange={(e) => updateField('price_target_low', e.target.value)}
              InputProps={{ startAdornment: <InputAdornment position="start">$</InputAdornment> }}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              fullWidth
              size="small"
              label="Volatility Threshold (%)"
              type="number"
              value={thresholds.volatility_threshold ?? ''}
              onChange={(e) => updateField('volatility_threshold', e.target.value)}
              InputProps={{ endAdornment: <InputAdornment position="end">%</InputAdornment> }}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              fullWidth
              size="small"
              label="Volume Anomaly Threshold"
              type="number"
              value={thresholds.volume_anomaly_threshold ?? ''}
              onChange={(e) => updateField('volume_anomaly_threshold', e.target.value)}
              InputProps={{ endAdornment: <InputAdornment position="end">x avg</InputAdornment> }}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              fullWidth
              size="small"
              label="RSI Overbought"
              type="number"
              value={thresholds.rsi_overbought ?? ''}
              onChange={(e) => updateField('rsi_overbought', e.target.value)}
            />
          </Grid>
          <Grid item xs={6}>
            <TextField
              fullWidth
              size="small"
              label="RSI Oversold"
              type="number"
              value={thresholds.rsi_oversold ?? ''}
              onChange={(e) => updateField('rsi_oversold', e.target.value)}
            />
          </Grid>
          <Grid item xs={12}>
            <TextField
              fullWidth
              size="small"
              label="Sentiment Alert Threshold"
              type="number"
              value={thresholds.sentiment_threshold ?? ''}
              onChange={(e) => updateField('sentiment_threshold', e.target.value)}
              helperText="Alert when sentiment score changes by more than this amount (0.0-1.0)"
            />
          </Grid>
        </Grid>

        {mutation.error && (
          <Alert severity="error" sx={{ mt: 2 }}>
            {(mutation.error as Error).message}
          </Alert>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose}>Cancel</Button>
        <Button
          variant="contained"
          onClick={() => mutation.mutate()}
          disabled={mutation.isPending}
        >
          {mutation.isPending ? 'Saving...' : 'Save Thresholds'}
        </Button>
      </DialogActions>
    </Dialog>
  );
}

// Template Selection Dialog
function TemplateSelectionDialog({
  open,
  onClose,
  onCreated,
}: {
  open: boolean;
  onClose: () => void;
  onCreated: (watchlistId: string) => void;
}) {
  const [step, setStep] = useState(0); // 0 = template selection, 1 = stock selection
  const [selectedTemplate, setSelectedTemplate] = useState<string | null>(null);
  const [selectedTickers, setSelectedTickers] = useState<string[]>([]);
  const [tickerFilter, setTickerFilter] = useState('');
  const [customName, setCustomName] = useState('');

  const templatesQ = useQuery({
    queryKey: ['index-templates'],
    queryFn: listIndexTemplates,
    enabled: open,
  });

  const templateDetailsQ = useQuery({
    queryKey: ['index-template', selectedTemplate],
    queryFn: () => getIndexTemplate(selectedTemplate!),
    enabled: !!selectedTemplate && step === 1,
  });

  const mutation = useMutation({
    mutationFn: (data: CreateWatchlistFromTemplateRequest) => createWatchlistFromTemplate(data),
    onSuccess: (response) => {
      setStep(0);
      setSelectedTemplate(null);
      setSelectedTickers([]);
      setCustomName('');
      setTickerFilter('');
      onCreated(response.watchlist_id);
    },
  });

  const templates = templatesQ.data || [];
  const selectedTemplateData = templates.find(t => t.id === selectedTemplate);
  const templateDetails = templateDetailsQ.data;

  // Group templates by category
  const templatesByCategory = templates.reduce((acc, template) => {
    if (!acc[template.category]) {
      acc[template.category] = [];
    }
    acc[template.category].push(template);
    return acc;
  }, {} as Record<string, IndexTemplateListItem[]>);

  // Filter tickers based on search
  const filteredTickers = templateDetails?.tickers.filter(ticker =>
    ticker.toLowerCase().includes(tickerFilter.toLowerCase())
  ) || [];

  const handleTemplateSelect = (templateId: string) => {
    setSelectedTemplate(templateId);
  };

  const handleNext = () => {
    if (step === 0 && selectedTemplate) {
      setStep(1);
      // Default to all tickers selected
      const template = templates.find(t => t.id === selectedTemplate);
      if (template) {
        // We'll set them once template details load
      }
    }
  };

  const handleBack = () => {
    setStep(0);
    setTickerFilter('');
  };

  const handleToggleTicker = (ticker: string) => {
    setSelectedTickers(prev =>
      prev.includes(ticker) ? prev.filter(t => t !== ticker) : [...prev, ticker]
    );
  };

  const handleSelectAll = () => {
    setSelectedTickers(templateDetails?.tickers || []);
  };

  const handleSelectNone = () => {
    setSelectedTickers([]);
  };

  const handleSelectTop = (count: number) => {
    setSelectedTickers(templateDetails?.tickers.slice(0, count) || []);
  };

  const handleCreate = () => {
    if (!selectedTemplate) return;
    mutation.mutate({
      template_id: selectedTemplate,
      custom_name: customName || undefined,
      selected_tickers: selectedTickers.length > 0 ? selectedTickers : undefined,
    });
  };

  // Auto-select all tickers when template details load
  useEffect(() => {
    if (templateDetails && selectedTickers.length === 0) {
      setSelectedTickers(templateDetails.tickers);
    }
  }, [templateDetails, selectedTickers.length]);

  return (
    <Dialog open={open} onClose={onClose} maxWidth="lg" fullWidth>
      <DialogTitle>
        {step === 0 ? 'Select Index Template' : 'Select Stocks'}
      </DialogTitle>
      <DialogContent>
        {/* Step 0: Template Selection */}
        {step === 0 && (
          <>
            {templatesQ.isLoading && (
              <Box display="flex" justifyContent="center" py={4}>
                <CircularProgress />
              </Box>
            )}

            {templatesQ.error && (
              <Alert severity="error" sx={{ mb: 2 }}>
                Failed to load templates: {(templatesQ.error as Error).message}
              </Alert>
            )}

            {!templatesQ.isLoading && !templatesQ.error && (
              <>
                <Typography variant="body2" color="text.secondary" mb={2}>
                  Select a predefined index or sector watchlist
                </Typography>

                {Object.entries(templatesByCategory).map(([category, categoryTemplates]) => (
                  <Box key={category} mb={3}>
                    <Typography variant="subtitle2" color="primary" mb={1}>
                      {category}
                    </Typography>
                    <Grid container spacing={2}>
                      {categoryTemplates.map(template => (
                        <Grid item xs={12} sm={6} md={4} key={template.id}>
                          <Card
                            sx={{
                              cursor: 'pointer',
                              border: '2px solid',
                              borderColor: selectedTemplate === template.id ? 'primary.main' : 'divider',
                              '&:hover': { borderColor: 'primary.main' },
                              transition: 'border-color 0.2s',
                            }}
                            onClick={() => handleTemplateSelect(template.id)}
                          >
                            <CardContent>
                              <Box display="flex" justifyContent="space-between" alignItems="flex-start">
                                <Box flex={1}>
                                  <Typography variant="subtitle1" fontWeight="bold">
                                    {template.name}
                                  </Typography>
                                  <Typography variant="body2" color="text.secondary" mt={0.5}>
                                    {template.description}
                                  </Typography>
                                </Box>
                                {selectedTemplate === template.id && (
                                  <CheckCircle color="primary" sx={{ ml: 1 }} />
                                )}
                              </Box>
                              <Chip
                                label={`${template.ticker_count} stocks`}
                                size="small"
                                variant="outlined"
                                sx={{ mt: 1 }}
                              />
                            </CardContent>
                          </Card>
                        </Grid>
                      ))}
                    </Grid>
                  </Box>
                ))}
              </>
            )}
          </>
        )}

        {/* Step 1: Stock Selection */}
        {step === 1 && (
          <>
            {templateDetailsQ.isLoading && (
              <Box display="flex" justifyContent="center" py={4}>
                <CircularProgress />
              </Box>
            )}

            {templateDetails && (
              <>
                <Box mb={2}>
                  <Typography variant="body2" color="text.secondary" gutterBottom>
                    Select stocks to include in your watchlist ({selectedTickers.length} of {templateDetails.tickers.length} selected)
                  </Typography>

                  {/* Quick Selection Buttons */}
                  <Box display="flex" gap={1} flexWrap="wrap" mt={2} mb={2}>
                    <Button
                      size="small"
                      variant={selectedTickers.length === templateDetails.tickers.length ? 'contained' : 'outlined'}
                      onClick={handleSelectAll}
                    >
                      Select All ({templateDetails.tickers.length})
                    </Button>
                    <Button size="small" variant="outlined" onClick={() => handleSelectTop(10)}>
                      Top 10
                    </Button>
                    <Button size="small" variant="outlined" onClick={() => handleSelectTop(25)}>
                      Top 25
                    </Button>
                    <Button size="small" variant="outlined" onClick={() => handleSelectTop(50)}>
                      Top 50
                    </Button>
                    <Button size="small" variant="outlined" color="error" onClick={handleSelectNone}>
                      Clear All
                    </Button>
                  </Box>

                  {/* Search Filter */}
                  <TextField
                    fullWidth
                    size="small"
                    placeholder="Search tickers..."
                    value={tickerFilter}
                    onChange={(e) => setTickerFilter(e.target.value)}
                    InputProps={{
                      startAdornment: (
                        <InputAdornment position="start">
                          <Search />
                        </InputAdornment>
                      ),
                    }}
                    sx={{ mb: 2 }}
                  />

                  {/* Custom Name */}
                  <TextField
                    fullWidth
                    size="small"
                    label="Watchlist Name (optional)"
                    value={customName}
                    onChange={(e) => setCustomName(e.target.value)}
                    placeholder={selectedTemplateData?.name}
                    helperText={`Default: "${selectedTemplateData?.name}"`}
                    sx={{ mb: 2 }}
                  />
                </Box>

                {/* Ticker List with Checkboxes */}
                <Paper variant="outlined" sx={{ maxHeight: 400, overflow: 'auto', p: 2 }}>
                  <Grid container spacing={1}>
                    {filteredTickers.map(ticker => (
                      <Grid item xs={6} sm={4} md={3} lg={2} key={ticker}>
                        <Box
                          sx={{
                            p: 0.5,
                            cursor: 'pointer',
                            borderRadius: 1,
                            border: '1px solid',
                            borderColor: selectedTickers.includes(ticker) ? 'primary.main' : 'divider',
                            bgcolor: selectedTickers.includes(ticker) ? 'primary.50' : 'transparent',
                            '&:hover': { bgcolor: 'action.hover' },
                          }}
                          onClick={() => handleToggleTicker(ticker)}
                        >
                          <Box display="flex" alignItems="center" gap={0.5}>
                            <Typography variant="body2" fontWeight={selectedTickers.includes(ticker) ? 'bold' : 'normal'}>
                              {ticker}
                            </Typography>
                            {selectedTickers.includes(ticker) && (
                              <CheckCircle sx={{ fontSize: 16 }} color="primary" />
                            )}
                          </Box>
                        </Box>
                      </Grid>
                    ))}
                  </Grid>
                </Paper>
              </>
            )}

            {mutation.isPending && (
              <Box mt={3} textAlign="center">
                <CircularProgress size={24} sx={{ mb: 1 }} />
                <Typography variant="body2" color="text.secondary">
                  Creating watchlist and adding {selectedTickers.length} stocks...
                </Typography>
              </Box>
            )}

            {mutation.isSuccess && mutation.data && (
              <Alert severity="success" sx={{ mt: 2 }}>
                ✅ Created "{mutation.data.name}" with {mutation.data.added_count} stocks
                {mutation.data.failed_count > 0 && ` (${mutation.data.failed_count} failed)`}
              </Alert>
            )}

            {mutation.error && (
              <Alert severity="error" sx={{ mt: 2 }}>
                {(mutation.error as Error).message}
              </Alert>
            )}
          </>
        )}
      </DialogContent>
      <DialogActions>
        <Button onClick={onClose} disabled={mutation.isPending}>
          Cancel
        </Button>
        {step === 1 && (
          <Button onClick={handleBack} disabled={mutation.isPending}>
            Back
          </Button>
        )}
        {step === 0 && (
          <Button
            variant="contained"
            onClick={handleNext}
            disabled={!selectedTemplate}
          >
            Next: Select Stocks
          </Button>
        )}
        {step === 1 && (
          <Button
            variant="contained"
            onClick={handleCreate}
            disabled={selectedTickers.length === 0 || mutation.isPending}
          >
            {mutation.isPending ? 'Creating...' : `Create with ${selectedTickers.length} Stocks`}
          </Button>
        )}
      </DialogActions>
    </Dialog>
  );
}

