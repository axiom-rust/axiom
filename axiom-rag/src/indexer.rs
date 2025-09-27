//! Document indexer for RAG

use std::collections::HashMap;
use std::path::Path;

use axiom_core::{Result, AxiomError};
use crate::document::{Document, DocumentProcessor, DocumentType};
use crate::embeddings::EmbeddingService;
use crate::vector_store::VectorStore;

/// Document indexer for processing and indexing documents
pub struct DocumentIndexer {
    /// Document processor
    processor: DocumentProcessor,
    /// Embedding service
    embedding_service: EmbeddingService,
    /// Vector store
    vector_store: Box<dyn VectorStore>,
    /// Index metadata
    metadata: IndexMetadata,
}

/// Metadata about the index
#[derive(Debug, Clone)]
pub struct IndexMetadata {
    /// Total number of documents
    pub total_documents: usize,
    /// Total number of chunks
    pub total_chunks: usize,
    /// Total size in bytes
    pub total_size_bytes: usize,
    /// Index creation time
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last updated time
    pub last_updated: chrono::DateTime<chrono::Utc>,
    /// Document types
    pub document_types: HashMap<DocumentType, usize>,
}

impl DocumentIndexer {
    /// Create a new document indexer
    pub fn new(
        processor: DocumentProcessor,
        embedding_service: EmbeddingService,
        vector_store: Box<dyn VectorStore>,
    ) -> Self {
        Self {
            processor,
            embedding_service,
            vector_store,
            metadata: IndexMetadata {
                total_documents: 0,
                total_chunks: 0,
                total_size_bytes: 0,
                created_at: chrono::Utc::now(),
                last_updated: chrono::Utc::now(),
                document_types: HashMap::new(),
            },
        }
    }

    /// Index a document
    pub async fn index_document(&mut self, document: Document) -> Result<()> {
        // Process the document
        let processed_document = self.processor.process_document(document).await?;

        // Generate embeddings for chunks
        let mut chunks_with_embeddings = Vec::new();
        for mut chunk in processed_document.chunks {
            if chunk.embedding.is_none() {
                let embedding = self.embedding_service.embed(&chunk.content).await?;
                chunk = chunk.with_embedding(embedding);
            }
            chunks_with_embeddings.push(chunk);
        }

        // Add to vector store
        self.vector_store.add_chunks(chunks_with_embeddings.clone()).await?;

        // Update metadata
        self.update_metadata(&processed_document);

        Ok(())
    }

    /// Index multiple documents
    pub async fn index_documents(&mut self, documents: Vec<Document>) -> Result<()> {
        for document in documents {
            self.index_document(document).await?;
        }
        Ok(())
    }

    /// Index a document from file
    pub async fn index_file(&mut self, file_path: &Path, document_type: DocumentType) -> Result<()> {
        let content = tokio::fs::read_to_string(file_path).await?;
        let document = Document::new(
            file_path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown"),
            content,
        ).with_type(document_type);

        self.index_document(document).await
    }

    /// Index multiple files
    pub async fn index_directory(&mut self, dir_path: &Path) -> Result<()> {
        let mut entries = tokio::fs::read_dir(dir_path).await?;
        
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_file() {
                let document_type = self.detect_document_type(&path);
                if let Err(e) = self.index_file(&path, document_type).await {
                    tracing::warn!("Failed to index file {:?}: {}", path, e);
                }
            }
        }

        Ok(())
    }

    /// Update an existing document
    pub async fn update_document(&mut self, document_id: &str, document: Document) -> Result<()> {
        // Remove existing chunks for this document
        self.remove_document_chunks(document_id).await?;

        // Index the updated document
        self.index_document(document).await
    }

    /// Remove a document from the index
    pub async fn remove_document(&mut self, document_id: &str) -> Result<()> {
        self.remove_document_chunks(document_id).await
    }

    /// Get index metadata
    pub fn get_metadata(&self) -> &IndexMetadata {
        &self.metadata
    }

    /// Get statistics about the index
    pub async fn get_statistics(&self) -> Result<IndexStatistics> {
        let total_chunks = self.vector_store.count().await?;
        
        Ok(IndexStatistics {
            total_documents: self.metadata.total_documents,
            total_chunks,
            total_size_bytes: self.metadata.total_size_bytes,
            average_chunks_per_document: if self.metadata.total_documents > 0 {
                total_chunks as f64 / self.metadata.total_documents as f64
            } else {
                0.0
            },
            document_types: self.metadata.document_types.clone(),
        })
    }

    /// Clear the entire index
    pub async fn clear_index(&mut self) -> Result<()> {
        self.vector_store.clear().await?;
        
        self.metadata = IndexMetadata {
            total_documents: 0,
            total_chunks: 0,
            total_size_bytes: 0,
            created_at: chrono::Utc::now(),
            last_updated: chrono::Utc::now(),
            document_types: HashMap::new(),
        };

        Ok(())
    }

    /// Update metadata after indexing a document
    fn update_metadata(&mut self, document: &Document) {
        self.metadata.total_documents += 1;
        self.metadata.total_chunks += document.chunk_count();
        self.metadata.total_size_bytes += document.size_bytes;
        self.metadata.last_updated = chrono::Utc::now();

        let count = self.metadata.document_types
            .entry(document.document_type.clone())
            .or_insert(0);
        *count += 1;
    }

    /// Remove all chunks for a document
    async fn remove_document_chunks(&mut self, document_id: &str) -> Result<()> {
        // This is a simplified implementation
        // In practice, you'd need to track which chunks belong to which document
        // and remove them accordingly
        Ok(())
    }

    /// Detect document type from file extension
    fn detect_document_type(&self, path: &Path) -> DocumentType {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            match extension.to_lowercase().as_str() {
                "txt" => DocumentType::Text,
                "md" | "markdown" => DocumentType::Markdown,
                "html" | "htm" => DocumentType::Html,
                "pdf" => DocumentType::Pdf,
                "doc" | "docx" => DocumentType::Word,
                "json" => DocumentType::Json,
                "csv" => DocumentType::Csv,
                _ => DocumentType::Other(extension.to_string()),
            }
        } else {
            DocumentType::Text
        }
    }
}

