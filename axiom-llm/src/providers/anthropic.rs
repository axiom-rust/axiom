//! Anthropic Claude provider implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use axiom_ai_core::{Message, StreamResponse, StreamChunk, ChunkType, Result, AxiomError};
use crate::gateway::{LlmRequest, LlmResponse};
use super::{LlmProvider, ProviderConfig, create_http_client, handle_provider_error};

/// Anthropic Claude provider implementation
pub struct AnthropicProvider {
    config: ProviderConfig,
    client: reqwest::Client,
    supported_models: Vec<String>,
}

impl AnthropicProvider {
    /// Create a new Anthropic provider
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let client = create_http_client(&config)?;
        let supported_models = vec![
            "claude-3-opus-20240229".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-haiku-20240307".to_string(),
            "claude-2.1".to_string(),
            "claude-2.0".to_string(),
            "claude-instant-1.2".to_string(),
        ];

        Ok(Self {
            config,
            client,
            supported_models,
        })
    }

    /// Get the base URL for Anthropic API
    fn get_base_url(&self) -> String {
        self.config.base_url.clone().unwrap_or_else(|| {
            "https://api.anthropic.com/v1".to_string()
        })
    }

    /// Convert Axiom messages to Anthropic format
    fn convert_messages(&self, messages: &[Message]) -> (String, Vec<AnthropicMessage>) {
        let mut system_message = String::new();
        let mut conversation_messages = Vec::new();

        for message in messages {
            match message.role {
                axiom_ai_core::MessageRole::System => {
                    if let Some(text) = message.text_content() {
                        system_message.push_str(text);
                        system_message.push('\n');
                    }
                }
                axiom_ai_core::MessageRole::User => {
                    if let Some(text) = message.text_content() {
                        conversation_messages.push(AnthropicMessage {
                            role: "user".to_string(),
                            content: text.to_string(),
                        });
                    }
                }
                axiom_ai_core::MessageRole::Assistant => {
                    if let Some(text) = message.text_content() {
                        conversation_messages.push(AnthropicMessage {
                            role: "assistant".to_string(),
                            content: text.to_string(),
                        });
                    }
                }
                axiom_ai_core::MessageRole::Tool => {
                    // Anthropic doesn't have tool messages in the same way
                    // We'll skip them for now
                }
            }
        }

        (system_message.trim().to_string(), conversation_messages)
    }

    /// Convert Anthropic response to Axiom format
    fn convert_response(&self, response: AnthropicResponse, model: &str) -> LlmResponse {
        let content = response.content.first()
            .map(|content| content.text.clone())
            .unwrap_or_default();

        let tokens_used = response.usage.map(|usage| usage.input_tokens + usage.output_tokens);

        LlmResponse::new(content, model, "anthropic")
            .with_tokens_used(tokens_used.unwrap_or(0))
            .with_generation_time(0) // Would be set by the caller
    }

    /// Convert streaming response to Axiom format
    fn convert_stream_chunk(&self, chunk: AnthropicStreamChunk) -> Option<StreamChunk> {
        if let Some(delta) = chunk.delta {
            if let Some(text) = delta.text {
                return Some(StreamChunk {
                    content: text,
                    chunk_type: ChunkType::Text,
                    metadata: None,
                    is_final: chunk.stop_reason.is_some(),
                });
            }
        }
        None
    }
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn name(&self) -> &str {
        "anthropic"
    }

    fn description(&self) -> &str {
        "Anthropic Claude models provider"
    }

    fn supported_models(&self) -> Vec<String> {
        self.supported_models.clone()
    }

    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse> {
        self.validate_request(&request)?;

        let start_time = std::time::Instant::now();
        let base_url = self.get_base_url();
        let url = format!("{}/messages", base_url);

        let (system, messages) = self.convert_messages(&request.messages);

        let anthropic_request = AnthropicRequest {
            model: request.model.clone(),
            max_tokens: request.max_tokens.unwrap_or(1000),
            messages,
            system: if system.is_empty() { None } else { Some(system) },
            temperature: request.temperature,
            top_p: request.top_p,
            stop_sequences: if request.stop_sequences.is_empty() {
                None
            } else {
                Some(request.stop_sequences)
            },
            stream: false,
        };

        let response = self.client
            .post(&url)
            .json(&anthropic_request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(handle_provider_error(status, &body));
        }

        let anthropic_response: AnthropicResponse = serde_json::from_str(&body)
            .map_err(|e| AxiomError::Serialization(serde_json::Error::from(e)))?;

        let mut llm_response = self.convert_response(anthropic_response, &request.model);
        llm_response.generation_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(llm_response)
    }

    async fn generate_stream(&self, _request: LlmRequest) -> Result<StreamResponse> {
        // For now, return a simple error indicating streaming is not implemented
        Err(AxiomError::LlmProvider("Streaming not implemented yet".to_string()))
    }

    fn get_cost_per_token(&self, model: &str) -> Option<f64> {
        match model {
            "claude-3-opus-20240229" => Some(0.015 / 1000.0), // $0.015 per 1K tokens
            "claude-3-sonnet-20240229" => Some(0.003 / 1000.0),
            "claude-3-haiku-20240307" => Some(0.00025 / 1000.0),
            "claude-2.1" => Some(0.008 / 1000.0),
            "claude-2.0" => Some(0.008 / 1000.0),
            "claude-instant-1.2" => Some(0.0008 / 1000.0),
            _ => None,
        }
    }

    fn get_max_context_length(&self, model: &str) -> Option<usize> {
        match model {
            "claude-3-opus-20240229" => Some(200000),
            "claude-3-sonnet-20240229" => Some(200000),
            "claude-3-haiku-20240307" => Some(200000),
            "claude-2.1" => Some(100000),
            "claude-2.0" => Some(100000),
            "claude-instant-1.2" => Some(100000),
            _ => None,
        }
    }
}

/// Anthropic API request format
#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    system: Option<String>,
    temperature: Option<f32>,
    top_p: Option<f32>,
    stop_sequences: Option<Vec<String>>,
    stream: bool,
}

/// Anthropic message format
#[derive(Debug, Serialize, Deserialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

/// Anthropic API response format
#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
    usage: Option<AnthropicUsage>,
}

/// Anthropic content format
#[derive(Debug, Deserialize)]
struct AnthropicContent {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

/// Anthropic usage format
#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

/// Anthropic streaming chunk format
#[derive(Debug, Deserialize)]
struct AnthropicStreamChunk {
    delta: Option<AnthropicDelta>,
    stop_reason: Option<String>,
}

/// Anthropic streaming delta format
#[derive(Debug, Deserialize)]
struct AnthropicDelta {
    text: Option<String>,
}
