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
  CircularProgress,
} from '@mui/material';
import { Upload, Refresh, TrendingUp, TrendingDown, CalendarToday } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  listPortfolios,
  listAccounts,
  importCSV,
  listCsvFiles,
  getLatestHoldings,
} from '../lib/endpoints';
import type { Account } from '../types';
import { formatCurrency, formatPercentage } from '../lib/formatters';

interface AccountsProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
  onAccountSelect?: (accountId: string) => void;
}

export function Accounts({ selectedPortfolioId, onPortfolioChange, onAccountSelect }: AccountsProps) {
  const qc = useQueryClient();
  const [isImportModalOpen, setIsImportModalOpen] = useState(false);
  const [selectedFilePath, setSelectedFilePath] = useState('');

  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const accountsQ = useQuery({
    queryKey: ['accounts', selectedPortfolioId],
    queryFn: () => listAccounts(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  const csvFilesQ = useQuery({
    queryKey: ['csvFiles'],
    queryFn: listCsvFiles,
    enabled: isImportModalOpen,
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

  const importM = useMutation({
    mutationFn: ({ portfolioId, path }: { portfolioId: string; path: string }) =>
      importCSV(portfolioId, path),
    onSuccess: async (data) => {
      await qc.invalidateQueries({ queryKey: ['accounts', selectedPortfolioId] });
      await qc.invalidateQueries({ queryKey: ['allAccountHoldings', selectedPortfolioId] });
      setIsImportModalOpen(false);
      setSelectedFilePath('');

      // Different message for activities vs holdings
      const isActivities = data.snapshot_date === 'N/A';
      const message = isActivities
        ? `Transaction import successful!\n\nTransactions imported: ${data.transactions_detected}\n\n${
            data.errors.length > 0 ? `Errors:\n${data.errors.join('\n')}` : 'No errors'
          }`
        : `Holdings import successful!\n\nAccounts created: ${data.accounts_created}\nHoldings created: ${data.holdings_created}\nTransactions detected: ${data.transactions_detected}\nSnapshot date: ${data.snapshot_date}\n\n${
            data.errors.length > 0 ? `Errors:\n${data.errors.join('\n')}` : 'No errors'
          }`;
      alert(message);
    },
    onError: (error: any) => {
      alert(`Import failed: ${error.response?.data || error.message}`);
    },
  });

  const handleImport = () => {
    if (selectedFilePath && selectedPortfolioId) {
      importM.mutate({ portfolioId: selectedPortfolioId, path: selectedFilePath });
    }
  };

  const handleOpenImportModal = () => {
    setIsImportModalOpen(true);
    setSelectedFilePath('');
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
          No accounts found. Import a CSV file to get started.
        </Alert>
      )}

      {/* Import CSV Modal */}
      <Dialog
        open={isImportModalOpen}
        onClose={() => setIsImportModalOpen(false)}
        maxWidth="sm"
        fullWidth
      >
        <DialogTitle>Import CSV File</DialogTitle>
        <DialogContent>
          <Alert severity="info" sx={{ mb: 3, mt: 1 }}>
            <Typography variant="body2">
              Select a CSV file from the backend/data directory.
              <br />
              <strong>Holdings files</strong>: Import account snapshots (AccountsHoldings-YYYYMMDD.csv)
              <br />
              <strong>Activities files</strong>: Import transactions (AccountActivities-ACCOUNT-YYYYMMDD.csv)
            </Typography>
          </Alert>

          {csvFilesQ.isLoading ? (
            <Box sx={{ display: 'flex', justifyContent: 'center', py: 3 }}>
              <CircularProgress />
            </Box>
          ) : csvFilesQ.data && csvFilesQ.data.length > 0 ? (
            <FormControl fullWidth>
              <InputLabel>CSV File</InputLabel>
              <Select
                value={selectedFilePath}
                onChange={(e) => setSelectedFilePath(e.target.value)}
                label="CSV File"
              >
                {csvFilesQ.data.map((file) => (
                  <MenuItem key={file.path} value={file.path}>
                    <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, width: '100%' }}>
                      <CalendarToday fontSize="small" color="action" />
                      <Box sx={{ flex: 1 }}>
                        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, mb: 0.5 }}>
                          <Chip
                            label={file.file_type === 'holdings' ? 'Holdings' : 'Activities'}
                            size="small"
                            color={file.file_type === 'holdings' ? 'primary' : 'secondary'}
                            sx={{ height: 20, fontSize: '0.7rem' }}
                          />
                          <Typography variant="body2">{file.name}</Typography>
                        </Box>
                        {file.date && (
                          <Typography variant="caption" color="textSecondary">
                            Date: {file.date}
                          </Typography>
                        )}
                      </Box>
                    </Box>
                  </MenuItem>
                ))}
              </Select>
            </FormControl>
          ) : (
            <Alert severity="warning">
              No CSV files found in the backend/data directory. Please add CSV files with formats:
              <br />
              - AccountsHoldings-YYYYMMDD.csv (for holdings snapshots)
              <br />
              - AccountActivities-ACCOUNT-YYYYMMDD.csv (for transaction records)
            </Alert>
          )}
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIsImportModalOpen(false)}>Cancel</Button>
          <Button
            onClick={handleImport}
            variant="contained"
            disabled={!selectedFilePath || importM.isPending}
          >
            {importM.isPending ? 'Importing...' : 'Import'}
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}
