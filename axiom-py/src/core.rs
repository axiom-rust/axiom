//! Core types and functionality exposed to Python

use pyo3::prelude::*;
use axiom_core::{
    Message as RustMessage, MessageRole as RustMessageRole, 
    Conversation as RustConversation, Config as RustConfig, ConfigBuilder as RustConfigBuilder
};

/// Python wrapper for Message
#[pyclass(name = "Message")]
#[derive(Clone)]
pub struct Message {
    inner: RustMessage,
}

#[pymethods]
impl Message {
    /// Create a system message
    #[staticmethod]
    fn system(text: &str) -> Self {
        Self {
            inner: RustMessage::system(text),
        }
    }

    /// Create a user message
    #[staticmethod]
    fn user(text: &str) -> Self {
        Self {
            inner: RustMessage::user(text),
        }
    }

    /// Create an assistant message
    #[staticmethod]
    fn assistant(text: &str) -> Self {
        Self {
            inner: RustMessage::assistant(text),
        }
    }

    /// Create a tool message
    #[staticmethod]
    fn tool(tool_call_id: &str, content: &str) -> Self {
        Self {
            inner: RustMessage::tool(tool_call_id, content),
        }
    }

    /// Get the text content if this is a text message
    fn text_content(&self) -> Option<String> {
        self.inner.text_content().map(|s| s.to_string())
    }

    /// Check if this message contains tool calls
    fn has_tool_calls(&self) -> bool {
        self.inner.has_tool_calls()
    }

    /// Get the role of the message
    fn role(&self) -> MessageRole {
        self.inner.role.clone().into()
    }

    /// Get the content of the message
    fn content(&self) -> String {
        self.inner.text_content().unwrap_or_default().to_string()
    }

    /// Get the message ID
    fn id(&self) -> String {
        self.inner.id.to_string()
    }

    /// Get the timestamp
    fn timestamp(&self) -> String {
        self.inner.timestamp.to_rfc3339()
    }
}

impl From<RustMessage> for Message {
    fn from(message: RustMessage) -> Self {
        Self { inner: message }
    }
}

/// Python wrapper for MessageRole
#[pyclass(name = "MessageRole")]
#[derive(Clone, PartialEq, Eq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

impl From<RustMessageRole> for MessageRole {
    fn from(role: RustMessageRole) -> Self {
        match role {
            RustMessageRole::System => MessageRole::System,
            RustMessageRole::User => MessageRole::User,
            RustMessageRole::Assistant => MessageRole::Assistant,
            RustMessageRole::Tool => MessageRole::Tool,
        }
    }
}

impl From<MessageRole> for RustMessageRole {
    fn from(role: MessageRole) -> Self {
        match role {
            MessageRole::System => RustMessageRole::System,
            MessageRole::User => RustMessageRole::User,
            MessageRole::Assistant => RustMessageRole::Assistant,
            MessageRole::Tool => RustMessageRole::Tool,
        }
    }
}

/// Python wrapper for Conversation
#[pyclass(name = "Conversation")]
pub struct Conversation {
    inner: RustConversation,
}

#[pymethods]
impl Conversation {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustConversation::new(),
        }
    }

    /// Add a message to the conversation
    fn add_message(&mut self, message: Message) {
        self.inner.add_message(message.inner.clone());
    }

    /// Get the last message
    fn last_message(&self) -> Option<Message> {
        self.inner.last_message().map(|msg| msg.clone().into())
    }

    /// Get messages by role
    fn messages_by_role(&self, role: MessageRole) -> Vec<Message> {
        self.inner
            .messages_by_role(role.into())
            .into_iter()
            .map(|msg| msg.clone().into())
            .collect()
    }

    /// Clear all messages
    fn clear(&mut self) {
        self.inner.clear();
    }

    /// Get the total number of messages
    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the conversation is empty
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get all messages
    fn messages(&self) -> Vec<Message> {
        self.inner
            .messages
            .iter()
            .map(|msg| msg.clone().into())
            .collect()
    }
}

impl From<RustConversation> for Conversation {
    fn from(conv: RustConversation) -> Self {
        Self { inner: conv }
    }
}

/// Python wrapper for Config
#[pyclass(name = "Config")]
pub struct Config {
    inner: RustConfig,
}

#[pymethods]
impl Config {
    /// Load configuration from environment variables
    #[staticmethod]
    fn from_env() -> PyResult<Self> {
        let config = RustConfig::from_env()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        Ok(Self { inner: config })
    }

    /// Get a configuration value
    fn get(&self, key: &str) -> Option<String> {
        // TODO: Implement proper config access once axiom-core Config API is finalized
        match key {
            "default_provider" => Some("openai".to_string()),
            "monitoring_enabled" => Some("true".to_string()),
            "rag_enabled" => Some("true".to_string()),
            _ => None,
        }
    }

    /// Set a configuration value
    fn set(&mut self, key: &str, value: &str) {
        // TODO: Implement proper config setting once axiom-core Config API is finalized
        println!("Setting config {} = {}", key, value);
    }
}

impl From<RustConfig> for Config {
    fn from(config: RustConfig) -> Self {
        Self { inner: config }
    }
}

/// Python wrapper for ConfigBuilder
#[pyclass(name = "ConfigBuilder")]
pub struct ConfigBuilder {
    inner: RustConfigBuilder,
}

#[pymethods]
impl ConfigBuilder {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustConfigBuilder::new(),
        }
    }

    /// Set the default provider
    fn default_provider<'a>(mut self_: PyRefMut<'a, Self>, provider: &str) -> PyRefMut<'a, Self> {
        let new_inner = std::mem::replace(&mut self_.inner, RustConfigBuilder::new());
        self_.inner = new_inner.default_provider(provider);
        self_
    }

    /// Enable monitoring
    fn enable_monitoring<'a>(mut self_: PyRefMut<'a, Self>, enabled: bool) -> PyRefMut<'a, Self> {
        let new_inner = std::mem::replace(&mut self_.inner, RustConfigBuilder::new());
        self_.inner = new_inner.enable_monitoring(enabled);
        self_
    }

    /// Enable RAG
    fn enable_rag<'a>(mut self_: PyRefMut<'a, Self>, enabled: bool) -> PyRefMut<'a, Self> {
        let new_inner = std::mem::replace(&mut self_.inner, RustConfigBuilder::new());
        self_.inner = new_inner.enable_rag(enabled);
        self_
    }

    /// Build the configuration
    fn build(mut self_: PyRefMut<Self>) -> Config {
        let inner = std::mem::replace(&mut self_.inner, RustConfigBuilder::new());
        Config {
            inner: inner.build(),
        }
    }
}

impl From<RustConfigBuilder> for ConfigBuilder {
    fn from(builder: RustConfigBuilder) -> Self {
        Self { inner: builder }
    }
}