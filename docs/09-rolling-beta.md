# Rustfolio Tutorial – Rolling Beta Analysis

> Audience: Beginners who want to understand how a stock reacts to the overall market  
> Goal: Learn what Beta means, how it changes over time, and how to forecast future market sensitivity.

---

# 📈 What Is Rolling Beta?

This page answers:

> "How sensitive is this stock to the market — and is that sensitivity changing?"

Beta measures how much a stock moves relative to a benchmark (e.g., S&P 500).

Rolling Beta shows how that relationship changes over time.

---

# 🧭 Page Overview

Main components:

- Portfolio Selector
- Ticker Selector
- Benchmark Selector (e.g., SPY / S&P 500)
- Historical Beta Tab
- Beta Forecast Tab
- Rolling Window Options (30, 60, 90 days)

---

# 📊 What Is Beta? (Beginner Explanation)

Beta compares a stock's movement to the market.

If:

- Beta = 1 → moves with market
- Beta > 1 → more volatile than market
- Beta < 1 → less volatile than market
- Beta < 0 → moves opposite market (rare)

Example:
If market rises 10%:

- Beta 1.5 stock may rise ~15%
- Beta 0.5 stock may rise ~5%

---

# 📉 Historical Beta (Rolling Beta)

This shows beta over time using a rolling window (e.g., 30-day window).

Why rolling?

Because beta changes depending on:

- Market conditions
- Company news
- Sector performance
- Economic regime

---

# 🔎 Example From Video

Example:

```
Rolling Beta Analysis: AMZN vs SPY
```

You see a line chart showing:

- Beta values over time
- How volatility relationship shifts

---

# 📅 Rolling Window Options

You can choose:

- 30 Days
- 60 Days
- 90 Days

Short window:
More responsive, more volatile beta

Long window:
Smoother, more stable estimate

---

# 🔮 Beta Forecast Tab

This section predicts future beta values.

Example:

```
Current Beta: 1.38
Forecast Beta: 1.32
```

This means:

The stock is currently more volatile than the market,
but may slightly decrease in sensitivity.

---

# 📊 Forecast Methods

Methods shown:

- Ensemble
- Mean Reversion
- Exp Smoothing

Let's simplify:

### Ensemble
Combines models → balanced forecast

### Mean Reversion
Assumes beta will move toward long-term average

### Exponential Smoothing
Gives more weight to recent data

---

# 📉 Confidence Interval

The shaded region shows:

95% confidence band.

Wider band:
More uncertainty

Narrow band:
More predictable sensitivity

---

# 🔄 Regime Changes

The page lists:

```
Detected regime changes
```

Example:
Beta changed from 1.20 → 1.35

This means:
The stock entered a higher market sensitivity phase.

---

# 🧠 Why Beta Matters

High beta stocks:

- Amplify market movements
- Increase portfolio volatility
- Increase drawdown risk

Low beta stocks:

- Reduce portfolio volatility
- Provide defensive exposure
- Stabilize performance

---

# 📊 Practical Portfolio Impact

If your portfolio contains mostly high beta stocks:

When market drops 10%,
your portfolio may drop significantly more.

If portfolio includes low beta stocks:

Losses may be cushioned.

---

# ⚠️ Important Reminder

Beta measures relative volatility,
not whether a stock is "good" or "bad."

High beta:
Good in bull markets  
Risky in bear markets  

Low beta:
Defensive in downturns  
May lag in rallies  

---

# 📌 How Beginners Should Use This Page

Step 1:
Check current beta of each large position

Step 2:
Identify high-beta concentration

Step 3:
Check if beta is rising

Step 4:
Use forecast to anticipate risk shifts

---

# 🔄 When To Use Rolling Beta

- Before market volatility events
- During earnings season
- When reallocating assets
- During macroeconomic uncertainty
- When portfolio volatility increases

---

# 📊 Combining Beta With Other Metrics

Beta + Correlation:
Shows how market-wide moves affect portfolio

Beta + Risk Score:
Shows both individual and systemic risk

Beta + Diversification:
Ensures not all assets amplify market swings

---

# ⚠️ Common Beginner Mistakes

❌ Confusing beta with return  
❌ Assuming beta is permanent  
❌ Ignoring rising beta trends  
❌ Not adjusting allocation in high-beta environments  

---

# ✅ What This Page Helps You Answer

- Is this stock aggressive or defensive?
- Is beta increasing?
- Will this stock amplify market volatility?
- Should I reduce high-beta exposure?
- How does this position affect overall portfolio stability?

---

# 📌 Final Takeaway

Correlation:
> "Do my stocks move together?"

Beta:
> "How strongly does this stock move with the market?"

Rolling Beta:
> "Is that relationship changing?"

Understanding beta helps you control systemic risk — not just individual stock volatility.

---

# Next Tutorial

Next page to explore:
→ **CVaR Analysis** (understanding tail risk and extreme losses)

Likely upcoming sections:
- Market Regime
- Volatility Forecast
- Trading Signals
- Sentiment Forecast
