//! Streaming response types and utilities

use serde::{Deserialize, Serialize};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio_stream::Stream;
use crate::AxiomError;

/// A chunk of streaming response data
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamChunk {
    /// Content of the chunk
    pub content: String,
    /// Type of the chunk
    pub chunk_type: ChunkType,
    /// Optional metadata
    pub metadata: Option<serde_json::Value>,
    /// Whether this is the final chunk
    pub is_final: bool,
}

/// Type of streaming chunk
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChunkType {
    /// Text content
    Text,
    /// Tool call
    ToolCall,
    /// Tool result
    ToolResult,
    /// Error
    Error,
    /// Metadata
    Metadata,
}

/// A streaming response that yields chunks of data
pub struct StreamResponse {
    inner: Pin<Box<dyn Stream<Item = Result<StreamChunk, AxiomError>> + Send + 'static>>,
}

impl StreamResponse {
    /// Create a new streaming response
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Result<StreamChunk, AxiomError>> + Send + 'static,
    {
        Self {
            inner: Box::pin(stream),
        }
    }

    /// Collect all chunks into a single string
    pub async fn collect_text(self) -> Result<String, AxiomError> {
        let mut result = String::new();
        let mut stream = self.inner;

        while let Some(chunk) = tokio_stream::StreamExt::next(&mut stream).await {
            let chunk = chunk?;
            if matches!(chunk.chunk_type, ChunkType::Text) {
                result.push_str(&chunk.content);
            }
        }

        Ok(result)
    }

    /// Collect all chunks into a vector
    pub async fn collect_all(self) -> Result<Vec<StreamChunk>, AxiomError> {
        let mut result = Vec::new();
        let mut stream = self.inner;

        while let Some(chunk) = tokio_stream::StreamExt::next(&mut stream).await {
            result.push(chunk?);
        }

        Ok(result)
    }

    /// Filter chunks by type
    pub fn filter_by_type(self, chunk_type: ChunkType) -> impl Stream<Item = Result<StreamChunk, AxiomError>> {
        tokio_stream::StreamExt::filter(self.inner, move |chunk| {
            match chunk {
                Ok(chunk) => chunk.chunk_type == chunk_type,
                Err(_) => true, // Always pass through errors
            }
        })
    }
}

impl Stream for StreamResponse {
    type Item = Result<StreamChunk, AxiomError>;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

/// Builder for creating streaming responses
pub struct StreamBuilder {
    chunks: Vec<StreamChunk>,
}

impl StreamBuilder {
    /// Create a new stream builder
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    /// Add a text chunk
    pub fn add_text(mut self, content: impl Into<String>) -> Self {
        self.chunks.push(StreamChunk {
            content: content.into(),
            chunk_type: ChunkType::Text,
            metadata: None,
            is_final: false,
        });
        self
    }

    /// Add a tool call chunk
    pub fn add_tool_call(mut self, content: impl Into<String>) -> Self {
        self.chunks.push(StreamChunk {
            content: content.into(),
            chunk_type: ChunkType::ToolCall,
            metadata: None,
            is_final: false,
        });
        self
    }

    /// Add a metadata chunk
    pub fn add_metadata(mut self, content: impl Into<String>, metadata: serde_json::Value) -> Self {
        self.chunks.push(StreamChunk {
            content: content.into(),
            chunk_type: ChunkType::Metadata,
            metadata: Some(metadata),
            is_final: false,
        });
        self
    }

    /// Mark the last chunk as final
    pub fn finalize(mut self) -> Self {
        if let Some(last_chunk) = self.chunks.last_mut() {
            last_chunk.is_final = true;
        }
        self
    }

    /// Build the streaming response
    pub fn build(self) -> StreamResponse {
        let chunks = self.chunks;
        let stream = tokio_stream::iter(chunks.into_iter().map(Ok));
        StreamResponse::new(stream)
    }
}

impl Default for StreamBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for working with streams
pub mod utils {
    use super::*;
    use futures::StreamExt;

    /// Merge multiple streams into one
    pub fn merge_streams(
        streams: Vec<StreamResponse>,
    ) -> impl Stream<Item = Result<StreamChunk, AxiomError>> {
        let streams: Vec<_> = streams.into_iter().map(|s| s.inner).collect();
        futures::stream::select_all(streams)
    }

    /// Buffer chunks and emit them in batches
    pub fn buffer_chunks(
        stream: StreamResponse,
        buffer_size: usize,
    ) -> impl Stream<Item = Result<Vec<StreamChunk>, AxiomError>> {
        stream
            .inner
            .chunks(buffer_size)
            .map(|chunk_batch| {
                chunk_batch
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
            })
    }
}
