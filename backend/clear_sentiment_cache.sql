-- Clear enhanced sentiment cache to force regeneration
DELETE FROM enhanced_sentiment_cache;

-- Also clear regular sentiment cache if needed
DELETE FROM sentiment_signal_cache;

SELECT 'Cache cleared successfully!' as result;
