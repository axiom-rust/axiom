//! RAG functionality exposed to Python

use pyo3::prelude::*;
use pyo3::types::PyDict;
use std::collections::HashMap;

use axiom_rag::{
    Document as RustDocument, VectorStore as RustVectorStore,
    RetrievalEngine as RustRetrievalEngine, DocumentProcessor as RustDocumentProcessor,
    EmbeddingService as RustEmbeddingService, VectorStoreFactory as RustVectorStoreFactory
};

/// Python wrapper for Document
#[pyclass(name = "Document")]
pub struct Document {
    inner: RustDocument,
}

#[pymethods]
impl Document {
    /// Create a new document
    #[new]
    fn new(id: &str, content: &str) -> Self {
        Self {
            inner: RustDocument::new(id, content),
        }
    }

    /// Set the document title
    fn with_title(mut self_: PyRef<Self>, title: &str) -> PyRef<Self> {
        self_.inner = self_.inner.with_title(title);
        self_
    }

    /// Set the document metadata
    fn with_metadata(mut self_: PyRef<Self>, metadata: HashMap<String, String>) -> PyRef<Self> {
        let json_metadata: HashMap<String, serde_json::Value> = metadata
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::String(v)))
            .collect();
        self_.inner = self_.inner.with_metadata(json_metadata);
        self_
    }

    /// Get the document ID
    fn id(&self) -> String {
        self.inner.id().to_string()
    }

    /// Get the document content
    fn content(&self) -> String {
        self.inner.content().to_string()
    }

    /// Get the document title
    fn title(&self) -> Option<String> {
        self.inner.title().map(|t| t.to_string())
    }

    /// Get the document metadata
    fn metadata(&self) -> HashMap<String, String> {
        self.inner
            .metadata()
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()
    }

    /// Get document chunks
    fn chunks(&self) -> Vec<DocumentChunk> {
        self.inner
            .get_chunks()
            .iter()
            .map(|chunk| chunk.clone().into())
            .collect()
    }

    /// Process the document
    fn process(&mut self, processor: DocumentProcessor) -> PyResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        rt.block_on(async {
            self.inner = processor.into().process_document(self.inner.clone()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(())
    }
}

impl From<RustDocument> for Document {
    fn from(document: RustDocument) -> Self {
        Self { inner: document }
    }
}

impl From<Document> for RustDocument {
    fn from(document: Document) -> Self {
        document.inner
    }
}

/// Python wrapper for DocumentChunk
#[pyclass(name = "DocumentChunk")]
pub struct DocumentChunk {
    inner: axiom_rag::DocumentChunk,
}

#[pymethods]
impl DocumentChunk {
    /// Get chunk content
    fn content(&self) -> String {
        self.inner.content.clone()
    }

    /// Get chunk metadata
    fn metadata(&self) -> HashMap<String, String> {
        self.inner
            .metadata
            .iter()
            .map(|(k, v)| (k.clone(), v.to_string()))
            .collect()
    }

    /// Get chunk embedding
    fn embedding(&self) -> Option<Vec<f32>> {
        self.inner.embedding.clone()
    }

    /// Get chunk score
    fn score(&self) -> f32 {
        self.inner.score
    }
}

impl From<axiom_rag::DocumentChunk> for DocumentChunk {
    fn from(chunk: axiom_rag::DocumentChunk) -> Self {
        Self { inner: chunk }
    }
}

/// Python wrapper for VectorStore
#[pyclass(name = "VectorStore")]
pub struct VectorStore {
    inner: Box<dyn RustVectorStore>,
}

#[pymethods]
impl VectorStore {
    /// Add a document to the vector store
    fn add_document(&mut self, document: Document) -> PyResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        rt.block_on(async {
            self.inner.add_document(document.into()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(())
    }

    /// Search for similar documents
    fn search(&self, query: &str, limit: Option<usize>) -> PyResult<Vec<SearchResult>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let results = rt.block_on(async {
            self.inner.search(query, limit).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    /// Get the number of documents in the store
    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the store is empty
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Clear all documents
    fn clear(&mut self) -> PyResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        rt.block_on(async {
            self.inner.clear().await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(())
    }
}

impl From<Box<dyn RustVectorStore>> for VectorStore {
    fn from(store: Box<dyn RustVectorStore>) -> Self {
        Self { inner: store }
    }
}

/// Python wrapper for SearchResult
#[pyclass(name = "SearchResult")]
pub struct SearchResult {
    inner: axiom_rag::SearchResult,
}

#[pymethods]
impl SearchResult {
    /// Get the document chunk
    fn chunk(&self) -> DocumentChunk {
        self.inner.chunk.clone().into()
    }

    /// Get the similarity score
    fn score(&self) -> f32 {
        self.inner.score
    }

    /// Get the document ID
    fn document_id(&self) -> String {
        self.inner.document_id.clone()
    }
}

impl From<axiom_rag::SearchResult> for SearchResult {
    fn from(result: axiom_rag::SearchResult) -> Self {
        Self { inner: result }
    }
}

/// Python wrapper for RetrievalEngine
#[pyclass(name = "RetrievalEngine")]
pub struct RetrievalEngine {
    inner: RustRetrievalEngine,
}

#[pymethods]
impl RetrievalEngine {
    /// Create a new retrieval engine
    #[new]
    fn new(vector_store: VectorStore, embedding_service: EmbeddingService, config: RetrievalConfig) -> Self {
        Self {
            inner: RustRetrievalEngine::new(
                vector_store.into(),
                embedding_service.into(),
                config.into(),
            ),
        }
    }

    /// Add a document to the retrieval engine
    fn add_document(&mut self, document: Document) -> PyResult<()> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        rt.block_on(async {
            self.inner.add_document(document.into()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(())
    }

    /// Search for relevant documents
    fn search(&self, query: &str, limit: Option<usize>) -> PyResult<Vec<SearchResult>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let results = rt.block_on(async {
            self.inner.search(query, limit).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(results.into_iter().map(|r| r.into()).collect())
    }

    /// Get the number of documents
    fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if the engine is empty
    fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl From<RustRetrievalEngine> for RetrievalEngine {
    fn from(engine: RustRetrievalEngine) -> Self {
        Self { inner: engine }
    }
}

impl From<RetrievalEngine> for RustRetrievalEngine {
    fn from(engine: RetrievalEngine) -> Self {
        engine.inner
    }
}

/// Python wrapper for DocumentProcessor
#[pyclass(name = "DocumentProcessor")]
pub struct DocumentProcessor {
    inner: RustDocumentProcessor,
}

#[pymethods]
impl DocumentProcessor {
    /// Create a new document processor
    #[new]
    fn new() -> Self {
        Self {
            inner: RustDocumentProcessor::default(),
        }
    }

    /// Process a document
    fn process_document(&self, document: Document) -> PyResult<Document> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let processed = rt.block_on(async {
            self.inner.process_document(document.into()).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(processed.into())
    }
}

impl From<RustDocumentProcessor> for DocumentProcessor {
    fn from(processor: RustDocumentProcessor) -> Self {
        Self { inner: processor }
    }
}

impl From<DocumentProcessor> for RustDocumentProcessor {
    fn from(processor: DocumentProcessor) -> Self {
        processor.inner
    }
}

/// Python wrapper for EmbeddingService
#[pyclass(name = "EmbeddingService")]
pub struct EmbeddingService {
    inner: RustEmbeddingService,
}

#[pymethods]
impl EmbeddingService {
    /// Create a new embedding service
    #[new]
    fn new(default_model: String) -> Self {
        Self {
            inner: RustEmbeddingService::new(default_model),
        }
    }

    /// Add an embedding model
    fn add_model(&mut self, name: String, model: Box<dyn EmbeddingModel>) -> PyResult<()> {
        // This would need to be implemented based on the actual EmbeddingService API
        Ok(())
    }

    /// Generate embeddings for text
    fn embed(&self, text: &str) -> PyResult<Vec<f32>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let embeddings = rt.block_on(async {
            self.inner.embed(text).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(embeddings)
    }

    /// Generate embeddings for multiple texts
    fn embed_batch(&self, texts: Vec<String>) -> PyResult<Vec<Vec<f32>>> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let embeddings = rt.block_on(async {
            self.inner.embed_batch(texts).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(embeddings)
    }
}

impl From<RustEmbeddingService> for EmbeddingService {
    fn from(service: RustEmbeddingService) -> Self {
        Self { inner: service }
    }
}

impl From<EmbeddingService> for RustEmbeddingService {
    fn from(service: EmbeddingService) -> Self {
        service.inner
    }
}

/// Python wrapper for VectorStoreFactory
#[pyclass(name = "VectorStoreFactory")]
pub struct VectorStoreFactory;

#[pymethods]
impl VectorStoreFactory {
    /// Create an in-memory vector store
    #[staticmethod]
    fn create_in_memory() -> VectorStore {
        VectorStore {
            inner: RustVectorStoreFactory::create_in_memory(),
        }
    }

    /// Create a Redis vector store
    #[staticmethod]
    fn create_redis(redis_url: &str) -> PyResult<VectorStore> {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        let store = rt.block_on(async {
            RustVectorStoreFactory::create_redis(redis_url).await
        }).map_err(|e| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(e.to_string()))?;
        
        Ok(VectorStore { inner: store })
    }
}

/// Python wrapper for RetrievalConfig
#[pyclass(name = "RetrievalConfig")]
pub struct RetrievalConfig {
    inner: axiom_rag::RetrievalConfig,
}

#[pymethods]
impl RetrievalConfig {
    /// Create a new retrieval configuration
    #[new]
    fn new() -> Self {
        Self {
            inner: axiom_rag::RetrievalConfig::default(),
        }
    }

    /// Set the similarity threshold
    fn with_similarity_threshold(mut self_: PyRef<Self>, threshold: f32) -> PyRef<Self> {
        self_.inner = self_.inner.with_similarity_threshold(threshold);
        self_
    }

    /// Set the maximum results
    fn with_max_results(mut self_: PyRef<Self>, max: usize) -> PyRef<Self> {
        self_.inner = self_.inner.with_max_results(max);
        self_
    }

    /// Set the chunk size
    fn with_chunk_size(mut self_: PyRef<Self>, size: usize) -> PyRef<Self> {
        self_.inner = self_.inner.with_chunk_size(size);
        self_
    }

    /// Set the chunk overlap
    fn with_chunk_overlap(mut self_: PyRef<Self>, overlap: usize) -> PyRef<Self> {
        self_.inner = self_.inner.with_chunk_overlap(overlap);
        self_
    }
}

impl From<axiom_rag::RetrievalConfig> for RetrievalConfig {
    fn from(config: axiom_rag::RetrievalConfig) -> Self {
        Self { inner: config }
    }
}

impl From<RetrievalConfig> for axiom_rag::RetrievalConfig {
    fn from(config: RetrievalConfig) -> Self {
        config.inner
    }
}

// Trait for embedding models
pub trait EmbeddingModel: Send + Sync {
    fn name(&self) -> &str;
    fn dimensions(&self) -> usize;
    fn embed(&self, text: &str) -> PyResult<Vec<f32>>;
    fn embed_batch(&self, texts: Vec<String>) -> PyResult<Vec<Vec<f32>>>;
}
