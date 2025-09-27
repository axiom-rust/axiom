//! Document processing and management for RAG

use std::collections::HashMap;
use chrono::{DateTime, Utc};

use axiom_core::{Result, AxiomError};

/// A document in the RAG system
#[derive(Debug, Clone)]
pub struct Document {
    /// Unique document ID
    pub id: String,
    /// Document content
    pub content: String,
    /// Document title
    pub title: Option<String>,
    /// Document metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Document chunks
    pub chunks: Vec<DocumentChunk>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Document size in bytes
    pub size_bytes: usize,
    /// Document type
    pub document_type: DocumentType,
}

/// A chunk of a document
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    /// Chunk ID
    pub id: String,
    /// Chunk content
    pub content: String,
    /// Chunk index in the document
    pub index: usize,
    /// Start position in the document
    pub start_pos: usize,
    /// End position in the document
    pub end_pos: usize,
    /// Chunk metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Embedding vector
    pub embedding: Option<Vec<f32>>,
}

/// Type of document
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentType {
    /// Plain text
    Text,
    /// Markdown
    Markdown,
    /// HTML
    Html,
    /// PDF
    Pdf,
    /// Word document
    Word,
    /// JSON
    Json,
    /// CSV
    Csv,
    /// Other
    Other(String),
}

impl Document {
    /// Create a new document
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        let content = content.into();
        let now = Utc::now();
        
        Self {
            id: id.into(),
            content: content.clone(),
            title: None,
            metadata: HashMap::new(),
            chunks: Vec::new(),
            created_at: now,
            updated_at: now,
            size_bytes: content.len(),
            document_type: DocumentType::Text,
        }
    }

    /// Set the document title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the document type
    pub fn with_type(mut self, document_type: DocumentType) -> Self {
        self.document_type = document_type;
        self
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Add a chunk to the document
    pub fn add_chunk(mut self, chunk: DocumentChunk) -> Self {
        self.chunks.push(chunk);
        self
    }

    /// Get the number of chunks
    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    /// Get a chunk by index
    pub fn get_chunk(&self, index: usize) -> Option<&DocumentChunk> {
        self.chunks.get(index)
    }

    /// Get all chunks
    pub fn get_chunks(&self) -> &[DocumentChunk] {
        &self.chunks
    }

    /// Check if the document has embeddings
    pub fn has_embeddings(&self) -> bool {
        self.chunks.iter().any(|chunk| chunk.embedding.is_some())
    }

    /// Get the total number of tokens (approximate)
    pub fn token_count(&self) -> usize {
        // Simple token estimation - in production, use a proper tokenizer
        self.content.split_whitespace().count()
    }
}

impl DocumentChunk {
    /// Create a new document chunk
    pub fn new(
        id: impl Into<String>,
        content: impl Into<String>,
        index: usize,
        start_pos: usize,
        end_pos: usize,
    ) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            index,
            start_pos,
            end_pos,
            metadata: HashMap::new(),
            embedding: None,
        }
    }

    /// Set the embedding for this chunk
    pub fn with_embedding(mut self, embedding: Vec<f32>) -> Self {
        self.embedding = Some(embedding);
        self
    }

    /// Add metadata to the chunk
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Get the chunk size in characters
    pub fn size(&self) -> usize {
        self.content.len()
    }

    /// Get the chunk size in tokens (approximate)
    pub fn token_count(&self) -> usize {
        self.content.split_whitespace().count()
    }

    /// Check if the chunk has an embedding
    pub fn has_embedding(&self) -> bool {
        self.embedding.is_some()
    }
}

/// Document processor for different types
pub struct DocumentProcessor {
    /// Chunk size for text splitting
    chunk_size: usize,
    /// Overlap between chunks
    chunk_overlap: usize,
    /// Minimum chunk size
    min_chunk_size: usize,
}

