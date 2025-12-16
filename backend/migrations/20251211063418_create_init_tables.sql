CREATE TABLE positions (
                           id UUID PRIMARY KEY,
                           portfolio_id UUID NOT NULL REFERENCES portfolios(id) ON DELETE CASCADE,
                           ticker TEXT NOT NULL,
                           shares NUMERIC NOT NULL CHECK (shares >= 0),
                           avg_buy_price NUMERIC NOT NULL CHECK (avg_buy_price >= 0),
                           created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE price_points (
                              id UUID PRIMARY KEY,
                              ticker TEXT NOT NULL,
                              date DATE NOT NULL,
                              close_price NUMERIC NOT NULL,
                              created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_price_ticker_date
    ON price_points(ticker, date);

CREATE TABLE transactions (
                              id UUID PRIMARY KEY,
                              portfolio_id UUID NOT NULL REFERENCES portfolios(id),
                              ticker TEXT NOT NULL,
                              quantity NUMERIC NOT NULL,
                              price NUMERIC NOT NULL,
                              side TEXT NOT NULL CHECK (side IN ('BUY', 'SELL')),
                              executed_at TIMESTAMPTZ NOT NULL
);
