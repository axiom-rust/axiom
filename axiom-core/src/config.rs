//! Configuration management for Axiom

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use crate::{Result, AxiomError};

/// Main configuration for Axiom
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// LLM provider configuration
    pub llm: LlmConfig,
    /// Memory configuration
    pub memory: MemoryConfig,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
    /// Agent configuration
    pub agent: AgentConfig,
    /// RAG configuration
    pub rag: RagConfig,
    /// WASM configuration
    pub wasm: WasmConfig,
    /// General settings
    pub general: GeneralConfig,
}

/// LLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    /// Default provider to use
    pub default_provider: String,
    /// Provider-specific configurations
    pub providers: HashMap<String, ProviderConfig>,
    /// Default model parameters
    pub default_params: ModelParams,
}

/// Provider-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// API key for the provider
    pub api_key: Option<String>,
    /// Base URL for the provider
    pub base_url: Option<String>,
    /// Default model for this provider
    pub default_model: Option<String>,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
    /// Additional provider-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per minute
    pub requests_per_minute: Option<u32>,
    /// Requests per hour
    pub requests_per_hour: Option<u32>,
    /// Tokens per minute
    pub tokens_per_minute: Option<u32>,
}

/// Model parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelParams {
    /// Temperature for generation
    pub temperature: f32,
    /// Maximum tokens to generate
    pub max_tokens: u32,
    /// Top-p sampling parameter
    pub top_p: f32,
    /// Top-k sampling parameter
    pub top_k: Option<u32>,
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
    /// Stop sequences
    pub stop_sequences: Vec<String>,
}

impl Default for ModelParams {
    fn default() -> Self {
        Self {
            temperature: 0.7,
            max_tokens: 1000,
            top_p: 1.0,
            top_k: None,
            presence_penalty: None,
            frequency_penalty: None,
            stop_sequences: Vec::new(),
        }
    }
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Type of memory backend
    pub backend: MemoryBackend,
    /// Maximum number of items to store
    pub max_items: Option<usize>,
    /// Maximum memory size in bytes
    pub max_size_bytes: Option<usize>,
    /// TTL for memory items in seconds
    pub ttl_seconds: Option<u64>,
    /// Backend-specific settings
    pub settings: HashMap<String, serde_json::Value>,
}

/// Memory backend types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryBackend {
    /// In-memory storage
    InMemory,
    /// Redis backend
    Redis,
    /// PostgreSQL backend
    Postgres,
    /// SQLite backend
    Sqlite,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable monitoring
    pub enabled: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval: u64,
    /// Log level
    pub log_level: String,
    /// Enable tracing
    pub enable_tracing: bool,
    /// Prometheus endpoint
    pub prometheus_endpoint: Option<String>,
    /// Jaeger endpoint for distributed tracing
    pub jaeger_endpoint: Option<String>,
}

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Maximum number of tool calls per turn
    pub max_tool_calls: u32,
    /// Tool call timeout in seconds
    pub tool_call_timeout: u64,
    /// Enable tool retries
    pub enable_tool_retries: bool,
    /// Maximum tool retries
    pub max_tool_retries: u32,
    /// Enable safety checks
    pub enable_safety_checks: bool,
    /// Safety check timeout in seconds
    pub safety_check_timeout: u64,
}

/// RAG configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// Enable RAG
    pub enabled: bool,
    /// Vector database backend
    pub vector_db: VectorDbConfig,
    /// Embedding model configuration
    pub embedding: EmbeddingConfig,
    /// Retrieval configuration
    pub retrieval: RetrievalConfig,
}

/// Vector database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorDbConfig {
    /// Backend type
    pub backend: VectorDbBackend,
    /// Connection settings
    pub connection: HashMap<String, serde_json::Value>,
    /// Index settings
    pub index: HashMap<String, serde_json::Value>,
}

/// Vector database backend types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VectorDbBackend {
    /// ChromaDB
    Chroma,
    /// Pinecone
    Pinecone,
    /// Weaviate
    Weaviate,
    /// Qdrant
    Qdrant,
    /// In-memory vector store
    InMemory,
}

/// Embedding configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingConfig {
    /// Model to use for embeddings
    pub model: String,
    /// Dimension of embeddings
    pub dimension: usize,
    /// Batch size for embedding generation
    pub batch_size: usize,
}

/// Retrieval configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalConfig {
    /// Number of documents to retrieve
    pub top_k: usize,
    /// Similarity threshold
    pub similarity_threshold: f32,
    /// Enable reranking
    pub enable_reranking: bool,
}

/// WASM configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    /// Enable WASM sandboxing
    pub enabled: bool,
    /// Maximum memory for WASM modules
    pub max_memory: usize,
    /// Maximum execution time in seconds
    pub max_execution_time: u64,
    /// Allowed WASM modules
    pub allowed_modules: Vec<String>,
}

