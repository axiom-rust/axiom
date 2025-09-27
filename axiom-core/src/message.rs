//! Message types for conversation handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Role of a message in a conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageRole {
    /// System message (instructions, context)
    System,
    /// User message (input from user)
    User,
    /// Assistant message (response from AI)
    Assistant,
    /// Tool message (result from tool execution)
    Tool,
}

/// Content of a message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageContent {
    /// Plain text content
    Text(String),
    /// Structured content with multiple parts
    Parts(Vec<MessagePart>),
}

/// Individual part of a message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessagePart {
    /// Text content
    Text(String),
    /// Image content (base64 encoded)
    Image { data: String, mime_type: String },
    /// Tool call
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    /// Tool result
    ToolResult {
        tool_call_id: String,
        content: String,
    },
}

/// A message in a conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    /// Role of the message
    pub role: MessageRole,
    /// Content of the message
    pub content: MessageContent,
    /// Optional metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Timestamp of the message
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Unique identifier for the message
    pub id: uuid::Uuid,
}

impl Message {
    /// Create a new message
    pub fn new(role: MessageRole, content: MessageContent) -> Self {
        Self {
            role,
            content,
            metadata: HashMap::new(),
            timestamp: chrono::Utc::now(),
            id: uuid::Uuid::new_v4(),
        }
    }

    /// Create a system message
    pub fn system(text: impl Into<String>) -> Self {
        Self::new(MessageRole::System, MessageContent::Text(text.into()))
    }

    /// Create a user message
    pub fn user(text: impl Into<String>) -> Self {
        Self::new(MessageRole::User, MessageContent::Text(text.into()))
    }

    /// Create an assistant message
    pub fn assistant(text: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, MessageContent::Text(text.into()))
    }

    /// Create a tool message
    pub fn tool(_tool_call_id: impl Into<String>, content: impl Into<String>) -> Self {
        Self::new(
            MessageRole::Tool,
            MessageContent::Text(content.into()),
        )
    }

    /// Add metadata to the message
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Get the text content if this is a text message
    pub fn text_content(&self) -> Option<&str> {
        match &self.content {
            MessageContent::Text(text) => Some(text),
            MessageContent::Parts(parts) => {
                if parts.len() == 1 {
                    match &parts[0] {
                        MessagePart::Text(text) => Some(text),
                        _ => None,
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Check if this message contains tool calls
    pub fn has_tool_calls(&self) -> bool {
        match &self.content {
            MessageContent::Text(_) => false,
            MessageContent::Parts(parts) => parts.iter().any(|part| matches!(part, MessagePart::ToolCall { .. })),
        }
    }

    /// Extract tool calls from the message
    pub fn tool_calls(&self) -> Vec<&MessagePart> {
        match &self.content {
            MessageContent::Text(_) => vec![],
            MessageContent::Parts(parts) => parts
                .iter()
                .filter(|part| matches!(part, MessagePart::ToolCall { .. }))
                .collect(),
        }
    }
}

/// A conversation is a sequence of messages
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Conversation {
    /// Messages in the conversation
    pub messages: Vec<Message>,
    /// Optional conversation metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Conversation {
    /// Create a new empty conversation
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a message to the conversation
    pub fn add_message(&mut self, message: Message) {
        self.messages.push(message);
    }

    /// Get the last message
    pub fn last_message(&self) -> Option<&Message> {
        self.messages.last()
    }

    /// Get messages by role
    pub fn messages_by_role(&self, role: MessageRole) -> Vec<&Message> {
        self.messages
            .iter()
            .filter(|msg| msg.role == role)
            .collect()
    }

    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Get the total number of messages
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    /// Check if the conversation is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
}
