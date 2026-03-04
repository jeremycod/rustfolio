import { useState } from 'react';
import {
    Box,
    Typography,
    TextField,
    Button,
    Paper,
    Alert,
    CircularProgress,
    Divider,
} from '@mui/material';
import { useAuth } from '../contexts/AuthContext';

export function ProfilePage() {
    const { user, logout, updateProfile, changePassword } = useAuth();

    // Profile edit state
    const [editEmail, setEditEmail] = useState(user?.email ?? '');
    const [editName, setEditName] = useState(user?.name ?? '');
    const [profileLoading, setProfileLoading] = useState(false);
    const [profileSuccess, setProfileSuccess] = useState('');
    const [profileError, setProfileError] = useState('');

    // Change password state
    const [currentPassword, setCurrentPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [passwordLoading, setPasswordLoading] = useState(false);
    const [passwordSuccess, setPasswordSuccess] = useState('');
    const [passwordError, setPasswordError] = useState('');

    // Logout state
    const [logoutLoading, setLogoutLoading] = useState(false);

    const handleProfileSave = async (e: React.FormEvent) => {
        e.preventDefault();
        setProfileError('');
        setProfileSuccess('');
        setProfileLoading(true);
        try {
            await updateProfile({
                email: editEmail,
                name: editName || undefined,
            });
            setProfileSuccess('Profile updated successfully.');
        } catch (err: unknown) {
            const axiosErr = err as { response?: { data?: string } };
            setProfileError(axiosErr.response?.data || 'Failed to update profile.');
        } finally {
            setProfileLoading(false);
        }
    };

    const handleChangePassword = async (e: React.FormEvent) => {
        e.preventDefault();
        setPasswordError('');
        setPasswordSuccess('');
        if (newPassword !== confirmPassword) {
            setPasswordError('New passwords do not match.');
            return;
        }
        if (newPassword.length < 8) {
            setPasswordError('Password must be at least 8 characters.');
            return;
        }
        setPasswordLoading(true);
        try {
            await changePassword(currentPassword, newPassword);
            setPasswordSuccess('Password changed successfully.');
            setCurrentPassword('');
            setNewPassword('');
            setConfirmPassword('');
        } catch (err: unknown) {
            const axiosErr = err as { response?: { data?: string } };
            setPasswordError(axiosErr.response?.data || 'Failed to change password.');
        } finally {
            setPasswordLoading(false);
        }
    };

    const handleLogout = async () => {
        setLogoutLoading(true);
        try {
            await logout();
        } finally {
            setLogoutLoading(false);
        }
    };

    return (
        <Box maxWidth={560}>
            <Typography variant="h5" fontWeight={600} mb={3}>
                Profile
            </Typography>

            {/* Account Info */}
            <Paper sx={{ p: 3, mb: 3 }}>
                <Typography variant="h6" mb={2}>Account Info</Typography>
                <Box
                    component="form"
                    onSubmit={handleProfileSave}
                    display="flex"
                    flexDirection="column"
                    gap={2}
                >
                    <TextField
                        label="Name (optional)"
                        value={editName}
                        onChange={e => setEditName(e.target.value)}
                        autoComplete="name"
                        fullWidth
                    />
                    <TextField
                        label="Email"
                        type="email"
                        value={editEmail}
                        onChange={e => setEditEmail(e.target.value)}
                        required
                        autoComplete="email"
                        fullWidth
                    />
                    {profileError && <Alert severity="error">{profileError}</Alert>}
                    {profileSuccess && <Alert severity="success">{profileSuccess}</Alert>}
                    <Button
                        type="submit"
                        variant="contained"
                        disabled={profileLoading}
                        sx={{ alignSelf: 'flex-start' }}
                    >
                        {profileLoading ? <CircularProgress size={22} /> : 'Save Changes'}
                    </Button>
                </Box>
            </Paper>

            {/* Change Password */}
            <Paper sx={{ p: 3, mb: 3 }}>
                <Typography variant="h6" mb={2}>Change Password</Typography>
                <Box
                    component="form"
                    onSubmit={handleChangePassword}
                    display="flex"
                    flexDirection="column"
                    gap={2}
                >
                    <TextField
                        label="Current Password"
                        type="password"
                        value={currentPassword}
                        onChange={e => setCurrentPassword(e.target.value)}
                        required
                        autoComplete="current-password"
                        fullWidth
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
                        value={confirmPassword}
                        onChange={e => setConfirmPassword(e.target.value)}
                        required
                        autoComplete="new-password"
                        fullWidth
                    />
                    {passwordError && <Alert severity="error">{passwordError}</Alert>}
                    {passwordSuccess && <Alert severity="success">{passwordSuccess}</Alert>}
                    <Button
                        type="submit"
                        variant="contained"
                        disabled={passwordLoading}
                        sx={{ alignSelf: 'flex-start' }}
                    >
                        {passwordLoading ? <CircularProgress size={22} /> : 'Change Password'}
                    </Button>
                </Box>
            </Paper>

            {/* Session / Logout */}
            <Paper sx={{ p: 3 }}>
                <Typography variant="h6" mb={1}>Session</Typography>
                <Typography variant="body2" color="text.secondary" mb={2}>
                    Signed in as {user?.email}
                </Typography>
                <Divider sx={{ mb: 2 }} />
                <Button
                    variant="outlined"
                    color="error"
                    onClick={handleLogout}
                    disabled={logoutLoading}
                >
                    {logoutLoading ? <CircularProgress size={22} /> : 'Sign Out'}
                </Button>
            </Paper>
        </Box>
    );
}
