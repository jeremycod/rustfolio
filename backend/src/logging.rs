use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Debug, Clone)]
pub struct LoggingConfig {
    pub loki_enabled: bool,
    pub loki_url: Option<String>,
    pub service_name: String,
    pub environment: String,
    pub log_level: String,
}

impl LoggingConfig {
    pub fn from_env() -> Self {
        Self {
            loki_enabled: std::env::var("LOKI_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .unwrap_or(false),
            loki_url: std::env::var("LOKI_URL").ok(),
            service_name: std::env::var("SERVICE_NAME")
                .unwrap_or_else(|_| "rustfolio".to_string()),
            environment: std::env::var("ENVIRONMENT")
                .unwrap_or_else(|_| "development".to_string()),
            log_level: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.loki_enabled && self.loki_url.is_none() {
            return Err("LOKI_ENABLED is true but LOKI_URL is not set".to_string());
        }
        Ok(())
    }
}

pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    config.validate()?;

    #[cfg(feature = "loki")]
    {
        if config.loki_enabled {
            if let Some(loki_url) = config.loki_url.clone() {
                tracing::info!("📊 Initializing logging with Loki at {}", loki_url);
                return init_with_loki(config, &loki_url);
            }
        }
    }

    // Fallback to console-only logging
    tracing::info!("📊 Initializing console-only logging");
    init_console_only(config)
}

fn init_console_only(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.log_level))
        .with(tracing_subscriber::fmt::layer())
        .init();

    Ok(())
}

#[cfg(feature = "loki")]
fn init_with_loki(config: LoggingConfig, loki_url: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::collections::HashMap;

    let url = url::Url::parse(loki_url)?;

    let mut labels = HashMap::new();
    labels.insert("service".to_string(), config.service_name.clone());
    labels.insert("environment".to_string(), config.environment.clone());

    let (loki_layer, task) = tracing_loki::builder()
        .label("service", &config.service_name)?
        .label("environment", &config.environment)?
        .build_url(url)?;

    // Spawn the background task that sends logs to Loki
    tokio::spawn(task);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(&config.log_level))
        .with(tracing_subscriber::fmt::layer())
        .with(loki_layer)
        .init();

    tracing::info!("✅ Loki logging initialized successfully");

    Ok(())
}
