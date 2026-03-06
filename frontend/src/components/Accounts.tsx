import { useMemo, useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Typography,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Button,
  Grid,
  Alert,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  LinearProgress,
  Chip,
  TextField,
} from '@mui/material';
import { Upload, Refresh, TrendingUp, TrendingDown, AddCircle } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  listPortfolios,
  listAccounts,
  uploadImportCSV,
  getLatestHoldings,
  createAccount,
} from '../lib/endpoints';
import type { Account } from '../types';
import { formatCurrency, formatPercentage } from '../lib/formatters';

interface AccountsProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
  onAccountSelect?: (accountId: string) => void;
}

function extractDateFromHoldingsFilename(filename: string): string | null {
  // AccountsHoldings-YYYYMMDD.csv → "YYYY-MM-DD"
  const parts = filename.split('-');
  if (parts.length >= 2) {
    const datePart = parts[1].replace('.csv', '');
    if (datePart.length === 8 && /^\d{8}$/.test(datePart)) {
      return `${datePart.slice(0, 4)}-${datePart.slice(4, 6)}-${datePart.slice(6, 8)}`;
    }
  }
  return null;
}

function extractAccountFromActivitiesFilename(filename: string): string | null {
  // AccountActivities-{account_number}-YYYYMMDD.csv → account_number
  const parts = filename.split('-');
  if (parts.length >= 3) {
    return parts[1];
  }
  return null;
}

