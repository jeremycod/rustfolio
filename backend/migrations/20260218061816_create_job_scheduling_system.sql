-- Sprint 19: Job Scheduling System
-- Check if job_status type exists, create if not
DO $$ BEGIN
    CREATE TYPE job_status AS ENUM ('running', 'success', 'failed', 'cancelled');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create job_runs table if not exists
CREATE TABLE IF NOT EXISTS job_runs (
    id SERIAL PRIMARY KEY,
    job_name VARCHAR(100) NOT NULL,
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    completed_at TIMESTAMP,
    status job_status NOT NULL DEFAULT 'running',
    error_message TEXT,
    items_processed INTEGER DEFAULT 0,
    items_failed INTEGER DEFAULT 0,
    duration_ms BIGINT
);

-- Create indexes if they don't exist
DO $$ BEGIN
    CREATE INDEX IF NOT EXISTS idx_job_runs_name ON job_runs(job_name);
    CREATE INDEX IF NOT EXISTS idx_job_runs_started ON job_runs(started_at DESC);
    CREATE INDEX IF NOT EXISTS idx_job_runs_status ON job_runs(status);
END $$;

-- Create job_config table if not exists
CREATE TABLE IF NOT EXISTS job_config (
    job_name VARCHAR(100) PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT true,
    schedule VARCHAR(100) NOT NULL,
    last_run TIMESTAMP,
    next_run TIMESTAMP,
    max_duration_minutes INTEGER DEFAULT 60,
    retry_count INTEGER DEFAULT 3,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Insert default job configurations (only if not exists)
-- Note: tokio-cron-scheduler uses 6-field format: second minute hour day month weekday
INSERT INTO job_config (job_name, schedule, max_duration_minutes)
VALUES
    ('refresh_prices', '0 0 2 * * *', 60),        -- 2:00 AM daily
    ('fetch_news', '0 30 2 * * *', 90),            -- 2:30 AM daily
    ('generate_forecasts', '0 0 4 * * *', 45),     -- 4:00 AM daily
    ('analyze_sec_filings', '0 30 4 * * *', 60),   -- 4:30 AM daily
    ('check_thresholds', '0 0 * * * *', 15),       -- Every hour
    ('warm_caches', '0 30 * * * *', 20),           -- Every hour at :30
    ('cleanup_cache', '0 0 3 * * SUN', 30),        -- Sunday 3:00 AM
    ('archive_snapshots', '0 30 3 * * SUN', 30)    -- Sunday 3:30 AM
ON CONFLICT (job_name) DO UPDATE SET schedule = EXCLUDED.schedule;

-- Add comments
COMMENT ON TABLE job_runs IS 'Sprint 19: Tracks execution history of background jobs';
COMMENT ON TABLE job_config IS 'Sprint 19: Configuration for scheduled jobs';
