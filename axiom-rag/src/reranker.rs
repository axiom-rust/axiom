//! Reranking models for improving retrieval quality

use std::collections::HashMap;

use axiom_core::{Result, AxiomError};
use crate::vector_store::SearchResult;

/// Trait for reranking models
#[async_trait::async_trait]
pub trait Reranker: Send + Sync {
    /// Rerank a list of search results
    async fn rerank(&self, query: &str, results: Vec<SearchResult>) -> Result<Vec<SearchResult>>;
    
    /// Get the name of the reranker
    fn name(&self) -> &str;
}

/// Simple reranker based on text similarity
pub struct TextSimilarityReranker {
    /// Name of the reranker
    name: String,
    /// Weight for content similarity
    content_weight: f32,
    /// Weight for metadata similarity
    metadata_weight: f32,
}

impl TextSimilarityReranker {
    /// Create a new text similarity reranker
    pub fn new(name: String) -> Self {
        Self {
            name,
            content_weight: 0.8,
            metadata_weight: 0.2,
        }
    }

    /// Set the weights for content and metadata
    pub fn with_weights(mut self, content_weight: f32, metadata_weight: f32) -> Self {
        self.content_weight = content_weight;
        self.metadata_weight = metadata_weight;
        self
    }

    /// Calculate text similarity between query and content
    fn calculate_text_similarity(&self, query: &str, content: &str) -> f32 {
        let query_words: Vec<&str> = query.to_lowercase().split_whitespace().collect();
        let content_words: Vec<&str> = content.to_lowercase().split_whitespace().collect();
        
        let mut matches = 0;
        for query_word in &query_words {
            if content_words.contains(query_word) {
                matches += 1;
            }
        }
        
        if query_words.is_empty() {
            0.0
        } else {
            matches as f32 / query_words.len() as f32
        }
    }

    /// Calculate metadata similarity
    fn calculate_metadata_similarity(&self, query: &str, metadata: &HashMap<String, serde_json::Value>) -> f32 {
        let mut similarity = 0.0;
        let mut count = 0;
        
        for (key, value) in metadata {
            if let Some(value_str) = value.as_str() {
                let key_similarity = self.calculate_text_similarity(query, key);
                let value_similarity = self.calculate_text_similarity(query, value_str);
                similarity += (key_similarity + value_similarity) / 2.0;
                count += 1;
            }
        }
        
        if count > 0 {
            similarity / count as f32
        } else {
            0.0
        }
    }
}

