//! Retrieval engine for RAG

use std::collections::HashMap;

use axiom_core::{Result, AxiomError};
use crate::document::DocumentChunk;
use crate::embeddings::{EmbeddingService, VectorOperations};
use crate::vector_store::{VectorStore, SearchResult};
use crate::reranker::Reranker;

/// Retrieval engine for RAG
pub struct RetrievalEngine {
    /// Vector store
    vector_store: Box<dyn VectorStore>,
    /// Embedding service
    embedding_service: EmbeddingService,
    /// Reranker (optional)
    reranker: Option<Box<dyn Reranker>>,
    /// Retrieval configuration
    config: RetrievalConfig,
}

/// Configuration for retrieval
#[derive(Debug, Clone)]
pub struct RetrievalConfig {
    /// Number of documents to retrieve initially
    pub initial_retrieval_count: usize,
    /// Number of documents to return after reranking
    pub final_retrieval_count: usize,
    /// Similarity threshold
    pub similarity_threshold: f32,
    /// Enable reranking
    pub enable_reranking: bool,
    /// Reranking model
    pub reranking_model: Option<String>,
}

impl Default for RetrievalConfig {
    fn default() -> Self {
        Self {
            initial_retrieval_count: 20,
            final_retrieval_count: 5,
            similarity_threshold: 0.7,
            enable_reranking: false,
            reranking_model: None,
        }
    }
}

impl RetrievalEngine {
    /// Create a new retrieval engine
    pub fn new(
        vector_store: Box<dyn VectorStore>,
        embedding_service: EmbeddingService,
        config: RetrievalConfig,
    ) -> Self {
        Self {
            vector_store,
            embedding_service,
            reranker: None,
            config,
        }
    }

    /// Set a reranker
    pub fn with_reranker(mut self, reranker: Box<dyn Reranker>) -> Self {
        self.reranker = Some(reranker);
        self
    }

    /// Add a document to the retrieval engine
    pub async fn add_document(&mut self, chunks: Vec<DocumentChunk>) -> Result<()> {
        // Generate embeddings for chunks that don't have them
        let mut processed_chunks = Vec::new();
        
        for mut chunk in chunks {
            if chunk.embedding.is_none() {
                let embedding = self.embedding_service.embed(&chunk.content).await?;
                chunk = chunk.with_embedding(embedding);
            }
            processed_chunks.push(chunk);
        }

        // Add to vector store
        self.vector_store.add_chunks(processed_chunks).await?;
        Ok(())
    }

    /// Search for relevant documents
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        // Generate query embedding
        let query_embedding = self.embedding_service.embed(query).await?;

        // Search vector store
        let mut results = self.vector_store.search(
            &query_embedding,
            self.config.initial_retrieval_count,
        ).await?;

        // Filter by similarity threshold
        results.retain(|result| result.score >= self.config.similarity_threshold);

        // Rerank if enabled
        if self.config.enable_reranking {
            if let Some(ref reranker) = self.reranker {
                results = reranker.rerank(query, results).await?;
            }
        }

        // Limit to final count
        results.truncate(self.config.final_retrieval_count);

        Ok(results)
    }

    /// Search with custom parameters
    pub async fn search_with_params(
        &self,
        query: &str,
        limit: Option<usize>,
        threshold: Option<f32>,
    ) -> Result<Vec<SearchResult>> {
        let query_embedding = self.embedding_service.embed(query).await?;
        
        let limit = limit.unwrap_or(self.config.initial_retrieval_count);
        let threshold = threshold.unwrap_or(self.config.similarity_threshold);

        let mut results = self.vector_store.search(&query_embedding, limit).await?;
        results.retain(|result| result.score >= threshold);

        if self.config.enable_reranking {
            if let Some(ref reranker) = self.reranker {
                results = reranker.rerank(query, results).await?;
            }
        }

        Ok(results)
    }

    /// Get a document chunk by ID
    pub async fn get_chunk(&self, id: &str) -> Result<Option<DocumentChunk>> {
        self.vector_store.get_chunk(id).await
    }

    /// Delete a document chunk
    pub async fn delete_chunk(&mut self, id: &str) -> Result<()> {
        self.vector_store.delete_chunk(id).await
    }

    /// Get the total number of chunks
    pub async fn count(&self) -> Result<usize> {
        self.vector_store.count().await
    }

    /// Clear all data
    pub async fn clear(&mut self) -> Result<()> {
        self.vector_store.clear().await
    }

    /// Update retrieval configuration
    pub fn update_config(&mut self, config: RetrievalConfig) {
        self.config = config;
    }

    /// Get current configuration
    pub fn get_config(&self) -> &RetrievalConfig {
        &self.config
    }
}

