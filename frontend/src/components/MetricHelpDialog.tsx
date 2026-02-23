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
