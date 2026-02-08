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
} from '@mui/material';
import { useState } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { resetAllData } from '../lib/endpoints';

export function Settings() {
  const [darkMode, setDarkMode] = useState(false);
  const [notifications, setNotifications] = useState(true);
  const [autoRefresh, setAutoRefresh] = useState(false);
  const [resetDialogOpen, setResetDialogOpen] = useState(false);

  const queryClient = useQueryClient();

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

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Settings
      </Typography>

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