//! OpenAI provider implementation

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use axiom_ai_core::{Message, StreamResponse, StreamChunk, ChunkType, Result, AxiomError};
use crate::gateway::{LlmRequest, LlmResponse};
use super::{LlmProvider, ProviderConfig, create_http_client, handle_provider_error};

/// OpenAI provider implementation
pub struct OpenAIProvider {
    config: ProviderConfig,
    client: reqwest::Client,
    supported_models: Vec<String>,
}

impl OpenAIProvider {
    /// Create a new OpenAI provider
    pub fn new(config: ProviderConfig) -> Result<Self> {
        let client = create_http_client(&config)?;
        let supported_models = vec![
            "gpt-3.5-turbo".to_string(),
            "gpt-3.5-turbo-16k".to_string(),
            "gpt-4".to_string(),
            "gpt-4o".to_string(),
            "gpt-4o-mini".to_string(),
            "gpt-4.1".to_string(),
            "gpt-4.1-mini".to_string(),
            "gpt-4-32k".to_string(),
            "gpt-4-turbo".to_string(),
            "text-davinci-003".to_string(),
            "text-davinci-002".to_string(),
            "text-curie-001".to_string(),
            "text-babbage-001".to_string(),
            "text-ada-001".to_string(),
        ];

        Ok(Self {
            config,
            client,
            supported_models,
        })
    }

    /// Get the base URL for OpenAI API
    fn get_base_url(&self) -> String {
        self.config.base_url.clone().unwrap_or_else(|| {
            "https://api.openai.com/v1".to_string()
        })
    }

    /// Convert Axiom messages to OpenAI format
    fn convert_messages(&self, messages: &[Message]) -> Vec<OpenAIMessage> {
        messages.iter().map(|msg| {
            let role = match msg.role {
                axiom_ai_core::MessageRole::System => "system",
                axiom_ai_core::MessageRole::User => "user",
                axiom_ai_core::MessageRole::Assistant => "assistant",
                axiom_ai_core::MessageRole::Tool => "tool",
            };

            let content = match &msg.content {
                axiom_ai_core::MessageContent::Text(text) => text.clone(),
                axiom_ai_core::MessageContent::Parts(parts) => {
                    // Convert parts to a single text content for now
                    // In a full implementation, you'd handle different part types
                    parts.iter()
                        .filter_map(|part| match part {
                            axiom_ai_core::MessagePart::Text(text) => Some(text.as_str()),
                            _ => None,
                        })
                        .collect::<Vec<_>>()
                        .join(" ")
                }
            };

            OpenAIMessage {
                role: role.to_string(),
                content,
            }
        }).collect()
    }

    /// Convert OpenAI response to Axiom format
    fn convert_response(&self, response: OpenAIResponse, model: &str) -> LlmResponse {
        let content = response.choices.first()
            .map(|choice| choice.message.content.clone())
            .unwrap_or_default();

        let tokens_used = response.usage.map(|usage| usage.total_tokens);

        LlmResponse::new(content, model, "openai")
            .with_tokens_used(tokens_used.unwrap_or(0))
            .with_generation_time(0) // Would be set by the caller
    }

    /// Convert streaming response to Axiom format
    fn convert_stream_chunk(&self, chunk: OpenAIStreamChunk) -> Option<StreamChunk> {
        if let Some(choice) = chunk.choices.first() {
            if let Some(delta) = &choice.delta {
                if !delta.content.is_empty() {
                    let content = &delta.content;
                    return Some(StreamChunk {
                        content: content.clone(),
                        chunk_type: ChunkType::Text,
                        metadata: None,
                        is_final: choice.finish_reason.is_some(),
                    });
                }
            }
        }
        None
    }
}

#[async_trait]
impl LlmProvider for OpenAIProvider {
    fn name(&self) -> &str {
        "openai"
    }

    fn description(&self) -> &str {
        "OpenAI GPT models provider"
    }

    fn supported_models(&self) -> Vec<String> {
        self.supported_models.clone()
    }

    async fn generate(&self, request: LlmRequest) -> Result<LlmResponse> {
        self.validate_request(&request)?;

        let start_time = std::time::Instant::now();
        let base_url = self.get_base_url();
        let url = format!("{}/chat/completions", base_url);

        let openai_request = OpenAIRequest {
            model: request.model.clone(),
            messages: self.convert_messages(&request.messages),
            temperature: request.temperature,
            max_tokens: request.max_tokens,
            top_p: request.top_p,
            stop: if request.stop_sequences.is_empty() {
                None
            } else {
                Some(request.stop_sequences)
            },
            stream: false,
        };

        let response = self.client
            .post(&url)
            .json(&openai_request)
            .send()
            .await?;

        let status = response.status();
        let body = response.text().await?;

        if !status.is_success() {
            return Err(handle_provider_error(status, &body));
        }

        let openai_response: OpenAIResponse = serde_json::from_str(&body)
            .map_err(|e| AxiomError::Serialization(serde_json::Error::from(e)))?;

        let mut llm_response = self.convert_response(openai_response, &request.model);
        llm_response.generation_time_ms = start_time.elapsed().as_millis() as u64;

        Ok(llm_response)
    }

    async fn generate_stream(&self, _request: LlmRequest) -> Result<StreamResponse> {
        // For now, return a simple error indicating streaming is not implemented
        Err(AxiomError::LlmProvider("Streaming not implemented yet".to_string()))
    }

    fn get_cost_per_token(&self, model: &str) -> Option<f64> {
        match model {
            "gpt-3.5-turbo" => Some(0.0005 / 1000.0), // $0.0005 per 1K tokens
            "gpt-3.5-turbo-16k" => Some(0.003 / 1000.0),
            "gpt-4" => Some(0.03 / 1000.0),
            "gpt-4-32k" => Some(0.06 / 1000.0),
            "gpt-4-turbo" => Some(0.01 / 1000.0),
            _ => None,
        }
    }

    fn get_max_context_length(&self, model: &str) -> Option<usize> {
        match model {
            "gpt-3.5-turbo" => Some(4096),
            "gpt-3.5-turbo-16k" => Some(16384),
            "gpt-4" => Some(8192),
            "gpt-4-32k" => Some(32768),
            "gpt-4-turbo" => Some(128000),
            _ => None,
        }
    }
}

/// OpenAI API request format
#[derive(Debug, Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
    top_p: Option<f32>,
    stop: Option<Vec<String>>,
    stream: bool,
}

/// OpenAI message format
#[derive(Debug, Serialize, Deserialize)]
struct OpenAIMessage {
    role: String,
    content: String,
}

/// OpenAI API response format
#[derive(Debug, Deserialize)]
struct OpenAIResponse {
    choices: Vec<OpenAIChoice>,
    usage: Option<OpenAIUsage>,
}

/// OpenAI choice format
#[derive(Debug, Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
    finish_reason: Option<String>,
}

/// OpenAI usage format
#[derive(Debug, Deserialize)]
struct OpenAIUsage {
    total_tokens: u32,
    prompt_tokens: u32,
    completion_tokens: u32,
}

/// OpenAI streaming chunk format
#[derive(Debug, Deserialize)]
struct OpenAIStreamChunk {
    choices: Vec<OpenAIStreamChoice>,
}

/// OpenAI streaming choice format
#[derive(Debug, Deserialize)]
struct OpenAIStreamChoice {
    delta: Option<OpenAIMessage>,
    finish_reason: Option<String>,
}
