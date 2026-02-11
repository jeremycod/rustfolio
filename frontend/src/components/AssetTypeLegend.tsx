import { Box, Typography, Paper, Grid, Collapse, IconButton } from '@mui/material';
import { ExpandMore, ExpandLess, Info } from '@mui/icons-material';
import { useState } from 'react';
import { AssetTypeChip } from './AssetTypeChip';

export function AssetTypeLegend() {
  const [expanded, setExpanded] = useState(false);

  const assetTypes = [
    { ticker: 'AAPL', name: 'Apple Inc', description: 'Individual stocks/equities' },
    { ticker: 'FID5494', name: 'Fidelity Fund', description: 'Mutual funds' },
    { ticker: 'VDY', name: 'Vanguard ETF', description: 'Exchange-traded funds' },
    { ticker: 'BND', name: 'Bond Fund', description: 'Fixed income securities' },
    { ticker: 'CGL', name: 'Gold ETF', description: 'Commodities and alternatives' },
  ];

  return (
    <Paper sx={{ p: 2, mb: 3, bgcolor: 'background.default' }}>
      <Box
        sx={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          cursor: 'pointer',
        }}
        onClick={() => setExpanded(!expanded)}
      >
        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
          <Info fontSize="small" color="primary" />
          <Typography variant="body2" fontWeight={500}>
            Asset Type Legend
          </Typography>
        </Box>
        <IconButton size="small">
          {expanded ? <ExpandLess /> : <ExpandMore />}
        </IconButton>
      </Box>

      <Collapse in={expanded}>
        <Box sx={{ mt: 2 }}>
          <Grid container spacing={2}>
            {assetTypes.map((asset) => (
              <Grid item xs={12} sm={6} md={4} key={asset.ticker}>
                <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                  <AssetTypeChip
                    ticker={asset.ticker}
                    holdingName={asset.name}
                    showTooltip={false}
                  />
                  <Typography variant="caption" color="text.secondary">
                    {asset.description}
                  </Typography>
                </Box>
              </Grid>
            ))}
          </Grid>
          <Typography variant="caption" color="text.secondary" sx={{ mt: 2, display: 'block' }}>
            Hover over any asset type chip to see more details about the investment type.
          </Typography>
        </Box>
      </Collapse>
    </Paper>
  );
}
