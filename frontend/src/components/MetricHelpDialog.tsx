import {
  Dialog,
  DialogTitle,
  DialogContent,
  IconButton,
  Typography,
  Box,
  Chip,
  Divider,
  Alert,
} from '@mui/material';
import { Close as CloseIcon, TrendingUp, TrendingDown, Remove } from '@mui/icons-material';

interface MetricHelpDialogProps {
  open: boolean;
  onClose: () => void;
  metricKey: string;
}

interface MetricHelp {
  title: string;
  description: string;
  interpretation: string;
  goodValues: { label: string; description: string; color: string }[];
  badValues: { label: string; description: string; color: string }[];
  example: string;
  formula?: string;
  additionalNotes?: string;
}

const METRIC_HELP: Record<string, MetricHelp> = {
  risk_score: {
    title: 'Risk Score',
    description: 'A composite score from 0-100 that combines multiple risk metrics into a single, easy-to-understand number. The risk score weighs volatility (40%), max drawdown (30%), beta (20%), and value at risk (10%) to give you an overall risk assessment.',
    interpretation: 'Think of the risk score as a "risk temperature" for your investment. Just like a thermometer tells you if it\'s hot or cold, the risk score tells you if an investment is risky or stable at a glance.',
    goodValues: [
      { label: '0-30', description: 'Low risk (conservative, stable)', color: '#4caf50' },
      { label: '30-50', description: 'Moderate-low risk (balanced)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '50-70', description: 'Moderate-high risk (aggressive)', color: '#ff9800' },
      { label: '70-100', description: 'High risk (very volatile, speculative)', color: '#f44336' },
    ],
    example: 'A blue-chip stock like Johnson & Johnson might have a risk score of 35 (moderate-low), while a small biotech startup could have a risk score of 85 (very high risk).',
    formula: 'Risk Score = (Volatility × 0.4) + (|Max Drawdown| × 0.3) + (Beta × 0.2) + (|VaR| × 0.1)',
    additionalNotes: 'The risk score is normalized to 0-100 for easy comparison across different securities. Scores above 60 indicate investments that may experience severe short-term losses and require strong emotional tolerance.',
  },
  volatility: {
    title: 'Volatility (Annualized)',
    description: 'Volatility measures how much the price of a security fluctuates over time. It is calculated as the standard deviation of daily returns, then annualized. Higher volatility means the price experiences larger and more frequent swings.',
    interpretation: 'Think of volatility as the "bumpiness" of your investment ride. A volatile stock can make big moves up or down in short periods, while a low-volatility stock tends to move more steadily.',
    goodValues: [
      { label: '< 15%', description: 'Very stable (defensive stocks, utilities)', color: '#4caf50' },
      { label: '15-25%', description: 'Moderate (blue-chip stocks, indices)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '25-40%', description: 'High (growth stocks, tech)', color: '#ff9800' },
      { label: '> 40%', description: 'Very high (small-cap, speculative)', color: '#f44336' },
    ],
    example: 'A stock with 30% volatility might typically move ±2% per day. One with 60% volatility might move ±4% per day.',
    formula: 'σ = √(252 × Var(daily_returns))',
    additionalNotes: 'Volatility is not inherently bad—it creates opportunities. However, higher volatility requires stronger risk tolerance and may not be suitable for short-term goals or conservative investors.',
  },
  max_drawdown: {
    title: 'Maximum Drawdown',
    description: 'The maximum observed loss from a peak to a trough before a new peak is achieved. It represents the worst possible loss you would have experienced if you bought at the absolute worst time and sold at the absolute worst time during the period.',
    interpretation: 'This tells you the worst-case scenario you faced in the measurement period. If the max drawdown is -25%, it means at some point the security lost 25% of its value from its previous high.',
    goodValues: [
      { label: '> -10%', description: 'Minimal drawdown (very stable)', color: '#4caf50' },
      { label: '-10% to -20%', description: 'Moderate decline (typical market corrections)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-20% to -35%', description: 'Significant drop (bear market territory)', color: '#ff9800' },
      { label: '< -35%', description: 'Severe decline (crisis-level losses)', color: '#f44336' },
    ],
    example: 'If a stock peaked at $100, dropped to $70, and later recovered to $95, the max drawdown is -30%.',
    additionalNotes: 'Recovery from drawdowns can take months or years. The S&P 500\'s max drawdown during the 2008 financial crisis was -56%. Consider your emotional ability to withstand such declines.',
  },
  beta: {
    title: 'Beta',
    description: 'Beta measures how much a security moves relative to a benchmark (usually a market index). A beta of 1.0 means the security moves in line with the market. Beta greater than 1.0 means more volatile than the market; less than 1.0 means less volatile.',
    interpretation: 'Beta tells you if your investment amplifies or dampens market movements. If the market is up 10%, a stock with beta 1.5 typically rises 15%, while one with beta 0.5 rises 5%.',
    goodValues: [
      { label: '0.5-0.8', description: 'Defensive (less volatile than market)', color: '#4caf50' },
      { label: '0.8-1.2', description: 'Market-like (moves with the market)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '1.2-1.5', description: 'Aggressive (amplifies market moves)', color: '#ff9800' },
      { label: '> 1.5', description: 'Very aggressive (extreme volatility)', color: '#f44336' },
    ],
    example: 'If the S&P 500 drops 10%, a stock with beta 1.3 would typically drop 13%, while one with beta 0.7 would drop 7%.',
    formula: 'β = Cov(stock_returns, market_returns) / Var(market_returns)',
    additionalNotes: 'Negative beta is rare but means the security moves opposite to the market (like gold or inverse ETFs). Beta changes over time and may not predict future behavior.',
  },
  systematic_risk: {
    title: 'Systematic Risk',
    description: 'The portion of a security\'s volatility that comes from market-wide factors (economy, interest rates, geopolitical events). This is the risk you cannot diversify away—even a perfectly diversified portfolio has systematic risk.',
    interpretation: 'This shows how much of your investment\'s ups and downs are due to overall market conditions versus company-specific factors. Higher systematic risk means the security is more affected by market sentiment.',
    goodValues: [
      { label: '< 10%', description: 'Low market sensitivity (defensive)', color: '#4caf50' },
      { label: '10-20%', description: 'Moderate market exposure', color: '#8bc34a' },
    ],
    badValues: [
      { label: '20-30%', description: 'High market sensitivity', color: '#ff9800' },
      { label: '> 30%', description: 'Very high market dependence', color: '#f44336' },
    ],
    example: 'A utility stock might have 12% systematic risk (stable, defensive), while a tech stock might have 28% systematic risk (moves heavily with market trends).',
    additionalNotes: 'Systematic risk is tied to beta and R². You can only reduce systematic risk by holding assets with different betas or moving to safer asset classes.',
  },
  idiosyncratic_risk: {
    title: 'Idiosyncratic Risk',
    description: 'The portion of a security\'s volatility that is specific to the company or asset itself—things like earnings reports, management changes, product launches, or lawsuits. This risk CAN be diversified away by owning multiple uncorrelated securities.',
    interpretation: 'This tells you how much the security moves for its own reasons, independent of the broader market. High idiosyncratic risk means company-specific news has a big impact.',
    goodValues: [
      { label: '< 15%', description: 'Stable, predictable company', color: '#4caf50' },
      { label: '15-25%', description: 'Moderate company-specific volatility', color: '#8bc34a' },
    ],
    badValues: [
      { label: '25-40%', description: 'High company-specific volatility', color: '#ff9800' },
      { label: '> 40%', description: 'Extreme uncertainty (speculative)', color: '#f44336' },
    ],
    example: 'A biotech stock awaiting FDA approval might have 50% idiosyncratic risk—its price depends heavily on trial results, not market conditions.',
    additionalNotes: 'Diversification is most effective at reducing idiosyncratic risk. Holding 20-30 uncorrelated stocks can eliminate most of this risk from your portfolio.',
  },
  r_squared: {
    title: 'R² (Coefficient of Determination)',
    description: 'R² measures what percentage of a security\'s price movements can be explained by movements in the benchmark. An R² of 75% means 75% of the security\'s volatility is due to market factors, and 25% is company-specific.',
    interpretation: 'Think of R² as showing how closely the security follows the market. High R² means it\'s market-driven; low R² means it marches to its own beat.',
    goodValues: [
      { label: '70-100%', description: 'Highly correlated with market (predictable)', color: '#4caf50' },
      { label: '40-70%', description: 'Moderately market-driven', color: '#8bc34a' },
    ],
    badValues: [
      { label: '20-40%', description: 'Loosely follows market', color: '#ff9800' },
      { label: '< 20%', description: 'Uncorrelated with market (unpredictable)', color: '#f44336' },
    ],
    example: 'An S&P 500 ETF has R² ≈ 100% vs the S&P 500. A gold mining stock might have R² of 30%—it doesn\'t follow the stock market closely.',
    additionalNotes: 'Low R² doesn\'t mean bad—it can provide diversification benefits. However, it also means beta and systematic risk estimates are less reliable.',
  },
  sharpe_ratio: {
    title: 'Sharpe Ratio',
    description: 'The Sharpe Ratio measures return per unit of total risk (volatility). It shows how much extra return you\'re getting for the volatility you\'re taking on. Calculated as (return - risk-free rate) / volatility.',
    interpretation: 'A higher Sharpe Ratio means better risk-adjusted returns—you\'re getting more bang for your buck in terms of risk taken. It helps compare investments with different volatility levels.',
    goodValues: [
      { label: '> 2.0', description: 'Excellent risk-adjusted returns', color: '#4caf50' },
      { label: '1.0-2.0', description: 'Good risk-adjusted returns', color: '#8bc34a' },
    ],
    badValues: [
      { label: '0-1.0', description: 'Acceptable but not optimal', color: '#ff9800' },
      { label: '< 0', description: 'Negative returns (losing money)', color: '#f44336' },
    ],
    example: 'If Stock A returns 15% with 20% volatility (Sharpe = 0.75) and Stock B returns 12% with 10% volatility (Sharpe = 1.2), Stock B has better risk-adjusted performance.',
    formula: 'Sharpe = (Return - Risk_Free_Rate) / Volatility',
    additionalNotes: 'Sharpe Ratios above 3 are rare and exceptional. The S&P 500\'s long-term Sharpe is around 0.4-0.5. Values depend on the time period measured.',
  },
  sortino_ratio: {
    title: 'Sortino Ratio',
    description: 'Similar to Sharpe Ratio, but only penalizes downside volatility (negative returns). The Sortino Ratio recognizes that upside volatility is good—you only care about the risk of losses, not gains.',
    interpretation: 'The Sortino Ratio gives a more accurate picture of risk for investors who care about downside protection. It\'s better than Sharpe for asymmetric return distributions.',
    goodValues: [
      { label: '> 2.0', description: 'Excellent downside-adjusted returns', color: '#4caf50' },
      { label: '1.0-2.0', description: 'Good downside protection', color: '#8bc34a' },
    ],
    badValues: [
      { label: '0-1.0', description: 'Moderate downside risk', color: '#ff9800' },
      { label: '< 0', description: 'Negative returns', color: '#f44336' },
    ],
    example: 'An investment that gains 5% most days but occasionally drops 3% will have a higher Sortino than Sharpe, because the upside volatility isn\'t penalized.',
    formula: 'Sortino = (Return - Risk_Free_Rate) / Downside_Deviation',
    additionalNotes: 'Sortino is generally higher than Sharpe for the same investment. It\'s preferred by investors focused on capital preservation and downside risk management.',
  },
  annualized_return: {
    title: 'Annualized Return',
    description: 'The average yearly return you would expect based on the historical period analyzed. It\'s the geometric mean of returns scaled to a yearly basis.',
    interpretation: 'This projects what you might earn per year if the historical pattern continues. It accounts for compounding—the "return on your returns."',
    goodValues: [
      { label: '> 15%', description: 'Excellent (above market average)', color: '#4caf50' },
      { label: '10-15%', description: 'Good (market-like returns)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '0-10%', description: 'Below market average', color: '#ff9800' },
      { label: '< 0%', description: 'Losing money', color: '#f44336' },
    ],
    example: 'The S&P 500\'s historical annualized return is about 10%. A stock with 20% annualized return has outperformed the market, but likely with higher risk.',
    additionalNotes: 'Past returns do not guarantee future performance. Higher returns usually come with higher volatility. Always consider the risk metrics alongside returns.',
  },
  var_95: {
    title: 'Value at Risk (95%)',
    description: 'VaR estimates the maximum loss you can expect on 95% of trading days. In other words, there\'s only a 5% chance (1 in 20 days) that your loss will exceed this amount.',
    interpretation: 'VaR gives you a worst-case scenario threshold for normal market conditions. If VaR(95%) is -3%, then 19 out of 20 days you should lose less than 3%.',
    goodValues: [
      { label: '> -2%', description: 'Low daily risk', color: '#4caf50' },
      { label: '-2% to -4%', description: 'Moderate daily risk', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-4% to -6%', description: 'High daily risk', color: '#ff9800' },
      { label: '< -6%', description: 'Very high daily risk', color: '#f44336' },
    ],
    example: 'If your $10,000 position has VaR(95%) of -$300 (-3%), there\'s a 5% chance you\'ll lose more than $300 in a single day.',
    formula: 'VaR = μ + (σ × z), where z is the 5th percentile of returns',
    additionalNotes: 'VaR doesn\'t tell you HOW MUCH you could lose in the worst 5% of cases—only the threshold. See Expected Shortfall for the average loss beyond VaR.',
  },
  var_99: {
    title: 'Value at Risk (99%)',
    description: 'VaR at 99% confidence estimates the maximum loss on 99% of trading days. There\'s only a 1% chance (1 in 100 days) that losses exceed this amount. This is a more conservative, extreme-risk measure.',
    interpretation: 'VaR(99%) captures rarer but more severe losses. Use this to understand tail risk—the potential for large losses during market turmoil.',
    goodValues: [
      { label: '> -3%', description: 'Low extreme risk', color: '#4caf50' },
      { label: '-3% to -5%', description: 'Moderate extreme risk', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-5% to -8%', description: 'High extreme risk', color: '#ff9800' },
      { label: '< -8%', description: 'Very high tail risk', color: '#f44336' },
    ],
    example: 'If VaR(99%) is -5%, then 99 out of 100 days your loss will be less than 5%, but on rare occasions it could be much worse.',
    additionalNotes: 'VaR(99%) is useful for stress testing and understanding crisis scenarios. The 2008 financial crisis and COVID-19 crash were "beyond VaR" events for many portfolios.',
  },
  expected_shortfall_95: {
    title: 'Expected Shortfall (95%)',
    description: 'Also called Conditional VaR (CVaR), Expected Shortfall answers: "If we hit the worst 5% of days, what\'s the average loss?" It measures tail risk—the severity of losses beyond the VaR threshold.',
    interpretation: 'While VaR tells you the threshold, Expected Shortfall tells you how bad things get when that threshold is breached. It\'s a more complete risk measure.',
    goodValues: [
      { label: '> -3%', description: 'Manageable tail losses', color: '#4caf50' },
      { label: '-3% to -5%', description: 'Moderate tail risk', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-5% to -8%', description: 'High tail risk', color: '#ff9800' },
      { label: '< -8%', description: 'Severe tail risk', color: '#f44336' },
    ],
    example: 'If VaR(95%) is -3% and ES(95%) is -5%, the worst 5% of days average a 5% loss—much worse than the 3% threshold.',
    formula: 'ES = Average(losses worse than VaR)',
    additionalNotes: 'Expected Shortfall is preferred by risk managers over VaR because it captures the full distribution of tail losses, not just the threshold. It\'s more conservative.',
  },
  expected_shortfall_99: {
    title: 'Expected Shortfall (99%)',
    description: 'The average loss on the worst 1% of trading days (1 in 100 days). This is your extreme crisis scenario measure—what happens during market meltdowns.',
    interpretation: 'ES(99%) tells you what to expect during rare but catastrophic market events. This is the risk that keeps risk managers up at night.',
    goodValues: [
      { label: '> -5%', description: 'Low crisis risk', color: '#4caf50' },
      { label: '-5% to -8%', description: 'Moderate crisis risk', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-8% to -12%', description: 'High crisis risk', color: '#ff9800' },
      { label: '< -12%', description: 'Extreme crisis risk', color: '#f44336' },
    ],
    example: 'If ES(99%) is -10%, on the worst 1% of days (2-3 times per year), you can expect to lose an average of 10%.',
    additionalNotes: 'During the 2020 COVID crash, many stocks had single-day losses exceeding their ES(99%). Use this metric to stress-test your emotional tolerance for severe drawdowns.',
  },
  average_correlation: {
    title: 'Average Correlation',
    description: 'The average correlation coefficient between all pairs of positions in your portfolio. Correlation measures how closely two assets move together, ranging from -100% (perfect opposite movement) to +100% (perfect synchronized movement).',
    interpretation: 'This tells you how diversified your portfolio truly is. High average correlation means most positions rise and fall together, reducing diversification benefits.',
    goodValues: [
      { label: '< 30%', description: 'Excellent diversification', color: '#4caf50' },
      { label: '30-50%', description: 'Good diversification', color: '#8bc34a' },
    ],
    badValues: [
      { label: '50-70%', description: 'Moderate concentration risk', color: '#ff9800' },
      { label: '> 70%', description: 'High concentration risk', color: '#f44336' },
    ],
    example: 'If your portfolio has 40% average correlation, when one stock moves 10%, other stocks typically move 4% in the same direction.',
    additionalNotes: 'During market crashes, correlations tend to spike toward 1.0 as "all correlations go to one." This metric is calculated during normal market conditions.',
  },
  diversification_score: {
    title: 'Diversification Score',
    description: 'A composite score from 0-10 that combines the number of positions with their correlation patterns. It adjusts for the fact that having many highly correlated positions provides false diversification.',
    interpretation: 'Think of this as a quality check on your diversification. You could own 50 tech stocks, but if they all move together, you\'re not truly diversified.',
    goodValues: [
      { label: '7-10', description: 'Excellent diversification', color: '#4caf50' },
      { label: '5-7', description: 'Good diversification', color: '#8bc34a' },
    ],
    badValues: [
      { label: '3-5', description: 'Moderate diversification', color: '#ff9800' },
      { label: '< 3', description: 'Poor diversification', color: '#f44336' },
    ],
    example: 'A portfolio with 20 stocks but 80% average correlation might score 3/10. A portfolio with 10 stocks and 30% average correlation might score 7/10.',
    formula: 'Score = √(N_positions) × (1 - avg_correlation) × adjustment_factors',
    additionalNotes: 'True diversification requires both quantity (many positions) and quality (low correlations). This score balances both factors.',
  },
  high_correlation_pairs: {
    title: 'High Correlation Pairs',
    description: 'The number of position pairs in your portfolio with correlation above 70%. When two positions are this highly correlated, they provide minimal diversification benefit to each other.',
    interpretation: 'Each highly correlated pair represents a potential concentration risk. If both positions move together, they act more like a single position than two independent investments.',
    goodValues: [
      { label: '0-1', description: 'Minimal concentration', color: '#4caf50' },
      { label: '2-3', description: 'Low concentration', color: '#8bc34a' },
    ],
    badValues: [
      { label: '4-6', description: 'Moderate concentration risk', color: '#ff9800' },
      { label: '> 6', description: 'High concentration risk', color: '#f44336' },
    ],
    example: 'If you own Apple, Microsoft, Amazon, and Google (all tech), you might have 6 high correlation pairs among these 4 stocks because big tech moves together.',
    additionalNotes: 'Consider reducing positions in one of each highly correlated pair, or adding uncorrelated assets like bonds, commodities, or international stocks.',
  },
  period_change: {
    title: 'Period Change',
    description: 'The total percentage change in price from the beginning to the end of the selected time period. It shows whether the investment gained or lost value, and by how much.',
    interpretation: 'This is your simple return for the period—how much money you made or lost. Positive values mean the price went up; negative values mean it went down.',
    goodValues: [
      { label: '> 20%', description: 'Strong gains', color: '#4caf50' },
      { label: '10-20%', description: 'Good gains', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-10% to -20%', description: 'Moderate losses', color: '#ff9800' },
      { label: '< -20%', description: 'Significant losses', color: '#f44336' },
    ],
    example: 'If a stock started the 90-day period at $100 and ended at $115, the period change is +15%.',
    additionalNotes: 'Period change doesn\'t account for volatility or risk taken—a stock might have had a wild ride to get that return. Consider looking at risk-adjusted metrics like Sharpe Ratio for the full picture.',
  },
  current_price: {
    title: 'Current Price',
    description: 'The most recent closing price for the security at the end of the selected period. This is the last known market price.',
    interpretation: 'This is simply what the stock is worth right now (or at the end of the period you\'re viewing). Compare it to the starting price to see if you\'re up or down.',
    goodValues: [
      { label: 'Any', description: 'Price itself isn\'t "good" or "bad"', color: '#2196f3' },
    ],
    badValues: [],
    example: 'If you\'re viewing 90 days of history and the stock closed today at $152.40, that\'s the current price shown.',
    additionalNotes: 'The current price is just a snapshot. What matters more is whether it\'s moving in the direction you want, and whether the risk level is acceptable.',
  },
  period_high: {
    title: 'Period High',
    description: 'The highest closing price reached during the selected time period. This represents the peak value the security achieved.',
    interpretation: 'This shows the best price you could have sold at during this period. If the current price is far below the period high, the stock has given back gains.',
    goodValues: [
      { label: 'Near current', description: 'Price at or near highs (strength)', color: '#4caf50' },
    ],
    badValues: [
      { label: 'Far above current', description: 'Price declined significantly from peak', color: '#ff9800' },
    ],
    example: 'If a stock hit $180 at some point in the 90-day period but is now at $150, the period high is $180—showing it gave back $30 of gains.',
    additionalNotes: 'When current price equals period high, the stock is making new highs—often a sign of momentum. When far below, it may indicate a correction or downtrend.',
  },
  period_low: {
    title: 'Period Low',
    description: 'The lowest closing price reached during the selected time period. This represents the trough or worst price the security hit.',
    interpretation: 'This shows how far the stock fell at its worst moment. If the current price is far above the period low, the stock has recovered strongly.',
    goodValues: [
      { label: 'Far below current', description: 'Stock recovered significantly', color: '#4caf50' },
    ],
    badValues: [
      { label: 'Near current', description: 'Price at or near lows (weakness)', color: '#ff9800' },
    ],
    example: 'If a stock dropped to $95 at its worst point but is now at $150, the period low is $95—showing a strong recovery of $55.',
    additionalNotes: 'When current price equals period low, the stock is making new lows—often a sign of weakness. When far above, it indicates recovery or uptrend.',
  },
  sentiment_score: {
    title: 'Sentiment Score',
    description: 'A numerical score from -1.0 (very negative) to +1.0 (very positive) that represents the overall sentiment from news articles, analyst reports, and social media about a stock. Calculated using natural language processing (NLP) on recent news.',
    interpretation: 'This tells you whether the "buzz" around a stock is positive or negative. High positive sentiment often precedes price gains; high negative sentiment may precede declines.',
    goodValues: [
      { label: '> 0.5', description: 'Very positive sentiment', color: '#4caf50' },
      { label: '0.2 to 0.5', description: 'Moderately positive', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-0.2 to -0.5', description: 'Moderately negative', color: '#ff9800' },
      { label: '< -0.5', description: 'Very negative sentiment', color: '#f44336' },
    ],
    example: 'A score of +0.7 means recent news is very positive (product launch, earnings beat). A score of -0.6 means very negative (lawsuit, earnings miss, regulatory issues).',
    additionalNotes: 'Sentiment is a leading indicator—it often changes before price does. However, extremely high sentiment can sometimes signal a top (everyone is bullish), while extreme negative sentiment can signal a bottom.',
  },
  sentiment_trend: {
    title: 'Sentiment Trend',
    description: 'The direction sentiment is moving over the recent period—improving, declining, or stable. This shows whether news is getting more positive or more negative.',
    interpretation: 'An improving trend means sentiment is becoming more positive over time. A declining trend means it\'s becoming more negative. This can help you catch momentum shifts early.',
    goodValues: [
      { label: 'Improving', description: 'Sentiment getting more positive', color: '#4caf50' },
      { label: 'Stable', description: 'Sentiment unchanged', color: '#2196f3' },
    ],
    badValues: [
      { label: 'Declining', description: 'Sentiment getting more negative', color: '#f44336' },
    ],
    example: 'If sentiment was +0.2 a week ago and is now +0.6, the trend is "improving"—news is getting more positive.',
    additionalNotes: 'Watch for divergences: if sentiment is declining but price is rising, the rally may be losing steam. If sentiment is improving but price is falling, it might be a buying opportunity.',
  },
  price_momentum: {
    title: 'Price Momentum',
    description: 'The recent direction of price movement—bullish (uptrend), bearish (downtrend), or neutral. This is typically calculated using moving averages or rate of change.',
    interpretation: 'Momentum tells you if the stock is in an uptrend or downtrend. Bullish momentum means prices are rising consistently; bearish momentum means they\'re falling.',
    goodValues: [
      { label: 'Bullish', description: 'Strong uptrend', color: '#4caf50' },
      { label: 'Neutral', description: 'No clear trend', color: '#2196f3' },
    ],
    badValues: [
      { label: 'Bearish', description: 'Downtrend', color: '#f44336' },
    ],
    example: 'If a stock has risen 15% over the past month with consistent gains, momentum is "bullish." If it\'s fallen 10% with consistent declines, momentum is "bearish."',
    additionalNotes: 'Momentum is a powerful force—stocks in strong uptrends tend to continue up, and vice versa. However, extremely strong momentum can sometimes signal an exhaustion point.',
  },
  divergence_signal: {
    title: 'Divergence Signal',
    description: 'A signal that occurs when sentiment and price move in opposite directions. Positive divergence: price falls but sentiment improves. Negative divergence: price rises but sentiment declines.',
    interpretation: 'Divergences can signal turning points. Positive divergence may indicate an upcoming rally (smart money is optimistic despite price drops). Negative divergence may warn of a correction.',
    goodValues: [
      { label: 'Positive', description: 'Sentiment improving while price falls (potential bottom)', color: '#4caf50' },
      { label: 'None', description: 'No divergence (alignment)', color: '#2196f3' },
    ],
    badValues: [
      { label: 'Negative', description: 'Sentiment declining while price rises (potential top)', color: '#ff9800' },
    ],
    example: 'Stock drops 10% on earnings miss, but sentiment quickly improves as analysts say "overreaction." This positive divergence might signal a bounce.',
    additionalNotes: 'Divergences are most reliable when combined with other indicators. Not all divergences lead to reversals—confirm with volume, technical patterns, or fundamental changes.',
  },
  beta_volatility: {
    title: 'Beta Volatility',
    description: 'Beta volatility (σ) measures how stable or unstable a security\'s beta is over time. It is calculated as the standard deviation of rolling beta values. High beta volatility indicates the correlation with the market is changing frequently, while low beta volatility means consistent market correlation.',
    interpretation: 'Think of beta volatility as measuring how "reliable" the beta metric is. A stable beta (low volatility) means the security\'s relationship with the market is predictable. High beta volatility suggests regime changes, company transitions, or market uncertainty affecting the correlation.',
    goodValues: [
      { label: '< 0.15', description: 'Very stable correlation (predictable)', color: '#4caf50' },
      { label: '0.15-0.30', description: 'Moderate stability (normal variation)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '0.30-0.50', description: 'Unstable correlation (changing dynamics)', color: '#ff9800' },
      { label: '> 0.50', description: 'Highly unstable (unreliable beta)', color: '#f44336' },
    ],
    example: 'A utility stock might have beta volatility of 0.10 (stable defensive nature), while a biotech stock might have 0.60 (correlation changes with news, trials, market sentiment).',
    formula: 'σ(β) = √(Var(rolling_beta_values))',
    additionalNotes: 'High beta volatility often occurs during market regime changes, company pivots, or sector rotations. When beta volatility is high, be cautious using beta for hedging or portfolio construction—the relationship may not hold.',
  },
  rolling_beta: {
    title: 'Rolling Beta',
    description: 'Rolling beta shows how a security\'s beta (correlation with the market) changes over time by calculating beta using a moving window (e.g., 30, 60, or 90 days). This reveals whether the security is becoming more or less correlated with market movements over time.',
    interpretation: 'Rolling beta helps you see if your investment\'s market sensitivity is increasing, decreasing, or staying constant. An upward trend means growing correlation with the market; a downward trend means becoming more independent from market movements.',
    goodValues: [
      { label: 'Stable trend', description: 'Beta stays within 0.2 range (predictable)', color: '#4caf50' },
      { label: 'Gradual change', description: 'Slow movement (manageable shifts)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Erratic swings', description: 'Beta changes rapidly (unstable)', color: '#ff9800' },
      { label: 'Regime changes', description: 'Sudden jumps (fundamental shift)', color: '#f44336' },
    ],
    example: 'A stock\'s beta trends from 0.8 to 1.4 over 6 months. This suggests the company is becoming more sensitive to market conditions—perhaps due to increased debt, sector momentum, or operational changes.',
    additionalNotes: 'Watch for regime changes where beta suddenly shifts—these often coincide with business model changes, sector rotations, or macroeconomic shifts. Different window sizes (30/60/90 days) reveal short-term vs. long-term correlation trends.',
  },
  beta_forecast: {
    title: 'Beta Forecast',
    description: 'A statistical prediction of what a security\'s beta will be in the future (typically 30-90 days ahead). Uses historical rolling beta patterns, mean reversion, and exponential smoothing to project future market correlation. Includes confidence intervals to show prediction uncertainty.',
    interpretation: 'Beta forecasting helps anticipate whether a security will become more or less correlated with the market. If forecast shows increasing beta, expect the security to become more volatile and market-dependent. Decreasing forecast beta suggests growing independence from market movements.',
    goodValues: [
      { label: 'Narrowing CI', description: 'High confidence prediction (reliable)', color: '#4caf50' },
      { label: 'Stable forecast', description: 'Minimal change predicted (consistency)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Wide CI', description: 'High uncertainty (use caution)', color: '#ff9800' },
      { label: 'Large shifts', description: 'Major changes expected (risky)', color: '#f44336' },
    ],
    example: 'Current beta is 1.2, forecast predicts 0.9 in 60 days. This suggests the security may become less volatile and more defensive, possibly due to mean reversion or changing market conditions.',
    formula: 'Ensemble: 60% Mean Reversion + 30% Exponential Smoothing + 10% Linear Regression',
    additionalNotes: 'Beta forecasts are estimates with uncertainty—watch for warnings about insufficient data or high volatility. Regime changes can invalidate forecasts. Use forecasts as one input among many, not as definitive predictions.',
  },
  combined_sentiment: {
    title: 'Combined Sentiment Score',
    description: 'A comprehensive sentiment score ranging from -100 (very negative) to +100 (very positive) that aggregates sentiment signals from multiple sources: news articles, SEC filings (8-Ks), and insider trading activity. The combined score weighs these sources based on their reliability and recency.',
    interpretation: 'This score gives you a holistic view of market sentiment around a security. Positive scores suggest optimism across multiple information sources, while negative scores indicate widespread concern. The multi-source approach reduces noise and provides more reliable sentiment signals than any single source alone.',
    goodValues: [
      { label: '> 50', description: 'Very positive (strong bullish signals)', color: '#4caf50' },
      { label: '20 to 50', description: 'Positive (moderately bullish)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-20 to -50', description: 'Negative (moderately bearish)', color: '#ff9800' },
      { label: '< -50', description: 'Very negative (strong bearish signals)', color: '#f44336' },
    ],
    example: 'A score of +65 means news is positive, SEC filings show favorable developments, and insiders are buying. A score of -40 might indicate negative news, concerning regulatory filings, and insider selling.',
    additionalNotes: 'Pay attention to divergence flags—when sources disagree (e.g., positive news but insider selling), it signals conflicting information that requires deeper investigation. High confidence levels mean sources agree; low confidence means mixed signals.',
  },
  combined_confidence: {
    title: 'Confidence Level',
    description: 'Measures how much agreement exists between different sentiment sources (news, SEC filings, insider activity). High confidence means all sources point in the same direction; low confidence indicates conflicting signals or limited data.',
    interpretation: 'Use confidence to assess reliability of the combined sentiment score. High confidence scores are more actionable. Low confidence suggests market uncertainty, conflicting narratives, or insufficient data—proceed with caution and gather more information.',
    goodValues: [
      { label: 'VERY HIGH', description: 'All sources agree (reliable signal)', color: '#4caf50' },
      { label: 'HIGH', description: 'Strong agreement (trustworthy)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'MEDIUM', description: 'Some disagreement (verify further)', color: '#ff9800' },
      { label: 'LOW', description: 'Conflicting signals (unreliable)', color: '#f44336' },
    ],
    example: 'Very High confidence with +60 score means news, filings, and insiders all bullish—strong signal. Low confidence with +20 score means mixed signals (e.g., positive news but insider selling)—requires investigation.',
    additionalNotes: 'Low confidence isn\'t always bad—it can signal an inflection point or developing story. Check divergence flags to understand what sources disagree on.',
  },
  news_sentiment_enhanced: {
    title: 'News Sentiment',
    description: 'Sentiment derived from analyzing recent news articles about the security. Uses natural language processing to assess whether news coverage is positive, negative, or neutral. Factors in article sources, publication dates, and relevance.',
    interpretation: 'News sentiment reflects how media portrays the company and how the market narrative is developing. Positive news sentiment often precedes price movements as investors react to coverage. Consider this alongside other sentiment sources for validation.',
    goodValues: [
      { label: '> 50', description: 'Highly positive coverage', color: '#4caf50' },
      { label: '20 to 50', description: 'Moderately positive coverage', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-20 to -50', description: 'Negative coverage', color: '#ff9800' },
      { label: '< -50', description: 'Very negative coverage', color: '#f44336' },
    ],
    example: 'After a product launch, news sentiment jumps to +70 as media covers the announcement positively. After a scandal, sentiment might drop to -60 as negative stories dominate.',
    additionalNotes: 'News sentiment can be noisy and reactive to short-term events. Look for sustained trends rather than single-day spikes. Compare with insider activity—if insiders are buying despite negative news, the negativity may be overblown.',
  },
  sec_filing_sentiment: {
    title: 'SEC Filing Sentiment',
    description: 'Sentiment extracted from analyzing SEC 8-K filings (material event reports) that companies must file when significant events occur. The sentiment reflects whether these events are positive (acquisitions, new contracts, earnings beats) or negative (lawsuits, executive departures, going concern warnings).',
    interpretation: 'SEC filings provide the most reliable corporate news because they\'re legally required disclosures. Unlike news or rumors, 8-Ks contain verified information directly from the company. Positive filing sentiment indicates corporate developments favor shareholders.',
    goodValues: [
      { label: '> 50', description: 'Highly favorable corporate events', color: '#4caf50' },
      { label: '20 to 50', description: 'Positive developments', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-20 to -50', description: 'Concerning events', color: '#ff9800' },
      { label: '< -50', description: 'Serious negative developments', color: '#f44336' },
    ],
    example: '8-K announcing major contract win yields +80 score. 8-K disclosing CEO departure and accounting restatement yields -70 score. Multiple small filings with neutral impact yield score near 0.',
    additionalNotes: 'Pay attention to the "Material Events" section to see what triggered the score. Critical importance events carry more weight. Absence of recent filings means no significant events—not necessarily good or bad.',
  },
  insider_sentiment: {
    title: 'Insider Activity Sentiment',
    description: 'Sentiment based on recent insider buying and selling transactions. Insiders (executives, directors, major shareholders) must report trades to the SEC. Heavy buying suggests insiders believe the stock is undervalued; heavy selling may indicate overvaluation or personal financial needs.',
    interpretation: 'Insiders have the best information about their company. Buying is generally a stronger signal than selling (insiders sell for many reasons unrelated to company prospects). Look for clusters of buying by multiple insiders or unusually large purchases.',
    goodValues: [
      { label: '> 50', description: 'Strong insider buying (bullish signal)', color: '#4caf50' },
      { label: '20 to 50', description: 'Net insider buying (positive)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-20 to -50', description: 'Net insider selling (caution)', color: '#ff9800' },
      { label: '< -50', description: 'Heavy insider selling (bearish)', color: '#f44336' },
    ],
    example: 'Three executives buy shares totaling $5M ahead of earnings: +75 sentiment. CEO sells 50% of holdings for "estate planning": -45 sentiment. Routine 10b5-1 plan sales: near 0 (pre-scheduled, less meaningful).',
    additionalNotes: 'Check transaction details—large open market purchases by C-suite executives are most significant. Automatic sales plans (10b5-1) are less informative. Compare with news and filings—insider buying during negative news is especially bullish.',
  },
  downside_deviation: {
    title: 'Downside Deviation (Semi-Deviation)',
    description: 'Downside deviation measures the volatility of returns that fall below a target threshold (Minimum Acceptable Return or MAR). Unlike standard deviation which penalizes both upside and downside volatility, downside deviation only focuses on negative returns below the target.',
    interpretation: 'This metric shows how much your investment fluctuates on the downside specifically. Higher downside deviation means larger and more frequent losses below your acceptable threshold. It\'s a more relevant risk measure for investors who don\'t consider upside volatility as "risk."',
    goodValues: [
      { label: '< 10%', description: 'Very low downside risk', color: '#4caf50' },
      { label: '10-15%', description: 'Low downside risk', color: '#8bc34a' },
    ],
    badValues: [
      { label: '15-25%', description: 'Moderate downside risk', color: '#ff9800' },
      { label: '> 25%', description: 'High downside risk', color: '#f44336' },
    ],
    example: 'A stock with 8% downside deviation means when it has negative returns below your target, those losses average around 8% annually. A stock with 30% downside deviation experiences severe losses below target.',
    formula: 'σ_downside = √(∑(min(return - MAR, 0)²) / N)',
    additionalNotes: 'Downside deviation is used in the Sortino Ratio calculation. It\'s particularly useful for risk-averse investors focused on capital preservation. The MAR (Minimum Acceptable Return) is typically set to the risk-free rate or 0%.',
  },
  portfolio_cvar_95: {
    title: 'Portfolio CVaR (95%)',
    description: 'Conditional Value at Risk (CVaR), also called Expected Shortfall, measures the average loss in the worst 5% of scenarios for your entire portfolio. While VaR tells you the threshold, CVaR tells you the average loss when that threshold is breached.',
    interpretation: 'This metric answers: "When things go bad (worst 5% of days), how bad is the average loss for my portfolio?" It provides a more complete picture of tail risk than VaR alone.',
    goodValues: [
      { label: '> -5%', description: 'Low portfolio tail risk', color: '#4caf50' },
      { label: '-5% to -10%', description: 'Moderate portfolio tail risk', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-10% to -15%', description: 'High portfolio tail risk', color: '#ff9800' },
      { label: '< -15%', description: 'Very high portfolio tail risk', color: '#f44336' },
    ],
    example: 'If your portfolio VaR(95%) is -3% and CVaR(95%) is -5%, it means: 95% of days you\'ll lose less than 3%, but in the worst 5% of days, your average loss is 5%.',
    formula: 'CVaR = Average(losses worse than VaR threshold)',
    additionalNotes: 'CVaR is always worse (more negative) than VaR because it captures the full distribution of tail losses. It\'s the preferred risk measure for portfolio managers because it satisfies the "coherent risk measure" properties.',
  },
  portfolio_cvar_99: {
    title: 'Portfolio CVaR (99%)',
    description: 'The average loss for your entire portfolio in the worst 1% of scenarios (roughly 2-3 worst days per year). This is your extreme crisis scenario measure for portfolio-level risk.',
    interpretation: 'This tells you what to expect during rare but catastrophic market events that affect your whole portfolio. Use this for stress testing and understanding maximum portfolio drawdown potential.',
    goodValues: [
      { label: '> -8%', description: 'Low portfolio crisis risk', color: '#4caf50' },
      { label: '-8% to -15%', description: 'Moderate portfolio crisis risk', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-15% to -25%', description: 'High portfolio crisis risk', color: '#ff9800' },
      { label: '< -25%', description: 'Extreme portfolio crisis risk', color: '#f44336' },
    ],
    example: 'Portfolio CVaR(99%) of -12% means on the worst 2-3 days per year, your portfolio loses an average of 12% of its value.',
    additionalNotes: 'This metric revealed itself dramatically during the 2020 COVID crash and 2008 financial crisis. Use it to ensure your portfolio can survive worst-case scenarios without forcing you to liquidate at the worst possible time.',
  },
  position_cvar_95: {
    title: 'Position CVaR (95%)',
    description: 'Expected Shortfall for an individual position, measuring the average loss of that specific holding in the worst 5% of scenarios. This helps identify which positions contribute most to portfolio tail risk.',
    interpretation: 'Use this to identify your riskiest positions in terms of tail risk. Positions with high CVaR values are the ones that will hurt most during market stress events.',
    goodValues: [
      { label: '> -4%', description: 'Low position tail risk', color: '#4caf50' },
      { label: '-4% to -8%', description: 'Moderate position tail risk', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-8% to -15%', description: 'High position tail risk', color: '#ff9800' },
      { label: '< -15%', description: 'Very high position tail risk', color: '#f44336' },
    ],
    example: 'A tech stock with -12% CVaR(95%) means in its worst 5% of days, it loses an average of 12%. A utility stock might have -3% CVaR(95%).',
    additionalNotes: 'Compare position CVaR values to identify concentration risk. Consider reducing positions with extreme CVaR values or hedging them during uncertain market conditions.',
  },
  mar: {
    title: 'Minimum Acceptable Return (MAR)',
    description: 'The threshold return below which you consider performance unacceptable. MAR is used as the benchmark for calculating downside deviation and Sortino Ratio. It\'s typically set to the risk-free rate (treasury yield) or 0%.',
    interpretation: 'MAR represents your personal risk tolerance threshold. Returns below this level are considered "downside" and contribute to downside risk metrics. Setting an appropriate MAR helps focus risk measurement on what matters to you.',
    goodValues: [
      { label: '0-3%', description: 'Conservative MAR (risk-free rate)', color: '#4caf50' },
      { label: '3-5%', description: 'Moderate MAR', color: '#8bc34a' },
    ],
    badValues: [
      { label: '5-8%', description: 'Aggressive MAR', color: '#ff9800' },
      { label: '> 8%', description: 'Very aggressive MAR', color: '#f44336' },
    ],
    example: 'If MAR is set to 2% (risk-free rate), any daily return below 2% annualized contributes to downside deviation. If you set MAR to 0%, only negative returns count as downside.',
    additionalNotes: 'Lower MAR makes the Sortino Ratio and downside deviation more strict (higher downside risk). Higher MAR is more lenient. Most investors use 0% or the current risk-free rate as MAR.',
  },
  market_regime: {
    title: 'Market Regime',
    description: 'Market regimes classify the current market environment into distinct states based on volatility patterns and price behavior. The system uses Hidden Markov Models (HMM) to detect four regimes: Bull (rising prices, moderate volatility), Bear (falling prices, elevated stress), Normal (stable, low volatility), and High Volatility (turbulent, unpredictable).',
    interpretation: 'Understanding the current regime helps you adjust your risk management strategy. Bull markets allow for more aggressive positioning, while Bear and High Volatility regimes warrant defensive measures. The regime detection combines statistical analysis with machine learning to identify regime shifts before they become obvious.',
    goodValues: [
      { label: 'Bull', description: 'Rising prices, positive momentum, moderate vol', color: '#4caf50' },
      { label: 'Normal', description: 'Stable conditions, low volatility, calm markets', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Bear', description: 'Falling prices, negative sentiment, stress', color: '#ff9800' },
      { label: 'High Volatility', description: 'Turbulent, large swings, uncertainty', color: '#f44336' },
    ],
    example: 'During the 2021 bull market, the regime stayed "Bull" for months with 85%+ confidence. During the COVID crash in March 2020, the system quickly shifted to "High Volatility" regime, signaling extreme risk.',
    additionalNotes: 'Regimes persist for weeks or months—they don\'t change daily. When regime confidence is low (below 70%), the market may be transitioning between states. Risk thresholds automatically adjust based on the detected regime.',
  },
  regime_confidence: {
    title: 'Regime Detection Confidence',
    description: 'The confidence level (0-100%) that the detected market regime is correct. This is calculated by the Hidden Markov Model based on how well current market behavior matches the expected patterns for each regime type. High confidence means the current data strongly supports the identified regime.',
    interpretation: 'Confidence tells you how certain the model is about the regime classification. Above 80% is very confident—the market clearly exhibits the regime\'s characteristics. Between 60-80% is moderate confidence—the regime is likely but not definitive. Below 60% suggests regime uncertainty or potential transition.',
    goodValues: [
      { label: '> 80%', description: 'Very confident (clear regime signals)', color: '#4caf50' },
      { label: '70-80%', description: 'Confident (solid regime identification)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '60-70%', description: 'Moderate (regime may be transitioning)', color: '#ff9800' },
      { label: '< 60%', description: 'Low confidence (uncertain regime)', color: '#f44336' },
    ],
    example: 'Bull regime with 92% confidence means market behavior strongly matches historical bull patterns. Bear regime with 55% confidence suggests the market might be transitioning between regimes—exercise caution.',
    additionalNotes: 'Low confidence often occurs at regime transition points—when markets shift from Bull to Bear or vice versa. During these periods, use additional indicators and be more conservative with risk.',
  },
  volatility_level: {
    title: 'Volatility Level (Market)',
    description: 'The current annualized volatility of the overall market, measured using a rolling window of recent returns. This represents how much the market index (e.g., S&P 500) is fluctuating. It\'s a key input for regime detection—stable volatility suggests Normal regimes, while spikes indicate High Volatility or Bear regimes.',
    interpretation: 'Market volatility sets the context for your portfolio risk. When market volatility is high (>25%), even conservative portfolios experience larger swings. When it\'s low (<15%), markets are calm and predictable. Your portfolio risk should be interpreted relative to market conditions.',
    goodValues: [
      { label: '< 12%', description: 'Very calm market (low risk environment)', color: '#4caf50' },
      { label: '12-18%', description: 'Normal market (typical conditions)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '18-30%', description: 'Elevated market stress (caution)', color: '#ff9800' },
      { label: '> 30%', description: 'Very high market turbulence (crisis)', color: '#f44336' },
    ],
    example: 'During calm periods like 2017, market volatility stayed around 10-12%. During the 2020 COVID crash, it spiked to 60-80%. The long-term average is about 15-18%.',
    additionalNotes: 'Market volatility tends to mean-revert—extremely high volatility rarely persists. Use this metric to gauge whether current market conditions are normal or exceptional. Portfolio risk metrics are more meaningful when viewed in context of market volatility.',
  },
  threshold_multiplier: {
    title: 'Risk Threshold Multiplier',
    description: 'A dynamic adjustment factor applied to risk thresholds based on the current market regime. In Bull/Normal regimes, the multiplier may tighten thresholds (< 1.0) to detect elevated risk earlier. In Bear/High Volatility regimes, it relaxes thresholds (> 1.0) to avoid false alarms, since higher volatility is expected.',
    interpretation: 'This multiplier helps provide regime-aware risk assessment. A 0.8x multiplier in a Bull market means the system is more sensitive—flagging risk earlier because volatility should be low. A 1.5x multiplier in a Bear market means higher tolerance—accepting that volatility is naturally elevated during stress.',
    goodValues: [
      { label: '0.7-0.9x', description: 'Tightened thresholds (sensitive detection)', color: '#4caf50' },
      { label: '0.9-1.1x', description: 'Standard thresholds (normal sensitivity)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '1.1-1.5x', description: 'Relaxed thresholds (crisis mode)', color: '#ff9800' },
      { label: '> 1.5x', description: 'Very relaxed (extreme market stress)', color: '#f44336' },
    ],
    example: 'In a Bull market with 0.85x multiplier, a base volatility threshold of 30% becomes 25.5% (30% × 0.85). In a Bear market with 1.4x multiplier, the same threshold becomes 42% (30% × 1.4), recognizing elevated volatility is normal.',
    formula: 'Adjusted_Threshold = Base_Threshold × Multiplier',
    additionalNotes: 'The multiplier prevents "crying wolf" during known volatile regimes while maintaining sensitivity during calm periods. It makes risk assessments context-aware and more actionable.',
  },
  hmm_probabilities: {
    title: 'HMM State Probabilities',
    description: 'Hidden Markov Model (HMM) probabilities represent the likelihood that the market is currently in each of the four possible regimes: Bull, Normal, Bear, and High Volatility. The probabilities sum to 100%. The regime with the highest probability is selected as the current regime.',
    interpretation: 'These probabilities reveal the model\'s confidence distribution across all regimes. If one regime has 85% and others have low probabilities, the model is very certain. If probabilities are split (e.g., 45% Bull, 40% Normal), the market is in a transitional or ambiguous state.',
    goodValues: [
      { label: 'One clear winner', description: 'One regime >70%, others low (decisive)', color: '#4caf50' },
      { label: 'Dominant regime', description: 'One regime 60-70%, others moderate', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Split decision', description: 'Two regimes 40-50% each (ambiguous)', color: '#ff9800' },
      { label: 'Uniform spread', description: 'All regimes 20-30% (very uncertain)', color: '#f44336' },
    ],
    example: 'Bull: 82%, Normal: 12%, Bear: 4%, High Vol: 2% indicates a strong bull market. Bull: 48%, Normal: 45%, Bear: 5%, High Vol: 2% suggests the market is between Bull and Normal—unclear regime.',
    additionalNotes: 'Watch for probability shifts over time—a rising Bear probability might signal an upcoming regime change. The HMM uses historical market patterns and volatility to calculate these probabilities.',
  },
  predicted_regime: {
    title: 'Predicted Market Regime',
    description: 'The forecasted market regime for a future time horizon (5, 10, or 30 days ahead). The prediction is made by the Hidden Markov Model using transition probabilities—the likelihood of moving from the current regime to another regime over time.',
    interpretation: 'Regime forecasts help you anticipate market conditions ahead. If a Bull regime is predicted to shift to High Volatility in 30 days, you might begin defensive positioning. Forecasts become less reliable at longer horizons due to increased uncertainty.',
    goodValues: [
      { label: 'Bull', description: 'Forecast expects favorable conditions', color: '#4caf50' },
      { label: 'Normal', description: 'Forecast expects stable conditions', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Bear', description: 'Forecast expects declining markets', color: '#ff9800' },
      { label: 'High Volatility', description: 'Forecast expects turbulent conditions', color: '#f44336' },
    ],
    example: 'Current regime is Bull, but 30-day forecast shows Bear with 65% confidence. This signals potential deterioration ahead—consider reducing risk exposure or adding hedges.',
    additionalNotes: 'Regime forecasts are probabilistic, not deterministic. Low forecast confidence (<60%) means the model is uncertain. Forecasts work best for 5-10 day horizons; 30-day forecasts have higher uncertainty.',
  },
  forecast_confidence: {
    title: 'Forecast Confidence',
    description: 'The confidence level (0-100%) in the regime forecast. This reflects how certain the model is about the predicted regime for the specified time horizon. Confidence naturally decreases as the forecast horizon extends further into the future.',
    interpretation: 'High forecast confidence (>70%) suggests the prediction is reliable based on current regime stability and transition patterns. Low confidence (<60%) means the forecast is uncertain—market conditions could evolve in multiple ways.',
    goodValues: [
      { label: '> 70%', description: 'High confidence (reliable forecast)', color: '#4caf50' },
      { label: '60-70%', description: 'Moderate confidence (reasonable forecast)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '50-60%', description: 'Low confidence (uncertain forecast)', color: '#ff9800' },
      { label: '< 50%', description: 'Very low confidence (unreliable)', color: '#f44336' },
    ],
    example: '5-day forecast with 85% confidence is highly reliable. 30-day forecast with 52% confidence should be viewed as one possible scenario among several.',
    additionalNotes: 'Confidence decreases with time horizon—30-day forecasts are inherently less reliable than 5-day forecasts. Use low-confidence forecasts as possibilities to monitor, not definitive predictions to trade on.',
  },
  transition_probability: {
    title: 'Regime Transition Probability',
    description: 'The probability (0-100%) that the market will transition from the current regime to a different regime by the forecast horizon. A high transition probability means regime change is likely; a low probability suggests the current regime will persist.',
    interpretation: 'This metric tells you whether to expect regime stability or change. High transition probability (>50%) signals potential regime shift ahead—be prepared for changing market conditions. Low transition probability (<20%) suggests the current regime is likely to continue.',
    goodValues: [
      { label: '< 20%', description: 'Very stable regime (low change risk)', color: '#4caf50' },
      { label: '20-40%', description: 'Stable regime (likely to persist)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '40-60%', description: 'Moderate transition risk (monitor)', color: '#ff9800' },
      { label: '> 60%', description: 'High transition risk (change likely)', color: '#f44336' },
    ],
    example: 'Bull regime with 15% transition probability in 30 days suggests the bull market will likely continue. Bull regime with 72% transition probability warns of likely regime shift—potentially to Bear or High Volatility.',
    additionalNotes: 'Transition probabilities are higher during regime uncertainty or market inflection points. Combine with forecast confidence—high transition probability with low confidence means unclear direction.',
  },
  sentiment_factors: {
    title: 'Sentiment Factors',
    description: 'A multi-source sentiment analysis framework that combines three independent data sources—news articles (40% weight), SEC regulatory filings (30% weight), and insider trading activity (30% weight)—into a comprehensive sentiment profile. Each source provides unique insights: news reflects public perception, SEC filings contain verified corporate events, and insider trades reveal informed insider confidence.',
    interpretation: 'Sentiment factors give you a 360-degree view of market sentiment around a security. By weighting multiple sources, the system reduces noise and identifies more reliable signals. When all sources align (all positive or all negative), confidence is high. When sources diverge, it signals conflicting information that warrants deeper investigation.',
    goodValues: [
      { label: 'All aligned positive', description: 'News, filings, and insiders all bullish (strong signal)', color: '#4caf50' },
      { label: 'Mostly positive', description: '2 of 3 sources positive (moderately bullish)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Mixed signals', description: 'Sources disagree (investigate further)', color: '#ff9800' },
      { label: 'All aligned negative', description: 'News, filings, and insiders all bearish (strong warning)', color: '#f44336' },
    ],
    example: 'AAPL shows +0.65 news sentiment (positive coverage of product launch), +0.45 SEC sentiment (strong earnings report), and +0.55 insider sentiment (CEO buying shares). Combined score: +0.57 with high confidence—all sources agree the outlook is positive.',
    additionalNotes: 'The 40/30/30 weighting gives news slightly more importance as it updates most frequently. SEC filings and insider trades are weighted equally as both provide high-quality, verified information from insiders.',
  },
  sentiment_momentum: {
    title: 'Sentiment Momentum',
    description: 'Measures how sentiment is changing over time by tracking the rate and direction of sentiment shifts across 7-day and 30-day windows. Sentiment momentum includes change metrics (how much sentiment has moved) and acceleration (whether the rate of change is increasing or decreasing).',
    interpretation: 'Momentum reveals whether sentiment is improving, deteriorating, or stable. Positive momentum (increasing sentiment) suggests growing optimism and may precede price gains. Negative momentum (declining sentiment) warns of deteriorating outlook. Acceleration tells you if the trend is speeding up or slowing down.',
    goodValues: [
      { label: 'Positive & accelerating', description: 'Sentiment improving and gaining strength', color: '#4caf50' },
      { label: 'Positive momentum', description: 'Sentiment steadily improving', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Negative momentum', description: 'Sentiment deteriorating', color: '#ff9800' },
      { label: 'Negative & accelerating', description: 'Sentiment rapidly worsening', color: '#f44336' },
    ],
    example: 'A stock with +0.15 sentiment 7 days ago and +0.35 today shows +0.20 positive momentum. If 30-day momentum is +0.10, acceleration is positive—sentiment is improving faster recently than over the longer term.',
    additionalNotes: 'Momentum is a leading indicator—it often changes before price does. Watch for momentum reversals: positive momentum turning negative can signal the end of a rally, while negative momentum turning positive may signal a bottom.',
  },
  sentiment_momentum_7d: {
    title: 'Sentiment Momentum (7-Day Change)',
    description: 'The change in combined sentiment score over the past 7 days. Calculated as: (Current Sentiment) - (Sentiment 7 days ago). Positive values indicate sentiment has improved over the past week; negative values indicate deterioration.',
    interpretation: 'This metric captures recent short-term shifts in market sentiment. A large positive 7-day change suggests rapidly improving sentiment—potentially driven by recent positive news, earnings, or events. A large negative change warns of quickly souring sentiment.',
    goodValues: [
      { label: '> +0.2', description: 'Strong recent improvement (very bullish)', color: '#4caf50' },
      { label: '+0.05 to +0.2', description: 'Moderate improvement (bullish)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-0.05 to -0.2', description: 'Moderate deterioration (bearish)', color: '#ff9800' },
      { label: '< -0.2', description: 'Rapid deterioration (very bearish)', color: '#f44336' },
    ],
    example: 'Sentiment was -0.10 last week and is now +0.15. The 7-day change is +0.25, indicating a significant positive shift—possibly due to earnings beat or positive product news released in the past week.',
    additionalNotes: '7-day momentum is sensitive to short-term events and news cycles. Compare with 30-day momentum to distinguish short-term spikes from sustained trends. Large 7-day moves often revert if not supported by fundamentals.',
  },
  sentiment_momentum_30d: {
    title: 'Sentiment Momentum (30-Day Change)',
    description: 'The change in combined sentiment score over the past 30 days. Calculated as: (Current Sentiment) - (Sentiment 30 days ago). This represents the longer-term trend in sentiment, smoothing out short-term noise and capturing sustained shifts.',
    interpretation: 'This metric reveals whether sentiment is in a sustained trend or just experiencing temporary fluctuations. Large 30-day momentum indicates a persistent sentiment shift that may have more lasting impact on price. Use this to distinguish genuine trend changes from short-term noise.',
    goodValues: [
      { label: '> +0.15', description: 'Strong sustained improvement (persistent bullish trend)', color: '#4caf50' },
      { label: '+0.05 to +0.15', description: 'Moderate sustained improvement', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-0.05 to -0.15', description: 'Sustained deterioration (persistent bearish trend)', color: '#ff9800' },
      { label: '< -0.15', description: 'Severe sustained decline (structural bearish shift)', color: '#f44336' },
    ],
    example: 'Sentiment has gradually improved from -0.20 a month ago to +0.05 now. The 30-day change is +0.25, showing a sustained positive trend—not just a one-day spike. This suggests improving fundamentals or a genuine narrative shift.',
    additionalNotes: '30-day momentum is more reliable than 7-day for identifying lasting trends. When 7-day and 30-day momentum point in the same direction, the trend is likely to continue. Divergence between them (e.g., positive 7-day but negative 30-day) signals potential trend reversal.',
  },
  sentiment_acceleration: {
    title: 'Sentiment Acceleration',
    description: 'Measures whether sentiment momentum is increasing or decreasing—essentially, the "momentum of momentum." Calculated as: (7-day momentum) - (30-day momentum) / 23. Positive acceleration means sentiment is improving faster recently; negative acceleration means the rate of improvement is slowing or deterioration is accelerating.',
    interpretation: 'Acceleration helps identify turning points and trend strength. Positive acceleration during an uptrend confirms strengthening bullish sentiment. Negative acceleration during an uptrend warns the rally may be losing steam. This metric catches inflection points earlier than simple momentum.',
    goodValues: [
      { label: '> +0.02', description: 'Accelerating positive (strengthening uptrend)', color: '#4caf50' },
      { label: '0 to +0.02', description: 'Stable positive (steady improvement)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-0.02 to 0', description: 'Decelerating (weakening trend)', color: '#ff9800' },
      { label: '< -0.02', description: 'Negative acceleration (rapid deterioration)', color: '#f44336' },
    ],
    example: 'Stock has +0.30 7-day momentum and +0.10 30-day momentum. Acceleration = (+0.30 - +0.10) / 23 ≈ +0.009. Positive acceleration shows sentiment improving faster recently—bullish signal suggesting the positive trend is strengthening.',
    formula: 'Acceleration = (Change_7d - Change_30d) / 23',
    additionalNotes: 'Watch for acceleration reversals: negative acceleration during a rally can signal exhaustion before momentum turns negative. Positive acceleration during a decline can signal a bottom is forming. Acceleration is most useful when combined with momentum direction.',
  },
  divergence_score: {
    title: 'Divergence Score',
    description: 'A quantitative measure (-1.0 to +1.0) of how much sentiment and price are moving in opposite directions. Positive divergence occurs when price falls but sentiment improves (potential bottom). Negative divergence occurs when price rises but sentiment deteriorates (potential top). The magnitude indicates divergence strength.',
    interpretation: 'Divergence scores identify situations where market price and underlying sentiment are misaligned. High absolute scores (> 0.5) indicate strong divergence—a signal that price may be about to reverse to align with sentiment. Scores near zero mean price and sentiment are moving together (no divergence).',
    goodValues: [
      { label: '+0.5 to +1.0', description: 'Strong positive divergence (potential bottom)', color: '#4caf50' },
      { label: '+0.2 to +0.5', description: 'Moderate positive divergence (watch for bounce)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-0.2 to -0.5', description: 'Moderate negative divergence (potential reversal)', color: '#ff9800' },
      { label: '-0.5 to -1.0', description: 'Strong negative divergence (warning of correction)', color: '#f44336' },
    ],
    example: 'Stock drops 10% over 2 weeks (negative price momentum), but sentiment improves from -0.30 to +0.15 (positive sentiment momentum). Divergence score: +0.65. This strong positive divergence suggests price may have overreacted and is due for a bounce as sentiment improves.',
    additionalNotes: 'Not all divergences lead to reversals—confirm with other indicators like volume, technical patterns, or fundamental changes. Divergences work best at extremes: strong negative divergence near all-time highs, or strong positive divergence near major support levels.',
  },
  reversal_probability: {
    title: 'Reversal Probability',
    description: 'The estimated probability (0-100%) that a price reversal will occur based on the detected divergence between sentiment and price. Calculated using machine learning models trained on historical divergence patterns and subsequent price movements. Higher probability means reversal is more likely.',
    interpretation: 'Use this probability to assess how actionable a divergence signal is. Probabilities above 60% indicate the divergence is statistically significant and historically has led to reversals. Below 40% means the divergence may not be strong enough to trigger a reversal—price and sentiment may simply re-align without major price movement.',
    goodValues: [
      { label: '> 70%', description: 'Very high reversal likelihood (strong signal)', color: '#4caf50' },
      { label: '60-70%', description: 'High reversal likelihood (reliable signal)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '40-60%', description: 'Moderate probability (uncertain)', color: '#ff9800' },
      { label: '< 40%', description: 'Low probability (weak signal)', color: '#f44336' },
    ],
    example: 'Positive divergence detected with 75% reversal probability. Historically, similar divergence patterns led to price reversals 75% of the time within 10-20 trading days. This is a statistically reliable signal worth acting on.',
    additionalNotes: 'Reversal probability is based on historical patterns—past performance doesn\'t guarantee future results. Higher magnitude divergences generally have higher reversal probabilities. Combine with stop-losses when trading on divergence signals in case the 25-30% non-reversal scenario occurs.',
  },
  sentiment_adjusted_forecast: {
    title: 'Sentiment-Adjusted Price Forecast',
    description: 'A price forecast that combines traditional technical forecasting (moving averages, momentum, trend analysis) with sentiment analysis from news, SEC filings, and insider activity. The base forecast is adjusted up or down based on sentiment factors, with the adjustment magnitude determined by sentiment strength and reliability.',
    interpretation: 'The sentiment-adjusted forecast provides a more complete picture than technical analysis alone. When sentiment is positive, the adjusted forecast will be higher than the base forecast—reflecting optimism that may drive price up. When sentiment is negative, the adjusted forecast is lower—incorporating pessimism that may pressure prices.',
    goodValues: [
      { label: 'Adjusted > Base', description: 'Positive sentiment adjustment (bullish outlook)', color: '#4caf50' },
      { label: 'Slight adjustment', description: 'Sentiment slightly positive or neutral', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Adjusted < Base', description: 'Negative sentiment adjustment (bearish outlook)', color: '#ff9800' },
      { label: 'Large downward adj', description: 'Strongly negative sentiment (warning)', color: '#f44336' },
    ],
    example: 'Base technical forecast predicts $150 in 30 days. Combined sentiment is +0.45 (positive). Sentiment-adjusted forecast: $158 (+5.3% adjustment). The positive sentiment suggests price may outperform technical expectations.',
    formula: 'Adjusted_Forecast = Base_Forecast × (1 + (Sentiment_Score × Adjustment_Factor))',
    additionalNotes: 'The forecast includes confidence intervals showing uncertainty. Wide intervals mean high uncertainty—use caution. Compare base vs adjusted forecasts: large divergence means sentiment is a strong factor. The model accounts for sentiment reliability—low-confidence sentiment has less impact on the adjustment.',
  },
  trading_signal_overall: {
    title: 'Overall Trading Recommendation',
    description: 'A comprehensive buy/sell/hold recommendation that synthesizes all individual trading signals (momentum, mean reversion, trend) into a single actionable decision. The recommendation includes an action (Buy/Sell/Hold), strength (Strong/Moderate/Weak), probability, and rationale explaining the decision.',
    interpretation: 'This is your "bottom line" signal—the aggregated recommendation across all technical factors. A "Strong Buy" with 75% probability means most technical indicators align bullish and historically similar setups produced positive returns 75% of the time. Use this as your primary decision signal, but review individual signals to understand what\'s driving it.',
    goodValues: [
      { label: 'Strong Buy', description: 'All signals align bullish (high probability)', color: '#4caf50' },
      { label: 'Moderate Buy', description: 'Most signals bullish (good probability)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Moderate Sell', description: 'Most signals bearish (caution)', color: '#ff9800' },
      { label: 'Strong Sell', description: 'All signals align bearish (high risk)', color: '#f44336' },
    ],
    example: 'Strong Buy with 78% probability: Momentum positive (RSI oversold recovery), Mean Reversion positive (price bouncing from lower Bollinger Band), and Trend positive (50-day MA crossing above 200-day MA). Three confirmations make this a high-confidence signal.',
    additionalNotes: 'The recommendation is only as good as current market conditions—it doesn\'t account for unexpected news or black swan events. Always combine with fundamental analysis, risk management, and your personal investment strategy. Past probability doesn\'t guarantee future results.',
  },
  trading_signal_momentum: {
    title: 'Momentum Signal',
    description: 'A signal based on momentum indicators (RSI, MACD, volume trends) that identifies whether price momentum is building in a bullish or bearish direction. Momentum signals work best when markets are trending strongly in one direction.',
    interpretation: 'Momentum is like a train gaining speed—once it starts moving, it tends to continue. A bullish momentum signal suggests buyers are in control and pushing prices higher. A bearish signal suggests sellers dominate. Momentum works best in trending markets but can give false signals in choppy, sideways markets.',
    goodValues: [
      { label: 'Bullish + High Conf', description: 'Strong upward momentum (trending up)', color: '#4caf50' },
      { label: 'Bullish + Med Conf', description: 'Moderate upward momentum', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Bearish + Med Conf', description: 'Moderate downward momentum', color: '#ff9800' },
      { label: 'Bearish + High Conf', description: 'Strong downward momentum (trending down)', color: '#f44336' },
    ],
    example: 'Bullish momentum signal with 72% probability: RSI crossed above 50 (gaining strength), MACD histogram turned positive (momentum shift), and volume increased on up days (confirming buying interest). This suggests upward price continuation is likely.',
    additionalNotes: 'Momentum signals can lag—by the time momentum is clear, much of the move may be done. They perform poorly in sideways/choppy markets where momentum constantly reverses. Best used in conjunction with trend confirmation. High RSI (>70) can signal overbought conditions where momentum may reverse.',
  },
  trading_signal_mean_reversion: {
    title: 'Mean Reversion Signal',
    description: 'A signal that identifies when price has deviated significantly from its average and is likely to "snap back" toward the mean. Based on Bollinger Bands, price deviations, and statistical measures. Mean reversion works best in range-bound, non-trending markets.',
    interpretation: 'Mean reversion is based on the idea that prices tend to return to their average over time. When a stock is extremely oversold (price near lower Bollinger Band), a bullish mean reversion signal suggests it may bounce back up. When extremely overbought, a bearish signal suggests it may pull back down.',
    goodValues: [
      { label: 'Bullish + High Conf', description: 'Oversold, likely to bounce up', color: '#4caf50' },
      { label: 'Bullish + Med Conf', description: 'Below average, recovery likely', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Bearish + Med Conf', description: 'Overbought, pullback likely', color: '#ff9800' },
      { label: 'Bearish + High Conf', description: 'Extremely overbought, correction likely', color: '#f44336' },
    ],
    example: 'Bullish mean reversion signal with 68% probability: Price touched the lower Bollinger Band (2 standard deviations below mean) and is starting to move back up. Historically, when this stock reached this level, it rebounded 68% of the time within 10 trading days.',
    additionalNotes: 'Mean reversion fails in strong trending markets—if a stock is trending down due to fundamental problems, it may keep going lower instead of reverting. Works best with stocks that have stable fundamentals and trade in a range. Can result in "catching a falling knife" if used carelessly during crashes.',
  },
  trading_signal_trend: {
    title: 'Trend Signal',
    description: 'A signal based on trend-following indicators (moving averages, trend strength, directional movement) that identifies the prevailing price direction and its strength. Trend signals aim to "ride the wave" of established trends.',
    interpretation: 'The trend is your friend—this signal tells you which direction the market is moving and whether the trend is strong or weakening. A bullish trend signal means the stock is in an uptrend and likely to continue higher. A bearish signal means it\'s in a downtrend. Trend signals help you avoid fighting the market.',
    goodValues: [
      { label: 'Bullish + High Conf', description: 'Strong uptrend (ride the wave)', color: '#4caf50' },
      { label: 'Bullish + Med Conf', description: 'Moderate uptrend (cautiously bullish)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Bearish + Med Conf', description: 'Moderate downtrend (caution)', color: '#ff9800' },
      { label: 'Bearish + High Conf', description: 'Strong downtrend (stay away)', color: '#f44336' },
    ],
    example: 'Bullish trend signal with 81% probability: 50-day moving average crossed above 200-day MA ("golden cross"), price consistently making higher highs and higher lows, and ADX shows strong trend strength. This is a classic strong uptrend setup.',
    additionalNotes: 'Trends don\'t last forever—even strong trends eventually reverse. Trend signals can be slow to react at trend reversals, causing you to hold too long. Best combined with momentum and mean reversion signals. "The trend is your friend until it ends"—be ready to exit when the trend weakens.',
  },
  trading_signal_combined: {
    title: 'Combined Signal',
    description: 'A meta-signal that combines all signal types (momentum, mean reversion, trend) with intelligent weighting based on current market conditions. When all signals align, confidence is high. When signals conflict, the combined signal reflects the uncertainty.',
    interpretation: 'The combined signal is your highest-level view—it tells you when all technical factors agree versus when they conflict. When momentum, mean reversion, and trend all point the same direction, you have strong confirmation. When they disagree, tread carefully—the market may be transitional or choppy.',
    goodValues: [
      { label: 'All Bullish', description: 'All signals agree bullish (strongest setup)', color: '#4caf50' },
      { label: 'Mostly Bullish', description: '2 of 3 signals bullish (good setup)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Mostly Bearish', description: '2 of 3 signals bearish (avoid)', color: '#ff9800' },
      { label: 'All Bearish', description: 'All signals agree bearish (high risk)', color: '#f44336' },
    ],
    example: 'Combined signal: Strong Bullish (85% probability). Momentum is bullish (RSI rising), Mean Reversion is neutral (price near average), Trend is bullish (uptrend confirmed). Two out of three signals bullish with no contradictions creates high confidence.',
    additionalNotes: 'The combined signal uses adaptive weighting—in trending markets, trend and momentum get more weight. In range-bound markets, mean reversion gets more weight. This makes it more robust than any single signal. However, it\'s still a technical signal and doesn\'t account for fundamentals, news, or macroeconomic factors.',
  },
  trading_signal_probability: {
    title: 'Signal Probability',
    description: 'The statistical likelihood that the signal will be correct based on historical backtesting. If a signal has 75% probability, it means in the past when this exact pattern occurred, the predicted outcome happened 75% of the time.',
    interpretation: 'Probability tells you how reliable the signal is based on history. Higher probability means the pattern has worked more consistently in the past. However, probability is not certainty—a 75% signal still fails 25% of the time, so risk management is essential.',
    goodValues: [
      { label: '> 70%', description: 'High probability (reliable pattern)', color: '#4caf50' },
      { label: '60-70%', description: 'Good probability (solid pattern)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '50-60%', description: 'Moderate probability (weak edge)', color: '#ff9800' },
      { label: '< 50%', description: 'Low probability (unreliable)', color: '#f44336' },
    ],
    example: 'Bullish signal with 68% probability means historically, when this exact combination of indicators appeared, the stock moved higher 68% of the time over the signal horizon. That\'s a meaningful edge, but not a guarantee.',
    additionalNotes: 'Probability is based on historical data—market conditions change, and past patterns may not repeat. Higher probability signals deserve more confidence but should still be combined with stop-losses and position sizing. Probabilities below 55% offer minimal edge and should generally be avoided.',
  },
  trading_signal_confidence: {
    title: 'Signal Confidence',
    description: 'A qualitative assessment (High/Medium/Low) of how strongly the indicators support the signal. Confidence considers the strength of individual indicators, agreement between indicators, and statistical significance.',
    interpretation: 'Confidence tells you how "convinced" the model is about the signal. High confidence means indicators strongly agree and show clear patterns. Low confidence means signals are weak or contradictory. Only act on high-confidence signals in uncertain markets.',
    goodValues: [
      { label: 'High', description: 'Strong indicator agreement (reliable)', color: '#4caf50' },
      { label: 'Medium', description: 'Moderate indicator agreement', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Low', description: 'Weak or conflicting indicators', color: '#f44336' },
    ],
    example: 'High Confidence Bullish Signal: RSI strongly oversold (25), MACD showing strong bullish divergence, price firmly bouncing off lower Bollinger Band, and volume confirming. All factors align clearly—high confidence. Medium Confidence: Some indicators bullish, some neutral, no clear pattern—medium confidence.',
    additionalNotes: 'Confidence is subjective and complements probability. You can have a high-probability signal with low confidence if the current market conditions differ from historical patterns. Prefer high-confidence signals, especially in volatile markets. Low-confidence signals should be avoided or traded with reduced position size.',
  },
  trading_signal_bullish_score: {
    title: 'Bullish Score',
    description: 'A numerical score representing the cumulative strength of all bullish factors detected. Each bullish indicator (RSI oversold recovery, bullish MACD crossover, golden cross, etc.) contributes to the score. Higher scores indicate more bullish factors are present.',
    interpretation: 'The bullish score quantifies how many positive signals are present. A high bullish score means multiple indicators are flashing buy signals simultaneously—stronger confirmation. Think of it as counting votes—the more bullish votes, the more confident you can be in a bullish outlook.',
    goodValues: [
      { label: '> 7', description: 'Many bullish factors (strong setup)', color: '#4caf50' },
      { label: '5-7', description: 'Several bullish factors (good setup)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '2-4', description: 'Few bullish factors (weak setup)', color: '#ff9800' },
      { label: '< 2', description: 'Very few bullish factors (avoid)', color: '#f44336' },
    ],
    example: 'Bullish Score: 8. Factors: RSI oversold recovery (+2), MACD bullish crossover (+2), price above 50-day MA (+1), volume increasing on up days (+1), golden cross forming (+2). Eight bullish factors create strong confirmation.',
    additionalNotes: 'A high bullish score is most meaningful when the bearish score is low—if both are high, the market is giving mixed signals. Compare bullish vs bearish scores to understand the signal clarity. More factors doesn\'t always mean better—quality matters more than quantity.',
  },
  trading_signal_bearish_score: {
    title: 'Bearish Score',
    description: 'A numerical score representing the cumulative strength of all bearish factors detected. Each bearish indicator (RSI overbought, bearish MACD crossover, death cross, etc.) contributes to the score. Higher scores indicate more bearish factors are present.',
    interpretation: 'The bearish score quantifies how many negative signals are present. A high bearish score means multiple indicators are flashing sell signals—stronger downside confirmation. Compare this to the bullish score to see which side has more weight.',
    goodValues: [
      { label: '< 2', description: 'Few bearish factors (bullish favorable)', color: '#4caf50' },
      { label: '2-4', description: 'Some bearish factors (neutral)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '5-7', description: 'Several bearish factors (caution)', color: '#ff9800' },
      { label: '> 7', description: 'Many bearish factors (high risk)', color: '#f44336' },
    ],
    example: 'Bearish Score: 9. Factors: RSI extremely overbought (+2), bearish MACD crossover (+2), death cross forming (+2), declining volume on up days (+1), price hitting resistance (+2). Nine bearish factors suggest strong downside risk.',
    additionalNotes: 'Even a high bearish score doesn\'t guarantee a decline—it just means risk is elevated. Use bearish scores to manage risk, reduce position sizes, or set tighter stop-losses. When both bullish and bearish scores are high, the market is indecisive—wait for clarity.',
  },
  rsi_14: {
    title: 'RSI (Relative Strength Index)',
    description: 'RSI is a momentum oscillator that measures the speed and magnitude of recent price changes on a scale of 0-100. It identifies whether a stock is overbought (RSI > 70), oversold (RSI < 30), or neutral (30-70). RSI helps spot potential reversals when price reaches extreme levels.',
    interpretation: 'Think of RSI as a "momentum battery" that charges up and depletes. When RSI is below 30 (oversold), the selling pressure is exhausted and a bounce is likely. When RSI is above 70 (overbought), buying pressure is exhausted and a pullback may occur. RSI between 40-60 suggests balanced momentum.',
    goodValues: [
      { label: '< 30', description: 'Oversold (bullish reversal likely)', color: '#4caf50' },
      { label: '30-40', description: 'Slightly oversold (bullish bias)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '60-70', description: 'Slightly overbought (bearish bias)', color: '#ff9800' },
      { label: '> 70', description: 'Overbought (bearish reversal likely)', color: '#f44336' },
    ],
    example: 'RSI at 28: Stock has been selling off heavily and is extremely oversold. Historically, when this stock\'s RSI drops below 30, it bounces back within 5-10 trading days 73% of the time. This is a bullish mean reversion opportunity.',
    formula: 'RSI = 100 - (100 / (1 + RS)), where RS = Average Gain / Average Loss over 14 periods',
    additionalNotes: 'RSI can remain overbought/oversold for extended periods during strong trends—don\'t fight a trend just because RSI is extreme. RSI divergence (price makes new high but RSI doesn\'t) is a powerful reversal signal. Works best in range-bound markets.',
  },
  macd: {
    title: 'MACD (Moving Average Convergence Divergence)',
    description: 'MACD tracks the relationship between two moving averages (typically 12-day and 26-day EMAs) and includes a signal line (9-day EMA). When the MACD line crosses above the signal line, it generates a bullish signal. When it crosses below, it\'s bearish. The histogram shows the distance between MACD and signal lines.',
    interpretation: 'MACD identifies momentum shifts and trend changes. A bullish crossover (MACD crosses above signal line) suggests upward momentum is building. A bearish crossover suggests downward momentum. The histogram getting taller means momentum is strengthening; getting shorter means momentum is weakening.',
    goodValues: [
      { label: 'Bullish Cross', description: 'MACD crosses above signal (buy signal)', color: '#4caf50' },
      { label: 'Positive Histogram', description: 'Above signal line (bullish momentum)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Negative Histogram', description: 'Below signal line (bearish momentum)', color: '#ff9800' },
      { label: 'Bearish Cross', description: 'MACD crosses below signal (sell signal)', color: '#f44336' },
    ],
    example: 'MACD crossed above signal line with histogram turning positive. This indicates momentum is shifting from bearish to bullish. The last 8 times this crossover occurred, the stock rose an average of 7.5% over the next 3 months.',
    formula: 'MACD = 12-day EMA - 26-day EMA; Signal Line = 9-day EMA of MACD; Histogram = MACD - Signal',
    additionalNotes: 'MACD is a lagging indicator—crossovers happen after the move has started. Works best when combined with other indicators. False signals are common in choppy markets. MACD divergence (price makes new low but MACD doesn\'t) is a strong reversal indicator.',
  },
  momentum_20d: {
    title: 'Momentum (20-Day)',
    description: 'Momentum measures the rate of price change over a 20-day period by comparing today\'s price to the price 20 days ago. Positive momentum means the stock is higher than 20 days ago; negative means it\'s lower. The magnitude shows how strong the move is.',
    interpretation: 'Momentum shows if the stock is gaining or losing steam. Rising momentum means the trend is accelerating—like a car speeding up. Falling momentum means the trend is decelerating—like a car slowing down. Zero momentum means the price is unchanged from 20 days ago.',
    goodValues: [
      { label: '> +5%', description: 'Strong positive momentum (accelerating up)', color: '#4caf50' },
      { label: '+1% to +5%', description: 'Moderate upward momentum', color: '#8bc34a' },
    ],
    badValues: [
      { label: '-1% to -5%', description: 'Moderate downward momentum', color: '#ff9800' },
      { label: '< -5%', description: 'Strong negative momentum (accelerating down)', color: '#f44336' },
    ],
    example: '20-day momentum of +8.5% means the stock is 8.5% higher than it was 20 trading days ago. This strong positive momentum suggests the uptrend has good strength and is likely to continue in the near term.',
    formula: 'Momentum = ((Current Price / Price 20 days ago) - 1) × 100',
    additionalNotes: 'Momentum is a leading indicator but can be volatile. Very high positive momentum can signal an overbought condition. Very high negative momentum can signal oversold. Best used with trend-following strategies rather than mean reversion.',
  },
  bollinger_bands: {
    title: 'Bollinger Bands',
    description: 'Bollinger Bands consist of a middle band (20-day moving average) and upper/lower bands set at 2 standard deviations above and below the middle. When price touches the lower band, it\'s oversold; upper band, it\'s overbought. Bands widen during high volatility and narrow during low volatility.',
    interpretation: 'Bollinger Bands create a "price envelope" that contains normal price action. When price reaches the lower band, it\'s stretched too far below average and likely to bounce back (mean reversion). When price reaches the upper band, it may pull back. Band squeezes (narrow bands) often precede big moves.',
    goodValues: [
      { label: 'At Lower Band', description: 'Oversold, mean reversion likely (bullish)', color: '#4caf50' },
      { label: 'Below Middle', description: 'Below average, potential support', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Above Middle', description: 'Above average, potential resistance', color: '#ff9800' },
      { label: 'At Upper Band', description: 'Overbought, pullback likely (bearish)', color: '#f44336' },
    ],
    example: 'Price touched the lower Bollinger Band at $145 when the middle band is at $152. The stock is 2 standard deviations below its 20-day average—statistically, it should revert back toward $152. This creates a mean reversion buy opportunity.',
    formula: 'Upper Band = SMA(20) + 2σ; Middle Band = SMA(20); Lower Band = SMA(20) - 2σ',
    additionalNotes: 'In strong trends, price can "walk the band"—staying near the upper band in uptrends or lower band in downtrends. Bollinger Band squeezes (bands very narrow) signal low volatility and often precede breakouts. Works best in range-bound markets for mean reversion.',
  },
  rsi_meanreversion: {
    title: 'RSI Mean Reversion',
    description: 'A specialized RSI signal focused on mean reversion opportunities. When RSI reaches extreme levels (< 30 oversold or > 70 overbought) and starts reversing back toward the middle (50), it signals that price is likely to revert toward its average.',
    interpretation: 'This indicator specifically looks for RSI extremes followed by reversals. When RSI is below 30 and starts rising, it signals oversold conditions ending—a bullish mean reversion. When RSI is above 70 and starts falling, it signals overbought conditions ending—a bearish mean reversion.',
    goodValues: [
      { label: 'RSI < 30 Rising', description: 'Oversold reversal (strong buy)', color: '#4caf50' },
      { label: 'RSI 30-40 Rising', description: 'Recovery from oversold (bullish)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'RSI 60-70 Falling', description: 'Pullback from overbought (bearish)', color: '#ff9800' },
      { label: 'RSI > 70 Falling', description: 'Overbought reversal (strong sell)', color: '#f44336' },
    ],
    example: 'RSI dropped to 25 (extremely oversold) and is now at 33 and rising. The reversal from oversold suggests selling pressure is exhausted and buyers are stepping in. This mean reversion signal has 71% success rate historically.',
    additionalNotes: 'Mean reversion works best when fundamentals are stable—if a stock is crashing due to bad news, RSI oversold may not lead to a bounce. Best for stocks that trade in ranges rather than strong trends. Combine with support levels for higher probability.',
  },
  price_deviation_sma50: {
    title: 'Price Deviation from 50-Day SMA',
    description: 'Measures how far the current price is from its 50-day simple moving average, expressed as a percentage. Large deviations (> ±10%) suggest the price has stretched too far from its average and may revert back. The 50-day SMA represents the medium-term average price.',
    interpretation: 'When price is significantly below the 50-day SMA (e.g., -8%), it\'s oversold relative to the medium-term trend and likely to bounce back up. When price is significantly above the 50-day SMA (e.g., +8%), it\'s overbought and may pull back. Small deviations (< ±5%) suggest normal price action.',
    goodValues: [
      { label: '< -8%', description: 'Significantly oversold (strong mean reversion)', color: '#4caf50' },
      { label: '-5% to -8%', description: 'Moderately oversold (mean reversion likely)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '+5% to +8%', description: 'Moderately overbought (pullback likely)', color: '#ff9800' },
      { label: '> +8%', description: 'Significantly overbought (strong pullback risk)', color: '#f44336' },
    ],
    example: 'Price is $142, 50-day SMA is $155. Deviation: -8.4%. Price has fallen significantly below its 50-day average and is stretched. Historically, when this stock deviates more than 8% below its 50-day SMA, it recovers back toward the average 68% of the time within 15 trading days.',
    formula: 'Deviation = ((Current Price / 50-day SMA) - 1) × 100',
    additionalNotes: 'During strong trends, deviations can persist longer than expected—don\'t fight a trend. Best for mean reversion in range-bound markets. Combine with RSI or Bollinger Bands for confirmation. The 50-day SMA is often watched by institutions as a key support/resistance level.',
  },
  moving_average_cross: {
    title: 'Moving Average Crossover',
    description: 'A signal generated when a shorter-term moving average (like 50-day) crosses above or below a longer-term moving average (like 200-day). The "golden cross" (50-day crosses above 200-day) is bullish; the "death cross" (50-day crosses below 200-day) is bearish.',
    interpretation: 'Moving average crossovers identify major trend changes. A golden cross signals the short-term trend is becoming stronger than the long-term trend—bullish momentum building. A death cross signals the opposite—bearish momentum taking over. These are among the most-watched technical signals.',
    goodValues: [
      { label: 'Golden Cross', description: '50-day crosses above 200-day (major buy)', color: '#4caf50' },
      { label: '50 > 200', description: 'Short-term above long-term (bullish)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '50 < 200', description: 'Short-term below long-term (bearish)', color: '#ff9800' },
      { label: 'Death Cross', description: '50-day crosses below 200-day (major sell)', color: '#f44336' },
    ],
    example: 'Golden Cross formed: 50-day MA crossed above 200-day MA last week. Historically, golden crosses on this stock led to an average gain of 12% over the next 6 months in 78% of cases. This is a strong bullish signal for the medium term.',
    additionalNotes: 'Moving average crosses are lagging signals—they confirm trends after they\'ve started, not predict them. Can result in whipsaws in choppy markets. Best used on longer timeframes (daily charts, not intraday). Golden crosses work better in bull markets; death crosses work better in bear markets.',
  },
  volume_trend: {
    title: 'Volume Trend',
    description: 'Analyzes whether trading volume is increasing or decreasing relative to recent averages. Rising volume on up days confirms buying interest; rising volume on down days confirms selling pressure. Volume validates price moves—strong moves on high volume are more reliable than moves on low volume.',
    interpretation: 'Volume is like the "fuel" behind price moves. If price rises on high volume, it shows strong buying conviction. If price rises on low volume, it\'s weak and may reverse. Rising volume in the direction of the trend confirms the trend; declining volume suggests the trend is losing momentum.',
    goodValues: [
      { label: 'High Vol + Up', description: 'Strong buying conviction (bullish)', color: '#4caf50' },
      { label: 'Rising Vol + Up', description: 'Increasing buying interest (bullish)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Rising Vol + Down', description: 'Increasing selling pressure (bearish)', color: '#ff9800' },
      { label: 'High Vol + Down', description: 'Strong selling conviction (bearish)', color: '#f44336' },
    ],
    example: 'Stock rose 3% today on volume 2.5x the 20-day average. The high volume confirms strong buying interest and validates the upward move. When volume confirms direction, the move is more likely to continue.',
    additionalNotes: 'Always consider volume with price. Price up + volume down = weak rally (likely to fail). Price down + volume down = weak selloff (may bounce). Volume spikes can signal institutional buying/selling. Earnings announcements and news often cause volume spikes.',
  },
  ema_alignment: {
    title: 'EMA Alignment (Exponential Moving Averages)',
    description: 'Analyzes the alignment of multiple exponential moving averages (typically 12-day, 26-day, and 50-day EMAs) to determine trend strength and direction. EMAs give more weight to recent prices than simple moving averages. When shorter EMAs are above longer EMAs, it signals an uptrend; when shorter EMAs are below longer EMAs, it signals a downtrend.',
    interpretation: 'EMA alignment shows the "trend stack". In a strong uptrend, you want to see 12-day EMA > 26-day EMA > 50-day EMA—all stacked properly with price on top. This shows consistent bullish momentum across short, medium, and longer timeframes. When the alignment is reversed (50 > 26 > 12), it signals a downtrend.',
    goodValues: [
      { label: 'Bullish Alignment', description: '12 > 26 > 50 (strong uptrend)', color: '#4caf50' },
      { label: 'Improving', description: 'EMAs converging bullishly', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Mixed', description: 'EMAs crossing/tangled (no clear trend)', color: '#ff9800' },
      { label: 'Bearish Alignment', description: '12 < 26 < 50 (strong downtrend)', color: '#f44336' },
    ],
    example: 'EMA Alignment: 12-day = $223.24, 26-day = $224.43, 50-day = $226.75. This shows bearish alignment (12 < 26 < 50) indicating the stock is in a downtrend across multiple timeframes. For a bullish setup, we\'d want to see 12 > 26 > 50.',
    additionalNotes: 'EMA alignment is powerful because it shows trend consensus across multiple timeframes. When all EMAs align in one direction, the trend is strong and reliable. When EMAs are tangled or crossing, the market is indecisive—wait for clear alignment before trading. EMA crossovers (12 crossing 26) can signal trend changes early.',
  },
  momentum_signal: {
    title: 'Momentum Signal (Meta-Signal)',
    description: 'A composite meta-signal that synthesizes multiple momentum indicators (RSI, MACD, price momentum) into a single momentum assessment. This aggregated signal shows whether overall momentum is bullish, bearish, or neutral by weighing all momentum factors together.',
    interpretation: 'The momentum meta-signal tells you the "momentum verdict" across all momentum indicators combined. When it shows bullish (probability > 55%), it means the majority of momentum indicators (RSI, MACD, price changes) are pointing up. This is used as one input to the Combined Signal along with mean reversion and trend signals.',
    goodValues: [
      { label: 'Bullish (>60%)', description: 'Strong aggregate momentum (multiple indicators align)', color: '#4caf50' },
      { label: 'Bullish (55-60%)', description: 'Moderate momentum (most indicators align)', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Bearish (55-60%)', description: 'Moderate negative momentum', color: '#ff9800' },
      { label: 'Bearish (>60%)', description: 'Strong negative momentum', color: '#f44336' },
    ],
    example: 'Momentum Signal: bullish at 58% probability (medium confidence). This means when you look at RSI, MACD, and 20-day price momentum together, they collectively show bullish momentum with 58% historical success rate.',
    additionalNotes: 'The momentum meta-signal is not a standalone recommendation—it\'s one of three inputs to the Combined Signal. Even if momentum is bullish, mean reversion might be bearish, or trend might be neutral. The Combined Signal weighs all three to give you the final verdict.',
  },
  meanreversion_signal: {
    title: 'Mean Reversion Signal (Meta-Signal)',
    description: 'A composite meta-signal that synthesizes mean reversion indicators (Bollinger Bands, RSI extremes, price deviation from moving averages) into a single mean reversion assessment. Shows whether the stock is oversold/overbought and likely to revert to its average.',
    interpretation: 'The mean reversion meta-signal tells you if the stock has stretched too far from its average and is likely to "snap back". Bullish mean reversion means the stock is oversold and likely to bounce up. Bearish mean reversion means the stock is overbought and likely to pull back down.',
    goodValues: [
      { label: 'Bullish (>60%)', description: 'Oversold—strong bounce likely', color: '#4caf50' },
      { label: 'Bullish (55-60%)', description: 'Slightly oversold—bounce possible', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Bearish (55-60%)', description: 'Slightly overbought—pullback possible', color: '#ff9800' },
      { label: 'Bearish (>60%)', description: 'Overbought—pullback likely', color: '#f44336' },
    ],
    example: 'Mean Reversion Signal: bearish at 61% probability (medium confidence). Price is 86% up the Bollinger Band (overbought), suggesting a pullback toward the average is likely. Historically, this pattern led to pullbacks 61% of the time.',
    additionalNotes: 'Mean reversion conflicts with momentum in trending markets—momentum says "keep going", mean reversion says "pull back". The Combined Signal weighs both. Mean reversion works best in range-bound markets; it fails in strong trends.',
  },
  trend_signal: {
    title: 'Trend Signal (Meta-Signal)',
    description: 'A composite meta-signal that synthesizes trend indicators (moving average crossovers, EMA alignment, volume trends) into a single trend assessment. Shows whether the stock is in an uptrend, downtrend, or no clear trend.',
    interpretation: 'The trend meta-signal is your "big picture" trend verdict. Bullish trend means moving averages are stacked properly and the stock is trending up. Bearish trend means the opposite. Neutral means no clear trend—the market is choppy or sideways.',
    goodValues: [
      { label: 'Bullish (>60%)', description: 'Strong uptrend confirmed', color: '#4caf50' },
      { label: 'Bullish (55-60%)', description: 'Moderate uptrend', color: '#8bc34a' },
    ],
    badValues: [
      { label: 'Bearish (55-60%)', description: 'Moderate downtrend', color: '#ff9800' },
      { label: 'Bearish (>60%)', description: 'Strong downtrend', color: '#f44336' },
    ],
    example: 'Trend Signal: neutral at 53% probability (low confidence). Golden cross present (50-day MA above 200-day) but EMAs are bearishly aligned. Mixed signals result in neutral verdict—no clear trend direction.',
    additionalNotes: 'The trend signal is the most reliable in trending markets but can give false signals in choppy, sideways markets. When trend confidence is low (<55%), the market is transitional—wait for a clearer trend before taking large positions.',
  },
  bollinger_band_percent_b: {
    title: 'Bollinger Band %B',
    description: 'Bollinger %B tells you where price is within the Bollinger Bands. 0% means price is at the lower band, 50% means at the middle (20-day average), 100% means at the upper band. Values above 100% or below 0% mean price has broken outside the bands—extreme conditions.',
    interpretation: '%B is like a "price thermometer" for the Bollinger Bands. When %B is near 0% (at lower band), the stock is oversold and likely to bounce. When %B is near 100% (at upper band), the stock is overbought and likely to pull back. 50% means price is at the average—neutral.',
    goodValues: [
      { label: '< 20%', description: 'Near lower band (oversold, bullish reversal)', color: '#4caf50' },
      { label: '20-40%', description: 'Below middle (potential support)', color: '#8bc34a' },
    ],
    badValues: [
      { label: '60-80%', description: 'Above middle (potential resistance)', color: '#ff9800' },
      { label: '> 80%', description: 'Near upper band (overbought, bearish reversal)', color: '#f44336' },
    ],
    example: 'Bollinger %B: 85.81%. Price is 86% of the way from the lower band to the upper band—very close to the upper band. This is overbought territory, suggesting a mean reversion pullback is likely.',
    formula: '%B = (Price - Lower Band) / (Upper Band - Lower Band) × 100',
    additionalNotes: '%B above 100% means price broke above the upper band (very overbought). %B below 0% means price broke below the lower band (very oversold). During strong trends, %B can stay extreme for extended periods—don\'t fight the trend.',
  },
};

export function MetricHelpDialog({ open, onClose, metricKey }: MetricHelpDialogProps) {
  const helpData = METRIC_HELP[metricKey];

  if (!helpData) {
    return null;
  }

  return (
    <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
      <DialogTitle>
        <Box display="flex" alignItems="center" justifyContent="space-between">
          <Typography variant="h6" fontWeight="bold">
            {helpData.title}
          </Typography>
          <IconButton
            edge="end"
            color="inherit"
            onClick={onClose}
            aria-label="close"
            size="small"
          >
            <CloseIcon />
          </IconButton>
        </Box>
      </DialogTitle>
      <DialogContent dividers>
        {/* Description */}
        <Box mb={3}>
          <Typography variant="subtitle2" color="primary" fontWeight="bold" gutterBottom>
            What is this metric?
          </Typography>
          <Typography variant="body2" color="text.secondary">
            {helpData.description}
          </Typography>
        </Box>

        {/* Interpretation */}
        <Box mb={3}>
          <Typography variant="subtitle2" color="primary" fontWeight="bold" gutterBottom>
            How to interpret it
          </Typography>
          <Typography variant="body2" color="text.secondary">
            {helpData.interpretation}
          </Typography>
        </Box>

        <Divider sx={{ my: 2 }} />

        {/* Good Values */}
        <Box mb={2}>
          <Box display="flex" alignItems="center" gap={1} mb={1.5}>
            <TrendingUp sx={{ color: '#4caf50' }} fontSize="small" />
            <Typography variant="subtitle2" fontWeight="bold">
              Good / Low Risk Values
            </Typography>
          </Box>
          <Box display="flex" flexDirection="column" gap={1}>
            {helpData.goodValues.map((item, index) => (
              <Box key={index} display="flex" alignItems="center" gap={1.5}>
                <Chip
                  label={item.label}
                  size="small"
                  sx={{
                    backgroundColor: item.color,
                    color: 'white',
                    fontWeight: 'bold',
                    minWidth: 80,
                  }}
                />
                <Typography variant="body2" color="text.secondary">
                  {item.description}
                </Typography>
              </Box>
            ))}
          </Box>
        </Box>

        {/* Bad Values */}
        <Box mb={3}>
          <Box display="flex" alignItems="center" gap={1} mb={1.5}>
            <TrendingDown sx={{ color: '#f44336' }} fontSize="small" />
            <Typography variant="subtitle2" fontWeight="bold">
              Concerning / High Risk Values
            </Typography>
          </Box>
          <Box display="flex" flexDirection="column" gap={1}>
            {helpData.badValues.map((item, index) => (
              <Box key={index} display="flex" alignItems="center" gap={1.5}>
                <Chip
                  label={item.label}
                  size="small"
                  sx={{
                    backgroundColor: item.color,
                    color: 'white',
                    fontWeight: 'bold',
                    minWidth: 80,
                  }}
                />
                <Typography variant="body2" color="text.secondary">
                  {item.description}
                </Typography>
              </Box>
            ))}
          </Box>
        </Box>

        <Divider sx={{ my: 2 }} />

        {/* Example */}
        <Box mb={3}>
          <Typography variant="subtitle2" color="primary" fontWeight="bold" gutterBottom>
            Practical Example
          </Typography>
          <Alert severity="info" sx={{ mt: 1 }}>
            <Typography variant="body2">{helpData.example}</Typography>
          </Alert>
        </Box>

        {/* Formula (if available) */}
        {helpData.formula && (
          <Box mb={3}>
            <Typography variant="subtitle2" color="primary" fontWeight="bold" gutterBottom>
              Formula
            </Typography>
            <Box
              sx={{
                p: 1.5,
                bgcolor: 'grey.100',
                borderRadius: 1,
                fontFamily: 'monospace',
                fontSize: '0.875rem',
              }}
            >
              <code>{helpData.formula}</code>
            </Box>
          </Box>
        )}

        {/* Additional Notes */}
        {helpData.additionalNotes && (
          <Box>
            <Typography variant="subtitle2" color="primary" fontWeight="bold" gutterBottom>
              Important Notes
            </Typography>
            <Alert severity="warning">
              <Typography variant="body2">{helpData.additionalNotes}</Typography>
            </Alert>
          </Box>
        )}
      </DialogContent>
    </Dialog>
  );
}
