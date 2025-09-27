//! Vector store implementations for RAG

use std::collections::HashMap;
use std::sync::Arc;

use axiom_core::{Result, AxiomError};
use crate::document::DocumentChunk;
use crate::embeddings::VectorOperations;

/// Trait for vector stores
#[async_trait::async_trait]
pub trait VectorStore: Send + Sync {
    /// Add a document chunk to the store
    async fn add_chunk(&mut self, chunk: DocumentChunk) -> Result<()>;

    /// Add multiple document chunks to the store
    async fn add_chunks(&mut self, chunks: Vec<DocumentChunk>) -> Result<()>;

    /// Search for similar chunks
    async fn search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>>;

    /// Get a chunk by ID
    async fn get_chunk(&self, id: &str) -> Result<Option<DocumentChunk>>;

    /// Delete a chunk by ID
    async fn delete_chunk(&mut self, id: &str) -> Result<()>;

    /// Get the number of chunks in the store
    async fn count(&self) -> Result<usize>;

    /// Clear all chunks
    async fn clear(&mut self) -> Result<()>;
}

/// Search result from vector store
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Document chunk
    pub chunk: DocumentChunk,
    /// Similarity score
    pub score: f32,
    /// Distance (if applicable)
    pub distance: Option<f32>,
}

/// In-memory vector store implementation
pub struct InMemoryVectorStore {
    /// Chunks storage
    chunks: HashMap<String, DocumentChunk>,
    /// Embeddings storage
    embeddings: HashMap<String, Vec<f32>>,
}

impl InMemoryVectorStore {
    /// Create a new in-memory vector store
    pub fn new() -> Self {
        Self {
            chunks: HashMap::new(),
            embeddings: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn add_chunk(&mut self, chunk: DocumentChunk) -> Result<()> {
        if let Some(embedding) = &chunk.embedding {
            self.embeddings.insert(chunk.id.clone(), embedding.clone());
        }
        self.chunks.insert(chunk.id.clone(), chunk);
        Ok(())
    }

    async fn add_chunks(&mut self, chunks: Vec<DocumentChunk>) -> Result<()> {
        for chunk in chunks {
            self.add_chunk(chunk).await?;
        }
        Ok(())
    }

    async fn search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();

        for (chunk_id, chunk) in &self.chunks {
            if let Some(embedding) = &chunk.embedding {
                let similarity = VectorOperations::cosine_similarity(query_embedding, embedding)?;
                results.push(SearchResult {
                    chunk: chunk.clone(),
                    score: similarity,
                    distance: None,
                });
            }
        }

        // Sort by similarity score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(limit);
        Ok(results)
    }

    async fn get_chunk(&self, id: &str) -> Result<Option<DocumentChunk>> {
        Ok(self.chunks.get(id).cloned())
    }

    async fn delete_chunk(&mut self, id: &str) -> Result<()> {
        self.chunks.remove(id);
        self.embeddings.remove(id);
        Ok(())
    }

    async fn count(&self) -> Result<usize> {
        Ok(self.chunks.len())
    }

    async fn clear(&mut self) -> Result<()> {
        self.chunks.clear();
        self.embeddings.clear();
        Ok(())
    }
}

/// Redis-based vector store implementation
pub struct RedisVectorStore {
    /// Redis client
    client: redis::Client,
    /// Key prefix for storing data
    key_prefix: String,
}

impl RedisVectorStore {
    /// Create a new Redis vector store
    pub fn new(redis_url: &str, key_prefix: String) -> Result<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| AxiomError::Internal(format!("Failed to connect to Redis: {}", e)))?;

        Ok(Self {
            client,
            key_prefix,
        })
    }

    /// Get the key for a chunk
    fn chunk_key(&self, chunk_id: &str) -> String {
        format!("{}:chunk:{}", self.key_prefix, chunk_id)
    }

    /// Get the key for an embedding
    fn embedding_key(&self, chunk_id: &str) -> String {
        format!("{}:embedding:{}", self.key_prefix, chunk_id)
    }

    /// Get the key for the index
    fn index_key(&self) -> String {
        format!("{}:index", self.key_prefix)
    }
}

