//! Embedding models and vector operations for RAG

use std::collections::HashMap;

use axiom_core::{Result, AxiomError};
use axiom_llm::LlmGateway;

/// Trait for embedding models
#[async_trait::async_trait]
pub trait EmbeddingModel: Send + Sync {
    /// Get the model name
    fn name(&self) -> &str;

    /// Get the embedding dimension
    fn dimension(&self) -> usize;

    /// Generate embeddings for text
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// Get the maximum input length
    fn max_input_length(&self) -> usize;
}

/// OpenAI embedding model
pub struct OpenAIEmbeddingModel {
    /// LLM gateway
    llm_gateway: std::sync::Arc<LlmGateway>,
    /// Model name
    model_name: String,
    /// Embedding dimension
    dimension: usize,
}

impl OpenAIEmbeddingModel {
    /// Create a new OpenAI embedding model
    pub fn new(llm_gateway: std::sync::Arc<LlmGateway>, model_name: String) -> Self {
        let dimension = match model_name.as_str() {
            "text-embedding-ada-002" => 1536,
            "text-embedding-3-small" => 1536,
            "text-embedding-3-large" => 3072,
            _ => 1536, // Default
        };

        Self {
            llm_gateway,
            model_name,
            dimension,
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingModel for OpenAIEmbeddingModel {
    fn name(&self) -> &str {
        &self.model_name
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // In a real implementation, you'd call the OpenAI embeddings API
        // For now, we'll return a mock embedding
        Ok(vec![0.0; self.dimension])
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for text in texts {
            let embedding = self.embed(text).await?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    fn max_input_length(&self) -> usize {
        8192 // OpenAI's limit
    }
}

/// Local embedding model using sentence transformers
pub struct LocalEmbeddingModel {
    /// Model name
    model_name: String,
    /// Embedding dimension
    dimension: usize,
    /// Maximum input length
    max_input_length: usize,
}

impl LocalEmbeddingModel {
    /// Create a new local embedding model
    pub fn new(model_name: String) -> Self {
        let dimension = match model_name.as_str() {
            "all-MiniLM-L6-v2" => 384,
            "all-mpnet-base-v2" => 768,
            "all-distilroberta-v1" => 768,
            _ => 384, // Default
        };

        Self {
            model_name,
            dimension,
            max_input_length: 512,
        }
    }
}

#[async_trait::async_trait]
impl EmbeddingModel for LocalEmbeddingModel {
    fn name(&self) -> &str {
        &self.model_name
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // In a real implementation, you'd use a local model like sentence-transformers
        // For now, we'll return a mock embedding
        Ok(vec![0.0; self.dimension])
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for text in texts {
            let embedding = self.embed(text).await?;
            embeddings.push(embedding);
        }
        Ok(embeddings)
    }

    fn max_input_length(&self) -> usize {
        self.max_input_length
    }
}

/// Embedding service for managing multiple models
pub struct EmbeddingService {
    /// Available models
    models: HashMap<String, Box<dyn EmbeddingModel>>,
    /// Default model
    default_model: String,
}

impl EmbeddingService {
    /// Create a new embedding service
    pub fn new(default_model: String) -> Self {
        Self {
            models: HashMap::new(),
            default_model,
        }
    }

    /// Add a model to the service
    pub fn add_model(mut self, name: String, model: Box<dyn EmbeddingModel>) -> Self {
        self.models.insert(name, model);
        self
    }

    /// Get a model by name
    pub fn get_model(&self, name: &str) -> Option<&dyn EmbeddingModel> {
        self.models.get(name).map(|m| m.as_ref())
    }

    /// Get the default model
    pub fn get_default_model(&self) -> Option<&dyn EmbeddingModel> {
        self.get_model(&self.default_model)
    }

    /// Generate embeddings using the default model
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let model = self.get_default_model()
            .ok_or_else(|| AxiomError::Internal("No default model available".to_string()))?;
        model.embed(text).await
    }

    /// Generate embeddings using a specific model
    pub async fn embed_with_model(&self, model_name: &str, text: &str) -> Result<Vec<f32>> {
        let model = self.get_model(model_name)
            .ok_or_else(|| AxiomError::Internal(format!("Model not found: {}", model_name)))?;
        model.embed(text).await
    }

    /// Generate embeddings for multiple texts using the default model
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let model = self.get_default_model()
            .ok_or_else(|| AxiomError::Internal("No default model available".to_string()))?;
        model.embed_batch(texts).await
    }

    /// Generate embeddings for multiple texts using a specific model
    pub async fn embed_batch_with_model(&self, model_name: &str, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let model = self.get_model(model_name)
            .ok_or_else(|| AxiomError::Internal(format!("Model not found: {}", model_name)))?;
        model.embed_batch(texts).await
    }

    /// List available models
    pub fn list_models(&self) -> Vec<&String> {
        self.models.keys().collect()
    }
}

/// Vector operations for embeddings
pub struct VectorOperations;

impl VectorOperations {
    /// Calculate cosine similarity between two vectors
    pub fn cosine_similarity(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AxiomError::Validation("Vector dimensions must match".to_string()));
        }

        let dot_product: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm_a == 0.0 || norm_b == 0.0 {
            return Ok(0.0);
        }

        Ok(dot_product / (norm_a * norm_b))
    }

    /// Calculate Euclidean distance between two vectors
    pub fn euclidean_distance(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AxiomError::Validation("Vector dimensions must match".to_string()));
        }

