import { useMemo, useState } from 'react';
import {
  Box,
  Grid,
  Card,
  CardContent,
  Typography,
  FormControl,
  InputLabel,
  Select,
  MenuItem,
  Button,
  Alert,
  Chip,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  TextField,
} from '@mui/material';
import { Add, Refresh } from '@mui/icons-material';
import { PieChart, Pie, Cell, ResponsiveContainer, Tooltip } from 'recharts';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  createPortfolio,
  listPortfolios,
  listPositions,
  getAnalytics,
} from '../lib/endpoints';

interface DashboardProps {
  selectedPortfolioId: string | null;
  onPortfolioChange: (id: string) => void;
}

const COLORS = ['#0088FE', '#00C49F', '#FFBB28', '#FF8042', '#8884D8'];

export function Dashboard({ selectedPortfolioId, onPortfolioChange }: DashboardProps) {
  const qc = useQueryClient();
  const [isModalOpen, setIsModalOpen] = useState(false);
  const [portfolioName, setPortfolioName] = useState('');

  const portfoliosQ = useQuery({
    queryKey: ['portfolios'],
    queryFn: listPortfolios,
  });

  const positionsQ = useQuery({
    queryKey: ['positions', selectedPortfolioId],
    queryFn: () => listPositions(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  const analyticsQ = useQuery({
    queryKey: ['analytics', selectedPortfolioId],
    queryFn: () => getAnalytics(selectedPortfolioId!),
    enabled: !!selectedPortfolioId,
  });

  const createPortfolioM = useMutation({
    mutationFn: (name: string) => createPortfolio(name),
    onSuccess: async () => {
      await qc.invalidateQueries({ queryKey: ['portfolios'] });
      setIsModalOpen(false);
      setPortfolioName('');
    },
  });

  const handleCreatePortfolio = () => {
    if (portfolioName.trim()) {
      createPortfolioM.mutate(portfolioName.trim());
    }
  };

  const portfolioValue = useMemo(() => {
    if (!positionsQ.data) return 0;
    return positionsQ.data.reduce((sum, pos) => sum + (parseFloat(pos.shares) * parseFloat(pos.avg_buy_price)), 0);
  }, [positionsQ.data]);

  const allocationData = useMemo(() => {
    if (!positionsQ.data) return [];
    return positionsQ.data.map((pos) => ({
      name: pos.ticker,
      value: parseFloat(pos.shares) * parseFloat(pos.avg_buy_price),
    }));
  }, [positionsQ.data]);

  return (
    <Box>
      <Typography variant="h4" gutterBottom>
        Dashboard
      </Typography>

      {/* Portfolio Selector */}
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
          startIcon={<Add />}
          onClick={() => setIsModalOpen(true)}
        >
          New Portfolio
        </Button>

        <Button
          variant="outlined"
          startIcon={<Refresh />}
          onClick={() => qc.invalidateQueries()}
        >
          Refresh
        </Button>
      </Box>

      {/* KPI Cards */}
      <Grid container spacing={3} sx={{ mb: 3 }}>
        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Total Portfolio Value
              </Typography>
              <Typography variant="h5">
                ${portfolioValue.toFixed(2)}
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Daily Change
              </Typography>
              <Typography variant="h5" color="success.main">
                +$0.00 (0.00%)
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                All-Time P/L
              </Typography>
              <Typography variant="h5" color="success.main">
                +$0.00 (0.00%)
              </Typography>
            </CardContent>
          </Card>
        </Grid>

        <Grid item xs={12} sm={6} md={3}>
          <Card>
            <CardContent>
              <Typography color="textSecondary" gutterBottom>
                Positions
              </Typography>
              <Typography variant="h5">
                {positionsQ.data?.length ?? 0}
              </Typography>
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      <Grid container spacing={3}>
        {/* Portfolio Allocation Chart */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Portfolio Allocation
              </Typography>
              {allocationData.length > 0 ? (
                <Box sx={{ height: 300 }}>
                  <ResponsiveContainer width="100%" height="100%">
                    <PieChart>
                      <Pie
                        data={allocationData}
                        cx="50%"
                        cy="50%"
                        innerRadius={60}
                        outerRadius={100}
                        paddingAngle={5}
                        dataKey="value"
                      >
                        {allocationData.map((entry, index) => (
                          <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                        ))}
                      </Pie>
                      <Tooltip formatter={(value: number) => [`$${value.toFixed(2)}`, 'Value']} />
                    </PieChart>
                  </ResponsiveContainer>
                </Box>
              ) : (
                <Typography color="textSecondary" sx={{ textAlign: 'center', py: 4 }}>
                  No positions to display
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>

        {/* Analytics Summary */}
        <Grid item xs={12} md={6}>
          <Card>
            <CardContent>
              <Typography variant="h6" gutterBottom>
                Analytics Summary
              </Typography>
              {analyticsQ.data ? (
                <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                  <Chip 
                    label={`Data Points: ${analyticsQ.data.meta.points}`} 
                    variant="outlined" 
                  />
                  <Chip 
                    label={`Range: ${analyticsQ.data.meta.start ?? '—'} → ${analyticsQ.data.meta.end ?? '—'}`}
                    variant="outlined"
                  />
                </Box>
              ) : (
                <Typography color="textSecondary">
                  No analytics data available
                </Typography>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {/* Alerts */}
      {(!positionsQ.data || positionsQ.data.length === 0) && (
        <Alert severity="info" sx={{ mt: 3 }}>
          Add at least one position, then generate prices using backend: 
          <Box component="code" sx={{ ml: 1, fontFamily: 'monospace' }}>
            POST /api/prices/AAPL/mock
          </Box>
        </Alert>
      )}

      {/* Create Portfolio Modal */}
      <Dialog open={isModalOpen} onClose={() => setIsModalOpen(false)} maxWidth="sm" fullWidth>
        <DialogTitle>Create New Portfolio</DialogTitle>
        <DialogContent>
          <TextField
            autoFocus
            margin="dense"
            label="Portfolio Name"
            fullWidth
            variant="outlined"
            value={portfolioName}
            onChange={(e) => setPortfolioName(e.target.value)}
            onKeyPress={(e) => e.key === 'Enter' && handleCreatePortfolio()}
          />
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setIsModalOpen(false)}>Cancel</Button>
          <Button 
            onClick={handleCreatePortfolio}
            variant="contained"
            disabled={!portfolioName.trim() || createPortfolioM.isPending}
          >
            Create
          </Button>
        </DialogActions>
      </Dialog>
    </Box>
  );
}