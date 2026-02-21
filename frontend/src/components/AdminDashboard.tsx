import { useState } from 'react';
import {
  Box,
  Typography,
  Paper,
  Tab,
  Tabs,
  Table,
  TableBody,
  TableCell,
  TableContainer,
  TableHead,
  TableRow,
  Button,
  Chip,
  Alert,
  CircularProgress,
  Card,
  CardContent,
  Grid,
  IconButton,
  Tooltip,
  Snackbar,
} from '@mui/material';
import {
  Refresh,
  PlayArrow,
  CheckCircle,
  Error,
  Schedule,
  HourglassEmpty,
  Cancel,
  Storage,
  Warning,
} from '@mui/icons-material';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import {
  listJobs,
  getRecentJobRuns,
  triggerJob,
  getCacheHealth,
} from '../lib/endpoints';
import type { JobStatus, CacheHealthLevel } from '../types';

interface TabPanelProps {
  children?: React.ReactNode;
  index: number;
  value: number;
}

function TabPanel(props: TabPanelProps) {
  const { children, value, index, ...other } = props;

  return (
    <div
      role="tabpanel"
      hidden={value !== index}
      id={`admin-tabpanel-${index}`}
      aria-labelledby={`admin-tab-${index}`}
      {...other}
    >
      {value === index && <Box sx={{ py: 3 }}>{children}</Box>}
    </div>
  );
}

function getStatusIcon(status: JobStatus) {
  switch (status) {
    case 'success':
      return <CheckCircle fontSize="small" color="success" />;
    case 'failed':
      return <Error fontSize="small" color="error" />;
    case 'running':
      return <HourglassEmpty fontSize="small" color="info" />;
    case 'cancelled':
      return <Cancel fontSize="small" color="warning" />;
    default:
      return <Schedule fontSize="small" />;
  }
}

function getStatusColor(status: JobStatus): "default" | "success" | "error" | "warning" | "info" {
  switch (status) {
    case 'success':
      return 'success';
    case 'failed':
      return 'error';
    case 'running':
      return 'info';
    case 'cancelled':
      return 'warning';
    default:
      return 'default';
  }
}

function getCacheHealthColor(status: CacheHealthLevel): "success" | "warning" | "error" {
  switch (status) {
    case 'healthy':
      return 'success';
    case 'degraded':
      return 'warning';
    case 'critical':
      return 'error';
    default:
      return 'warning';
  }
}

function formatDuration(ms: number | null): string {
  if (ms === null) return '-';
  if (ms < 1000) return `${ms}ms`;
  if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
  return `${(ms / 60000).toFixed(1)}m`;
}

function formatDateTime(dateStr: string | null): string {
  if (!dateStr) return '-';
  const date = new Date(dateStr);
  return date.toLocaleString();
}

