-- Add optimization cache population job to the schedule
-- This job runs every 6 hours to pre-calculate portfolio optimization recommendations

INSERT INTO job_config (job_name, schedule, max_duration_minutes, enabled)
VALUES
    ('populate_optimization_cache', '0 0 */6 * * *', 120, true)  -- Every 6 hours
ON CONFLICT (job_name) DO UPDATE SET
    schedule = EXCLUDED.schedule,
    max_duration_minutes = EXCLUDED.max_duration_minutes;

COMMENT ON TABLE job_config IS 'Configuration for scheduled background jobs including optimization cache population';
