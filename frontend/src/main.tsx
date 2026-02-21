import React from "react";
import ReactDOM from "react-dom/client";
import { ThemeProvider, createTheme } from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';
import App from "./App";
import "./index.css";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { PreferencesProvider, usePreferences } from './contexts/PreferencesContext';

const queryClient = new QueryClient();

// Theme wrapper component that uses preferences
function ThemedApp() {
    const { darkMode } = usePreferences();

    const theme = React.useMemo(
        () =>
            createTheme({
                palette: {
                    mode: darkMode ? 'dark' : 'light',
                    primary: {
                        main: '#1976d2',
                    },
                    secondary: {
                        main: '#dc004e',
                    },
                    success: {
                        main: '#2e7d32',
                    },
                    error: {
                        main: '#d32f2f',
                    },
                },
                typography: {
                    fontFamily: 'system-ui, -apple-system, Segoe UI, Roboto, Helvetica, Arial, sans-serif',
                    h1: {
                        fontSize: '2.5rem',
                        fontWeight: 600,
                    },
                    h2: {
                        fontSize: '2rem',
                        fontWeight: 500,
                    },
                },
                components: {
                    MuiButton: {
                        styleOverrides: {
                            root: {
                                textTransform: 'none',
                            },
                        },
                    },
                },
            }),
        [darkMode]
    );

    return (
        <ThemeProvider theme={theme}>
            <CssBaseline />
            <QueryClientProvider client={queryClient}>
                <App />
            </QueryClientProvider>
        </ThemeProvider>
    );
}

ReactDOM.createRoot(document.getElementById("root")!).render(
    <React.StrictMode>
        <PreferencesProvider>
            <ThemedApp />
        </PreferencesProvider>
    </React.StrictMode>
);