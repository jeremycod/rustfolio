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
} from '@mui/material';
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
  { text: 'Admin', icon: <AdminPanelSettings />, path: 'admin' },
  { text: 'Settings', icon: <Settings />, path: 'settings' },
];

interface LayoutProps {
  children: React.ReactNode;
  currentPage: string;
  onPageChange: (page: string) => void;
}

export function Layout({ children, currentPage, onPageChange }: LayoutProps) {
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
                <ListItemIcon>{item.icon}</ListItemIcon>
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