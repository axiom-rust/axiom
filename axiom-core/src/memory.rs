//! Memory system for conversation and context management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

use crate::{Message, Result, AxiomError};

/// A memory item that can be stored and retrieved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    /// Unique identifier for the memory item
    pub id: uuid::Uuid,
    /// Content of the memory
    pub content: String,
    /// Type of memory
    pub memory_type: MemoryType,
    /// Importance score (0.0 to 1.0)
    pub importance: f32,
    /// Timestamp when created
    pub created_at: DateTime<Utc>,
    /// Timestamp when last accessed
    pub last_accessed: DateTime<Utc>,
    /// Access count
    pub access_count: u32,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Type of memory
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryType {
    /// Short-term memory (conversation context)
    ShortTerm,
    /// Long-term memory (persistent knowledge)
    LongTerm,
    /// Working memory (current task context)
    Working,
    /// Episodic memory (specific events)
    Episodic,
    /// Semantic memory (facts and knowledge)
    Semantic,
}

impl MemoryItem {
    /// Create a new memory item
    pub fn new(
        content: impl Into<String>,
        memory_type: MemoryType,
        importance: f32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4(),
            content: content.into(),
            memory_type,
            importance: importance.clamp(0.0, 1.0),
            created_at: now,
            last_accessed: now,
            access_count: 0,
            metadata: HashMap::new(),
        }
    }

    /// Update the last accessed time and increment access count
    pub fn touch(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Add metadata to the memory item
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Calculate a relevance score for a query
    pub fn relevance_score(&self, query: &str) -> f32 {
        // Simple relevance calculation based on content similarity
        // In a real implementation, you'd use more sophisticated methods
        let content_lower = self.content.to_lowercase();
        let query_lower = query.to_lowercase();
        
        let common_words = content_lower
            .split_whitespace()
            .filter(|word| query_lower.contains(word))
            .count();
        
        let total_words = content_lower.split_whitespace().count();
        
        if total_words == 0 {
            0.0
        } else {
            (common_words as f32 / total_words as f32) * self.importance
        }
    }
}

/// A memory system for storing and retrieving information
#[async_trait::async_trait]
pub trait Memory: Send + Sync {
    /// Store a memory item
    async fn store(&mut self, item: MemoryItem) -> Result<()>;

    /// Retrieve memory items by query
    async fn retrieve(&mut self, query: &str, limit: Option<usize>) -> Result<Vec<MemoryItem>>;

    /// Retrieve memory items by type
    async fn retrieve_by_type(
        &mut self,
        memory_type: MemoryType,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>>;

    /// Update a memory item
    async fn update(&mut self, id: uuid::Uuid, item: MemoryItem) -> Result<()>;

    /// Delete a memory item
    async fn delete(&mut self, id: uuid::Uuid) -> Result<()>;

    /// Clear all memory
    async fn clear(&mut self) -> Result<()>;

    /// Get memory statistics
    async fn stats(&self) -> Result<MemoryStats>;
}

/// Statistics about memory usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of memory items
    pub total_items: usize,
    /// Items by type
    pub items_by_type: HashMap<MemoryType, usize>,
    /// Total memory size in bytes
    pub total_size_bytes: usize,
    /// Average importance score
    pub avg_importance: f32,
}

/// In-memory implementation of the Memory trait
pub struct InMemoryMemory {
    items: HashMap<uuid::Uuid, MemoryItem>,
}