#[async_trait::async_trait]
impl VectorStore for RedisVectorStore {
    async fn add_chunk(&mut self, chunk: DocumentChunk) -> Result<()> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AxiomError::Internal(format!("Failed to get Redis connection: {}", e)))?;

        // Store chunk data
        let chunk_data = serde_json::to_string(&chunk)
            .map_err(|e| AxiomError::Serialization(format!("Failed to serialize chunk: {}", e)))?;
        
        redis::cmd("SET")
            .arg(self.chunk_key(&chunk.id))
            .arg(chunk_data)
            .query_async(&mut conn)
            .await
            .map_err(|e| AxiomError::Internal(format!("Failed to store chunk: {}", e)))?;

        // Store embedding if available
        if let Some(embedding) = &chunk.embedding {
            let embedding_data = serde_json::to_string(embedding)
                .map_err(|e| AxiomError::Serialization(format!("Failed to serialize embedding: {}", e)))?;
            
            redis::cmd("SET")
                .arg(self.embedding_key(&chunk.id))
                .arg(embedding_data)
                .query_async(&mut conn)
                .await
                .map_err(|e| AxiomError::Internal(format!("Failed to store embedding: {}", e)))?;
        }

        // Add to index
        redis::cmd("SADD")
            .arg(self.index_key())
            .arg(&chunk.id)
            .query_async(&mut conn)
            .await
            .map_err(|e| AxiomError::Internal(format!("Failed to add to index: {}", e)))?;

        Ok(())
    }

    async fn add_chunks(&mut self, chunks: Vec<DocumentChunk>) -> Result<()> {
        for chunk in chunks {
            self.add_chunk(chunk).await?;
        }
        Ok(())
    }

    async fn search(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AxiomError::Internal(format!("Failed to get Redis connection: {}", e)))?;

        // Get all chunk IDs
        let chunk_ids: Vec<String> = redis::cmd("SMEMBERS")
            .arg(self.index_key())
            .query_async(&mut conn)
            .await
            .map_err(|e| AxiomError::Internal(format!("Failed to get chunk IDs: {}", e)))?;

        let mut results = Vec::new();

        for chunk_id in chunk_ids {
            // Get chunk data
            let chunk_data: Option<String> = redis::cmd("GET")
                .arg(self.chunk_key(&chunk_id))
                .query_async(&mut conn)
                .await
                .map_err(|e| AxiomError::Internal(format!("Failed to get chunk: {}", e)))?;

            if let Some(data) = chunk_data {
                let chunk: DocumentChunk = serde_json::from_str(&data)
                    .map_err(|e| AxiomError::Serialization(format!("Failed to deserialize chunk: {}", e)))?;

                // Get embedding
                let embedding_data: Option<String> = redis::cmd("GET")
                    .arg(self.embedding_key(&chunk_id))
                    .query_async(&mut conn)
                    .await
                    .map_err(|e| AxiomError::Internal(format!("Failed to get embedding: {}", e)))?;

                if let Some(embedding_str) = embedding_data {
                    let embedding: Vec<f32> = serde_json::from_str(&embedding_str)
                        .map_err(|e| AxiomError::Serialization(format!("Failed to deserialize embedding: {}", e)))?;

                    let similarity = VectorOperations::cosine_similarity(query_embedding, &embedding)?;
                    results.push(SearchResult {
                        chunk,
                        score: similarity,
                        distance: None,
                    });
                }
            }
        }

        // Sort by similarity score (descending)
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(limit);
        Ok(results)
    }

    async fn get_chunk(&self, id: &str) -> Result<Option<DocumentChunk>> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AxiomError::Internal(format!("Failed to get Redis connection: {}", e)))?;

        let chunk_data: Option<String> = redis::cmd("GET")
            .arg(self.chunk_key(id))
            .query_async(&mut conn)
            .await
            .map_err(|e| AxiomError::Internal(format!("Failed to get chunk: {}", e)))?;

        if let Some(data) = chunk_data {
            let chunk: DocumentChunk = serde_json::from_str(&data)
                .map_err(|e| AxiomError::Serialization(format!("Failed to deserialize chunk: {}", e)))?;
            Ok(Some(chunk))
        } else {
            Ok(None)
        }
    }

    async fn delete_chunk(&mut self, id: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AxiomError::Internal(format!("Failed to get Redis connection: {}", e)))?;

        // Delete chunk data
        redis::cmd("DEL")
            .arg(self.chunk_key(id))
            .query_async(&mut conn)
            .await
            .map_err(|e| AxiomError::Internal(format!("Failed to delete chunk: {}", e)))?;

        // Delete embedding
        redis::cmd("DEL")
            .arg(self.embedding_key(id))
            .query_async(&mut conn)
            .await
            .map_err(|e| AxiomError::Internal(format!("Failed to delete embedding: {}", e)))?;

        // Remove from index
        redis::cmd("SREM")
            .arg(self.index_key())
            .arg(id)
            .query_async(&mut conn)
            .await
            .map_err(|e| AxiomError::Internal(format!("Failed to remove from index: {}", e)))?;

        Ok(())
    }

    async fn count(&self) -> Result<usize> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AxiomError::Internal(format!("Failed to get Redis connection: {}", e)))?;

        let count: usize = redis::cmd("SCARD")
            .arg(self.index_key())
            .query_async(&mut conn)
            .await
            .map_err(|e| AxiomError::Internal(format!("Failed to get count: {}", e)))?;

        Ok(count)
    }

    async fn clear(&mut self) -> Result<()> {
        let mut conn = self.client.get_async_connection().await
            .map_err(|e| AxiomError::Internal(format!("Failed to get Redis connection: {}", e)))?;

        // Get all keys with the prefix
        let keys: Vec<String> = redis::cmd("KEYS")
            .arg(format!("{}:*", self.key_prefix))
            .query_async(&mut conn)
            .await
            .map_err(|e| AxiomError::Internal(format!("Failed to get keys: {}", e)))?;

        if !keys.is_empty() {
            redis::cmd("DEL")
                .arg(keys)
                .query_async(&mut conn)
                .await
                .map_err(|e| AxiomError::Internal(format!("Failed to delete keys: {}", e)))?;
        }

        Ok(())
    }
}

