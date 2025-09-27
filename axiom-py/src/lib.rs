//! Python bindings for Axiom
//!
//! This module provides Python bindings for the Axiom AI framework,
//! allowing Python developers to use Axiom's streaming-first, production-ready
//! capabilities from Python code.

use pyo3::prelude::*;

mod core;
// mod agents;
// mod llm;
// mod rag;
// mod monitoring;
// mod wasm;

/// Python module for Axiom
#[pymodule]
fn axiom_py(_py: Python, m: &PyModule) -> PyResult<()> {
    // Core types and functionality
    m.add_class::<core::Message>()?;
    m.add_class::<core::MessageRole>()?;
    // m.add_class::<core::MessageContent>()?; // TODO: Fix enum implementation
    m.add_class::<core::Conversation>()?;
    m.add_class::<core::Config>()?;
    m.add_class::<core::ConfigBuilder>()?;

    // TODO: Add other modules once dependency conflicts are resolved
    // Agent functionality
    // m.add_class::<agents::Agent>()?;
    // m.add_class::<agents::AgentConfig>()?;
    // m.add_class::<agents::AgentResult>()?;

    // LLM providers
    // m.add_class::<llm::LlmGateway>()?;
    // m.add_class::<llm::OpenAIProvider>()?;
    // m.add_class::<llm::AnthropicProvider>()?;
    // m.add_class::<llm::ProviderConfig>()?;

    // RAG functionality
    // m.add_class::<rag::Document>()?;
    // m.add_class::<rag::VectorStore>()?;
    // m.add_class::<rag::RetrievalEngine>()?;

    // Monitoring
    // m.add_class::<monitoring::MetricsCollector>()?;
    // m.add_class::<monitoring::HealthCheckManager>()?;

    // WASM sandboxing
    // m.add_class::<wasm::WasmSandbox>()?;
    // m.add_class::<wasm::SecurityPolicy>()?;

    // Module documentation
    m.add("__version__", "0.1.0")?;
    m.add("__doc__", "Axiom Python bindings - A streaming-first, production-ready LangChain alternative")?;

    Ok(())
}