impl InMemoryMemory {
    /// Create a new in-memory memory system
    pub fn new() -> Self {
        Self {
            items: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl Memory for InMemoryMemory {
    async fn store(&mut self, item: MemoryItem) -> Result<()> {
        self.items.insert(item.id, item);
        Ok(())
    }

    async fn retrieve(&mut self, query: &str, limit: Option<usize>) -> Result<Vec<MemoryItem>> {
        let mut items: Vec<MemoryItem> = self.items.values().cloned().collect();
        
        // Sort by relevance score
        items.sort_by(|a, b| {
            let score_a = a.relevance_score(query);
            let score_b = b.relevance_score(query);
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        // Apply limit
        if let Some(limit) = limit {
            items.truncate(limit);
        }

        // Update access counts
        for item in &items {
            if let Some(stored_item) = self.items.get_mut(&item.id) {
                stored_item.touch();
            }
        }

        Ok(items)
    }

    async fn retrieve_by_type(
        &mut self,
        memory_type: MemoryType,
        limit: Option<usize>,
    ) -> Result<Vec<MemoryItem>> {
        let mut items: Vec<MemoryItem> = self
            .items
            .values()
            .filter(|item| item.memory_type == memory_type)
            .cloned()
            .collect();

        // Sort by importance and last accessed
        items.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then(b.last_accessed.cmp(&a.last_accessed))
        });

        if let Some(limit) = limit {
            items.truncate(limit);
        }

        // Update access counts
        for item in &items {
            if let Some(stored_item) = self.items.get_mut(&item.id) {
                stored_item.touch();
            }
        }

        Ok(items)
    }

    async fn update(&mut self, id: uuid::Uuid, item: MemoryItem) -> Result<()> {
        if self.items.contains_key(&id) {
            self.items.insert(id, item);
            Ok(())
        } else {
            Err(AxiomError::Memory(format!("Memory item not found: {}", id)))
        }
    }

    async fn delete(&mut self, id: uuid::Uuid) -> Result<()> {
        if self.items.remove(&id).is_some() {
            Ok(())
        } else {
            Err(AxiomError::Memory(format!("Memory item not found: {}", id)))
        }
    }

    async fn clear(&mut self) -> Result<()> {
        self.items.clear();
        Ok(())
    }

    async fn stats(&self) -> Result<MemoryStats> {
        let total_items = self.items.len();
        let mut items_by_type = HashMap::new();
        let mut total_importance = 0.0;
        let mut total_size = 0;

        for item in self.items.values() {
            *items_by_type.entry(item.memory_type.clone()).or_insert(0) += 1;
            total_importance += item.importance;
            total_size += item.content.len();
        }

        let avg_importance = if total_items > 0 {
            total_importance / total_items as f32
        } else {
            0.0
        };

        Ok(MemoryStats {
            total_items,
            items_by_type,
            total_size_bytes: total_size,
            avg_importance,
        })
    }
}

impl Default for InMemoryMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// A conversation memory that manages message history
pub struct ConversationMemory {
    memory: Box<dyn Memory>,
    max_messages: Option<usize>,
}

impl ConversationMemory {
    /// Create a new conversation memory
    pub fn new(memory: Box<dyn Memory>, max_messages: Option<usize>) -> Self {
        Self {
            memory,
            max_messages,
        }
    }

    /// Add a message to the conversation memory
    pub async fn add_message(&mut self, message: Message) -> Result<()> {
        let memory_item = MemoryItem::new(
            message.text_content().unwrap_or("").to_string(),
            MemoryType::ShortTerm,
            0.5, // Default importance for conversation messages
        );

        self.memory.store(memory_item).await?;

        // Trim messages if we exceed the limit
        if let Some(max) = self.max_messages {
            let stats = self.memory.stats().await?;
            if stats.total_items > max {
                // In a real implementation, you'd implement a more sophisticated trimming strategy
                // For now, we'll just clear old short-term memories
                let old_memories = self.memory.retrieve_by_type(MemoryType::ShortTerm, Some(max)).await?;
                // Keep only the most recent ones
                for (i, item) in old_memories.iter().enumerate() {
                    if i >= max {
                        self.memory.delete(item.id).await?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Get relevant context for a query
    pub async fn get_context(&mut self, query: &str, limit: usize) -> Result<Vec<MemoryItem>> {
        self.memory.retrieve(query, Some(limit)).await
    }

    /// Clear the conversation memory
    pub async fn clear(&mut self) -> Result<()> {
        self.memory.clear().await
    }
}
