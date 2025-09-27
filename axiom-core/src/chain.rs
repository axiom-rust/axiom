//! Chain composition and execution framework

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{Message, StreamResponse, Result, AxiomError};

/// A chain is a composable unit of execution that can be chained together
#[async_trait]
pub trait Chain: Send + Sync {
    /// Execute the chain with the given input
    async fn execute(&self, input: ChainInput) -> Result<ChainOutput>;

    /// Execute the chain and return a streaming response
    async fn stream(&self, input: ChainInput) -> Result<StreamResponse>;

    /// Get the name of the chain
    fn name(&self) -> &str;

    /// Get the description of the chain
    fn description(&self) -> &str;
}

/// Input to a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainInput {
    /// Messages in the conversation
    pub messages: Vec<Message>,
    /// Additional context data
    pub context: HashMap<String, serde_json::Value>,
    /// Configuration parameters
    pub config: ChainConfig,
}

/// Output from a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainOutput {
    /// Response messages
    pub messages: Vec<Message>,
    /// Additional output data
    pub data: HashMap<String, serde_json::Value>,
    /// Metadata about the execution
    pub metadata: ChainMetadata,
}

/// Configuration for chain execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Temperature for generation
    pub temperature: Option<f32>,
    /// Maximum tokens to generate
    pub max_tokens: Option<u32>,
    /// Top-p sampling parameter
    pub top_p: Option<f32>,
    /// Whether to stream the response
    pub stream: bool,
    /// Additional parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

impl Default for ChainConfig {
    fn default() -> Self {
        Self {
            temperature: Some(0.7),
            max_tokens: Some(1000),
            top_p: Some(1.0),
            stream: false,
            parameters: HashMap::new(),
        }
    }
}

/// Metadata about chain execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainMetadata {
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Number of tokens used
    pub tokens_used: Option<u32>,
    /// Cost of the operation
    pub cost: Option<f64>,
    /// Additional metadata
    pub data: HashMap<String, serde_json::Value>,
}

/// A sequential chain that executes multiple chains in order
pub struct SequentialChain {
    chains: Vec<Box<dyn Chain>>,
    name: String,
    description: String,
}

impl SequentialChain {
    /// Create a new sequential chain
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            chains: Vec::new(),
            name: name.into(),
            description: description.into(),
        }
    }

    /// Add a chain to the sequence
    pub fn add_chain(mut self, chain: Box<dyn Chain>) -> Self {
        self.chains.push(chain);
        self
    }
}

#[async_trait]
impl Chain for SequentialChain {
    async fn execute(&self, input: ChainInput) -> Result<ChainOutput> {
        let start_time = std::time::Instant::now();
        let mut final_output = ChainOutput {
            messages: input.messages.clone(),
            data: input.context.clone(),
            metadata: ChainMetadata {
                execution_time_ms: 0,
                tokens_used: None,
                cost: None,
                data: HashMap::new(),
            },
        };

        for chain in &self.chains {
            let chain_input = ChainInput {
                messages: final_output.messages,
                context: final_output.data,
                config: input.config.clone(),
            };

            let chain_output = chain.execute(chain_input).await?;
            final_output.messages = chain_output.messages;
            final_output.data = chain_output.data;
        }

        final_output.metadata.execution_time_ms = start_time.elapsed().as_millis() as u64;
        Ok(final_output)
    }

    async fn stream(&self, input: ChainInput) -> Result<StreamResponse> {
        // For sequential chains, we'll execute the first chain that supports streaming
        // and pass the output to subsequent chains
        if let Some(first_chain) = self.chains.first() {
            first_chain.stream(input).await
        } else {
            Err(AxiomError::ChainExecution("No chains in sequential chain".to_string()))
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// A parallel chain that executes multiple chains concurrently
pub struct ParallelChain {
    chains: Vec<Box<dyn Chain>>,
    name: String,
    description: String,
}

impl ParallelChain {
    /// Create a new parallel chain
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            chains: Vec::new(),
            name: name.into(),
            description: description.into(),
        }
    }

    /// Add a chain to the parallel execution
    pub fn add_chain(mut self, chain: Box<dyn Chain>) -> Self {
        self.chains.push(chain);
        self
    }
}

#[async_trait]
impl Chain for ParallelChain {
    async fn execute(&self, input: ChainInput) -> Result<ChainOutput> {
        let start_time = std::time::Instant::now();
        
        // Execute all chains in parallel
        let futures: Vec<_> = self.chains
            .iter()
            .map(|chain| chain.execute(input.clone()))
            .collect();

        let results = futures::future::try_join_all(futures).await?;

        // Combine results (this is a simple implementation - in practice you might want more sophisticated merging)
        let mut combined_messages = Vec::new();
        let mut combined_data = HashMap::new();
        let mut total_tokens = 0;
        let mut total_cost = 0.0;

        for result in results {
            combined_messages.extend(result.messages);
            combined_data.extend(result.data);
            if let Some(tokens) = result.metadata.tokens_used {
                total_tokens += tokens;
            }
            if let Some(cost) = result.metadata.cost {
                total_cost += cost;
            }
        }

        Ok(ChainOutput {
            messages: combined_messages,
            data: combined_data,
            metadata: ChainMetadata {
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                tokens_used: Some(total_tokens),
                cost: Some(total_cost),
                data: HashMap::new(),
            },
        })
    }

    async fn stream(&self, input: ChainInput) -> Result<StreamResponse> {
        // For parallel chains, we'll merge the streams from all chains
        let futures: Vec<_> = self.chains
            .iter()
            .map(|chain| chain.stream(input.clone()))
            .collect();

        let results = futures::future::try_join_all(futures).await?;
        
        // Merge all streams
        use crate::stream::utils::merge_streams;
        Ok(StreamResponse::new(merge_streams(results)))
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }
}

/// Builder for creating chains
pub struct ChainBuilder {
    name: String,
    description: String,
    chains: Vec<Box<dyn Chain>>,
    chain_type: ChainType,
}

#[derive(Debug, Clone)]
enum ChainType {
    Sequential,
    Parallel,
}

impl ChainBuilder {
    /// Create a new chain builder
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            chains: Vec::new(),
            chain_type: ChainType::Sequential,
        }
    }

    /// Set the chain type to parallel
    pub fn parallel(mut self) -> Self {
        self.chain_type = ChainType::Parallel;
        self
    }

    /// Add a chain to the builder
    pub fn add_chain(mut self, chain: Box<dyn Chain>) -> Self {
        self.chains.push(chain);
        self
    }

    /// Build the chain
    pub fn build(self) -> Box<dyn Chain> {
        match self.chain_type {
            ChainType::Sequential => {
                let mut seq_chain = SequentialChain::new(self.name, self.description);
                for chain in self.chains {
                    seq_chain = seq_chain.add_chain(chain);
                }
                Box::new(seq_chain)
            }
            ChainType::Parallel => {
                let mut par_chain = ParallelChain::new(self.name, self.description);
                for chain in self.chains {
                    par_chain = par_chain.add_chain(chain);
                }
                Box::new(par_chain)
            }
        }
    }
}
