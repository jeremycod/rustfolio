import { useState } from 'react';
import {
    Box, Button, TextField, Typography, Paper, Tab, Tabs, Alert, CircularProgress
} from '@mui/material';
import { useAuth } from '../contexts/AuthContext';

export function LoginPage() {
    const [tab, setTab] = useState(0);
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [name, setName] = useState('');
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);
    const { login, register } = useAuth();

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        setLoading(true);
        try {
            if (tab === 0) {
                await login(email, password);
            } else {
                await register(email, password, name || undefined);
                await login(email, password);
            }
        } catch (err: unknown) {
            const msg = err instanceof Error ? err.message : 'Authentication failed';
            // Try to extract server error message from axios response
            const axiosErr = err as { response?: { data?: string } };
            setError(axiosErr.response?.data || msg);
        } finally {
            setLoading(false);
        }
    };

    return (
        <Box
            sx={{
                minHeight: '100vh',
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                bgcolor: 'background.default',
            }}
        >
            <Paper sx={{ p: 4, width: '100%', maxWidth: 400 }}>
                <Typography variant="h5" fontWeight={600} mb={2} textAlign="center">
                    Rustfolio
                </Typography>
                <Tabs value={tab} onChange={(_, v) => setTab(v)} centered sx={{ mb: 3 }}>
                    <Tab label="Sign In" />
                    <Tab label="Register" />
                </Tabs>
                <Box component="form" onSubmit={handleSubmit} display="flex" flexDirection="column" gap={2}>
                    {tab === 1 && (
                        <TextField
                            label="Name (optional)"
                            value={name}
                            onChange={e => setName(e.target.value)}
                            autoComplete="name"
                        />
                    )}
                    <TextField
                        label="Email"
                        type="email"
                        value={email}
                        onChange={e => setEmail(e.target.value)}
                        required
                        autoComplete="email"
                    />
                    <TextField
                        label="Password"
                        type="password"
                        value={password}
                        onChange={e => setPassword(e.target.value)}
                        required
                        autoComplete={tab === 0 ? 'current-password' : 'new-password'}
                    />
                    {error && <Alert severity="error">{error}</Alert>}
                    <Button
                        type="submit"
                        variant="contained"
                        disabled={loading}
                        size="large"
                    >
                        {loading ? <CircularProgress size={24} /> : tab === 0 ? 'Sign In' : 'Register'}
                    </Button>
                </Box>
            </Paper>
        </Box>
    );
}