#[async_trait::async_trait]
impl Reranker for TextSimilarityReranker {
    async fn rerank(&self, query: &str, mut results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        for result in &mut results {
            let content_similarity = self.calculate_text_similarity(query, &result.chunk.content);
            let metadata_similarity = self.calculate_metadata_similarity(query, &result.chunk.metadata);
            
            // Combine with original score
            let rerank_score = (content_similarity * self.content_weight) + 
                              (metadata_similarity * self.metadata_weight);
            
            // Blend with original score
            result.score = (result.score * 0.7) + (rerank_score * 0.3);
        }

        // Sort by new score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(results)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// BM25-based reranker
pub struct BM25Reranker {
    /// Name of the reranker
    name: String,
    /// BM25 parameters
    k1: f32,
    b: f32,
    /// Document frequencies
    doc_frequencies: HashMap<String, usize>,
    /// Total number of documents
    total_docs: usize,
    /// Average document length
    avg_doc_length: f32,
}

impl BM25Reranker {
    /// Create a new BM25 reranker
    pub fn new(name: String) -> Self {
        Self {
            name,
            k1: 1.2,
            b: 0.75,
            doc_frequencies: HashMap::new(),
            total_docs: 0,
            avg_doc_length: 0.0,
        }
    }

    /// Set BM25 parameters
    pub fn with_parameters(mut self, k1: f32, b: f32) -> Self {
        self.k1 = k1;
        self.b = b;
        self
    }

    /// Build the index from a collection of documents
    pub fn build_index(&mut self, documents: &[String]) {
        self.doc_frequencies.clear();
        self.total_docs = documents.len();
        
        let mut total_length = 0;
        
        for doc in documents {
            let words = self.tokenize(doc);
            total_length += words.len();
            
            for word in words {
                *self.doc_frequencies.entry(word).or_insert(0) += 1;
            }
        }
        
        self.avg_doc_length = if self.total_docs > 0 {
            total_length as f32 / self.total_docs as f32
        } else {
            0.0
        };
    }

    /// Calculate BM25 score for a document
    fn calculate_bm25_score(&self, query: &str, document: &str) -> f32 {
        let query_terms = self.tokenize(query);
        let doc_terms = self.tokenize(document);
        let doc_length = doc_terms.len() as f32;
        
        let mut score = 0.0;
        
        for term in query_terms {
            let term_freq = doc_terms.iter().filter(|&&t| t == term).count() as f32;
            let doc_freq = self.doc_frequencies.get(&term).copied().unwrap_or(0) as f32;
            
            if doc_freq > 0.0 {
                let idf = ((self.total_docs as f32 - doc_freq + 0.5) / (doc_freq + 0.5)).ln();
                let tf = (term_freq * (self.k1 + 1.0)) / 
                        (term_freq + self.k1 * (1.0 - self.b + self.b * (doc_length / self.avg_doc_length)));
                
                score += idf * tf;
            }
        }
        
        score
    }

    /// Simple tokenization
    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split_whitespace()
            .map(|s| s.to_string())
            .collect()
    }
}

#[async_trait::async_trait]
impl Reranker for BM25Reranker {
    async fn rerank(&self, query: &str, mut results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        for result in &mut results {
            let bm25_score = self.calculate_bm25_score(query, &result.chunk.content);
            
            // Blend with original score
            result.score = (result.score * 0.5) + (bm25_score * 0.5);
        }

        // Sort by new score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(results)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Cross-encoder reranker using a neural model
pub struct CrossEncoderReranker {
    /// Name of the reranker
    name: String,
    /// Model name
    model_name: String,
    /// LLM gateway for inference
    llm_gateway: std::sync::Arc<axiom_llm::LlmGateway>,
}

impl CrossEncoderReranker {
    /// Create a new cross-encoder reranker
    pub fn new(
        name: String,
        model_name: String,
        llm_gateway: std::sync::Arc<axiom_llm::LlmGateway>,
    ) -> Self {
        Self {
            name,
            model_name,
            llm_gateway,
        }
    }

    /// Calculate relevance score using cross-encoder
    async fn calculate_relevance_score(&self, query: &str, content: &str) -> Result<f32> {
        // In a real implementation, you'd use a proper cross-encoder model
        // For now, we'll use a simple heuristic based on text similarity
        
        let query_lower = query.to_lowercase();
        let content_lower = content.to_lowercase();
        
        let query_words: Vec<&str> = query_lower.split_whitespace().collect();
        let content_words: Vec<&str> = content_lower.split_whitespace().collect();
        
        let mut matches = 0;
        for query_word in &query_words {
            if content_words.contains(query_word) {
                matches += 1;
            }
        }
        
        let word_similarity = if query_words.is_empty() {
            0.0
        } else {
            matches as f32 / query_words.len() as f32
        };
        
        // Add some randomness to simulate model uncertainty
        let noise = (rand::random::<f32>() - 0.5) * 0.1;
        Ok((word_similarity + noise).clamp(0.0, 1.0))
    }
}

#[async_trait::async_trait]
impl Reranker for CrossEncoderReranker {
    async fn rerank(&self, query: &str, mut results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        for result in &mut results {
            let relevance_score = self.calculate_relevance_score(query, &result.chunk.content).await?;
            
            // Use the cross-encoder score as the primary score
            result.score = relevance_score;
        }

        // Sort by new score
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(results)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

/// Ensemble reranker that combines multiple rerankers
pub struct EnsembleReranker {
    /// Name of the ensemble
    name: String,
    /// List of rerankers
    rerankers: Vec<Box<dyn Reranker>>,
    /// Weights for each reranker
    weights: Vec<f32>,
}

impl EnsembleReranker {
    /// Create a new ensemble reranker
    pub fn new(name: String) -> Self {
        Self {
            name,
            rerankers: Vec::new(),
            weights: Vec::new(),
        }
    }

    /// Add a reranker to the ensemble
    pub fn add_reranker(mut self, reranker: Box<dyn Reranker>, weight: f32) -> Self {
        self.rerankers.push(reranker);
        self.weights.push(weight);
        self
    }

    /// Normalize weights to sum to 1.0
    fn normalize_weights(&mut self) {
        let total_weight: f32 = self.weights.iter().sum();
        if total_weight > 0.0 {
            for weight in &mut self.weights {
                *weight /= total_weight;
            }
        }
    }
}

#[async_trait::async_trait]
impl Reranker for EnsembleReranker {
    async fn rerank(&self, query: &str, results: Vec<SearchResult>) -> Result<Vec<SearchResult>> {
        if self.rerankers.is_empty() {
            return Ok(results);
        }

        let mut all_scores = Vec::new();
        
        // Get scores from each reranker
        for reranker in &self.rerankers {
            let reranked = reranker.rerank(query, results.clone()).await?;
            let scores: Vec<f32> = reranked.iter().map(|r| r.score).collect();
            all_scores.push(scores);
        }

        // Combine scores using weighted average
        let mut combined_results = results;
        for (i, result) in combined_results.iter_mut().enumerate() {
            let mut combined_score = 0.0;
            for (j, scores) in all_scores.iter().enumerate() {
                if i < scores.len() {
                    combined_score += scores[i] * self.weights[j];
                }
            }
            result.score = combined_score;
        }

        // Sort by combined score
        combined_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        Ok(combined_results)
    }

    fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::DocumentChunk;

    #[tokio::test]
    async fn test_text_similarity_reranker() {
        let reranker = TextSimilarityReranker::new("test".to_string());
        
        let chunk = DocumentChunk::new("test", "Hello world", 0, 0, 11);
        let results = vec![SearchResult {
            chunk,
            score: 0.5,
            distance: None,
        }];
        
        let reranked = reranker.rerank("hello", results).await.unwrap();
        assert_eq!(reranked.len(), 1);
    }

    #[tokio::test]
    async fn test_bm25_reranker() {
        let mut reranker = BM25Reranker::new("test".to_string());
        
        // Build index with sample documents
        let documents = vec![
            "Hello world".to_string(),
            "Hello universe".to_string(),
        ];
        reranker.build_index(&documents);
        
        let chunk = DocumentChunk::new("test", "Hello world", 0, 0, 11);
        let results = vec![SearchResult {
            chunk,
            score: 0.5,
            distance: None,
        }];
        
        let reranked = reranker.rerank("hello", results).await.unwrap();
        assert_eq!(reranked.len(), 1);
    }
}
