//! Axiom Agents - Agent framework with tools, memory, and execution planning
//!
//! This crate provides a comprehensive agent framework that includes:
//! - Tool execution and management
//! - Memory systems for conversation and context
//! - Execution planning and reasoning
//! - Safety guards and validation

pub mod agent;
pub mod planner;
pub mod executor;
pub mod safety;
pub mod tools;

pub use agent::*;
pub use planner::*;
pub use executor::*;
pub use safety::*;
pub use tools::*;

/// Re-exports commonly used types for convenience
pub mod prelude {
    pub use crate::{
        Agent, AgentConfig, AgentState, AgentResult,
        Planner, ExecutionPlan, PlanStep,
        Executor, ExecutionContext,
        SafetyGuard, SafetyResult,
    };
    pub use axiom_ai_core::prelude::*;
    pub use axiom_ai_llm::prelude::*;
}
