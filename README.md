# Axiom

A streaming-first, production-ready LangChain alternative built in Rust.

## 🚀 Key Features

- **Streaming-First**: Built from the ground up for real-time streaming responses
- **Production-Ready**: Comprehensive monitoring, retries, safety guards, and observability
- **Vendor-Agnostic**: Unified interface for all LLM providers (OpenAI, Anthropic, etc.)
- **WASM Sandboxing**: Safe agent execution with WebAssembly isolation
- **Optimized RAG**: High-performance retrieval-augmented generation with vector search
- **Memory Efficient**: Rust's zero-cost abstractions and memory safety
- **Agent Framework**: Complete agent system with tools, memory, and execution planning
- **Monitoring**: Built-in metrics, tracing, health checks, and alerting

## 🏗️ Architecture

```
axiom-core/          # Core abstractions and traits
├── Message types and conversation handling
├── Streaming response system
├── Chain composition framework
├── Tool system and execution
├── Memory management
└── Configuration system

axiom-llm/           # LLM gateway and provider implementations
├── Unified LLM interface
├── OpenAI provider
├── Anthropic/Claude provider
├── Retry logic and circuit breakers
├── Rate limiting
└── Monitoring and metrics

axiom-agents/        # Agent framework with tools and memory
├── Agent execution engine
├── Planning system (simple and LLM-based)
├── Tool execution framework
├── Safety guards and validation
└── Built-in tools (calculator, weather, search, etc.)

axiom-rag/           # Optimized RAG layer with vector search
├── Document processing and chunking
├── Embedding models and vector operations
├── Vector stores (in-memory, Redis)
├── Retrieval engine with hybrid search
├── Document indexing
└── Reranking models

axiom-wasm/          # WASM sandboxing for safe execution
├── WASM sandbox with security policies
├── Resource limits and monitoring
├── Module management and validation
└── Safe execution runtime

axiom-monitoring/    # Observability, metrics, and monitoring
├── Metrics collection and export
├── Distributed tracing
├── Health checking
└── Alerting and notifications

axiom-examples/      # Example applications and demos
├── Basic agent example
├── RAG implementation
├── WASM sandbox demo
└── Monitoring showcase
```

## 🚀 Quick Start

### Basic Agent

```rust
use axiom_core::prelude::*;
use axiom_llm::{LlmGateway, OpenAIProvider, ProviderConfig};
use axiom_agents::{Agent, AgentConfig, SimplePlanner, SimpleExecutor, SimpleSafetyGuard};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create LLM gateway
    let openai_config = ProviderConfig::new()
        .with_api_key("your-openai-api-key");
    
    let openai_provider = OpenAIProvider::new(openai_config)?;
    let llm_gateway = std::sync::Arc::new(
        LlmGateway::new("openai")
            .add_provider("openai".to_string(), std::sync::Arc::new(openai_provider))
    );

    // Create agent components
    let memory = Box::new(axiom_core::InMemoryMemory::new());
    let planner = Box::new(SimplePlanner::new(10));
    let executor = Box::new(SimpleExecutor::new(llm_gateway.clone(), "gpt-3.5-turbo".to_string()));
    let safety_guard = Box::new(SimpleSafetyGuard::new());
    let config = AgentConfig::default();

    // Create agent
    let mut agent = Agent::new(
        llm_gateway,
        memory,
        planner,
        executor,
        safety_guard,
        config,
    );

    // Process a message
    let message = Message::user("What is 2 + 2?");
    let result = agent.process_message(message).await?;

    println!("Agent response: {:?}", result);
    Ok(())
}
```

### RAG Implementation

```rust
use axiom_rag::prelude::*;
use axiom_llm::{LlmGateway, OpenAIProvider, ProviderConfig};

#[tokio::main]
async fn main() -> Result<()> {
    // Create embedding service
    let embedding_service = EmbeddingService::new("local".to_string())
        .add_model("local".to_string(), Box::new(LocalEmbeddingModel::new("all-MiniLM-L6-v2".to_string())));

    // Create vector store
    let vector_store = VectorStoreFactory::create_in_memory();

    // Create retrieval engine
    let config = RetrievalConfig::default();
    let mut retrieval_engine = RetrievalEngine::new(vector_store, embedding_service, config);

    // Create and index documents
    let documents = vec![
        Document::new("doc1", "Rust is a systems programming language that focuses on safety and performance.")
            .with_title("Rust Programming Language"),
        Document::new("doc2", "Machine learning is a subset of artificial intelligence that focuses on algorithms.")
            .with_title("Machine Learning"),
    ];

    // Process and index documents
    let processor = DocumentProcessor::default();
    for mut document in documents {
        document = processor.process_document(document).await?;
        let chunks = document.get_chunks().to_vec();
        retrieval_engine.add_document(chunks).await?;
    }

    // Search for relevant documents
    let query = "What is Rust?";
    let results = retrieval_engine.search(query).await?;

    println!("Found {} relevant documents:", results.len());
    for (i, result) in results.iter().enumerate() {
        println!("{}. {} (score: {:.3})", i + 1, result.chunk.content, result.score);
    }

    Ok(())
}
```