impl DocumentProcessor {
    /// Create a new document processor
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
            min_chunk_size: chunk_size / 4,
        }
    }

    /// Process a document and create chunks
    pub async fn process_document(&self, mut document: Document) -> Result<Document> {
        match document.document_type {
            DocumentType::Text => self.process_text_document(document).await,
            DocumentType::Markdown => self.process_markdown_document(document).await,
            DocumentType::Html => self.process_html_document(document).await,
            DocumentType::Json => self.process_json_document(document).await,
            DocumentType::Csv => self.process_csv_document(document).await,
            _ => self.process_text_document(document).await,
        }
    }

    /// Process a text document
    async fn process_text_document(&self, mut document: Document) -> Result<Document> {
        let chunks = self.split_text(&document.content)?;
        
        for (i, chunk_content) in chunks.iter().enumerate() {
            let chunk = DocumentChunk::new(
                format!("{}_{}", document.id, i),
                chunk_content.clone(),
                i,
                i * self.chunk_size,
                (i + 1) * self.chunk_size,
            );
            document = document.add_chunk(chunk);
        }

        Ok(document)
    }

    /// Process a markdown document
    async fn process_markdown_document(&self, mut document: Document) -> Result<Document> {
        // Simple markdown processing - in production, use a proper markdown parser
        let content = self.clean_markdown(&document.content);
        let chunks = self.split_text(&content)?;
        
        for (i, chunk_content) in chunks.iter().enumerate() {
            let chunk = DocumentChunk::new(
                format!("{}_{}", document.id, i),
                chunk_content.clone(),
                i,
                i * self.chunk_size,
                (i + 1) * self.chunk_size,
            ).with_metadata("type".to_string(), serde_json::Value::String("markdown".to_string()));
            
            document = document.add_chunk(chunk);
        }

        Ok(document)
    }

    /// Process an HTML document
    async fn process_html_document(&self, mut document: Document) -> Result<Document> {
        // Simple HTML processing - in production, use a proper HTML parser
        let content = self.clean_html(&document.content);
        let chunks = self.split_text(&content)?;
        
        for (i, chunk_content) in chunks.iter().enumerate() {
            let chunk = DocumentChunk::new(
                format!("{}_{}", document.id, i),
                chunk_content.clone(),
                i,
                i * self.chunk_size,
                (i + 1) * self.chunk_size,
            ).with_metadata("type".to_string(), serde_json::Value::String("html".to_string()));
            
            document = document.add_chunk(chunk);
        }

        Ok(document)
    }

    /// Process a JSON document
    async fn process_json_document(&self, mut document: Document) -> Result<Document> {
        // Parse JSON and extract text content
        let json_value: serde_json::Value = serde_json::from_str(&document.content)
            .map_err(|e| AxiomError::Serialization(format!("Invalid JSON: {}", e)))?;
        
        let content = self.extract_text_from_json(&json_value);
        let chunks = self.split_text(&content)?;
        
        for (i, chunk_content) in chunks.iter().enumerate() {
            let chunk = DocumentChunk::new(
                format!("{}_{}", document.id, i),
                chunk_content.clone(),
                i,
                i * self.chunk_size,
                (i + 1) * self.chunk_size,
            ).with_metadata("type".to_string(), serde_json::Value::String("json".to_string()));
            
            document = document.add_chunk(chunk);
        }

        Ok(document)
    }

    /// Process a CSV document
    async fn process_csv_document(&self, mut document: Document) -> Result<Document> {
        // Simple CSV processing - in production, use a proper CSV parser
        let lines: Vec<&str> = document.content.lines().collect();
        let mut content = String::new();
        
        for line in lines {
            content.push_str(line);
            content.push('\n');
        }
        
        let chunks = self.split_text(&content)?;
        
        for (i, chunk_content) in chunks.iter().enumerate() {
            let chunk = DocumentChunk::new(
                format!("{}_{}", document.id, i),
                chunk_content.clone(),
                i,
                i * self.chunk_size,
                (i + 1) * self.chunk_size,
            ).with_metadata("type".to_string(), serde_json::Value::String("csv".to_string()));
            
            document = document.add_chunk(chunk);
        }

        Ok(document)
    }

    /// Split text into chunks
    fn split_text(&self, text: &str) -> Result<Vec<String>> {
        let mut chunks = Vec::new();
        let mut start = 0;
        let mut chunk_index = 0;

        while start < text.len() {
            let end = std::cmp::min(start + self.chunk_size, text.len());
            let chunk = &text[start..end];
            
            // Skip chunks that are too small (except for the last one)
            if chunk.len() >= self.min_chunk_size || end == text.len() {
                chunks.push(chunk.to_string());
                chunk_index += 1;
            }
            
            start = end - self.chunk_overlap;
            if start >= text.len() {
                break;
            }
        }

        Ok(chunks)
    }

    /// Clean markdown content
    fn clean_markdown(&self, content: &str) -> String {
        // Simple markdown cleaning - remove headers, links, etc.
        content
            .lines()
            .map(|line| {
                if line.starts_with('#') {
                    line.trim_start_matches('#').trim()
                } else if line.starts_with('[') && line.contains(']') {
                    // Extract link text
                    if let Some(start) = line.find('[') {
                        if let Some(end) = line.find(']') {
                            &line[start + 1..end]
                        } else {
                            line
                        }
                    } else {
                        line
                    }
                } else {
                    line
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Clean HTML content
    fn clean_html(&self, content: &str) -> String {
        // Simple HTML cleaning - remove tags
        content
            .replace("<[^>]*>", "")
            .replace("&nbsp;", " ")
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
    }

    /// Extract text content from JSON
    fn extract_text_from_json(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(map) => {
                map.values()
                    .map(|v| self.extract_text_from_json(v))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            serde_json::Value::Array(arr) => {
                arr.iter()
                    .map(|v| self.extract_text_from_json(v))
                    .collect::<Vec<_>>()
                    .join(" ")
            }
            _ => value.to_string(),
        }
    }
}

impl Default for DocumentProcessor {
    fn default() -> Self {
        Self::new(1000, 100)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document::new("test", "Hello, world!")
            .with_title("Test Document")
            .with_type(DocumentType::Text);

        assert_eq!(doc.id, "test");
        assert_eq!(doc.content, "Hello, world!");
        assert_eq!(doc.title, Some("Test Document".to_string()));
        assert_eq!(doc.document_type, DocumentType::Text);
    }

    #[test]
    fn test_document_chunk_creation() {
        let chunk = DocumentChunk::new("chunk_1", "Hello", 0, 0, 5)
            .with_metadata("type", serde_json::Value::String("text".to_string()));

        assert_eq!(chunk.id, "chunk_1");
        assert_eq!(chunk.content, "Hello");
        assert_eq!(chunk.index, 0);
        assert_eq!(chunk.start_pos, 0);
        assert_eq!(chunk.end_pos, 5);
    }

    #[tokio::test]
    async fn test_document_processing() {
        let processor = DocumentProcessor::new(100, 20);
        let doc = Document::new("test", "This is a test document with some content that should be split into chunks.");
        
        let processed = processor.process_document(doc).await.unwrap();
        assert!(processed.chunk_count() > 0);
    }
}
