import { useState, useEffect } from 'react';
import {
    Dialog,
    DialogTitle,
    DialogContent,
    DialogActions,
    Button,
    TextField,
    Select,
    MenuItem,
    FormControl,
    InputLabel,
    Box,
    Typography,
    RadioGroup,
    FormControlLabel,
    Radio,
    Checkbox,
    FormGroup,
    FormLabel
} from '@mui/material';
import { useQuery } from '@tanstack/react-query';
import { listPortfolios } from '../lib/endpoints';
import type {
    AlertRule,
    CreateAlertRuleRequest,
    UpdateAlertRuleRequest,
    Comparison,
    NotificationChannel,
    AlertRuleType
} from '../types';

interface AlertRuleFormDialogProps {
    open: boolean;
    onClose: () => void;
    onSubmit: (data: CreateAlertRuleRequest | UpdateAlertRuleRequest) => void;
    editRule?: AlertRule | null;
}

export default function AlertRuleFormDialog({
    open,
    onClose,
    onSubmit,
    editRule
}: AlertRuleFormDialogProps) {
    const portfoliosQ = useQuery({
        queryKey: ['portfolios'],
        queryFn: listPortfolios,
        enabled: open
    });

    // Form state
    const [name, setName] = useState('');
    const [description, setDescription] = useState('');
    const [scope, setScope] = useState<'portfolio' | 'ticker'>('portfolio');
    const [portfolioId, setPortfolioId] = useState('');
    const [ticker, setTicker] = useState('');
    const [ruleType, setRuleType] = useState<AlertRuleType>('price_change');
    const [threshold, setThreshold] = useState<number>(5);
    const [comparison, setComparison] = useState<Comparison>('gt');
    const [channels, setChannels] = useState<NotificationChannel[]>(['email', 'in_app']);
    const [cooldownHours, setCooldownHours] = useState<number>(24);

    // Rule-specific configs
    const [priceChangePercentage, setPriceChangePercentage] = useState<number>(5);
    const [priceChangeDirection, setPriceChangeDirection] = useState<string>('either');
    const [priceChangeTimeframe, setPriceChangeTimeframe] = useState<string>('daily');

    const [volatilityThreshold, setVolatilityThreshold] = useState<number>(50);

    const [drawdownPercentage, setDrawdownPercentage] = useState<number>(10);

    const [riskMetric, setRiskMetric] = useState<string>('risk_score');
    const [riskThresholdValue, setRiskThresholdValue] = useState<number>(75);

    const [sentimentThreshold, setSentimentThreshold] = useState<number>(0);
    const [sentimentTrend, setSentimentTrend] = useState<string>('');

    const [divergenceType, setDivergenceType] = useState<string>('rsi');

    // Load edit rule data
    useEffect(() => {
        if (editRule) {
            setName(editRule.name);
            setDescription(editRule.description || '');
            setScope(editRule.portfolio_id ? 'portfolio' : 'ticker');
            setPortfolioId(editRule.portfolio_id || '');
            setTicker(editRule.ticker || '');
            setRuleType(editRule.rule_type as AlertRuleType);
            setThreshold(editRule.threshold);
            setComparison(editRule.comparison as Comparison);
            setChannels(editRule.notification_channels as NotificationChannel[]);
            setCooldownHours(editRule.cooldown_hours);
        } else {
            // Reset form
            resetForm();
        }
    }, [editRule, open]);

    const resetForm = () => {
        setName('');
        setDescription('');
        setScope('portfolio');
        setPortfolioId('');
        setTicker('');
        setRuleType('price_change');
        setThreshold(5);
        setComparison('gt');
        setChannels(['email', 'in_app']);
        setCooldownHours(24);
        setPriceChangePercentage(5);
        setPriceChangeDirection('either');
        setPriceChangeTimeframe('daily');
        setVolatilityThreshold(50);
        setDrawdownPercentage(10);
        setRiskMetric('risk_score');
        setRiskThresholdValue(75);
        setSentimentThreshold(0);
        setSentimentTrend('');
        setDivergenceType('rsi');
    };

    const handleChannelToggle = (channel: NotificationChannel) => {
        if (channels.includes(channel)) {
            setChannels(channels.filter(c => c !== channel));
        } else {
            setChannels([...channels, channel]);
        }
    };

    const handleSubmit = () => {
        // Build the rule_type tagged union based on rule type
        let ruleTypeData: any;

        switch (ruleType) {
            case 'price_change':
                ruleTypeData = {
                    type: 'price_change',
                    config: {
                        percentage: priceChangePercentage,
                        direction: priceChangeDirection,
                        timeframe: priceChangeTimeframe
                    }
                };
                break;
            case 'volatility_spike':
                ruleTypeData = {
                    type: 'volatility_spike',
                    config: {
                        threshold: volatilityThreshold
                    }
                };
                break;
            case 'drawdown_exceeded':
                ruleTypeData = {
                    type: 'drawdown_exceeded',
                    config: {
                        percentage: drawdownPercentage
                    }
                };
                break;
            case 'risk_threshold':
                ruleTypeData = {
                    type: 'risk_threshold',
                    config: {
                        metric: riskMetric,
                        threshold: riskThresholdValue
                    }
                };
                break;
            case 'sentiment_change':
                ruleTypeData = {
                    type: 'sentiment_change',
                    config: {
                        sentiment_threshold: sentimentThreshold,
                        trend: sentimentTrend || undefined
                    }
                };
                break;
            case 'divergence':
                ruleTypeData = {
                    type: 'divergence',
                    config: {
                        divergence_type: divergenceType
                    }
                };
                break;
        }

        // Build the request based on rule type
        const baseData: any = {
            name,
            description: description || undefined,
            threshold,
            comparison,
            notification_channels: channels,
            cooldown_hours: cooldownHours,
            rule_type: ruleTypeData
        };

        // Add scope
        if (scope === 'portfolio') {
            baseData.portfolio_id = portfolioId;
        } else {
            baseData.ticker = ticker;
        }

        // For create, we need the full data; for update, we can omit unchanged fields
        if (editRule) {
            // Update - only send changed fields
            const updateData: UpdateAlertRuleRequest = {
                name: baseData.name,
                description: baseData.description,
                threshold: baseData.threshold,
                comparison: baseData.comparison,
                notification_channels: baseData.notification_channels,
                cooldown_hours: baseData.cooldown_hours,
                enabled: editRule.enabled
            };
            onSubmit(updateData);
        } else {
            // Create
            onSubmit(baseData as CreateAlertRuleRequest);
        }
    };

    const isFormValid = () => {
        if (!name || name.length < 3) return false;
        if (scope === 'portfolio' && !portfolioId) return false;
        if (scope === 'ticker' && !ticker) return false;
        if (channels.length === 0) return false;
        if (cooldownHours < 1 || cooldownHours > 168) return false;
        return true;
    };

    const renderRuleTypeConfig = () => {
        switch (ruleType) {
            case 'price_change':
                return (
                    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                        <TextField
                            label="Percentage Change"
                            type="number"
                            value={priceChangePercentage}
                            onChange={(e) => setPriceChangePercentage(Number(e.target.value))}
                            fullWidth
                            helperText="Percentage change threshold (0-100)"
                            inputProps={{ min: 0, max: 100 }}
                        />
                        <FormControl fullWidth>
                            <InputLabel>Direction</InputLabel>
                            <Select
                                value={priceChangeDirection}
                                label="Direction"
                                onChange={(e) => setPriceChangeDirection(e.target.value)}
                            >
                                <MenuItem value="up">Up</MenuItem>
                                <MenuItem value="down">Down</MenuItem>
                                <MenuItem value="either">Either</MenuItem>
                            </Select>
                        </FormControl>
                        <FormControl fullWidth>
                            <InputLabel>Timeframe</InputLabel>
                            <Select
                                value={priceChangeTimeframe}
                                label="Timeframe"
                                onChange={(e) => setPriceChangeTimeframe(e.target.value)}
                            >
                                <MenuItem value="intraday">Intraday</MenuItem>
                                <MenuItem value="daily">Daily</MenuItem>
                                <MenuItem value="weekly">Weekly</MenuItem>
                                <MenuItem value="monthly">Monthly</MenuItem>
                            </Select>
                        </FormControl>
                    </Box>
                );

            case 'volatility_spike':
                return (
                    <TextField
                        label="Volatility Threshold"
                        type="number"
                        value={volatilityThreshold}
                        onChange={(e) => setVolatilityThreshold(Number(e.target.value))}
                        fullWidth
                        helperText="Volatility threshold percentage"
                        inputProps={{ min: 0 }}
                    />
                );

            case 'drawdown_exceeded':
                return (
                    <TextField
                        label="Drawdown Percentage"
                        type="number"
                        value={drawdownPercentage}
                        onChange={(e) => setDrawdownPercentage(Number(e.target.value))}
                        fullWidth
                        helperText="Maximum drawdown percentage (0-100)"
                        inputProps={{ min: 0, max: 100 }}
                    />
                );

            case 'risk_threshold':
                return (
                    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                        <FormControl fullWidth>
                            <InputLabel>Risk Metric</InputLabel>
                            <Select
                                value={riskMetric}
                                label="Risk Metric"
                                onChange={(e) => setRiskMetric(e.target.value)}
                            >
                                <MenuItem value="risk_score">Risk Score</MenuItem>
                                <MenuItem value="volatility">Volatility</MenuItem>
                                <MenuItem value="sharpe_ratio">Sharpe Ratio</MenuItem>
                                <MenuItem value="max_drawdown">Max Drawdown</MenuItem>
                                <MenuItem value="var_95">VaR 95%</MenuItem>
                                <MenuItem value="cvar_95">CVaR 95%</MenuItem>
                            </Select>
                        </FormControl>
                        <TextField
                            label="Threshold Value"
                            type="number"
                            value={riskThresholdValue}
                            onChange={(e) => setRiskThresholdValue(Number(e.target.value))}
                            fullWidth
                            helperText="Threshold for the selected metric"
                        />
                    </Box>
                );

            case 'sentiment_change':
                return (
                    <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                        <TextField
                            label="Sentiment Threshold"
                            type="number"
                            value={sentimentThreshold}
                            onChange={(e) => setSentimentThreshold(Number(e.target.value))}
                            fullWidth
                            helperText="Sentiment threshold (-1.0 to 1.0)"
                            inputProps={{ min: -1, max: 1, step: 0.1 }}
                        />
                        <FormControl fullWidth>
                            <InputLabel>Trend (Optional)</InputLabel>
                            <Select
                                value={sentimentTrend}
                                label="Trend (Optional)"
                                onChange={(e) => setSentimentTrend(e.target.value)}
                            >
                                <MenuItem value="">Any</MenuItem>
                                <MenuItem value="improving">Improving</MenuItem>
                                <MenuItem value="deteriorating">Deteriorating</MenuItem>
                            </Select>
                        </FormControl>
                    </Box>
                );

            case 'divergence':
                return (
                    <FormControl fullWidth>
                        <InputLabel>Divergence Type</InputLabel>
                        <Select
                            value={divergenceType}
                            label="Divergence Type"
                            onChange={(e) => setDivergenceType(e.target.value)}
                        >
                            <MenuItem value="rsi">RSI</MenuItem>
                            <MenuItem value="macd">MACD</MenuItem>
                            <MenuItem value="volume">Volume</MenuItem>
                        </Select>
                    </FormControl>
                );

            default:
                return null;
        }
    };

    return (
        <Dialog open={open} onClose={onClose} maxWidth="md" fullWidth>
            <DialogTitle>{editRule ? 'Edit Alert Rule' : 'Create Alert Rule'}</DialogTitle>
            <DialogContent>
                <Box sx={{ display: 'flex', flexDirection: 'column', gap: 3, pt: 2 }}>
                    {/* Basic Info */}
                    <Box>
                        <Typography variant="subtitle2" gutterBottom>
                            Basic Information
                        </Typography>
                        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
                            <TextField
                                label="Rule Name"
                                value={name}
                                onChange={(e) => setName(e.target.value)}
                                fullWidth
                                required
                                helperText="A descriptive name for this alert rule"
                            />
                            <TextField
                                label="Description"
                                value={description}
                                onChange={(e) => setDescription(e.target.value)}
                                fullWidth
                                multiline
                                rows={2}
                                helperText="Optional description"
                            />
                        </Box>
                    </Box>

                    {/* Scope Selector */}
                    <Box>
                        <Typography variant="subtitle2" gutterBottom>
                            Scope
                        </Typography>
                        <RadioGroup
                            value={scope}
                            onChange={(e) => setScope(e.target.value as 'portfolio' | 'ticker')}
                            row
                        >
                            <FormControlLabel value="portfolio" control={<Radio />} label="Portfolio" />
                            <FormControlLabel value="ticker" control={<Radio />} label="Ticker" />
                        </RadioGroup>
                        {scope === 'portfolio' ? (
                            <FormControl fullWidth sx={{ mt: 1 }}>
                                <InputLabel>Portfolio</InputLabel>
                                <Select
                                    value={portfolioId}
                                    label="Portfolio"
                                    onChange={(e) => setPortfolioId(e.target.value)}
                                >
                                    {portfoliosQ.data?.map((p) => (
                                        <MenuItem key={p.id} value={p.id}>
                                            {p.name}
                                        </MenuItem>
                                    ))}
                                </Select>
                            </FormControl>
                        ) : (
                            <TextField
                                label="Ticker Symbol"
                                value={ticker}
                                onChange={(e) => setTicker(e.target.value.toUpperCase())}
                                fullWidth
                                sx={{ mt: 1 }}
                                helperText="e.g., AAPL, MSFT"
                            />
                        )}
                    </Box>

                    {/* Rule Type */}
                    <Box>
                        <Typography variant="subtitle2" gutterBottom>
                            Alert Type
                        </Typography>
                        <FormControl fullWidth>
                            <InputLabel>Rule Type</InputLabel>
                            <Select
                                value={ruleType}
                                label="Rule Type"
                                onChange={(e) => setRuleType(e.target.value as AlertRuleType)}
                            >
                                <MenuItem value="price_change">Price Change</MenuItem>
                                <MenuItem value="volatility_spike">Volatility Spike</MenuItem>
                                <MenuItem value="drawdown_exceeded">Drawdown Exceeded</MenuItem>
                                <MenuItem value="risk_threshold">Risk Threshold</MenuItem>
                                <MenuItem value="sentiment_change">Sentiment Change</MenuItem>
                                <MenuItem value="divergence">Divergence</MenuItem>
                            </Select>
                        </FormControl>
                    </Box>

                    {/* Rule Configuration */}
                    <Box>
                        <Typography variant="subtitle2" gutterBottom>
                            Rule Configuration
                        </Typography>
                        {renderRuleTypeConfig()}
                    </Box>

                    {/* Threshold and Comparison */}
                    <Box>
                        <Typography variant="subtitle2" gutterBottom>
                            Threshold Settings
                        </Typography>
                        <Box sx={{ display: 'flex', gap: 2 }}>
                            <FormControl sx={{ minWidth: 150 }}>
                                <InputLabel>Comparison</InputLabel>
                                <Select
                                    value={comparison}
                                    label="Comparison"
                                    onChange={(e) => setComparison(e.target.value as Comparison)}
                                >
                                    <MenuItem value="gt">Greater Than</MenuItem>
                                    <MenuItem value="gte">Greater or Equal</MenuItem>
                                    <MenuItem value="lt">Less Than</MenuItem>
                                    <MenuItem value="lte">Less or Equal</MenuItem>
                                    <MenuItem value="eq">Equal</MenuItem>
                                </Select>
                            </FormControl>
                            <TextField
                                label="Threshold"
                                type="number"
                                value={threshold}
                                onChange={(e) => setThreshold(Number(e.target.value))}
                                fullWidth
                                helperText="Value to trigger the alert"
                            />
                        </Box>
                    </Box>

                    {/* Notification Settings */}
                    <Box>
                        <Typography variant="subtitle2" gutterBottom>
                            Notification Settings
                        </Typography>
                        <FormGroup>
                            <FormControlLabel
                                control={
                                    <Checkbox
                                        checked={channels.includes('email')}
                                        onChange={() => handleChannelToggle('email')}
                                    />
                                }
                                label="Email"
                            />
                            <FormControlLabel
                                control={
                                    <Checkbox
                                        checked={channels.includes('in_app')}
                                        onChange={() => handleChannelToggle('in_app')}
                                    />
                                }
                                label="In-App"
                            />
                            <FormControlLabel
                                control={
                                    <Checkbox
                                        checked={channels.includes('webhook')}
                                        onChange={() => handleChannelToggle('webhook')}
                                    />
                                }
                                label="Webhook"
                            />
                        </FormGroup>
                        <TextField
                            label="Cooldown Hours"
                            type="number"
                            value={cooldownHours}
                            onChange={(e) => setCooldownHours(Number(e.target.value))}
                            fullWidth
                            sx={{ mt: 2 }}
                            helperText="Hours to wait before triggering again (1-168)"
                            inputProps={{ min: 1, max: 168 }}
                        />
                    </Box>
                </Box>
            </DialogContent>
            <DialogActions>
                <Button onClick={onClose}>Cancel</Button>
                <Button
                    onClick={handleSubmit}
                    variant="contained"
                    disabled={!isFormValid()}
                >
                    {editRule ? 'Update' : 'Create'}
                </Button>
            </DialogActions>
        </Dialog>
    );
}