### WASM Sandboxing

```rust
use axiom_wasm::{WasmSandbox, WasmSandboxBuilder, SecurityPolicy, ResourceLimits};

#[tokio::main]
async fn main() -> Result<()> {
    // Create security policy
    let security_policy = SecurityPolicy::new()
        .forbid_import("env")
        .forbid_import("wasi_snapshot_preview1")
        .with_memory_protection(true)
        .with_stack_protection(true);

    // Create resource limits
    let resource_limits = ResourceLimits::new()
        .with_max_module_size(1024 * 1024) // 1MB
        .with_max_memory_bytes(16 * 1024 * 1024) // 16MB
        .with_max_execution_time(5000) // 5 seconds
        .with_max_executions(100);

    // Create WASM sandbox
    let mut sandbox = WasmSandboxBuilder::new()
        .with_security_policy(security_policy)
        .with_resource_limits(resource_limits)
        .build()?;

    // Load a WASM module (example with empty bytes)
    match sandbox.load_module("example".to_string(), &[]).await {
        Ok(module) => {
            println!("Module loaded: {:?}", module.metadata);
        }
        Err(e) => {
            println!("Failed to load module: {}", e);
        }
    }

    Ok(())
}
```

## 📦 Installation

Add Axiom to your `Cargo.toml`:

```toml
[dependencies]
axiom-core = "0.1.0"
axiom-llm = "0.1.0"
axiom-agents = "0.1.0"
axiom-rag = "0.1.0"
axiom-wasm = "0.1.0"
axiom-monitoring = "0.1.0"
```

## 🔧 Configuration

Axiom supports configuration through environment variables or configuration files:

```rust
use axiom_core::Config;

// Load from environment
let config = Config::from_env()?;

// Load from file
let config = Config::from_file("axiom.toml").await?;

// Create programmatically
let config = ConfigBuilder::new()
    .default_provider("openai")
    .add_provider("openai", ProviderConfig::new().with_api_key("your-key"))
    .enable_monitoring(true)
    .enable_rag(true)
    .build();
```

## 🛠️ Examples

Run the examples:

```bash
# Run all examples
cargo run --bin axiom-examples -- all

# Run specific examples
cargo run --bin axiom-examples -- basic-agent
cargo run --bin axiom-examples -- rag
cargo run --bin axiom-examples -- wasm-sandbox
cargo run --bin axiom-examples -- monitoring
```

## 🔍 Monitoring and Observability

Axiom includes comprehensive monitoring capabilities:

- **Metrics**: Prometheus-compatible metrics export
- **Tracing**: Distributed tracing with Jaeger/Zipkin support
- **Health Checks**: Built-in health checking for all components
- **Alerting**: Configurable alerting with multiple notification channels

```rust
use axiom_monitoring::{MetricsCollector, TracingConfig, HealthCheckManager};

// Initialize metrics
let metrics = MetricsCollector::new()?;

// Initialize tracing
let tracing_config = TracingConfig::new("my-service".to_string())
    .with_jaeger("http://localhost:14268/api/traces".to_string());
tracing_config.init()?;

// Set up health checks
let mut health_manager = HealthCheckManager::new();
health_manager.add_checker(Box::new(BasicHealthChecker::new("my-service".to_string())));
```

## 🚀 Performance

Axiom is built for performance:

- **Zero-copy streaming**: Efficient memory usage for streaming responses
- **Async/await**: Non-blocking I/O throughout
- **Connection pooling**: Reuse HTTP connections for LLM providers
- **Batch processing**: Efficient batch operations for embeddings and vector operations
- **Memory safety**: Rust's ownership system prevents memory leaks and data races

## 🔒 Security

Security is built-in:

- **WASM sandboxing**: Isolate untrusted code execution
- **Input validation**: Comprehensive validation of all inputs
- **Rate limiting**: Prevent abuse and ensure fair usage
- **Safety guards**: Configurable safety checks for agent actions
- **Secure defaults**: Secure-by-default configuration

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Inspired by [LangChain](https://github.com/langchain-ai/langchain) and the broader AI agent ecosystem
- Built with [Tokio](https://tokio.rs/) for async runtime
- Uses [Wasmtime](https://wasmtime.dev/) for WebAssembly execution
- Vector operations powered by [Candle](https://github.com/huggingface/candle)

## 📚 Documentation

- [API Documentation](https://docs.rs/axiom)
- [Examples](https://github.com/axiom-ai/axiom/tree/main/axiom-examples)
- [Architecture Guide](docs/architecture.md)
- [Security Guide](docs/security.md)