export function Accounts({ selectedPortfolioId, onPortfolioChange, onAccountSelect }: AccountsProps) {
  const qc = useQueryClient();
  const [isImportModalOpen, setIsImportModalOpen] = useState(false);
  const [importFormat, setImportFormat] = useState<'rj_holdings' | 'rj_activities' | ''>('');
  const [importFile, setImportFile] = useState<File | null>(null);
  const [importSnapshotDate, setImportSnapshotDate] = useState(new Date().toISOString().slice(0, 10));
  const [importAccountId, setImportAccountId] = useState('');
  const [detectedAccountNumber, setDetectedAccountNumber] = useState('');
  const [isCreateAccountOpen, setIsCreateAccountOpen] = useState(false);
  const [newAccountNumber, setNewAccountNumber] = useState('');
  const [newAccountNickname, setNewAccountNickname] = useState('');
  const [newClientName, setNewClientName] = useState('');

  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const accountsQ = useQuery({
    queryKey: ['accounts', selectedPortfolioId],
    queryFn: () => listAccounts(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  // Fetch holdings for all accounts
  const accountHoldingsQueries = useQuery({
    queryKey: ['allAccountHoldings', selectedPortfolioId, accountsQ.data],
    queryFn: async () => {
      if (!accountsQ.data || accountsQ.data.length === 0) return [];

      const holdingsPromises = accountsQ.data.map(async (account) => {
        try {
          const holdings = await getLatestHoldings(account.id);
          return { accountId: account.id, holdings };
        } catch (error) {
          console.error(`Failed to fetch holdings for account ${account.id}:`, error);
          return { accountId: account.id, holdings: [] };
        }
      });

      return Promise.all(holdingsPromises);
    },
    enabled: !!selectedPortfolioId && !!accountsQ.data && accountsQ.data.length > 0,
  });

  const resetImportState = () => {
    setImportFormat('');
    setImportFile(null);
    setImportSnapshotDate(new Date().toISOString().slice(0, 10));
    setImportAccountId('');
    setDetectedAccountNumber('');
  };

  const handleFormatChange = (newFormat: 'rj_holdings' | 'rj_activities') => {
    setImportFormat(newFormat);
    setImportFile(null);
    setImportSnapshotDate(new Date().toISOString().slice(0, 10));
    setImportAccountId('');
    setDetectedAccountNumber('');
  };

  const handleImportFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0] || null;
    setImportFile(file);
    if (!file) return;

    if (importFormat === 'rj_holdings') {
      const date = extractDateFromHoldingsFilename(file.name);
      if (date) setImportSnapshotDate(date);
    } else if (importFormat === 'rj_activities') {
      const accountNum = extractAccountFromActivitiesFilename(file.name);
      setDetectedAccountNumber(accountNum || '');
      if (accountNum && accountsQ.data) {
        const match = accountsQ.data.find((a: Account) => a.account_number === accountNum);
        setImportAccountId(match ? match.id : '');
      } else {
        setImportAccountId('');
      }
    }
  };

  const uploadImportM = useMutation({
    mutationFn: async () => {
      if (!importFile || !selectedPortfolioId || !importFormat) {
        throw new Error('Missing required fields');
      }
      const content = await importFile.text();
      return uploadImportCSV(selectedPortfolioId, {
        filename: importFile.name,
        content,
        format: importFormat,
        account_id: importFormat === 'rj_activities' ? importAccountId || undefined : undefined,
        snapshot_date: importFormat === 'rj_holdings' ? importSnapshotDate : undefined,
      });
    },
    onSuccess: (data) => {
      qc.invalidateQueries({ queryKey: ['accounts', selectedPortfolioId] });
      qc.invalidateQueries({ queryKey: ['allAccountHoldings', selectedPortfolioId] });
      setIsImportModalOpen(false);
      resetImportState();

      const isActivities = data.snapshot_date === 'N/A';
      const message = isActivities
        ? `Transaction import successful!\n\nTransactions imported: ${data.transactions_detected}\n\n${data.errors.length > 0 ? `Errors:\n${data.errors.join('\n')}` : 'No errors'}`
        : `Holdings import successful!\n\nAccounts created: ${data.accounts_created}\nHoldings created: ${data.holdings_created}\nTransactions detected: ${data.transactions_detected}\nSnapshot date: ${data.snapshot_date}\n\n${data.errors.length > 0 ? `Errors:\n${data.errors.join('\n')}` : 'No errors'}`;
      alert(message);
    },
    onError: (error: any) => {
      alert(`Import failed: ${error.response?.data || error.message}`);
    },
  });

  const createAccountM = useMutation({
    mutationFn: (data: { account_number: string; account_nickname: string; client_name?: string }) =>
      createAccount(selectedPortfolioId!, data),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['accounts', selectedPortfolioId] });
      setIsCreateAccountOpen(false);
      setNewAccountNumber('');
      setNewAccountNickname('');
      setNewClientName('');
    },
    onError: (error: any) => {
      alert(`Failed to create account: ${error.response?.data || error.message}`);
    },
  });

  const handleCreateAccount = () => {
    if (newAccountNumber.trim() && newAccountNickname.trim() && selectedPortfolioId) {
      createAccountM.mutate({
        account_number: newAccountNumber.trim(),
        account_nickname: newAccountNickname.trim(),
        client_name: newClientName.trim() || undefined,
      });
    }
  };

  const handleOpenImportModal = () => {
    setIsImportModalOpen(true);
    resetImportState();
  };

  // Calculate account summaries
  const accountSummaries = useMemo(() => {
    if (!accountsQ.data || !accountHoldingsQueries.data) {
      return [];
    }

    return accountsQ.data.map((account) => {
      const accountHoldings = accountHoldingsQueries.data.find(
        (h) => h.accountId === account.id
      );

      if (!accountHoldings || accountHoldings.holdings.length === 0) {
        return {
          account,
          bookValue: 0,
          marketValue: 0,
          gainLoss: 0,
          gainLossPct: 0,
        };
      }

      // Calculate totals from holdings
      const marketValue = accountHoldings.holdings.reduce((sum, holding) => {
        return sum + (parseFloat(holding.market_value) || 0);
      }, 0);

      const gainLoss = accountHoldings.holdings.reduce((sum, holding) => {
        return sum + (parseFloat(holding.gain_loss || '0') || 0);
      }, 0);

      // Book value = Market value - Gain/Loss
      const bookValue = marketValue - gainLoss;
      const gainLossPct = bookValue > 0 ? (gainLoss / bookValue) * 100 : 0;

      return {
        account,
        bookValue,
        marketValue,
        gainLoss,
        gainLossPct,
      };
    });
  }, [accountsQ.data, accountHoldingsQueries.data]);

  const totalSummary = useMemo(() => {
    return accountSummaries.reduce(
      (acc, curr) => ({
        bookValue: acc.bookValue + curr.bookValue,
        marketValue: acc.marketValue + curr.marketValue,
        gainLoss: acc.gainLoss + curr.gainLoss,
      }),
      { bookValue: 0, marketValue: 0, gainLoss: 0 }
    );
  }, [accountSummaries]);

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Accounts
      </Typography>

      {/* Portfolio Selector and Actions */}
      <Box sx={{ display: 'flex', gap: 2, alignItems: 'center', mb: 3 }}>
        <FormControl sx={{ minWidth: 200 }}>
          <InputLabel>Portfolio</InputLabel>
          <Select
            value={selectedPortfolioId ?? ''}
            onChange={(e) => onPortfolioChange(e.target.value)}
            label="Portfolio"
          >
            {(portfoliosQ.data ?? []).map((p) => (
              <MenuItem key={p.id} value={p.id}>
                {p.name}
              </MenuItem>
            ))}
          </Select>
        </FormControl>

        <Button
          variant="contained"
          startIcon={<AddCircle />}
          onClick={() => setIsCreateAccountOpen(true)}
          disabled={!selectedPortfolioId}
        >
          Add Account
        </Button>

        <Button
          variant="outlined"
          startIcon={<Upload />}
          onClick={handleOpenImportModal}
          disabled={!selectedPortfolioId}
        >
          Import CSV
        </Button>

        <Button
          variant="outlined"
          startIcon={<Refresh />}
          onClick={() => qc.invalidateQueries()}
        >
          Refresh
        </Button>
      </Box>

      {/* Summary Cards */}
      {selectedPortfolioId && accountsQ.data && accountsQ.data.length > 0 && (
        <Card sx={{ mb: 3, bgcolor: 'primary.dark', color: 'white' }}>
          <CardContent>
            <Typography variant="h6" gutterBottom>
              Consolidated Summary
            </Typography>
            <Grid container spacing={3}>
              <Grid item xs={12} sm={4}>
                <Typography color="rgba(255,255,255,0.7)" variant="body2">
                  Total Book Value
                </Typography>
                <Typography variant="h5">
                  {formatCurrency(totalSummary.bookValue)}
                </Typography>
              </Grid>
              <Grid item xs={12} sm={4}>
                <Typography color="rgba(255,255,255,0.7)" variant="body2">
                  Total Market Value
                </Typography>
                <Typography variant="h5">
                  {formatCurrency(totalSummary.marketValue)}
                </Typography>
              </Grid>
              <Grid item xs={12} sm={4}>
                <Typography color="rgba(255,255,255,0.7)" variant="body2">
                  Total Gain/Loss
                </Typography>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <Typography variant="h5">
                    {formatCurrency(totalSummary.gainLoss)}
                  </Typography>
                  {totalSummary.gainLoss >= 0 ? (
                    <TrendingUp color="success" />
                  ) : (
                    <TrendingDown color="error" />
                  )}
                </Box>
              </Grid>
            </Grid>
          </CardContent>
        </Card>
      )}

      {/* Account Cards */}
      <Grid container spacing={3}>
        {accountSummaries.map(({ account, bookValue, marketValue, gainLoss, gainLossPct }) => (
          <Grid item xs={12} key={account.id}>
            <Card
              sx={{
                cursor: onAccountSelect ? 'pointer' : 'default',
                '&:hover': onAccountSelect ? { boxShadow: 4 } : {},
              }}
              onClick={() => onAccountSelect?.(account.id)}
            >
              <CardContent>
                <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', mb: 2 }}>
                  <Box>
                    <Typography variant="h6" gutterBottom>
                      {account.account_nickname}
                    </Typography>
                    <Chip label={account.account_number} size="small" variant="outlined" />
                    {account.client_name && (
                      <Typography variant="body2" color="textSecondary" sx={{ mt: 1 }}>
                        {account.client_name}
                      </Typography>
                    )}
                  </Box>
                  <Chip
                    label={gainLoss >= 0 ? 'Gain' : 'Loss'}
                    color={gainLoss >= 0 ? 'success' : 'error'}
                    size="small"
                  />
                </Box>

                <Grid container spacing={2}>
                  <Grid item xs={3}>
                    <Typography variant="body2" color="textSecondary">
                      Book Value
                    </Typography>
                    <Typography variant="h6">{formatCurrency(bookValue)}</Typography>
                  </Grid>
                  <Grid item xs={3}>
                    <Typography variant="body2" color="textSecondary">
                      Market Value
                    </Typography>
                    <Typography variant="h6">{formatCurrency(marketValue)}</Typography>
                  </Grid>
                  <Grid item xs={3}>
                    <Typography variant="body2" color="textSecondary">
                      Gain/Loss
                    </Typography>
                    <Typography
                      variant="h6"
                      color={gainLoss >= 0 ? 'success.main' : 'error.main'}
                    >
                      {formatCurrency(gainLoss)}
                    </Typography>
                  </Grid>
                  <Grid item xs={3}>
                    <Typography variant="body2" color="textSecondary">
                      G/L (%)
                    </Typography>
                    <Typography
                      variant="h6"
                      color={gainLossPct >= 0 ? 'success.main' : 'error.main'}
                    >
                      {formatPercentage(gainLossPct)}
                    </Typography>
                  </Grid>
                </Grid>

                <Box sx={{ mt: 2 }}>
                  <LinearProgress
                    variant="determinate"
                    value={100}
                    sx={{ height: 8, borderRadius: 1 }}
                  />
                </Box>
              </CardContent>
            </Card>
          </Grid>
        ))}
      </Grid>

      {/* Empty State */}
      {selectedPortfolioId && accountsQ.data && accountsQ.data.length === 0 && (
        <Alert severity="info" sx={{ mt: 3 }}>
          No accounts found. Add an account manually or import a CSV file to get started.
        </Alert>
      )}

      {/* Create Account Dialog */}
      <Dialog
        open={isCreateAccountOpen}
        onClose={() => setIsCreateAccountOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Add Account</DialogTitle>
        <DialogContent sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 1 }}>
          <TextField
            label="Account Number"
            value={newAccountNumber}
            onChange={(e) => setNewAccountNumber(e.target.value)}
            fullWidth
            required
            placeholder="e.g. RRSP-1234"
          />
          <TextField
            label="Account Nickname"
            value={newAccountNickname}
            onChange={(e) => setNewAccountNickname(e.target.value)}
            fullWidth
            required
            placeholder="e.g. My RRSP"
          />
          <TextField
            label="Client Name (optional)"
            value={newClientName}
            onChange={(e) => setNewClientName(e.target.value)}
            fullWidth
            placeholder="e.g. John Doe"
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIsCreateAccountOpen(false)}>Cancel</Button>
          <Button
            onClick={handleCreateAccount}
            variant="contained"
            disabled={!newAccountNumber.trim() || !newAccountNickname.trim() || createAccountM.isPending}
          >
            {createAccountM.isPending ? 'Creating...' : 'Create Account'}
          </Button>
        </DialogActions>
      </Dialog>

      {/* Import CSV Modal */}
      <Dialog open={isImportModalOpen} onClose={() => setIsImportModalOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Import CSV</DialogTitle>
        <DialogContent sx={{ display: 'flex', flexDirection: 'column', gap: 3, mt: 1 }}>

          {/* Step 1: Format */}
          <Box>
            <Typography variant="subtitle2" gutterBottom>Format</Typography>
            <Box sx={{ display: 'flex', gap: 1 }}>
              <Button
                variant={importFormat === 'rj_holdings' ? 'contained' : 'outlined'}
                onClick={() => handleFormatChange('rj_holdings')}
                size="small"
              >
                Raymond James — Holdings
              </Button>
              <Button
                variant={importFormat === 'rj_activities' ? 'contained' : 'outlined'}
                onClick={() => handleFormatChange('rj_activities')}
                size="small"
              >
                Raymond James — Activities
              </Button>
            </Box>
          </Box>

          {/* Step 2: File picker (only show after format selected) */}
          {importFormat && (
            <Box>
              <Typography variant="subtitle2" gutterBottom>CSV File</Typography>
              <Button variant="outlined" component="label" startIcon={<Upload />}>
                {importFile ? importFile.name : 'Choose File'}
                <input
                  type="file"
                  accept=".csv"
                  hidden
                  onChange={handleImportFileChange}
                />
              </Button>
              {importFile && (
                <Typography variant="caption" color="text.secondary" sx={{ ml: 1 }}>
                  {(importFile.size / 1024).toFixed(1)} KB
                </Typography>
              )}
            </Box>
          )}

          {/* Step 3: Holdings — snapshot date */}
          {importFormat === 'rj_holdings' && importFile && (
            <TextField
              label="Snapshot Date"
              type="date"
              value={importSnapshotDate}
              onChange={(e) => setImportSnapshotDate(e.target.value)}
              InputLabelProps={{ shrink: true }}
              fullWidth
              helperText="Date of this holdings snapshot (auto-detected from filename)"
            />
          )}

          {/* Step 3: Activities — account picker */}
          {importFormat === 'rj_activities' && importFile && (
            <Box>
              <FormControl fullWidth>
                <InputLabel>Account</InputLabel>
                <Select
                  value={importAccountId}
                  onChange={(e) => setImportAccountId(e.target.value)}
                  label="Account"
                >
                  {(accountsQ.data ?? []).map((a) => (
                    <MenuItem key={a.id} value={a.id}>
                      {a.account_nickname} ({a.account_number})
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
              {detectedAccountNumber && (
                <Typography variant="caption" color={importAccountId ? 'success.main' : 'warning.main'} sx={{ mt: 0.5, display: 'block' }}>
                  {importAccountId
                    ? `Detected from filename: ${detectedAccountNumber}`
                    : `Could not find account "${detectedAccountNumber}" — please select manually`}
                </Typography>
              )}
              {!detectedAccountNumber && (
                <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>
                  Could not detect account from filename — please select manually
                </Typography>
              )}
            </Box>
          )}

        </DialogContent>
        <DialogActions>
          <Button onClick={() => { setIsImportModalOpen(false); resetImportState(); }}>Cancel</Button>
          <Button
            onClick={() => uploadImportM.mutate()}
            variant="contained"
            startIcon={<Upload />}
            disabled={
              !importFile ||
              !importFormat ||
              (importFormat === 'rj_activities' && !importAccountId) ||
              uploadImportM.isPending
            }
          >
            {uploadImportM.isPending ? 'Importing...' : 'Import'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
