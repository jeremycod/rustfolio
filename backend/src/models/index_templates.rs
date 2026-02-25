use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub ticker_count: usize,
    pub tickers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexTemplateListItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub ticker_count: usize,
}

impl From<&IndexTemplate> for IndexTemplateListItem {
    fn from(template: &IndexTemplate) -> Self {
        Self {
            id: template.id.clone(),
            name: template.name.clone(),
            description: template.description.clone(),
            category: template.category.clone(),
            ticker_count: template.ticker_count,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWatchlistFromTemplateRequest {
    pub template_id: String,
    pub custom_name: Option<String>,
    pub selected_tickers: Option<Vec<String>>, // None = all tickers, Some = specific selection
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWatchlistFromTemplateResponse {
    pub watchlist_id: String,
    pub name: String,
    pub added_count: usize,
    pub failed_count: usize,
    pub failed_tickers: Vec<String>,
}

/// Get all available index templates
pub fn get_all_templates() -> Vec<IndexTemplate> {
    vec![
        // US Market Indices
        IndexTemplate {
            id: "sp500".to_string(),
            name: "S&P 500".to_string(),
            description: "Top 500 large-cap US companies".to_string(),
            category: "US Indices".to_string(),
            ticker_count: get_sp500_tickers().len(),
            tickers: get_sp500_tickers(),
        },
        IndexTemplate {
            id: "nasdaq100".to_string(),
            name: "NASDAQ 100".to_string(),
            description: "Top 100 non-financial companies on NASDAQ".to_string(),
            category: "US Indices".to_string(),
            ticker_count: get_nasdaq100_tickers().len(),
            tickers: get_nasdaq100_tickers(),
        },
        IndexTemplate {
            id: "dowjones".to_string(),
            name: "Dow Jones Industrial Average".to_string(),
            description: "30 prominent US blue-chip companies".to_string(),
            category: "US Indices".to_string(),
            ticker_count: 30,
            tickers: get_dowjones_tickers(),
        },
        // Sector ETFs
        IndexTemplate {
            id: "tech_leaders".to_string(),
            name: "Tech Leaders".to_string(),
            description: "Major technology companies (FAANG+)".to_string(),
            category: "Sectors".to_string(),
            ticker_count: 15,
            tickers: get_tech_leaders_tickers(),
        },
        IndexTemplate {
            id: "finance_leaders".to_string(),
            name: "Financial Leaders".to_string(),
            description: "Major banks and financial institutions".to_string(),
            category: "Sectors".to_string(),
            ticker_count: 15,
            tickers: get_finance_leaders_tickers(),
        },
        IndexTemplate {
            id: "energy_leaders".to_string(),
            name: "Energy Leaders".to_string(),
            description: "Major oil, gas, and renewable energy companies".to_string(),
            category: "Sectors".to_string(),
            ticker_count: 10,
            tickers: get_energy_leaders_tickers(),
        },
        // International
        IndexTemplate {
            id: "tsx60".to_string(),
            name: "S&P/TSX 60".to_string(),
            description: "60 largest companies on Toronto Stock Exchange".to_string(),
            category: "International".to_string(),
            ticker_count: get_tsx60_tickers().len(),
            tickers: get_tsx60_tickers(),
        },
        // Popular Lists
        IndexTemplate {
            id: "magnificent7".to_string(),
            name: "Magnificent 7".to_string(),
            description: "The seven largest tech companies (2024)".to_string(),
            category: "Popular".to_string(),
            ticker_count: 7,
            tickers: vec![
                "AAPL".to_string(), "MSFT".to_string(), "GOOGL".to_string(),
                "AMZN".to_string(), "NVDA".to_string(), "META".to_string(),
                "TSLA".to_string(),
            ],
        },
        IndexTemplate {
            id: "dividend_aristocrats".to_string(),
            name: "Dividend Aristocrats (Sample)".to_string(),
            description: "Companies with 25+ years of dividend increases".to_string(),
            category: "Popular".to_string(),
            ticker_count: 15,
            tickers: get_dividend_aristocrats_tickers(),
        },
    ]
}

/// Get template by ID
pub fn get_template_by_id(id: &str) -> Option<IndexTemplate> {
    get_all_templates().into_iter().find(|t| t.id == id)
}

// Sample ticker lists - in production, these would be comprehensive or loaded from a database

fn get_sp500_tickers() -> Vec<String> {
    // Top 100 S&P 500 by market cap (comprehensive sample)
    vec![
        // Mega Cap (Top 10)
        "AAPL", "MSFT", "GOOGL", "AMZN", "NVDA", "META", "TSLA", "BRK.B", "UNH", "XOM",
        // Large Cap Tech
        "AVGO", "ORCL", "ADBE", "CRM", "CSCO", "ACN", "NFLX", "AMD", "INTC", "QCOM",
        "TXN", "AMAT", "ADI", "INTU", "NOW", "PANW", "SNPS", "CDNS", "LRCX", "KLAC",
        // Large Cap Healthcare (UNH already in Mega Cap, removed duplicate)
        "JNJ", "LLY", "ABBV", "MRK", "PFE", "TMO", "ABT", "DHR", "BMY", "SYK",
        "AMGN", "GILD", "CVS", "CI", "ELV", "ISRG", "VRTX", "REGN", "ZTS", "BSX",
        // Large Cap Finance
        "JPM", "BAC", "WFC", "GS", "MS", "C", "BLK", "SCHW", "AXP", "USB",
        "PNC", "TFC", "COF", "BK", "AIG", "MET", "PRU", "ALL", "TRV", "AFL",
        // Large Cap Consumer (AMZN, TSLA already in Mega Cap, removed duplicates)
        "HD", "WMT", "PG", "COST", "MCD", "NKE", "DIS", "CMCSA", "BKNG", "TJX",
        "PEP", "KO", "PM", "TGT", "SBUX", "LOW", "MDLZ", "CL", "GIS", "KHC",
        // Large Cap Industrial & Energy (XOM already in Mega Cap, removed duplicate)
        "CVX", "COP", "SLB", "BA", "CAT", "RTX", "HON", "UPS", "LMT", "PSX",
        "DE", "MMM", "GE", "EMR", "ITW", "ETN", "PH", "CMI", "FDX", "NSC",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn get_nasdaq100_tickers() -> Vec<String> {
    // Top 80 NASDAQ 100 stocks (comprehensive)
    vec![
        // Tech Giants
        "AAPL", "MSFT", "AMZN", "NVDA", "META", "GOOGL", "GOOG", "TSLA",
        // Tech Large Cap
        "AVGO", "NFLX", "ADBE", "CSCO", "CRM", "PEP", "AMD", "QCOM",
        "INTC", "TXN", "INTU", "AMAT", "ADI", "LRCX", "KLAC", "SNPS",
        "CDNS", "MRVL", "PANW", "WDAY", "FTNT", "DDOG", "TEAM", "ZS",
        // Consumer & Retail
        "COST", "SBUX", "BKNG", "MAR", "ABNB", "LULU", "ROST", "DLTR",
        "ORLY", "AZO", "POOL", "ULTA", "CHTR", "CMCSA", "NFLX", "DIS",
        // Healthcare & Biotech
        "AMGN", "GILD", "REGN", "VRTX", "BIIB", "ILMN", "MRNA", "SGEN",
        "ALGN", "IDXX", "DXCM", "EXAS", "ISRG", "HOLX", "TECH", "PODD",
        // Other
        "HON", "ADP", "PAYX", "CTAS", "VRSK", "WBD", "EA", "ATVI",
        "PYPL", "MELI", "ADSK", "ANSS", "CPRT", "FAST", "ODFL", "PCAR",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn get_dowjones_tickers() -> Vec<String> {
    // All 30 Dow Jones stocks
    vec![
        "AAPL", "AMGN", "AXP", "BA", "CAT", "CRM", "CSCO", "CVX", "DIS",
        "DOW", "GS", "HD", "HON", "IBM", "INTC", "JNJ", "JPM", "KO",
        "MCD", "MMM", "MRK", "MSFT", "NKE", "PG", "TRV", "UNH", "V",
        "VZ", "WMT", "WBA",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn get_tech_leaders_tickers() -> Vec<String> {
    vec![
        "AAPL", "MSFT", "GOOGL", "AMZN", "META", "NVDA", "TSLA", "NFLX",
        "ADBE", "CRM", "INTC", "AMD", "QCOM", "AVGO", "ORCL",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn get_finance_leaders_tickers() -> Vec<String> {
    vec![
        "JPM", "BAC", "WFC", "C", "GS", "MS", "BLK", "SCHW", "AXP", "USB",
        "PNC", "TFC", "BK", "COF", "AIG",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn get_energy_leaders_tickers() -> Vec<String> {
    vec![
        "XOM", "CVX", "COP", "SLB", "EOG", "MPC", "PSX", "VLO", "OXY", "HES",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn get_tsx60_tickers() -> Vec<String> {
    // Top 40 TSX 60 stocks
    vec![
        // Big Banks
        "RY", "TD", "BMO", "BNS", "CM",
        // Tech & Telecom
        "SHOP", "BCE", "T", "QBR.B", "TELUS",
        // Energy
        "ENB", "CNQ", "SU", "TRP", "PPL", "IMO", "CVE", "WCP", "ARX",
        // Industrials & Transportation
        "CNR", "CP", "WCN", "TOU", "CSU", "GIB.A",
        // Materials & Mining
        "ABX", "NTR", "FM", "CCO", "K",
        // Finance & Insurance
        "MFC", "SLF", "POW", "GWO", "IFC",
        // Real Estate
        "BN",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

fn get_dividend_aristocrats_tickers() -> Vec<String> {
    // Sample dividend aristocrats
    vec![
        "JNJ", "PG", "KO", "PEP", "MCD", "WMT", "TGT", "LOW", "CAT", "MMM",
        "GD", "EMR", "ITW", "SHW", "ECL",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}
