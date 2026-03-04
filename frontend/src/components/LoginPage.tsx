import { useState } from 'react';
import {
    Box, Button, TextField, Typography, Paper, Tab, Tabs, Alert, CircularProgress, Link
} from '@mui/material';
import { useAuth } from '../contexts/AuthContext';
import { requestPasswordReset, resetPassword } from '../lib/endpoints';

export function LoginPage() {
    const [tab, setTab] = useState(0);
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [name, setName] = useState('');
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);
    const { login, register } = useAuth();

    // Forgot password state
    const [showForgot, setShowForgot] = useState(false);
    const [forgotEmail, setForgotEmail] = useState('');
    const [resetToken, setResetToken] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmNewPassword, setConfirmNewPassword] = useState('');
    const [forgotStep, setForgotStep] = useState<'request' | 'reset'>('request');
    const [forgotLoading, setForgotLoading] = useState(false);
    const [forgotError, setForgotError] = useState('');
    const [forgotSuccess, setForgotSuccess] = useState('');

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
            const axiosErr = err as { response?: { data?: string } };
            setError(axiosErr.response?.data || msg);
        } finally {
            setLoading(false);
        }
    };

    const handleRequestReset = async (e: React.FormEvent) => {
        e.preventDefault();
        setForgotError('');
        setForgotSuccess('');
        setForgotLoading(true);
        try {
            const result = await requestPasswordReset(forgotEmail);
            if (result.reset_token) {
                setResetToken(result.reset_token);
                setForgotStep('reset');
                setForgotSuccess('Reset token generated. Enter it below along with your new password.');
            } else {
                setForgotSuccess('If that email is registered, a reset token has been generated. Check the server logs or contact your administrator.');
            }
        } catch (err: unknown) {
            const axiosErr = err as { response?: { data?: string } };
            setForgotError(axiosErr.response?.data || 'Failed to request reset.');
        } finally {
            setForgotLoading(false);
        }
    };

    const handleResetPassword = async (e: React.FormEvent) => {
        e.preventDefault();
        setForgotError('');
        if (newPassword !== confirmNewPassword) {
            setForgotError('Passwords do not match.');
            return;
        }
        if (newPassword.length < 8) {
            setForgotError('Password must be at least 8 characters.');
            return;
        }
        setForgotLoading(true);
        try {
            await resetPassword(resetToken, newPassword);
            setForgotSuccess('Password reset successfully. You can now sign in.');
            setForgotStep('request');
            setShowForgot(false);
            setTab(0);
            setEmail(forgotEmail);
            setForgotEmail('');
            setResetToken('');
            setNewPassword('');
            setConfirmNewPassword('');
        } catch (err: unknown) {
            const axiosErr = err as { response?: { data?: string } };
            setForgotError(axiosErr.response?.data || 'Invalid or expired token.');
        } finally {
            setForgotLoading(false);
        }
    };

    const cancelForgot = () => {
        setShowForgot(false);
        setForgotStep('request');
        setForgotEmail('');
        setResetToken('');
        setNewPassword('');
        setConfirmNewPassword('');
        setForgotError('');
        setForgotSuccess('');
    };

    if (showForgot) {
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
                    <Typography variant="h5" fontWeight={600} mb={1} textAlign="center">
                        Rustfolio
                    </Typography>
                    <Typography variant="h6" mb={3} textAlign="center">
                        Reset Password
                    </Typography>

                    {forgotStep === 'request' ? (
                        <Box
                            component="form"
                            onSubmit={handleRequestReset}
                            display="flex"
                            flexDirection="column"
                            gap={2}
                        >
                            <Typography variant="body2" color="text.secondary">
                                Enter your email address to generate a reset token.
                            </Typography>
                            <TextField
                                label="Email"
                                type="email"
                                value={forgotEmail}
                                onChange={e => setForgotEmail(e.target.value)}
                                required
                                autoComplete="email"
                                fullWidth
                            />
                            {forgotError && <Alert severity="error">{forgotError}</Alert>}
                            {forgotSuccess && <Alert severity="info">{forgotSuccess}</Alert>}
                            <Button
                                type="submit"
                                variant="contained"
                                disabled={forgotLoading}
                                size="large"
                                fullWidth
                            >
                                {forgotLoading ? <CircularProgress size={24} /> : 'Request Reset Token'}
                            </Button>
                            <Link
                                component="button"
                                type="button"
                                variant="body2"
                                onClick={cancelForgot}
                                textAlign="center"
                            >
                                Back to Sign In
                            </Link>
                        </Box>
                    ) : (
                        <Box
                            component="form"
                            onSubmit={handleResetPassword}
                            display="flex"
                            flexDirection="column"
                            gap={2}
                        >
                            {forgotSuccess && <Alert severity="success">{forgotSuccess}</Alert>}
                            <TextField
                                label="Reset Token"
                                value={resetToken}
                                onChange={e => setResetToken(e.target.value)}
                                required
                                fullWidth
                                helperText="Token was pre-filled from the server response"
                            />
                            <TextField
                                label="New Password"
                                type="password"
                                value={newPassword}
                                onChange={e => setNewPassword(e.target.value)}
                                required
                                autoComplete="new-password"
                                helperText="Minimum 8 characters"
                                fullWidth
                            />
                            <TextField
                                label="Confirm New Password"
                                type="password"
                                value={confirmNewPassword}
                                onChange={e => setConfirmNewPassword(e.target.value)}
                                required
                                autoComplete="new-password"
                                fullWidth
                            />
                            {forgotError && <Alert severity="error">{forgotError}</Alert>}
                            <Button
                                type="submit"
                                variant="contained"
                                disabled={forgotLoading}
                                size="large"
                                fullWidth
                            >
                                {forgotLoading ? <CircularProgress size={24} /> : 'Set New Password'}
                            </Button>
                            <Link
                                component="button"
                                type="button"
                                variant="body2"
                                onClick={cancelForgot}
                                textAlign="center"
                            >
                                Back to Sign In
                            </Link>
                        </Box>
                    )}
                </Paper>
            </Box>
        );
    }

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
                    {tab === 0 && (
                        <Box textAlign="right" mt={-1}>
                            <Link
                                component="button"
                                type="button"
                                variant="body2"
                                onClick={() => {
                                    setShowForgot(true);
                                    setForgotEmail(email);
                                }}
                            >
                                Forgot password?
                            </Link>
                        </Box>
                    )}
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