/// General configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Application name
    pub app_name: String,
    /// Application version
    pub app_version: String,
    /// Environment (dev, staging, prod)
    pub environment: String,
    /// Debug mode
    pub debug: bool,
    /// Data directory
    pub data_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            llm: LlmConfig {
                default_provider: "openai".to_string(),
                providers: HashMap::new(),
                default_params: ModelParams::default(),
            },
            memory: MemoryConfig {
                backend: MemoryBackend::InMemory,
                max_items: Some(10000),
                max_size_bytes: Some(100 * 1024 * 1024), // 100MB
                ttl_seconds: Some(3600), // 1 hour
                settings: HashMap::new(),
            },
            monitoring: MonitoringConfig {
                enabled: true,
                metrics_interval: 60,
                log_level: "info".to_string(),
                enable_tracing: true,
                prometheus_endpoint: Some("0.0.0.0:9090".to_string()),
                jaeger_endpoint: None,
            },
            agent: AgentConfig {
                max_tool_calls: 10,
                tool_call_timeout: 30,
                enable_tool_retries: true,
                max_tool_retries: 3,
                enable_safety_checks: true,
                safety_check_timeout: 5,
            },
            rag: RagConfig {
                enabled: false,
                vector_db: VectorDbConfig {
                    backend: VectorDbBackend::InMemory,
                    connection: HashMap::new(),
                    index: HashMap::new(),
                },
                embedding: EmbeddingConfig {
                    model: "text-embedding-ada-002".to_string(),
                    dimension: 1536,
                    batch_size: 100,
                },
                retrieval: RetrievalConfig {
                    top_k: 5,
                    similarity_threshold: 0.7,
                    enable_reranking: false,
                },
            },
            wasm: WasmConfig {
                enabled: false,
                max_memory: 64 * 1024 * 1024, // 64MB
                max_execution_time: 30,
                allowed_modules: Vec::new(),
            },
            general: GeneralConfig {
                app_name: "axiom".to_string(),
                app_version: "0.1.0".to_string(),
                environment: "dev".to_string(),
                debug: false,
                data_dir: "./data".to_string(),
            },
        }
    }
}

/// Configuration builder
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new() -> Self {
        Self {
            config: Config::default(),
        }
    }

    /// Set the default LLM provider
    pub fn default_provider(mut self, provider: impl Into<String>) -> Self {
        self.config.llm.default_provider = provider.into();
        self
    }

    /// Add a provider configuration
    pub fn add_provider(mut self, name: impl Into<String>, provider: ProviderConfig) -> Self {
        self.config.llm.providers.insert(name.into(), provider);
        self
    }

    /// Set memory backend
    pub fn memory_backend(mut self, backend: MemoryBackend) -> Self {
        self.config.memory.backend = backend;
        self
    }

    /// Enable monitoring
    pub fn enable_monitoring(mut self, enabled: bool) -> Self {
        self.config.monitoring.enabled = enabled;
        self
    }

    /// Enable RAG
    pub fn enable_rag(mut self, enabled: bool) -> Self {
        self.config.rag.enabled = enabled;
        self
    }

    /// Enable WASM sandboxing
    pub fn enable_wasm(mut self, enabled: bool) -> Self {
        self.config.wasm.enabled = enabled;
        self
    }

    /// Set the environment
    pub fn environment(mut self, env: impl Into<String>) -> Self {
        self.config.general.environment = env.into();
        self
    }

    /// Build the configuration
    pub fn build(self) -> Config {
        self.config
    }
}

impl Config {
    /// Load configuration from a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: Config = toml::from_str(&content)
            .map_err(|e| AxiomError::Config(format!("Failed to parse config file: {}", e)))?;
        Ok(config)
    }

    /// Save configuration to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| AxiomError::Config(format!("Failed to serialize config: {}", e)))?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let mut config = Config::default();

        // Load from environment variables
        if let Ok(provider) = std::env::var("AXIOM_DEFAULT_PROVIDER") {
            config.llm.default_provider = provider;
        }

        if let Ok(api_key) = std::env::var("AXIOM_API_KEY") {
            config.llm.providers.insert(
                "openai".to_string(),
                ProviderConfig {
                    api_key: Some(api_key),
                    base_url: None,
                    default_model: None,
                    rate_limit: None,
                    settings: HashMap::new(),
                },
            );
        }

        if let Ok(env) = std::env::var("AXIOM_ENVIRONMENT") {
            config.general.environment = env;
        }

        if let Ok(debug) = std::env::var("AXIOM_DEBUG") {
            config.general.debug = debug.parse().unwrap_or(false);
        }

        Ok(config)
    }

    /// Get provider configuration
    pub fn get_provider(&self, name: &str) -> Option<&ProviderConfig> {
        self.llm.providers.get(name)
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate LLM configuration
        if self.llm.providers.is_empty() {
            return Err(AxiomError::Config("No LLM providers configured".to_string()));
        }

        if !self.llm.providers.contains_key(&self.llm.default_provider) {
            return Err(AxiomError::Config(format!(
                "Default provider '{}' not found in providers",
                self.llm.default_provider
            )));
        }

        // Validate model parameters
        if self.llm.default_params.temperature < 0.0 || self.llm.default_params.temperature > 2.0 {
            return Err(AxiomError::Config("Temperature must be between 0.0 and 2.0".to_string()));
        }

        if self.llm.default_params.top_p < 0.0 || self.llm.default_params.top_p > 1.0 {
            return Err(AxiomError::Config("Top-p must be between 0.0 and 1.0".to_string()));
        }

        // Validate memory configuration
        if let Some(max_items) = self.memory.max_items {
            if max_items == 0 {
                return Err(AxiomError::Config("Max items must be greater than 0".to_string()));
            }
        }

        // Validate agent configuration
        if self.agent.max_tool_calls == 0 {
            return Err(AxiomError::Config("Max tool calls must be greater than 0".to_string()));
        }

        Ok(())
    }
}
