import { useState } from 'react';
import {
  Box,
  Drawer,
  AppBar,
  Toolbar,
  List,
  Typography,
  ListItem,
  ListItemButton,
  ListItemIcon,
  ListItemText,
  Badge,
} from '@mui/material';
import { useQuery } from '@tanstack/react-query';
import { getUnreadNotificationCount } from '../lib/endpoints';
import {
  Dashboard as DashboardIcon,
  AccountBalance,
  Analytics,
  Settings,
  AccountBalanceWallet,
  Assessment,
  Security,
  Compare,
  GridOn,
  Timeline,
  AdminPanelSettings,
  Notifications as NotificationsIcon,
  NotificationsActive,
  History,
  TrendingDown,
  ShowChart,
  SignalCellularAlt,
  Psychology,
  FilterList,
  Visibility,
  Savings,
  Category,
} from '@mui/icons-material';

const drawerWidth = 240;

const menuItems = [
  { text: 'Dashboard', icon: <DashboardIcon />, path: 'dashboard' },
  { text: 'Accounts', icon: <AccountBalanceWallet />, path: 'accounts' },
  { text: 'Portfolio', icon: <AccountBalance />, path: 'holdings' },
  { text: 'Analytics', icon: <Analytics />, path: 'analytics' },
  { text: 'Portfolio Risk', icon: <Security />, path: 'portfolio-risk' },
  { text: 'Risk Analysis', icon: <Assessment />, path: 'risk' },
  { text: 'Risk Comparison', icon: <Compare />, path: 'risk-comparison' },
  { text: 'Correlations', icon: <GridOn />, path: 'correlations' },
  { text: 'Rolling Beta', icon: <Timeline />, path: 'rolling-beta' },
  // Phase 1 features
  { text: 'CVaR Analysis', icon: <TrendingDown />, path: 'cvar' },
  { text: 'Downside Risk', icon: <TrendingDown />, path: 'downside-risk' },
  { text: 'Market Regime', icon: <ShowChart />, path: 'market-regime' },
  // Phase 2 features
  { text: 'Volatility Forecast', icon: <ShowChart />, path: 'volatility-forecast' },
  { text: 'Trading Signals', icon: <SignalCellularAlt />, path: 'trading-signals' },
  { text: 'Sentiment Forecast', icon: <Psychology />, path: 'sentiment-forecast' },
  // Phase 3 features
  { text: 'Stock Screener', icon: <FilterList />, path: 'screening' },
  { text: 'Watchlists', icon: <Visibility />, path: 'watchlists' },
  { text: 'Long-Term Guidance', icon: <Savings />, path: 'long-term-guidance' },
  { text: 'Factor Portfolio', icon: <Category />, path: 'factor-portfolio' },
  // Notifications & Admin
  { text: 'Alert Rules', icon: <NotificationsActive />, path: 'alerts' },
  { text: 'Notifications', icon: <NotificationsIcon />, path: 'notifications' },
  { text: 'Alert History', icon: <History />, path: 'alert-history' },
  { text: 'Admin', icon: <AdminPanelSettings />, path: 'admin' },
  { text: 'Settings', icon: <Settings />, path: 'settings' },
];

interface LayoutProps {
  children: React.ReactNode;
  currentPage: string;
  onPageChange: (page: string) => void;
}

export function Layout({ children, currentPage, onPageChange }: LayoutProps) {
  // Fetch unread notification count with auto-refresh
  const { data: notificationCount } = useQuery({
    queryKey: ['notificationCount'],
    queryFn: getUnreadNotificationCount,
    refetchInterval: 30000, // Refresh every 30 seconds
  });

  const unreadCount = notificationCount?.unread || 0;

  return (
    <Box sx={{ display: 'flex' }}>
      <AppBar
        position="fixed"
        sx={{ width: `calc(100% - ${drawerWidth}px)`, ml: `${drawerWidth}px` }}
      >
        <Toolbar>
          <Typography variant="h6" noWrap component="div">
            Rustfolio
          </Typography>
        </Toolbar>
      </AppBar>
      
      <Drawer
        sx={{
          width: drawerWidth,
          flexShrink: 0,
          '& .MuiDrawer-paper': {
            width: drawerWidth,
            boxSizing: 'border-box',
          },
        }}
        variant="permanent"
        anchor="left"
      >
        <Toolbar />
        <List>
          {menuItems.map((item) => (
            <ListItem key={item.text} disablePadding>
              <ListItemButton
                selected={currentPage === item.path}
                onClick={() => onPageChange(item.path)}
              >
                <ListItemIcon>
                  {item.path === 'notifications' ? (
                    <Badge badgeContent={unreadCount} color="error">
                      {item.icon}
                    </Badge>
                  ) : (
                    item.icon
                  )}
                </ListItemIcon>
                <ListItemText primary={item.text} />
              </ListItemButton>
            </ListItem>
          ))}
        </List>
      </Drawer>
      
      <Box
        component="main"
        sx={{ flexGrow: 1, bgcolor: 'background.default', p: 3 }}
      >
        <Toolbar />
        {children}
      </Box>
    </Box>
  );
}