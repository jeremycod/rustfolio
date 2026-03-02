# Rustfolio Tutorial – Volatility Forecasting (GARCH)

> Audience: Beginners who want to predict future volatility  
> Goal: Predict future price volatility.

---

# 📊 What Is GARCH?

GARCH = Generalized Autoregressive Conditional Heteroskedasticity

(Don't worry about the name!)

It's a statistical model capturing:

- Volatility clustering
- Shock persistence
- Risk expansion phases

---

# 🔍 What GARCH Does

GARCH recognizes:

> "High volatility tends to follow high volatility"

If today is volatile,
tomorrow is likely volatile too.

This is called volatility clustering.

---

# 📈 Key Outputs

## Current Volatility

Example:

```
18.5%
```

Current measured volatility.

---

## Forecast Volatility

Example:

```
22.3%
```

Predicted volatility for next period.

If forecast > current:
Expect larger price swings ahead.

---

## Persistence (Beta)

Example:

```
0.85
```

How long volatility shocks last.

Higher beta = volatility persists longer.

---

## Shock Impact (Alpha)

Example:

```
0.12
```

How much new shocks affect volatility.

Higher alpha = more reactive to news.

---

# 🧠 Why It Matters

If volatility forecast is rising:
Expect larger price swings.

Useful for:

- Position sizing
- Options trading
- Risk control
- Entry/exit timing

---

# 📊 Volatility Clustering Example

Day 1: Low volatility  
Day 2: Market shock → high volatility  
Day 3: Still high volatility  
Day 4: Still elevated  
Day 5: Gradually declining  

GARCH captures this pattern.

---

# 📌 How Beginners Should Use This Page

Step 1:
Check current vs forecast volatility

Step 2:
If forecast is rising, reduce position sizes

Step 3:
If forecast is falling, market may stabilize

Step 4:
Monitor persistence and shock impact

---

# ⚠️ Important Reminder

GARCH forecasts volatility, not direction.

It tells you:
> "How much prices will move"

Not:
> "Which direction prices will move"

---

# 🔄 When To Use Volatility Forecast

- Before earnings announcements
- During market uncertainty
- When planning entries/exits
- For options strategies
- When adjusting position sizes

---

# ✅ What This Page Helps You Answer

- Is volatility increasing?
- How long will volatility persist?
- Should I reduce position sizes?
- Is the market stabilizing?
- When should I expect calmer conditions?

---

# 📌 Final Takeaway

Market Regime tells you:
> "What environment are we in?"

Volatility Forecast tells you:
> "How much will prices swing?"

Both help you manage risk proactively.

---

# Next Tutorial

Next page to explore:
→ **Trading Signals** (technical buy/sell indicators)
