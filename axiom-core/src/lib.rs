//! Axiom Core - Core abstractions and traits for the Axiom framework
//!
//! This crate provides the fundamental building blocks for building AI applications
//! with streaming-first, production-ready capabilities.

pub mod error;
pub mod message;
pub mod stream;
pub mod chain;
pub mod tool;
pub mod memory;
pub mod config;

pub use error::{AxiomError, Result};
pub use message::*;
pub use stream::*;
pub use chain::*;
pub use tool::*;
pub use memory::*;
pub use config::*;

/// Re-exports commonly used types for convenience
pub mod prelude {
    pub use crate::{
        AxiomError, Result,
        Message, MessageRole, MessageContent,
        StreamChunk, StreamResponse,
        Chain, ChainBuilder,
        Tool, ToolResult,
        Memory, MemoryItem,
        Config, ConfigBuilder,
    };
}
