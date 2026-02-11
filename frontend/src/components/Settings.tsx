import {
  Box,
  Typography,
  Paper,
  FormControlLabel,
  Switch,
  Divider,
  Button,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogContentText,
  DialogActions,
  List,
  ListItem,
  ListItemText,
  ListItemSecondaryAction,
  IconButton,
  Alert,
} from '@mui/material';
import { Delete, Warning } from '@mui/icons-material';
import { useState } from 'react';
import { useMutation, useQueryClient, useQuery } from '@tanstack/react-query';
import { resetAllData, listPortfolios, deletePortfolio } from '../lib/endpoints';
import { RiskThresholdSettings } from './RiskThresholdSettings';
import type { Portfolio } from '../types';

export function Settings() {
  const [darkMode, setDarkMode] = useState(false);
  const [notifications, setNotifications] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [resetDialogOpen, setResetDialogOpen] = useState(false);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [portfolioToDelete, setPortfolioToDelete] = useState<Portfolio | null>(null);

  const queryClient = useQueryClient();

  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const resetMutation = useMutation({
    mutationFn: resetAllData,
    onSuccess: (data) => {
      queryClient.invalidateQueries();
      alert(`Success! ${data.message}\n\nTables cleared: ${data.tables_cleared.join(', ')}`);
      setResetDialogOpen(false);
    },
    onError: (error: any) => {
      alert(`Failed to reset data: ${error.response?.data || error.message}`);
    },
  });

  const deletePortfolioMutation = useMutation({
    mutationFn: deletePortfolio,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['portfolios'] });
      alert(`Portfolio "${portfolioToDelete?.name}" has been deleted successfully.`);
      setDeleteDialogOpen(false);
      setPortfolioToDelete(null);
    },
    onError: (error: any) => {
      alert(`Failed to delete portfolio: ${error.response?.data || error.message}`);
    },
  });

  const handleDeleteClick = (portfolio: Portfolio) => {
    setPortfolioToDelete(portfolio);
    setDeleteDialogOpen(true);
  };

  const handleConfirmDelete = () => {
    if (portfolioToDelete) {
      deletePortfolioMutation.mutate(portfolioToDelete.id);
    }
  };

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Settings
      </Typography>

      {/* Portfolio Management Section */}
      <Paper sx={{ p: 3, mb: 4 }}>
        <Typography variant="h6" gutterBottom>
          Portfolio Management
        </Typography>
        <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
          Manage your portfolios. Deleting a portfolio will permanently remove all associated accounts, holdings, transactions, and cash flows.
        </Typography>

        {portfoliosQ.isLoading && (
          <Typography color="text.secondary">Loading portfolios...</Typography>
        )}

        {portfoliosQ.isError && (
          <Alert severity="error" sx={{ mb: 2 }}>
            Failed to load portfolios
          </Alert>
        )}

        {portfoliosQ.data && portfoliosQ.data.length === 0 && (
          <Alert severity="info">
            No portfolios found. Create a portfolio from the Dashboard.
          </Alert>
        )}

        {portfoliosQ.data && portfoliosQ.data.length > 0 && (
          <>
            <List>
              {portfoliosQ.data.map((portfolio) => (
                <ListItem key={portfolio.id} divider>
                  <ListItemText
                    primary={portfolio.name}
                    secondary={`Created: ${new Date(portfolio.created_at).toLocaleDateString()}`}
                  />
                  <ListItemSecondaryAction>
                    <IconButton
                      edge="end"
                      color="error"
                      onClick={() => handleDeleteClick(portfolio)}
                      disabled={deletePortfolioMutation.isPending}
                    >
                      <Delete />
                    </IconButton>
                  </ListItemSecondaryAction>
                </ListItem>
              ))}
            </List>
            {portfoliosQ.data.length === 1 && (
              <Alert severity="warning" sx={{ mt: 2 }} icon={<Warning />}>
                This is your only portfolio. Deleting it will remove all your data.
              </Alert>
            )}
          </>
        )}
      </Paper>

      {/* Risk Thresholds Section */}
      <Box sx={{ mb: 4 }}>
        <RiskThresholdSettings />
      </Box>

      <Paper sx={{ p: 3 }}>
        <Typography variant="h6" gutterBottom>
          Preferences
        </Typography>

        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          <FormControlLabel
            control={
              <Switch
                checked={darkMode}
                onChange={(e) => setDarkMode(e.target.checked)}
              />
            }
            label="Dark Mode"
          />

          <FormControlLabel
            control={
              <Switch
                checked={notifications}
                onChange={(e) => setNotifications(e.target.checked)}
              />
            }
            label="Enable Notifications"
          />

          <FormControlLabel
            control={
              <Switch
                checked={autoRefresh}
                onChange={(e) => setAutoRefresh(e.target.checked)}
              />
            }
            label="Auto-refresh Data"
          />
        </Box>

        <Divider sx={{ my: 3 }} />

        <Typography variant="h6" gutterBottom>
          Data Management
        </Typography>

        <Box sx={{ display: 'flex', gap: 2 }}>
          <Button
            variant="outlined"
            color="warning"
            onClick={() => {
              queryClient.clear();
              alert('Cache cleared successfully!');
            }}
          >
            Clear Cache
          </Button>
          <Button
            variant="outlined"
            color="error"
            onClick={() => setResetDialogOpen(true)}
            disabled={resetMutation.isPending}
          >
            Reset All Data
          </Button>
        </Box>
      </Paper>

      {/* Delete Portfolio Confirmation Dialog */}
      <Dialog
        open={deleteDialogOpen}
        onClose={() => !deletePortfolioMutation.isPending && setDeleteDialogOpen(false)}
      >
        <DialogTitle>Delete Portfolio?</DialogTitle>
        <DialogContent>
          <DialogContentText>
            Are you sure you want to delete the portfolio <strong>"{portfolioToDelete?.name}"</strong>?
            <br />
            <br />
            This will permanently delete:
            <br />
            • All accounts in this portfolio
            <br />
            • All holdings snapshots
            <br />
            • All transactions
            <br />
            • All cash flows
            <br />
            • All related data
            <br />
            <br />
            <strong>This action cannot be undone!</strong>
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button
            onClick={() => {
              setDeleteDialogOpen(false);
              setPortfolioToDelete(null);
            }}
            disabled={deletePortfolioMutation.isPending}
          >
            Cancel
          </Button>
          <Button
            onClick={handleConfirmDelete}
            color="error"
            variant="contained"
            disabled={deletePortfolioMutation.isPending}
          >
            {deletePortfolioMutation.isPending ? 'Deleting...' : 'Yes, Delete Portfolio'}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Reset Confirmation Dialog */}
      <Dialog
        open={resetDialogOpen}
        onClose={() => setResetDialogOpen(false)}
      >
        <DialogTitle>Reset All Data?</DialogTitle>
        <DialogContent>
          <DialogContentText>
            This will permanently delete ALL data from the database including:
            <br />
            • All portfolios
            <br />
            • All accounts
            <br />
            • All holdings snapshots
            <br />
            • All transactions
            <br />
            • All cash flows
            <br />
            • All price data
            <br />
            <br />
            <strong>This action cannot be undone!</strong>
          </DialogContentText>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setResetDialogOpen(false)}>
            Cancel
          </Button>
          <Button
            onClick={() => resetMutation.mutate()}
            color="error"
            variant="contained"
            disabled={resetMutation.isPending}
          >
            {resetMutation.isPending ? 'Resetting...' : 'Yes, Delete Everything'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}