export function AdminDashboard() {
  const [currentTab, setCurrentTab] = useState(0);
  const [triggeringJob, setTriggeringJob] = useState<string | null>(null);
  const [snackbarOpen, setSnackbarOpen] = useState(false);
  const [snackbarMessage, setSnackbarMessage] = useState('');
  const [snackbarSeverity, setSnackbarSeverity] = useState<'success' | 'error' | 'info'>('info');
  const queryClient = useQueryClient();

  // Fetch jobs list with auto-refresh every 30 seconds
  const jobsQuery = useQuery({
    queryKey: ['admin-jobs'],
    queryFn: listJobs,
    refetchInterval: 30000,
  });

  // Fetch recent job runs
  const recentRunsQuery = useQuery({
    queryKey: ['admin-recent-runs'],
    queryFn: () => getRecentJobRuns(50),
    refetchInterval: 10000, // Refresh every 10 seconds
  });

  // Fetch cache health
  const cacheHealthQuery = useQuery({
    queryKey: ['admin-cache-health'],
    queryFn: getCacheHealth,
    refetchInterval: 60000, // Refresh every minute
  });

  // Mutation for triggering jobs
  const triggerJobMutation = useMutation({
    mutationFn: triggerJob,
    onSuccess: (response: any, jobName) => {
      queryClient.invalidateQueries({ queryKey: ['admin-jobs'] });
      queryClient.invalidateQueries({ queryKey: ['admin-recent-runs'] });

      // Check the actual job status from the response
      if (response.status === 'success') {
        setSnackbarMessage(`✅ ${jobName}: ${response.message}`);
        setSnackbarSeverity('success');
      } else if (response.status === 'failed') {
        setSnackbarMessage(`❌ ${jobName}: ${response.error_message || response.message}`);
        setSnackbarSeverity('error');
      } else {
        setSnackbarMessage(`ℹ️ ${jobName}: ${response.message}`);
        setSnackbarSeverity('info');
      }
      setSnackbarOpen(true);
    },
    onError: (error: any, jobName) => {
      const errorMsg = error.response?.data?.message || error.response?.data || error.message;
      setSnackbarMessage(`Failed to trigger "${jobName}": ${errorMsg}`);
      setSnackbarSeverity('error');
      setSnackbarOpen(true);
    },
    onSettled: () => {
      setTriggeringJob(null);
    },
  });

  const handleTriggerJob = (jobName: string) => {
    setTriggeringJob(jobName);
    triggerJobMutation.mutate(jobName);
  };

  const handleRefresh = () => {
    queryClient.invalidateQueries({ queryKey: ['admin-jobs'] });
    queryClient.invalidateQueries({ queryKey: ['admin-recent-runs'] });
    queryClient.invalidateQueries({ queryKey: ['admin-cache-health'] });
  };

  return (
    <Box>
      <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', mb: 3 }}>
        <Typography variant="h4">Admin Dashboard</Typography>
        <Tooltip title="Refresh all data">
          <IconButton onClick={handleRefresh} color="primary">
            <Refresh />
          </IconButton>
        </Tooltip>
      </Box>

      <Paper sx={{ width: '100%' }}>
        <Tabs
          value={currentTab}
          onChange={(_, newValue) => setCurrentTab(newValue)}
          aria-label="admin dashboard tabs"
        >
          <Tab label="Jobs Overview" />
          <Tab label="Recent Runs" />
          <Tab label="Cache Health" />
          <Tab label="Manual Controls" />
        </Tabs>

        {/* Jobs Overview Tab */}
        <TabPanel value={currentTab} index={0}>
          {jobsQuery.isLoading && (
            <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
              <CircularProgress />
            </Box>
          )}

          {jobsQuery.isError && (
            <Alert severity="error" sx={{ mb: 2 }}>
              Failed to load jobs: {(jobsQuery.error as any)?.message || 'Unknown error'}
            </Alert>
          )}

          {jobsQuery.data && (
            <>
              <Alert severity="info" sx={{ mb: 2 }}>
                Jobs with a <Chip label="Manual" size="small" color="primary" variant="outlined" sx={{ fontSize: '0.65rem', height: '18px', mx: 0.5 }} /> badge
                support manual triggering. Other jobs run automatically on schedule only.
                A "failed" status indicates the last scheduled run failed - you can manually retry analytics jobs.
              </Alert>
              <TableContainer>
              <Table>
                <TableHead>
                  <TableRow>
                    <TableCell>Job Name</TableCell>
                    <TableCell>Schedule</TableCell>
                    <TableCell>Description</TableCell>
                    <TableCell>Last Run</TableCell>
                    <TableCell>Status</TableCell>
                    <TableCell>Next Run</TableCell>
                    <TableCell align="right">Actions</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {jobsQuery.data.map((job) => (
                    <TableRow key={job.name}>
                      <TableCell>
                        <Typography variant="body2" fontWeight="bold">
                          {job.name}
                        </Typography>
                      </TableCell>
                      <TableCell>
                        <Chip label={job.schedule} size="small" variant="outlined" />
                      </TableCell>
                      <TableCell>{job.description}</TableCell>
                      <TableCell>{formatDateTime(job.last_run)}</TableCell>
                      <TableCell>
                        {job.last_status ? (
                          <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                            {getStatusIcon(job.last_status)}
                            <Chip
                              label={job.last_status}
                              size="small"
                              color={getStatusColor(job.last_status)}
                            />
                          </Box>
                        ) : (
                          <Chip label="Not run yet" size="small" variant="outlined" />
                        )}
                      </TableCell>
                      <TableCell>{formatDateTime(job.next_run)}</TableCell>
                      <TableCell align="right">
                        <Button
                          size="small"
                          startIcon={
                            triggeringJob === job.name ? (
                              <CircularProgress size={16} />
                            ) : (
                              <PlayArrow />
                            )
                          }
                          onClick={() => handleTriggerJob(job.name)}
                          disabled={triggeringJob === job.name}
                          variant="outlined"
                        >
                          Run Now
                        </Button>
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
            </>
          )}
        </TabPanel>

        {/* Recent Runs Tab */}
        <TabPanel value={currentTab} index={1}>
          {recentRunsQuery.isLoading && (
            <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
              <CircularProgress />
            </Box>
          )}

          {recentRunsQuery.isError && (
            <Alert severity="error" sx={{ mb: 2 }}>
              Failed to load recent runs: {(recentRunsQuery.error as any)?.message || 'Unknown error'}
            </Alert>
          )}

          {recentRunsQuery.data && (
            <TableContainer>
              <Table size="small">
                <TableHead>
                  <TableRow>
                    <TableCell>Job Name</TableCell>
                    <TableCell>Started</TableCell>
                    <TableCell>Completed</TableCell>
                    <TableCell>Status</TableCell>
                    <TableCell align="right">Duration</TableCell>
                    <TableCell align="right">Processed</TableCell>
                    <TableCell align="right">Failed</TableCell>
                    <TableCell>Error</TableCell>
                  </TableRow>
                </TableHead>
                <TableBody>
                  {recentRunsQuery.data.map((run) => (
                    <TableRow key={run.id}>
                      <TableCell>
                        <Typography variant="body2">{run.job_name}</Typography>
                      </TableCell>
                      <TableCell>{formatDateTime(run.started_at)}</TableCell>
                      <TableCell>{formatDateTime(run.completed_at)}</TableCell>
                      <TableCell>
                        <Box sx={{ display: 'flex', alignItems: 'center', gap: 1 }}>
                          {getStatusIcon(run.status)}
                          <Chip
                            label={run.status}
                            size="small"
                            color={getStatusColor(run.status)}
                          />
                        </Box>
                      </TableCell>
                      <TableCell align="right">{formatDuration(run.duration_ms)}</TableCell>
                      <TableCell align="right">
                        {run.items_processed !== null ? run.items_processed : '-'}
                      </TableCell>
                      <TableCell align="right">
                        {run.items_failed !== null && run.items_failed > 0 ? (
                          <Chip label={run.items_failed} size="small" color="error" />
                        ) : (
                          '-'
                        )}
                      </TableCell>
                      <TableCell>
                        {run.error_message && (
                          <Tooltip title={run.error_message}>
                            <Chip label="View" size="small" color="error" variant="outlined" />
                          </Tooltip>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            </TableContainer>
          )}
        </TabPanel>

        {/* Cache Health Tab */}
        <TabPanel value={currentTab} index={2}>
          {cacheHealthQuery.isLoading && (
            <Box sx={{ display: 'flex', justifyContent: 'center', p: 4 }}>
              <CircularProgress />
            </Box>
          )}

          {cacheHealthQuery.isError && (
            <Alert severity="error" sx={{ mb: 2 }}>
              Failed to load cache health: {(cacheHealthQuery.error as any)?.message || 'Unknown error'}
            </Alert>
          )}

          {cacheHealthQuery.data && (
            <>
              {/* Overall Status Card */}
              <Card sx={{ mb: 3 }}>
                <CardContent>
                  <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, mb: 2 }}>
                    <Storage fontSize="large" color="primary" />
                    <Box>
                      <Typography variant="h6">Overall Cache Health</Typography>
                      <Typography variant="caption" color="text.secondary">
                        Last checked: {formatDateTime(cacheHealthQuery.data.checked_at)}
                      </Typography>
                    </Box>
                    <Box sx={{ ml: 'auto' }}>
                      <Chip
                        label={cacheHealthQuery.data.status.toUpperCase()}
                        color={getCacheHealthColor(cacheHealthQuery.data.status)}
                        size="large"
                        sx={{ fontWeight: 'bold' }}
                      />
                    </Box>
                  </Box>

                  <Grid container spacing={2}>
                    <Grid item xs={12} sm={6} md={3}>
                      <Box sx={{ textAlign: 'center', p: 2, bgcolor: 'background.default', borderRadius: 1 }}>
                        <Typography variant="h4" color="primary">
                          {cacheHealthQuery.data.summary.total_entries}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          Total Entries
                        </Typography>
                      </Box>
                    </Grid>
                    <Grid item xs={12} sm={6} md={3}>
                      <Box sx={{ textAlign: 'center', p: 2, bgcolor: 'background.default', borderRadius: 1 }}>
                        <Typography variant="h4" color="success.main">
                          {cacheHealthQuery.data.summary.freshness_pct.toFixed(1)}%
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          Freshness
                        </Typography>
                      </Box>
                    </Grid>
                    <Grid item xs={12} sm={6} md={3}>
                      <Box sx={{ textAlign: 'center', p: 2, bgcolor: 'background.default', borderRadius: 1 }}>
                        <Typography variant="h4" color="warning.main">
                          {cacheHealthQuery.data.summary.total_stale}
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          Stale Entries
                        </Typography>
                      </Box>
                    </Grid>
                    <Grid item xs={12} sm={6} md={3}>
                      <Box sx={{ textAlign: 'center', p: 2, bgcolor: 'background.default', borderRadius: 1 }}>
                        <Typography variant="h4" color="error.main">
                          {cacheHealthQuery.data.summary.error_rate_pct.toFixed(1)}%
                        </Typography>
                        <Typography variant="body2" color="text.secondary">
                          Error Rate
                        </Typography>
                      </Box>
                    </Grid>
                  </Grid>
                </CardContent>
              </Card>

              {/* Cache Tables Detail */}
              <Typography variant="h6" gutterBottom>
                Cache Tables
              </Typography>
              <TableContainer>
                <Table>
                  <TableHead>
                    <TableRow>
                      <TableCell>Table Name</TableCell>
                      <TableCell align="right">Total</TableCell>
                      <TableCell align="right">Fresh</TableCell>
                      <TableCell align="right">Stale</TableCell>
                      <TableCell align="right">Calculating</TableCell>
                      <TableCell align="right">Errors</TableCell>
                      <TableCell align="right">Avg Age (hrs)</TableCell>
                    </TableRow>
                  </TableHead>
                  <TableBody>
                    {cacheHealthQuery.data.tables.map((table) => (
                      <TableRow key={table.table_name}>
                        <TableCell>
                          <Typography variant="body2" fontWeight="bold">
                            {table.table_name}
                          </Typography>
                        </TableCell>
                        <TableCell align="right">{table.total_entries}</TableCell>
                        <TableCell align="right">
                          <Chip label={table.fresh_entries} size="small" color="success" />
                        </TableCell>
                        <TableCell align="right">
                          {table.stale_entries > 0 ? (
                            <Chip label={table.stale_entries} size="small" color="warning" />
                          ) : (
                            '-'
                          )}
                        </TableCell>
                        <TableCell align="right">
                          {table.calculating_entries > 0 ? (
                            <Chip label={table.calculating_entries} size="small" color="info" />
                          ) : (
                            '-'
                          )}
                        </TableCell>
                        <TableCell align="right">
                          {table.error_entries > 0 ? (
                            <Chip label={table.error_entries} size="small" color="error" />
                          ) : (
                            '-'
                          )}
                        </TableCell>
                        <TableCell align="right">
                          {table.avg_age_hours !== null
                            ? table.avg_age_hours.toFixed(1)
                            : '-'}
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </TableContainer>
            </>
          )}
        </TabPanel>

        {/* Manual Controls Tab */}
        <TabPanel value={currentTab} index={3}>
          <Alert severity="info" sx={{ mb: 3 }}>
            <Typography variant="body2" fontWeight="bold" gutterBottom>
              Manual Job Controls
            </Typography>
            <Typography variant="body2">
              All jobs can be manually triggered from this interface for testing and maintenance purposes.
              Results appear in the Recent Runs tab. Jobs also run automatically on their configured schedules.
            </Typography>
          </Alert>

          {jobsQuery.data && (
            <Grid container spacing={2}>
              {jobsQuery.data.map((job) => (
                <Grid item xs={12} sm={6} md={4} key={job.name}>
                  <Card>
                    <CardContent>
                      <Typography variant="h6" gutterBottom>
                        {job.name}
                      </Typography>
                      <Typography variant="body2" color="text.secondary" paragraph>
                        {job.description}
                      </Typography>
                      <Box sx={{ display: 'flex', gap: 1, mb: 2 }}>
                        <Chip label={job.schedule} size="small" />
                        {job.last_status && (
                          <Chip
                            label={job.last_status}
                            size="small"
                            color={getStatusColor(job.last_status)}
                          />
                        )}
                      </Box>
                      <Button
                        fullWidth
                        variant="contained"
                        startIcon={
                          triggeringJob === job.name ? (
                            <CircularProgress size={16} />
                          ) : (
                            <PlayArrow />
                          )
                        }
                        onClick={() => handleTriggerJob(job.name)}
                        disabled={triggeringJob === job.name}
                      >
                        {triggeringJob === job.name ? 'Triggering...' : 'Run Now'}
                      </Button>
                    </CardContent>
                  </Card>
                </Grid>
              ))}
            </Grid>
          )}
        </TabPanel>
      </Paper>

      {/* Snackbar for job trigger feedback */}
      <Snackbar
        open={snackbarOpen}
        autoHideDuration={6000}
        onClose={() => setSnackbarOpen(false)}
        anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
      >
        <Alert
          onClose={() => setSnackbarOpen(false)}
          severity={snackbarSeverity}
          variant="filled"
          sx={{ width: '100%' }}
        >
          {snackbarMessage}
        </Alert>
      </Snackbar>
    </Box>
  );
}
