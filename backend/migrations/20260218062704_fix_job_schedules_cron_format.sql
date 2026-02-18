-- Fix job schedules to use correct 6-field cron format (second minute hour day month weekday)
-- tokio-cron-scheduler requires seconds field, not standard 5-field Unix cron

UPDATE job_config SET schedule = '0 0 2 * * *' WHERE job_name = 'refresh_prices';
UPDATE job_config SET schedule = '0 30 2 * * *' WHERE job_name = 'fetch_news';
UPDATE job_config SET schedule = '0 0 4 * * *' WHERE job_name = 'generate_forecasts';
UPDATE job_config SET schedule = '0 30 4 * * *' WHERE job_name = 'analyze_sec_filings';
UPDATE job_config SET schedule = '0 0 * * * *' WHERE job_name = 'check_thresholds';
UPDATE job_config SET schedule = '0 30 * * * *' WHERE job_name = 'warm_caches';
UPDATE job_config SET schedule = '0 0 3 * * SUN' WHERE job_name = 'cleanup_cache';
UPDATE job_config SET schedule = '0 30 3 * * SUN' WHERE job_name = 'archive_snapshots';
