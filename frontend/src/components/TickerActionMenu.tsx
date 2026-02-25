import { useState } from 'react';
import {
  Box,
  Typography,
  IconButton,
  Menu,
  MenuItem,
  ListItemIcon,
  ListItemText,
  Divider,
  Chip,
} from '@mui/material';
import {
  ArrowForward,
  MoreVert,
  TrendingUp,
  ShowChart,
  Psychology,
  Assessment,
  Timeline,
} from '@mui/icons-material';

export type TickerAction =
  | 'risk-analysis'
  | 'volatility-forecast'
  | 'trading-signals'
  | 'sentiment-forecast'
  | 'rolling-beta';

interface TickerActionMenuProps {
  ticker: string;
  variant?: 'text' | 'chip' | 'icon';
  onNavigate: (ticker: string, page: string) => void;
  size?: 'small' | 'medium';
  showLabel?: boolean;
  disabledActions?: TickerAction[];
}

export function TickerActionMenu({
  ticker,
  variant = 'text',
  onNavigate,
  size = 'medium',
  showLabel = true,
  disabledActions = [],
}: TickerActionMenuProps) {
  const [anchorEl, setAnchorEl] = useState<null | HTMLElement>(null);
  const open = Boolean(anchorEl);

  const handleClick = (event: React.MouseEvent<HTMLElement>) => {
    event.stopPropagation();
    setAnchorEl(event.currentTarget);
  };

  const handleClose = () => {
    setAnchorEl(null);
  };

  const handleAction = (page: string) => {
    handleClose();
    onNavigate(ticker, page);
  };

  const actions = [
    {
      id: 'risk-analysis' as TickerAction,
      label: 'Risk Analysis',
      icon: <Assessment fontSize="small" />,
      page: 'risk',
      description: 'View comprehensive risk metrics',
    },
    {
      id: 'volatility-forecast' as TickerAction,
      label: 'Volatility Forecast',
      icon: <ShowChart fontSize="small" />,
      page: 'volatility-forecast',
      description: 'GARCH volatility predictions',
    },
    {
      id: 'trading-signals' as TickerAction,
      label: 'Trading Signals',
      icon: <TrendingUp fontSize="small" />,
      page: 'trading-signals',
      description: 'Buy/sell signal analysis',
    },
    {
      id: 'sentiment-forecast' as TickerAction,
      label: 'Sentiment Forecast',
      icon: <Psychology fontSize="small" />,
      page: 'sentiment-forecast',
      description: 'Market sentiment predictions',
    },
    {
      id: 'rolling-beta' as TickerAction,
      label: 'Rolling Beta',
      icon: <Timeline fontSize="small" />,
      page: 'rolling-beta',
      description: 'Beta correlation over time',
    },
  ];

  const availableActions = actions.filter(
    action => !disabledActions.includes(action.id)
  );

  // Render different variants
  if (variant === 'chip') {
    return (
      <>
        <Chip
          label={ticker}
          size={size}
          onClick={handleClick}
          onDelete={handleClick}
          deleteIcon={<MoreVert />}
          sx={{
            cursor: 'pointer',
            fontWeight: 'bold',
            '&:hover': {
              bgcolor: 'primary.light',
              color: 'primary.contrastText',
            },
          }}
        />
        <Menu
          anchorEl={anchorEl}
          open={open}
          onClose={handleClose}
          onClick={(e) => e.stopPropagation()}
        >
          <MenuItem disabled sx={{ opacity: 1, fontWeight: 'bold' }}>
            <Typography variant="subtitle2">{ticker} - Quick Actions</Typography>
          </MenuItem>
          <Divider />
          {availableActions.map(action => (
            <MenuItem key={action.id} onClick={() => handleAction(action.page)}>
              <ListItemIcon>{action.icon}</ListItemIcon>
              <ListItemText
                primary={action.label}
                secondary={action.description}
                secondaryTypographyProps={{ variant: 'caption' }}
              />
            </MenuItem>
          ))}
        </Menu>
      </>
    );
  }

  if (variant === 'icon') {
    return (
      <>
        <IconButton
          size={size}
          onClick={handleClick}
          sx={{ ml: 0.5 }}
        >
          <MoreVert fontSize="small" />
        </IconButton>
        <Menu
          anchorEl={anchorEl}
          open={open}
          onClose={handleClose}
          onClick={(e) => e.stopPropagation()}
        >
          <MenuItem disabled sx={{ opacity: 1, fontWeight: 'bold' }}>
            <Typography variant="subtitle2">{ticker} - Quick Actions</Typography>
          </MenuItem>
          <Divider />
          {availableActions.map(action => (
            <MenuItem key={action.id} onClick={() => handleAction(action.page)}>
              <ListItemIcon>{action.icon}</ListItemIcon>
              <ListItemText
                primary={action.label}
                secondary={action.description}
                secondaryTypographyProps={{ variant: 'caption' }}
              />
            </MenuItem>
          ))}
        </Menu>
      </>
    );
  }

  // Default: text variant
  return (
    <>
      <Box
        display="inline-flex"
        alignItems="center"
        gap={0.5}
        sx={{
          cursor: 'pointer',
          '&:hover': {
            color: 'primary.main',
          },
        }}
        onClick={handleClick}
      >
        <Typography fontWeight="bold" component="span">
          {ticker}
        </Typography>
        {showLabel && (
          <ArrowForward sx={{ fontSize: 14, opacity: 0.6 }} />
        )}
      </Box>
      <Menu
        anchorEl={anchorEl}
        open={open}
        onClose={handleClose}
        onClick={(e) => e.stopPropagation()}
      >
        <MenuItem disabled sx={{ opacity: 1, fontWeight: 'bold' }}>
          <Typography variant="subtitle2">{ticker} - Quick Actions</Typography>
        </MenuItem>
        <Divider />
        {availableActions.map(action => (
          <MenuItem key={action.id} onClick={() => handleAction(action.page)}>
            <ListItemIcon>{action.icon}</ListItemIcon>
            <ListItemText
              primary={action.label}
              secondary={action.description}
              secondaryTypographyProps={{ variant: 'caption' }}
            />
          </MenuItem>
        ))}
      </Menu>
    </>
  );
}