        let distance_squared: f32 = a.iter()
            .zip(b.iter())
            .map(|(x, y)| (x - y).powi(2))
            .sum();

        Ok(distance_squared.sqrt())
    }

    /// Calculate dot product between two vectors
    pub fn dot_product(a: &[f32], b: &[f32]) -> Result<f32> {
        if a.len() != b.len() {
            return Err(AxiomError::Validation("Vector dimensions must match".to_string()));
        }

        Ok(a.iter().zip(b.iter()).map(|(x, y)| x * y).sum())
    }

    /// Normalize a vector to unit length
    pub fn normalize(vector: &mut [f32]) -> Result<()> {
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        
        if norm == 0.0 {
            return Err(AxiomError::Validation("Cannot normalize zero vector".to_string()));
        }

        for x in vector.iter_mut() {
            *x /= norm;
        }

        Ok(())
    }

    /// Calculate the magnitude of a vector
    pub fn magnitude(vector: &[f32]) -> f32 {
        vector.iter().map(|x| x * x).sum::<f32>().sqrt()
    }

    /// Add two vectors
    pub fn add(a: &[f32], b: &[f32]) -> Result<Vec<f32>> {
        if a.len() != b.len() {
            return Err(AxiomError::Validation("Vector dimensions must match".to_string()));
        }

        Ok(a.iter().zip(b.iter()).map(|(x, y)| x + y).collect())
    }

    /// Subtract two vectors
    pub fn subtract(a: &[f32], b: &[f32]) -> Result<Vec<f32>> {
        if a.len() != b.len() {
            return Err(AxiomError::Validation("Vector dimensions must match".to_string()));
        }

        Ok(a.iter().zip(b.iter()).map(|(x, y)| x - y).collect())
    }

    /// Scale a vector by a scalar
    pub fn scale(vector: &[f32], scalar: f32) -> Vec<f32> {
        vector.iter().map(|x| x * scalar).collect()
    }
}

/// Embedding cache for storing computed embeddings
pub struct EmbeddingCache {
    /// Cache storage
    cache: HashMap<String, Vec<f32>>,
    /// Maximum cache size
    max_size: usize,
}

impl EmbeddingCache {
    /// Create a new embedding cache
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    /// Get an embedding from cache
    pub fn get(&self, key: &str) -> Option<&Vec<f32>> {
        self.cache.get(key)
    }

    /// Store an embedding in cache
    pub fn put(&mut self, key: String, embedding: Vec<f32>) {
        if self.cache.len() >= self.max_size {
            // Remove oldest entry (simple LRU)
            if let Some(oldest_key) = self.cache.keys().next().cloned() {
                self.cache.remove(&oldest_key);
            }
        }
        self.cache.insert(key, embedding);
    }

    /// Check if cache contains a key
    pub fn contains(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn size(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        let similarity = VectorOperations::cosine_similarity(&a, &b).unwrap();
        assert_eq!(similarity, 0.0);

        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let similarity = VectorOperations::cosine_similarity(&a, &b).unwrap();
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_euclidean_distance() {
        let a = vec![0.0, 0.0];
        let b = vec![3.0, 4.0];
        let distance = VectorOperations::euclidean_distance(&a, &b).unwrap();
        assert_eq!(distance, 5.0);
    }

    #[test]
    fn test_vector_normalization() {
        let mut vector = vec![3.0, 4.0];
        VectorOperations::normalize(&mut vector).unwrap();
        let magnitude = VectorOperations::magnitude(&vector);
        assert!((magnitude - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_embedding_cache() {
        let mut cache = EmbeddingCache::new(2);
        cache.put("key1".to_string(), vec![1.0, 2.0, 3.0]);
        cache.put("key2".to_string(), vec![4.0, 5.0, 6.0]);
        
        assert!(cache.contains("key1"));
        assert!(cache.contains("key2"));
        assert_eq!(cache.size(), 2);
    }
}
