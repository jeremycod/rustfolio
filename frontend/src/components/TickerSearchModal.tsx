import { useState } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  TextField,
  List,
  ListItem,
  ListItemButton,
  ListItemText,
  Typography,
  Box,
  CircularProgress,
  Alert,
} from '@mui/material';
import { useQuery } from '@tanstack/react-query';
import { searchTickers } from '../lib/endpoints';
import type { TickerMatch } from '../types';

interface TickerSearchModalProps {
  open: boolean;
  onClose: () => void;
  onSelect: (ticker: string) => void;
}

export function TickerSearchModal({ open, onClose, onSelect }: TickerSearchModalProps) {
  const [searchTerm, setSearchTerm] = useState('');

  const searchQuery = useQuery({
    queryKey: ['ticker-search', searchTerm],
    queryFn: () => searchTickers(searchTerm),
    enabled: searchTerm.length >= 2,
  });

  const handleSelect = (ticker: TickerMatch) => {
    onSelect(ticker.symbol);
    onClose();
    setSearchTerm('');
  };

  const handleClose = () => {
    onClose();
    setSearchTerm('');
  };

  return (
    <Dialog open={open} onClose={handleClose} maxWidth="md" fullWidth>
      <DialogTitle>Search Ticker</DialogTitle>
      <DialogContent>
        <TextField
          autoFocus
          fullWidth
          label="Search by company name or ticker"
          variant="outlined"
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          placeholder="e.g. Apple, AAPL"
          sx={{ mb: 2 }}
        />

        {searchTerm.length < 2 && (
          <Typography variant="body2" color="text.secondary">
            Enter at least 2 characters to search
          </Typography>
        )}

        {searchQuery.isLoading && (
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1, py: 2 }}>
            <CircularProgress size={20} />
            <Typography>Searching...</Typography>
          </Box>
        )}

        {searchQuery.isError && (
          <Alert severity="error">Failed to search tickers</Alert>
        )}

        {searchQuery.data && searchQuery.data.length === 0 && (
          <Alert severity="info">No tickers found</Alert>
        )}

        {searchQuery.data && searchQuery.data.length > 0 && (
          <List sx={{ maxHeight: 400, overflow: 'auto' }}>
            {searchQuery.data.map((ticker) => (
              <ListItem key={ticker.symbol} disablePadding>
                <ListItemButton onClick={() => handleSelect(ticker)}>
                  <ListItemText
                    primary={
                      <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                        <Typography variant="body1" fontWeight="bold">
                          {ticker.symbol}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          {ticker.name}
                        </Typography>
                      </Box>
                    }
                    secondary={
                      <Typography variant="caption" color="text.secondary">
                        {ticker._type} • {ticker.region} • {ticker.currency}
                      </Typography>
                    }
                  />
                </ListItemButton>
              </ListItem>
            ))}
          </List>
        )}
      </DialogContent>
    </Dialog>
  );
}