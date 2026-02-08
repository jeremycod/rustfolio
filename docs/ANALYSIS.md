# Analysis of the rustfolio and stock_research_sidekick Projects

## Overview of the rustfolio project

**Architecture** – The rustfolio repository is a full‑stack portfolio tracker built with a Rust back end (Axum + SQLx) and a React/TypeScript front end. The rustfolio_project_description.md notes that the back end exposes REST APIs to manage portfolios, positions, transactions and price data, and the front end renders dashboards and charts. The core features include adding/editing positions, tracking daily price history (real or mock), visualising portfolio value over time and computing basic trend indicators (moving average, linear regression). Stretch goals mention advanced analytics such as volatility, drawdown and risk scores.

**Back‑end structure** – backend/src/main.rs creates a Postgres connection, sets up a price_provider (Alpha Vantage) and exposes a router with endpoints for health checks, accounts, portfolios, positions, prices and analytics. The create_app function registers nested routers for /api/portfolios, /api/positions, /api/prices and /api/analytics, among others.

**Analytics** – The analytics service fetches a portfolio's value history from the database and computes simple moving averages (SMA), exponential moving averages (EMA) and a linear regression trend line over the values. These computations are implemented in indicators.rs as pure functions. The service also calculates current allocations and normalises them. At present, the analytics endpoint returns these basic time‑series indicators but does not calculate volatility, drawdown or risk metrics.

**Price service** – The project defines a PriceProvider trait and an AlphaVantageProvider implementation to fetch daily history and search tickers. The price_service can fetch a ticker's history, return the latest price, generate mock data and refresh prices from the API while handling rate limits. Endpoints support searching for tickers, retrieving price history, and updating data.

## Overview of the stock_research_sidekick project

**Purpose** – This Python project uses LangGraph to orchestrate a tool‑using agent that generates a grounded stock research brief. According to its project guide, the input is a ticker symbol, a look‑back window (7/30/90 days) and a depth parameter (quick/standard/deep). The output is a Markdown brief that summarises price context (return, a volatility proxy and drawdown), clusters recent news into themes with citations, and proposes a watchlist or follow‑up questions. The design intentionally separates fact‑gathering tools from the LLM: tools fetch price data (Alpha Vantage) and news (Serper), while the LLM plans the research, clusters news and composes the brief.

**Price feature computation** – The node compute_price_features.py computes percentage change, a volatility proxy (standard deviation of daily returns) and maximum drawdown over the chosen window. These metrics provide risk/return context for the brief.

**News analysis** – The pipeline fetches recent news articles, deduplicates them and then calls GPT‑4 to cluster them into themes (via cluster_themes.py). Another GPT call summarises each theme with bullets and citations (summarize_themes.py). A final LLM call composes the Markdown brief using the price features and theme summaries. Thus the project demonstrates how to incorporate LLMs for research planning, clustering unstructured news and generating natural‑language reports.

## Opportunities to expand Rustfolio using ideas from Stock Research Sidekick

### 1. Advanced analytics and risk metrics

**Volatility, drawdown and risk scores** – The rustfolio documentation lists volatility, max drawdown and risk scores as stretch features, but these metrics are currently computed only in the Python sidekick. You could port the _pct, volatility and max‑drawdown logic from compute_price_features.py into Rust. Additional risk metrics to consider:

- **Sharpe/Sortino ratios** – risk‑adjusted return metrics.
- **Beta and correlation with a market index** to measure systematic risk.
- **Value at Risk (VaR) and Expected Shortfall** to quantify potential losses.
- **Portfolio volatility** computed from position weights and covariances.

**Scenario and "what‑if" analysis** – Provide tools for simulating how the portfolio's value would change under different market scenarios (e.g., a 10 % drop in technology stocks). This aligns with the "what‑if investment simulations" goal. You could implement a small engine in Rust or call a Python micro‑service to run Monte Carlo simulations based on historical returns.

### 2. LLM‑based research and reporting within Rustfolio

**Narrative analytics** – Following the sidekick's pattern, you can integrate an LLM service (OpenAI or open‑source) to generate natural‑language summaries of the portfolio's performance and risk. For example, after computing returns, volatility and drawdown for each holding and the overall portfolio, call the LLM with a prompt like "Explain the past month's performance of this portfolio and highlight the biggest contributors to risk." The model can translate numeric metrics into bullet points and highlight trends, similar to how compose_brief.py builds a markdown brief. This would make Rustfolio more user‑friendly by providing digestible explanations alongside charts.

**News aggregation and thematic summaries** – Using the sidekick's news pipeline as inspiration, add an optional service that fetches recent news for each holding (via Serper or another news API), clusters the articles into themes and summarises them. Present a "recent themes" section in the portfolio dashboard. This can alert the user to company‑specific events (earnings, product launches, lawsuits) that may affect positions.

