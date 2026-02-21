import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  Button,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  IconButton,
  Alert,
  CircularProgress,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
  Grid,
} from '@mui/material';
import { Add, Refresh, Edit, Delete, Analytics, Search } from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  createPosition,
  deletePosition,
  listPositions,
  listPortfolios,
  updatePosition,
  updatePrices,
} from '../lib/endpoints';
import { TickerSearchModal } from './TickerSearchModal';
import { RiskBadge } from './RiskBadge';

interface HoldingsProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
}

export function Holdings({ selectedPortfolioId, onPortfolioChange }: HoldingsProps) {
  const qc = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [isEditModalOpen, setIsEditModalOpen] = useState(false);
  const [isTickerSearchOpen, setIsTickerSearchOpen] = useState(false);
  const [editingPosition, setEditingPosition] = useState<any>(null);
  const [formData, setFormData] = useState({
    ticker: '',
    shares: '',
    avg_buy_price: '',
  });
  const [editFormData, setEditFormData] = useState({
    shares: '',
    avg_buy_price: '',
  });

  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const positionsQ = useQuery({
    queryKey: ['positions', selectedPortfolioId],
    queryFn: () => listPositions(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  const createPositionM = useMutation({
    mutationFn: (args: {
      portfolioId: string;
      ticker: string;
      shares: number;
      avg_buy_price: number;
    }) =>
      createPosition(args.portfolioId, {
        ticker: args.ticker,
        shares: args.shares,
        avg_buy_price: args.avg_buy_price,
      }),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ['positions', selectedPortfolioId] });
      setIsModalOpen(false);
      setFormData({ ticker: '', shares: '', avg_buy_price: '' });
    },
  });

  const handleCreatePosition = () => {
    if (selectedPortfolioId && formData.ticker && formData.shares && formData.avg_buy_price) {
      createPositionM.mutate({
        portfolioId: selectedPortfolioId,
        ticker: formData.ticker.toUpperCase(),
        shares: parseInt(formData.shares),
        avg_buy_price: parseFloat(formData.avg_buy_price),
      });
    }
  };

  const handleInputChange = (field: keyof typeof formData) => (e: React.ChangeEvent<HTMLInputElement>) => {
    setFormData(prev => ({ ...prev, [field]: e.target.value }));
  };

  const handleEditInputChange = (field: keyof typeof editFormData) => (e: React.ChangeEvent<HTMLInputElement>) => {
    setEditFormData(prev => ({ ...prev, [field]: e.target.value }));
  };

  const handleTickerSelect = (ticker: string) => {
    setFormData(prev => ({ ...prev, ticker }));
  };

  const handleEditPosition = (position: any) => {
    setEditingPosition(position);
    setEditFormData({
      shares: typeof position.shares === 'string' ? position.shares : position.shares.toString(),
      avg_buy_price: typeof position.avg_buy_price === 'string' ? position.avg_buy_price : position.avg_buy_price.toString(),
    });
    setIsEditModalOpen(true);
  };

  const handleUpdatePosition = () => {
    if (editingPosition && editFormData.shares && editFormData.avg_buy_price) {
      updatePositionM.mutate({
        id: editingPosition.id,
        data: {
          shares: parseInt(editFormData.shares),
          avg_buy_price: parseFloat(editFormData.avg_buy_price),
        },
      });
    }
  };

  const updatePricesM = useMutation({
    mutationFn: (ticker: string) => updatePrices(ticker),
    onSuccess: async () => {
      await qc.invalidateQueries();
    },
  });

  const deletePositionM = useMutation({
    mutationFn: (positionId: string) => deletePosition(positionId),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ['positions', selectedPortfolioId] });
    },
  });

  const updatePositionM = useMutation({
    mutationFn: ({ id, data }: { id: string; data: { shares: number; avg_buy_price: number } }) => 
      updatePosition(id, data),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ['positions', selectedPortfolioId] });
      setIsEditModalOpen(false);
      setEditingPosition(null);
      setEditFormData({ shares: '', avg_buy_price: '' });
    },
  });

  const formatCurrency = (value: number | string) => {
    const num = typeof value === 'string' ? parseFloat(value) : value;
    return `$${num.toFixed(2)}`;
  };
  const formatPercent = (value: number | string) => {
    const num = typeof value === 'string' ? parseFloat(value) : value;
    return `${num.toFixed(2)}%`;
  };

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Holdings
      </Typography>

      {/* Portfolio Selector */}
      <Box sx={{ mb: 3 }}>
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
      </Box>

      <Paper>
        <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', p: 2 }}>
          <Typography variant="h6">Positions</Typography>
          
          {selectedPortfolioId && (
            <Button
              variant="contained"
              startIcon={<Add />}
              onClick={() => setIsModalOpen(true)}
            >
              Add Position
            </Button>
          )}
        </Box>

        {positionsQ.isLoading && (
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, p: 3 }}>
            <CircularProgress size={20} />
            <Typography>Loading positions...</Typography>
          </Box>
        )}

        {positionsQ.isError && (
          <Alert severity="error" sx={{ m: 2 }}>Failed to load positions</Alert>
        )}

        {positionsQ.data && positionsQ.data.length > 0 && (
          <TableContainer>
            <Table>
              <TableHead>
                <TableRow>
                  <TableCell>Ticker</TableCell>
                  <TableCell align="right">Shares</TableCell>
                  <TableCell align="right">Avg Buy Price</TableCell>
                  <TableCell align="right">Current Price</TableCell>
                  <TableCell align="right">Market Value</TableCell>
                  <TableCell align="right">Unrealized P/L</TableCell>
                  <TableCell align="center">Risk</TableCell>
                  <TableCell align="center">Actions</TableCell>
                </TableRow>
              </TableHead>
              <TableBody>
                {positionsQ.data.map((pos: any) => {
                  const shares = parseFloat(pos.shares);
                  const avgBuyPrice = parseFloat(pos.avg_buy_price);
                  const currentPrice = avgBuyPrice * 1.05; // Mock 5% gain
                  const marketValue = shares * currentPrice;
                  const unrealizedPL = shares * (currentPrice - avgBuyPrice);
                  const unrealizedPercent = ((currentPrice - avgBuyPrice) / avgBuyPrice) * 100;
                  const isPositive = unrealizedPL >= 0;

                  return (
                    <TableRow key={pos.id}>
                      <TableCell>
                        <Typography variant="body2" fontWeight="bold">
                          {pos.ticker}
                        </Typography>
                      </TableCell>
                      <TableCell align="right">{pos.shares}</TableCell>
                      <TableCell align="right">{formatCurrency(pos.avg_buy_price)}</TableCell>
                      <TableCell align="right">{formatCurrency(currentPrice)}</TableCell>
                      <TableCell align="right">{formatCurrency(marketValue)}</TableCell>
                      <TableCell align="right">
                        <Box sx={{ color: isPositive ? 'success.main' : 'error.main' }}>
                          <Typography variant="body2">
                            {formatCurrency(unrealizedPL)}
                          </Typography>
                          <Typography variant="caption">
                            ({formatPercent(unrealizedPercent)})
                          </Typography>
                        </Box>
                      </TableCell>
                      <TableCell align="center">
                        <RiskBadge ticker={pos.ticker} />
                      </TableCell>
                      <TableCell align="center">
                        <Box sx={{ display: 'flex', gap: 0.5 }}>
                          <IconButton
                            size="small"
                            onClick={() => updatePricesM.mutate(pos.ticker)}
                            disabled={updatePricesM.isPending}
                            title="Update Price"
                          >
                            <Refresh fontSize="small" />
                          </IconButton>
                          <IconButton 
                            size="small" 
                            title="Edit Position"
                            onClick={() => handleEditPosition(pos)}
                          >
                            <Edit fontSize="small" />
                          </IconButton>
                          <IconButton size="small" title="View Analytics">
                            <Analytics fontSize="small" />
                          </IconButton>
                          <IconButton 
                            size="small" 
                            color="error" 
                            title="Delete Position"
                            onClick={() => deletePositionM.mutate(pos.id)}
                            disabled={deletePositionM.isPending}
                          >
                            <Delete fontSize="small" />
                          </IconButton>
                        </Box>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </TableContainer>
        )}

        {(!positionsQ.data || positionsQ.data.length === 0) && !positionsQ.isLoading && (
          <Alert severity="info" sx={{ m: 2 }}>
            No positions found. Add positions to get started.
          </Alert>
        )}
      </Paper>

      {/* Add Position Modal */}
      <Dialog open={isModalOpen} onClose={() => setIsModalOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Add New Position</DialogTitle>
        <DialogContent>
          <Grid container spacing={2} sx={{ mt: 1 }}>
            <Grid item xs={12}>
              <Box sx={{ display: 'flex', gap: 1 }}>
                <TextField
                  autoFocus
                  label="Ticker Symbol"
                  fullWidth
                  variant="outlined"
                  value={formData.ticker}
                  onChange={handleInputChange('ticker')}
                  placeholder="e.g. AAPL"
                />
                <Button
                  variant="outlined"
                  startIcon={<Search />}
                  onClick={() => setIsTickerSearchOpen(true)}
                  sx={{ minWidth: 'auto', px: 2 }}
                >
                  Search
                </Button>
              </Box>
            </Grid>
            <Grid item xs={6}>
              <TextField
                label="Shares"
                fullWidth
                variant="outlined"
                type="number"
                value={formData.shares}
                onChange={handleInputChange('shares')}
                placeholder="10"
              />
            </Grid>
            <Grid item xs={6}>
              <TextField
                label="Average Buy Price"
                fullWidth
                variant="outlined"
                type="number"
                value={formData.avg_buy_price}
                onChange={handleInputChange('avg_buy_price')}
                placeholder="150.00"
                inputProps={{ step: 0.01 }}
              />
            </Grid>
          </Grid>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIsModalOpen(false)}>Cancel</Button>
          <Button 
            onClick={handleCreatePosition}
            variant="contained"
            disabled={!formData.ticker || !formData.shares || !formData.avg_buy_price || createPositionM.isPending}
          >
            Add Position
          </Button>
        </DialogActions>
      </Dialog>

      {/* Edit Position Modal */}
      <Dialog open={isEditModalOpen} onClose={() => setIsEditModalOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Edit Position - {editingPosition?.ticker}</DialogTitle>
        <DialogContent>
          <Grid container spacing={2} sx={{ mt: 1 }}>
            <Grid item xs={6}>
              <TextField
                autoFocus
                label="Shares"
                fullWidth
                variant="outlined"
                type="number"
                value={editFormData.shares}
                onChange={handleEditInputChange('shares')}
              />
            </Grid>
            <Grid item xs={6}>
              <TextField
                label="Average Buy Price"
                fullWidth
                variant="outlined"
                type="number"
                value={editFormData.avg_buy_price}
                onChange={handleEditInputChange('avg_buy_price')}
                inputProps={{ step: 0.01 }}
              />
            </Grid>
          </Grid>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIsEditModalOpen(false)}>Cancel</Button>
          <Button 
            onClick={handleUpdatePosition}
            variant="contained"
            disabled={!editFormData.shares || !editFormData.avg_buy_price || updatePositionM.isPending}
          >
            Update Position
          </Button>
        </DialogActions>
      </Dialog>

      {/* Ticker Search Modal */}
      <TickerSearchModal
        open={isTickerSearchOpen}
        onClose={() => setIsTickerSearchOpen(false)}
        onSelect={handleTickerSelect}
      />
    </Box>
  );
}