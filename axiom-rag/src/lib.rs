//! Axiom RAG - Optimized RAG layer with vector search and retrieval
//!
//! This crate provides a high-performance retrieval-augmented generation (RAG) system
//! with optimized vector search, document processing, and retrieval mechanisms.

pub mod vector_store;
pub mod embeddings;
pub mod retrieval;
pub mod document;
pub mod indexer;
pub mod reranker;

pub use vector_store::*;
pub use embeddings::*;
pub use retrieval::*;
pub use document::*;
pub use indexer::*;
pub use reranker::*;

/// Re-exports commonly used types for convenience
pub mod prelude {
    pub use crate::{
        VectorStore, EmbeddingModel, Document, DocumentIndexer,
        RetrievalEngine, Reranker, SearchResult,
    };
    pub use axiom_core::prelude::*;
    pub use axiom_llm::prelude::*;
}