**Watchlist and follow‑up questions** – Generate a watchlist with prompts for further research. For instance, after summarising news and price movements, the LLM could suggest questions like "How will the company's pending acquisition affect its debt ratio?" This feature encourages deeper analysis without making buy/sell recommendations.

### 3. Forecasting and predictive analytics

**Time‑series forecasting** – Beyond linear regression, implement more sophisticated models (e.g., ARIMA, Prophet or machine‑learning regressors) to forecast short‑term price movements or portfolio value. Models can be trained offline (in Python) and exposed to Rust via a REST or gRPC micro‑service. Predictions should include confidence intervals and be clearly labelled as speculative.

**Rolling regression and beta calculation** – For each position, compute a rolling regression against a benchmark index to estimate beta and forecast sensitivity to market moves. Provide a "risk contribution" chart showing which positions contribute most to portfolio variance.

**Sentiment‑aware signals** – Combine price data with sentiment extracted from news articles (via LLM classification) to create basic sentiment indicators. For example, tag themes as positive/negative and correlate them with price movements. This remains experimental but could help users spot momentum or risk shifts.

### 4. System architecture considerations

**Micro‑service design** – The existing Rust back end can continue to handle CRUD operations, price fetching and basic analytics. For heavy computations and LLM calls, spin up a Python micro‑service that re‑uses the sidekick's modules. The Rust service can call this Python service asynchronously when needed and cache results in the database.

**Background jobs** – Schedule tasks to refresh price data and news for holdings in the background (e.g., via cron or a task queue). This reduces latency when the user opens the dashboard.

**Extensible API** – Add new endpoints such as /api/analytics/{portfolio_id}/risk to return advanced risk metrics, and /api/analysis/{portfolio_id}/brief to return the LLM‑generated narrative and news summary. The front end can then display these sections alongside existing charts.

### 5. User experience improvements

**Interactive Q&A** – Embed a chat panel in the dashboard where the user can ask natural‑language questions about their portfolio. Use the analytics and news data as context for the LLM to answer queries like "Why did my portfolio drop last week?" or "Which holding has the highest volatility?" Ensure answers avoid buy/sell advice.

**Alerts and notifications** – Allow users to set up price or volatility alerts and receive notifications or e‑mails. Use the LLM to suggest potential reasons for large moves, referencing recent news themes.

**Educational tooltips** – Provide contextual tooltips explaining risk metrics (e.g., "Volatility measures how widely a stock's price varies over time; higher values indicate greater uncertainty"). These can be generated once via an LLM and stored.

## Conclusion

The rustfolio project already has a solid foundation for managing portfolios and computing basic trend indicators. The stock_research_sidekick demonstrates how to use LLMs to plan research, cluster news and compose human‑readable briefs. By integrating the sidekick's approaches—particularly risk metric computation, news summarisation and LLM‑driven narratives—rustfolio can evolve from a pure tracker into an intelligent assistant that helps users understand their holdings and potential risks. Such enhancements should be offered as educational tools and avoid making direct investment recommendations. With careful design (e.g., micro‑services and background tasks) and clear communication of limitations, these AI features can significantly enrich the user experience.

## Per-Position Risk Warnings

To give you per‑position risk warnings in addition to general alerts/notifications, you can expand the analytics layer and add some logic that flags positions when specific risk metrics breach user‑defined thresholds. Here are a few ideas:

**Compute position‑level volatility and drawdown**: The sidekick prototype already contains functions that calculate percentage change, a volatility proxy (standard deviation of daily returns) and maximum drawdown. Port this logic into Rustfolio's backend so that each position's price history yields a set of risk metrics (e.g. daily return volatility, trailing drawdown, rolling beta versus a market index). You can then define thresholds (e.g. volatility above X%, drawdown more than Y%) that trigger a warning.

**Create a risk‑scoring function**: Combine several metrics—recent price declines, high volatility, earnings/news sentiment, and perhaps correlation with the rest of the portfolio—into a simple score or letter grade. The stretch goals listed in your project doc already mention adding volatility, max drawdown and risk scores; a composite score would build on that and make comparisons across positions easier.

**Watch for technical signals**: Use your existing trend indicators (SMA, EMA and linear regression slope) as warning signals. For example, if the price falls below a long‑term moving average or the regression slope turns negative for multiple periods, flag the position for review.

**Incorporate news and sentiment**: If you decide to integrate the news‑clustering pipeline from the stock sidekick, you can add another warning when negative themes (e.g. lawsuits, earnings misses) dominate recent coverage. An LLM could summarise why the position might be risky and suggest follow‑up questions or research topics.

**Expose settings in the UI**: Allow users to customise risk thresholds, e.g. "alert me when a position's drawdown exceeds 15%." Display warnings alongside each position in the holdings table and link to a detailed panel that shows the underlying metrics and a narrative explanation.

By surfacing position‑specific risk metrics and offering configurable thresholds, Rustfolio can provide proactive warnings and guidance without making buy/sell recommendations.