/// Vector store factory
pub struct VectorStoreFactory;

impl VectorStoreFactory {
    /// Create an in-memory vector store
    pub fn create_in_memory() -> Box<dyn VectorStore> {
        Box::new(InMemoryVectorStore::new())
    }

    /// Create a Redis vector store
    pub fn create_redis(redis_url: &str, key_prefix: String) -> Result<Box<dyn VectorStore>> {
        Ok(Box::new(RedisVectorStore::new(redis_url, key_prefix)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::DocumentChunk;

    #[tokio::test]
    async fn test_in_memory_vector_store() {
        let mut store = InMemoryVectorStore::new();
        
        let chunk = DocumentChunk::new("test", "Hello world", 0, 0, 11)
            .with_embedding(vec![1.0, 0.0, 0.0]);
        
        store.add_chunk(chunk).await.unwrap();
        
        let query_embedding = vec![1.0, 0.0, 0.0];
        let results = store.search(&query_embedding, 10).await.unwrap();
        
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].score, 1.0);
    }

    #[tokio::test]
    async fn test_vector_store_count() {
        let mut store = InMemoryVectorStore::new();
        
        assert_eq!(store.count().await.unwrap(), 0);
        
        let chunk = DocumentChunk::new("test", "Hello world", 0, 0, 11);
        store.add_chunk(chunk).await.unwrap();
        
        assert_eq!(store.count().await.unwrap(), 1);
    }
}