/// Hybrid retrieval engine that combines multiple retrieval methods
pub struct HybridRetrievalEngine {
    /// Multiple retrieval engines
    engines: Vec<RetrievalEngine>,
    /// Fusion strategy
    fusion_strategy: FusionStrategy,
    /// Configuration
    config: HybridRetrievalConfig,
}

/// Fusion strategy for combining results from multiple engines
#[derive(Debug, Clone)]
pub enum FusionStrategy {
    /// Reciprocal rank fusion
    ReciprocalRankFusion { k: f32 },
    /// Weighted combination
    Weighted { weights: Vec<f32> },
    /// Maximum score
    Maximum,
    /// Average score
    Average,
}

/// Configuration for hybrid retrieval
#[derive(Debug, Clone)]
pub struct HybridRetrievalConfig {
    /// Number of results to retrieve from each engine
    pub results_per_engine: usize,
    /// Final number of results
    pub final_result_count: usize,
    /// Similarity threshold
    pub similarity_threshold: f32,
}

impl Default for HybridRetrievalConfig {
    fn default() -> Self {
        Self {
            results_per_engine: 10,
            final_result_count: 5,
            similarity_threshold: 0.7,
        }
    }
}

impl HybridRetrievalEngine {
    /// Create a new hybrid retrieval engine
    pub fn new(engines: Vec<RetrievalEngine>, fusion_strategy: FusionStrategy) -> Self {
        Self {
            engines,
            fusion_strategy,
            config: HybridRetrievalConfig::default(),
        }
    }

    /// Search using all engines and fuse results
    pub async fn search(&self, query: &str) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();

        // Search each engine
        for engine in &self.engines {
            let results = engine.search_with_params(
                query,
                Some(self.config.results_per_engine),
                Some(self.config.similarity_threshold),
            ).await?;
            all_results.push(results);
        }

        // Fuse results
        let fused_results = self.fuse_results(all_results)?;

        // Limit to final count
        let final_results = fused_results
            .into_iter()
            .take(self.config.final_result_count)
            .collect();

