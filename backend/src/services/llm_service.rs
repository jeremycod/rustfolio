use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error};
use uuid::Uuid;

use crate::errors::LlmError;

/// Configuration for LLM service
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub enabled: bool,
    pub provider: String,
    pub api_key: Option<String>,
    pub max_tokens: usize,
    pub temperature: f32,
}

impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: "openai".to_string(),
            api_key: None,
            max_tokens: 500,
            temperature: 0.7,
        }
    }
}

/// Trait for LLM providers
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Generate a completion from a prompt
    async fn generate_completion(&self, prompt: String) -> Result<String, LlmError>;

    /// Generate a summary of text with a maximum length
    #[allow(dead_code)]
    async fn generate_summary(&self, text: String, max_length: usize) -> Result<String, LlmError>;

    /// Get text embedding (vector representation)
    #[allow(dead_code)]
    async fn get_embedding(&self, text: String) -> Result<Vec<f32>, LlmError>;
}

/// OpenAI API request/response structures
#[derive(Debug, Serialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    max_tokens: usize,
    temperature: f32,
}

#[derive(Debug, Serialize, Clone)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<OpenAiChoice>,
    usage: Option<OpenAiUsage>,
}

#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    message: OpenAiResponseMessage,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponseMessage {
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct OpenAiEmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAiEmbeddingResponse {
    data: Vec<OpenAiEmbeddingData>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct OpenAiEmbeddingData {
    embedding: Vec<f32>,
}

/// OpenAI provider implementation
pub struct OpenAiProvider {
    api_key: String,
    model: String,
    max_tokens: usize,
    temperature: f32,
    client: Client,
}

impl OpenAiProvider {
    pub fn new(api_key: String, max_tokens: usize, temperature: f32) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            api_key,
            model: "gpt-4o-mini".to_string(),
            max_tokens,
            temperature,
            client,
        }
    }

    async fn call_openai_with_retry(&self, request: OpenAiRequest) -> Result<OpenAiResponse, LlmError> {
        let mut retry_count = 0;
        let max_retries = 3;
        let mut delay = Duration::from_secs(1);

        loop {
            match self.call_openai(&request).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    retry_count += 1;
                    if retry_count >= max_retries {
                        error!("OpenAI API call failed after {} retries: {}", max_retries, e);
                        return Err(e);
                    }

                    warn!("OpenAI API call failed (attempt {}/{}): {}. Retrying in {:?}...",
                          retry_count, max_retries, e, delay);
                    tokio::time::sleep(delay).await;
                    delay *= 2; // Exponential backoff: 1s, 2s, 4s
                }
            }
        }
    }

    async fn call_openai(&self, request: &OpenAiRequest) -> Result<OpenAiResponse, LlmError> {
        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::Timeout
                } else {
                    LlmError::NetworkError(e.to_string())
                }
            })?;

        let status = response.status();

        if status == 429 {
            return Err(LlmError::RateLimited);
        }

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!("HTTP {}: {}", status, error_text)));
        }

        response.json::<OpenAiResponse>()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))
    }
}

#[async_trait]
impl LlmProvider for OpenAiProvider {
    async fn generate_completion(&self, prompt: String) -> Result<String, LlmError> {
        info!("Generating LLM completion (model: {}, max_tokens: {})", self.model, self.max_tokens);

        let request = OpenAiRequest {
            model: self.model.clone(),
            messages: vec![
                OpenAiMessage {
                    role: "system".to_string(),
                    content: "You are a helpful portfolio analysis assistant. Provide educational insights about portfolio performance and risk. Do NOT give buy/sell recommendations.".to_string(),
                },
                OpenAiMessage {
                    role: "user".to_string(),
                    content: prompt,
                },
            ],
            max_tokens: self.max_tokens,
            temperature: self.temperature,
        };

        let response = self.call_openai_with_retry(request).await?;

        let content = response.choices
            .first()
            .ok_or_else(|| LlmError::InvalidResponse("No choices in response".to_string()))?
            .message
            .content
            .clone();

        if let Some(usage) = response.usage {
            info!("LLM completion generated. Tokens: {} prompt + {} completion = {} total",
                  usage.prompt_tokens, usage.completion_tokens, usage.total_tokens);
        }

        Ok(content)
    }

    async fn generate_summary(&self, text: String, max_length: usize) -> Result<String, LlmError> {
        let prompt = format!(
            "Summarize the following text in no more than {} words:\n\n{}",
            max_length, text
        );

        self.generate_completion(prompt).await
    }