/// Statistics about the index
#[derive(Debug, Clone)]
pub struct IndexStatistics {
    /// Total number of documents
    pub total_documents: usize,
    /// Total number of chunks
    pub total_chunks: usize,
    /// Total size in bytes
    pub total_size_bytes: usize,
    /// Average chunks per document
    pub average_chunks_per_document: f64,
    /// Document types and their counts
    pub document_types: HashMap<DocumentType, usize>,
}

/// Batch indexer for processing large numbers of documents
pub struct BatchIndexer {
    /// Document indexer
    indexer: DocumentIndexer,
    /// Batch size
    batch_size: usize,
    /// Processing queue
    queue: Vec<Document>,
}

impl BatchIndexer {
    /// Create a new batch indexer
    pub fn new(
        indexer: DocumentIndexer,
        batch_size: usize,
    ) -> Self {
        Self {
            indexer,
            batch_size,
            queue: Vec::new(),
        }
    }

    /// Add a document to the processing queue
    pub fn add_document(&mut self, document: Document) {
        self.queue.push(document);
    }

    /// Add multiple documents to the processing queue
    pub fn add_documents(&mut self, documents: Vec<Document>) {
        self.queue.extend(documents);
    }

    /// Process the queue in batches
    pub async fn process_batch(&mut self) -> Result<()> {
        if self.queue.is_empty() {
            return Ok(());
        }

        let batch: Vec<Document> = self.queue
            .drain(0..std::cmp::min(self.batch_size, self.queue.len()))
            .collect();

        self.indexer.index_documents(batch).await
    }

    /// Process all remaining documents
    pub async fn process_all(&mut self) -> Result<()> {
        while !self.queue.is_empty() {
            self.process_batch().await?;
        }
        Ok(())
    }

    /// Get the number of documents in the queue
    pub fn queue_size(&self) -> usize {
        self.queue.len()
    }

    /// Clear the queue
    pub fn clear_queue(&mut self) {
        self.queue.clear();
    }
}

/// Incremental indexer for updating existing indexes
pub struct IncrementalIndexer {
    /// Document indexer
    indexer: DocumentIndexer,
    /// Change tracker
    change_tracker: ChangeTracker,
}

/// Tracks changes to documents
pub struct ChangeTracker {
    /// Document checksums
    checksums: HashMap<String, u64>,
    /// Last modification times
    modification_times: HashMap<String, std::time::SystemTime>,
}

impl ChangeTracker {
    /// Create a new change tracker
    pub fn new() -> Self {
        Self {
            checksums: HashMap::new(),
            modification_times: HashMap::new(),
        }
    }

    /// Check if a document has changed
    pub fn has_changed(&self, document_id: &str, content: &str) -> bool {
        let current_checksum = self.calculate_checksum(content);
        
        if let Some(stored_checksum) = self.checksums.get(document_id) {
            current_checksum != *stored_checksum
        } else {
            true // New document
        }
    }

    /// Update the checksum for a document
    pub fn update_checksum(&mut self, document_id: &str, content: &str) {
        let checksum = self.calculate_checksum(content);
        self.checksums.insert(document_id.to_string(), checksum);
    }

    /// Calculate a simple checksum for content
    fn calculate_checksum(&self, content: &str) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }
}

impl IncrementalIndexer {
    /// Create a new incremental indexer
    pub fn new(indexer: DocumentIndexer) -> Self {
        Self {
            indexer,
            change_tracker: ChangeTracker::new(),
        }
    }

    /// Index a document if it has changed
    pub async fn index_if_changed(&mut self, document: Document) -> Result<bool> {
        if self.change_tracker.has_changed(&document.id, &document.content) {
            self.indexer.index_document(document.clone()).await?;
            self.change_tracker.update_checksum(&document.id, &document.content);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Get the change tracker
    pub fn change_tracker(&self) -> &ChangeTracker {
        &self.change_tracker
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vector_store::InMemoryVectorStore;
    use crate::embeddings::{EmbeddingService, LocalEmbeddingModel};

    #[tokio::test]
    async fn test_document_indexer() {
        let processor = DocumentProcessor::default();
        let embedding_service = EmbeddingService::new("test".to_string())
            .add_model("test".to_string(), Box::new(LocalEmbeddingModel::new("test".to_string())));
        let vector_store = Box::new(InMemoryVectorStore::new());
        
        let mut indexer = DocumentIndexer::new(processor, embedding_service, vector_store);
        
        let document = Document::new("test", "Hello world");
        indexer.index_document(document).await.unwrap();
        
        let stats = indexer.get_statistics().await.unwrap();
        assert_eq!(stats.total_documents, 1);
    }

    #[test]
    fn test_change_tracker() {
        let mut tracker = ChangeTracker::new();
        
        let content1 = "Hello world";
        let content2 = "Hello world";
        let content3 = "Hello universe";
        
        assert!(tracker.has_changed("doc1", content1));
        tracker.update_checksum("doc1", content1);
        
        assert!(!tracker.has_changed("doc1", content2));
        assert!(tracker.has_changed("doc1", content3));
    }
}