        Ok(final_results)
    }

    /// Fuse results from multiple engines
    fn fuse_results(&self, mut all_results: Vec<Vec<SearchResult>>) -> Result<Vec<SearchResult>> {
        match &self.fusion_strategy {
            FusionStrategy::ReciprocalRankFusion { k } => {
                self.reciprocal_rank_fusion(all_results, *k)
            }
            FusionStrategy::Weighted { weights } => {
                self.weighted_fusion(all_results, weights)
            }
            FusionStrategy::Maximum => {
                self.maximum_fusion(all_results)
            }
            FusionStrategy::Average => {
                self.average_fusion(all_results)
            }
        }
    }

    /// Reciprocal rank fusion
    fn reciprocal_rank_fusion(
        &self,
        all_results: Vec<Vec<SearchResult>>,
        k: f32,
    ) -> Result<Vec<SearchResult>> {
        let mut scores: HashMap<String, f32> = HashMap::new();

        for results in all_results {
            for (rank, result) in results.iter().enumerate() {
                let score = 1.0 / (k + rank as f32);
                *scores.entry(result.chunk.id.clone()).or_insert(0.0) += score;
            }
        }

        let mut fused_results: Vec<SearchResult> = scores
            .into_iter()
            .map(|(chunk_id, score)| {
                // Find the original result to get the chunk
                let mut chunk = None;
                for results in &all_results {
                    for result in results {
                        if result.chunk.id == chunk_id {
                            chunk = Some(result.chunk.clone());
                            break;
                        }
                    }
                    if chunk.is_some() {
                        break;
                    }
                }

                SearchResult {
                    chunk: chunk.unwrap(),
                    score,
                    distance: None,
                }
            })
            .collect();

        fused_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(fused_results)
    }

    /// Weighted fusion
    fn weighted_fusion(
        &self,
        all_results: Vec<Vec<SearchResult>>,
        weights: &[f32],
    ) -> Result<Vec<SearchResult>> {
        if all_results.len() != weights.len() {
            return Err(AxiomError::Validation("Number of engines must match number of weights".to_string()));
        }

        let mut scores: HashMap<String, f32> = HashMap::new();

        for (results, weight) in all_results.iter().zip(weights.iter()) {
            for result in results {
                *scores.entry(result.chunk.id.clone()).or_insert(0.0) += result.score * weight;
            }
        }

        let mut fused_results: Vec<SearchResult> = scores
            .into_iter()
            .map(|(chunk_id, score)| {
                let mut chunk = None;
                for results in &all_results {
                    for result in results {
                        if result.chunk.id == chunk_id {
                            chunk = Some(result.chunk.clone());
                            break;
                        }
                    }
                    if chunk.is_some() {
                        break;
                    }
                }

                SearchResult {
                    chunk: chunk.unwrap(),
                    score,
                    distance: None,
                }
            })
            .collect();

        fused_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(fused_results)
    }

    /// Maximum fusion
    fn maximum_fusion(&self, all_results: Vec<Vec<SearchResult>>) -> Result<Vec<SearchResult>> {
        let mut scores: HashMap<String, f32> = HashMap::new();

        for results in all_results {
            for result in results {
                let entry = scores.entry(result.chunk.id.clone()).or_insert(0.0);
                *entry = entry.max(result.score);
            }
        }

        let mut fused_results: Vec<SearchResult> = scores
            .into_iter()
            .map(|(chunk_id, score)| {
                let mut chunk = None;
                for results in &all_results {
                    for result in results {
                        if result.chunk.id == chunk_id {
                            chunk = Some(result.chunk.clone());
                            break;
                        }
                    }
                    if chunk.is_some() {
                        break;
                    }
                }

                SearchResult {
                    chunk: chunk.unwrap(),
                    score,
                    distance: None,
                }
            })
            .collect();

        fused_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(fused_results)
    }

    /// Average fusion
    fn average_fusion(&self, all_results: Vec<Vec<SearchResult>>) -> Result<Vec<SearchResult>> {
        let mut scores: HashMap<String, (f32, usize)> = HashMap::new();

        for results in all_results {
            for result in results {
                let entry = scores.entry(result.chunk.id.clone()).or_insert((0.0, 0));
                entry.0 += result.score;
                entry.1 += 1;
            }
        }

        let mut fused_results: Vec<SearchResult> = scores
            .into_iter()
            .map(|(chunk_id, (total_score, count))| {
                let score = total_score / count as f32;
                
                let mut chunk = None;
                for results in &all_results {
                    for result in results {
                        if result.chunk.id == chunk_id {
                            chunk = Some(result.chunk.clone());
                            break;
                        }
                    }
                    if chunk.is_some() {
                        break;
                    }
                }

                SearchResult {
                    chunk: chunk.unwrap(),
                    score,
                    distance: None,
                }
            })
            .collect();

        fused_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        Ok(fused_results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector_store::InMemoryVectorStore;
    use crate::embeddings::{EmbeddingService, LocalEmbeddingModel};

    #[tokio::test]
    async fn test_retrieval_engine() {
        let vector_store = Box::new(InMemoryVectorStore::new());
        let embedding_service = EmbeddingService::new("test".to_string())
            .add_model("test".to_string(), Box::new(LocalEmbeddingModel::new("test".to_string())));
        
        let config = RetrievalConfig::default();
        let engine = RetrievalEngine::new(vector_store, embedding_service, config);
        
        // Test search (will return empty results since no documents are added)
        let results = engine.search("test query").await.unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_fusion_strategies() {
        let engines = vec![];
        let strategy = FusionStrategy::ReciprocalRankFusion { k: 60.0 };
        let _hybrid_engine = HybridRetrievalEngine::new(engines, strategy);
    }
}