    async fn get_embedding(&self, text: String) -> Result<Vec<f32>, LlmError> {
        info!("Generating embedding for text (length: {} chars)", text.len());

        let request = OpenAiEmbeddingRequest {
            model: "text-embedding-3-small".to_string(),
            input: text,
        };

        let response = self.client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    LlmError::Timeout
                } else {
                    LlmError::NetworkError(e.to_string())
                }
            })?;

        let status = response.status();

        if status == 429 {
            return Err(LlmError::RateLimited);
        }

        if !status.is_success() {
            let error_text = response.text().await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(LlmError::ApiError(format!("HTTP {}: {}", status, error_text)));
        }

        let embedding_response: OpenAiEmbeddingResponse = response.json()
            .await
            .map_err(|e| LlmError::InvalidResponse(e.to_string()))?;

        let embedding = embedding_response.data
            .first()
            .ok_or_else(|| LlmError::InvalidResponse("No embedding data in response".to_string()))?
            .embedding
            .clone();

        info!("Embedding generated (dimension: {})", embedding.len());

        Ok(embedding)
    }
}

/// Cached response with expiration
#[derive(Debug, Clone)]
struct CachedResponse {
    content: String,
    created_at: Instant,
}

/// LLM response cache with TTL
pub struct LlmCache {
    cache: Arc<RwLock<HashMap<String, CachedResponse>>>,
    ttl: Duration,
}

impl LlmCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }

    pub async fn get(&self, key: &str) -> Option<String> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(key) {
            if cached.created_at.elapsed() < self.ttl {
                info!("Cache hit for key: {}", &key[..key.len().min(50)]);
                return Some(cached.content.clone());
            }
        }
        None
    }

    pub async fn set(&self, key: String, value: String) {
        let mut cache = self.cache.write().await;
        cache.insert(key.clone(), CachedResponse {
            content: value,
            created_at: Instant::now(),
        });
        info!("Cached response (key: {})", &key[..key.len().min(50)]);
    }

    #[allow(dead_code)]
    pub async fn clear_expired(&self) {
        let mut cache = self.cache.write().await;
        let initial_count = cache.len();
        cache.retain(|_, v| v.created_at.elapsed() < self.ttl);
        let removed_count = initial_count - cache.len();
        if removed_count > 0 {
            info!("Cleared {} expired cache entries", removed_count);
        }
    }
}

/// Rate limit tracker for a user
#[derive(Debug, Clone)]
struct RateLimitEntry {
    count: usize,
    window_start: Instant,
}

/// Per-user rate limiter
pub struct RateLimiter {
    limits: Arc<RwLock<HashMap<Uuid, RateLimitEntry>>>,
    max_requests_per_hour: usize,
    window_duration: Duration,
}

impl RateLimiter {
    pub fn new(max_requests_per_hour: usize) -> Self {
        Self {
            limits: Arc::new(RwLock::new(HashMap::new())),
            max_requests_per_hour,
            window_duration: Duration::from_secs(3600), // 1 hour
        }
    }

    pub async fn check_and_increment(&self, user_id: Uuid) -> Result<(), LlmError> {
        let mut limits = self.limits.write().await;
        let now = Instant::now();

        let entry = limits.entry(user_id).or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
        });

        // Reset window if expired
        if now.duration_since(entry.window_start) >= self.window_duration {
            entry.count = 0;
            entry.window_start = now;
        }

        // Check if rate limit exceeded
        if entry.count >= self.max_requests_per_hour {
            warn!("Rate limit exceeded for user: {}", user_id);
            return Err(LlmError::RateLimited);
        }

        // Increment counter
        entry.count += 1;
        info!("LLM request count for user {}: {}/{}", user_id, entry.count, self.max_requests_per_hour);

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn cleanup_expired(&self) {
        let mut limits = self.limits.write().await;
        let now = Instant::now();
        let initial_count = limits.len();
        limits.retain(|_, v| now.duration_since(v.window_start) < self.window_duration);
        let removed_count = initial_count - limits.len();
        if removed_count > 0 {
            info!("Cleaned up {} expired rate limit entries", removed_count);
        }
    }
}

/// LLM service with provider abstraction, caching, and rate limiting
pub struct LlmService {
    #[allow(dead_code)]
    config: LlmConfig,
    provider: Option<Arc<dyn LlmProvider>>,
    cache: LlmCache,
    rate_limiter: RateLimiter,
}

