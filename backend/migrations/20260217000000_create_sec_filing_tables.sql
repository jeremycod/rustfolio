-- SEC Filing Types Enum
CREATE TYPE filing_type AS ENUM ('8-k', 'form4', '10-q', '10-k');

-- Event Importance Enum
CREATE TYPE event_importance AS ENUM ('critical', 'high', 'medium', 'low');

-- Transaction Type Enum
CREATE TYPE transaction_type AS ENUM ('purchase', 'sale', 'grant', 'exercise');

-- Confidence Level Enums
CREATE TYPE insider_confidence AS ENUM ('high', 'medium', 'low', 'none');
CREATE TYPE confidence_level AS ENUM ('very_high', 'high', 'medium', 'low');

-- Raw SEC filings cache
CREATE TABLE sec_filings (
    id SERIAL PRIMARY KEY,
    ticker VARCHAR(10) NOT NULL,
    filing_type filing_type NOT NULL,
    filing_date DATE NOT NULL,
    accession_number VARCHAR(25) NOT NULL UNIQUE,
    filing_url TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_sec_filings_ticker ON sec_filings(ticker);
CREATE INDEX idx_sec_filings_date ON sec_filings(filing_date);
CREATE INDEX idx_sec_filings_type ON sec_filings(filing_type);

-- Material events (8-K analysis)
CREATE TABLE material_events (
    id SERIAL PRIMARY KEY,
    ticker VARCHAR(10) NOT NULL,
    event_date DATE NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    sentiment_score DOUBLE PRECISION NOT NULL,
    summary TEXT NOT NULL,
    importance event_importance NOT NULL,
    filing_url TEXT NOT NULL,
    accession_number VARCHAR(25) REFERENCES sec_filings(accession_number),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_material_events_ticker ON material_events(ticker);
CREATE INDEX idx_material_events_date ON material_events(event_date);
CREATE INDEX idx_material_events_importance ON material_events(importance);

-- Insider transactions (Form 4)
CREATE TABLE insider_transactions (
    id SERIAL PRIMARY KEY,
    ticker VARCHAR(10) NOT NULL,
    transaction_date DATE NOT NULL,
    reporting_person VARCHAR(255) NOT NULL,
    title VARCHAR(100),
    transaction_type transaction_type NOT NULL,
    shares BIGINT NOT NULL,
    price_per_share DECIMAL(12, 4),
    ownership_after BIGINT,
    accession_number VARCHAR(25) REFERENCES sec_filings(accession_number),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_insider_transactions_ticker ON insider_transactions(ticker);
CREATE INDEX idx_insider_transactions_date ON insider_transactions(transaction_date);
CREATE INDEX idx_insider_transactions_type ON insider_transactions(transaction_type);

-- Enhanced sentiment cache
CREATE TABLE enhanced_sentiment_cache (
    ticker VARCHAR(10) NOT NULL PRIMARY KEY,
    calculated_at TIMESTAMP NOT NULL,
    expires_at TIMESTAMP NOT NULL,

    -- Component scores
    news_sentiment DOUBLE PRECISION NOT NULL,
    news_confidence VARCHAR(20) NOT NULL,
    sec_filing_score DOUBLE PRECISION,
    insider_sentiment_score DOUBLE PRECISION,

    -- Combined analysis
    combined_sentiment DOUBLE PRECISION NOT NULL,
    confidence_level confidence_level NOT NULL,

    -- Metadata
    material_events JSONB NOT NULL,
    insider_activity JSONB NOT NULL,
    divergence_flags JSONB NOT NULL
);

CREATE INDEX idx_enhanced_sentiment_expires ON enhanced_sentiment_cache(expires_at);
CREATE INDEX idx_enhanced_sentiment_confidence ON enhanced_sentiment_cache(confidence_level);

COMMENT ON TABLE sec_filings IS 'Raw SEC filing metadata from Edgar';
COMMENT ON TABLE material_events IS 'Analyzed 8-K material events with sentiment scores';
COMMENT ON TABLE insider_transactions IS 'Form 4 insider trading transactions';
COMMENT ON TABLE enhanced_sentiment_cache IS 'Multi-source sentiment analysis with 12-hour TTL';
