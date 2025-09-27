//! Axiom WASM - WebAssembly sandboxing for safe agent execution
//!
//! This crate provides WebAssembly sandboxing capabilities for running
//! untrusted code safely within the Axiom framework.

pub mod sandbox;
pub mod runtime;
pub mod security;
pub mod modules;

pub use sandbox::*;
pub use runtime::*;
pub use security::*;
pub use modules::*;

/// Re-exports commonly used types for convenience
pub mod prelude {
    pub use crate::{
        WasmSandbox, WasmRuntime, WasmModule, WasmExecutionResult,
        SecurityPolicy, ResourceLimits,
    };
    pub use axiom_core::prelude::*;
}