impl LlmService {
    pub fn new(config: LlmConfig) -> Self {
        let provider = if config.enabled {
            if let Some(api_key) = &config.api_key {
                if !api_key.is_empty() {
                    info!("Initializing LLM service with provider: {}", config.provider);
                    match config.provider.as_str() {
                        "openai" => {
                            let provider = OpenAiProvider::new(
                                api_key.clone(),
                                config.max_tokens,
                                config.temperature,
                            );
                            Some(Arc::new(provider) as Arc<dyn LlmProvider>)
                        },
                        _ => {
                            warn!("Unknown LLM provider: {}. LLM features disabled.", config.provider);
                            None
                        }
                    }
                } else {
                    warn!("LLM API key is empty. LLM features disabled.");
                    None
                }
            } else {
                warn!("LLM API key not configured. LLM features disabled.");
                None
            }
        } else {
            info!("LLM features are disabled in configuration");
            None
        };

        Self {
            config,
            provider,
            cache: LlmCache::new(Duration::from_secs(3600)), // 1 hour TTL
            rate_limiter: RateLimiter::new(50), // 50 requests per hour per user
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.provider.is_some()
    }

    /// Generate completion with rate limiting and caching
    pub async fn generate_completion_for_user(
        &self,
        user_id: Uuid,
        prompt: String,
    ) -> Result<String, LlmError> {
        // Check rate limit first
        self.rate_limiter.check_and_increment(user_id).await?;

        // Check cache
        let cache_key = format!("completion:{}", Self::hash_prompt(&prompt));
        if let Some(cached) = self.cache.get(&cache_key).await {
            return Ok(cached);
        }

        // Call provider
        let provider = self.provider.as_ref()
            .ok_or(LlmError::Disabled)?;

        let result = provider.generate_completion(prompt).await?;

        // Cache the result
        self.cache.set(cache_key, result.clone()).await;

        Ok(result)
    }

    /// Generate completion without rate limiting (for internal use)
    #[allow(dead_code)]
    pub async fn generate_completion(&self, prompt: String) -> Result<String, LlmError> {
        let provider = self.provider.as_ref()
            .ok_or(LlmError::Disabled)?;

        provider.generate_completion(prompt).await
    }

    #[allow(dead_code)]
    pub async fn generate_summary(&self, text: String, max_length: usize) -> Result<String, LlmError> {
        let provider = self.provider.as_ref()
            .ok_or(LlmError::Disabled)?;

        provider.generate_summary(text, max_length).await
    }

    #[allow(dead_code)]
    pub async fn get_embedding(&self, text: String) -> Result<Vec<f32>, LlmError> {
        let provider = self.provider.as_ref()
            .ok_or(LlmError::Disabled)?;

        provider.get_embedding(text).await
    }

    /// Hash prompt for cache key generation
    fn hash_prompt(prompt: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Clean up expired cache entries and rate limits
    #[allow(dead_code)]
    pub async fn cleanup(&self) {
        self.cache.clear_expired().await;
        self.rate_limiter.cleanup_expired().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llm_config_default() {
        let config = LlmConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.provider, "openai");
        assert_eq!(config.max_tokens, 500);
        assert_eq!(config.temperature, 0.7);
    }

    #[test]
    fn test_llm_service_disabled_by_default() {
        let config = LlmConfig::default();
        let service = LlmService::new(config);
        assert!(!service.is_enabled());
    }

    #[tokio::test]
    async fn test_llm_service_returns_disabled_error() {
        let config = LlmConfig::default();
        let service = LlmService::new(config);

        let result = service.generate_completion("test".to_string()).await;
        assert!(matches!(result, Err(LlmError::Disabled)));
    }

    #[tokio::test]
    async fn test_cache_stores_and_retrieves() {
        let cache = LlmCache::new(Duration::from_secs(60));
        cache.set("test_key".to_string(), "test_value".to_string()).await;

        let result = cache.get("test_key").await;
        assert_eq!(result, Some("test_value".to_string()));
    }

    #[tokio::test]
    async fn test_cache_expires() {
        let cache = LlmCache::new(Duration::from_millis(100));
        cache.set("test_key".to_string(), "test_value".to_string()).await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        let result = cache.get("test_key").await;
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(3);
        let user_id = Uuid::new_v4();

        assert!(limiter.check_and_increment(user_id).await.is_ok());
        assert!(limiter.check_and_increment(user_id).await.is_ok());
        assert!(limiter.check_and_increment(user_id).await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(2);
        let user_id = Uuid::new_v4();

        assert!(limiter.check_and_increment(user_id).await.is_ok());
        assert!(limiter.check_and_increment(user_id).await.is_ok());

        let result = limiter.check_and_increment(user_id).await;
        assert!(matches!(result, Err(LlmError::RateLimited)));
    }
}
