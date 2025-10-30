// ABOUTME: AI service for making structured generation calls to Anthropic Claude
// ABOUTME: Handles API requests, response parsing, and usage tracking

use std::env;

use futures::stream::Stream;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{error, info};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_MODEL: &str = "claude-sonnet-4-20250514"; // Claude Sonnet 4 (May 2025)
const DEFAULT_MAX_TOKENS: u32 = 4096;
const DEFAULT_TEMPERATURE: f32 = 0.7;

/// Calculate appropriate max_tokens for a given model
fn get_max_tokens_for_model(model: &str) -> u32 {
    // Claude 3 family (Opus, Sonnet, Haiku)
    if model.contains("claude-3-opus") || model.contains("claude-3-sonnet") {
        4096
    } else if model.contains("claude-3-haiku") {
        1024
    }
    // Claude Sonnet/Haiku 4/5
    else if model.contains("claude-sonnet") {
        4096
    } else if model.contains("claude-haiku") {
        1024
    }
    // Default to safe value for unknown models
    else {
        4096
    }
}

#[derive(Debug, Error)]
pub enum AIServiceError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("API error: {0}")]
    ApiError(String),

    #[error("Failed to parse response: {0}")]
    ParseError(String),

    #[error("No API key configured")]
    NoApiKey,

    #[error("Invalid response format")]
    InvalidResponse,
}

pub type AIServiceResult<T> = Result<T, AIServiceError>;

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    temperature: f32,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    #[allow(dead_code)]
    id: String,
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Usage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl Usage {
    pub fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

#[derive(Debug)]
pub struct AIResponse<T> {
    pub data: T,
    pub usage: Usage,
}

/// AI service for making structured generation calls
pub struct AIService {
    client: Client,
    api_key: Option<String>,
    model: String,
}

impl AIService {
    /// Create HTTP client with timeout configuration
    fn create_client() -> Client {
        Client::builder()
            .timeout(std::time::Duration::from_secs(600))
            .connect_timeout(std::time::Duration::from_secs(10))
            .build()
            .expect("Failed to build HTTP client")
    }

    /// Creates a new AI service instance
    /// API key is fetched from ANTHROPIC_API_KEY environment variable
    /// Model can be overridden with ANTHROPIC_MODEL environment variable
    pub fn new() -> Self {
        let api_key = env::var("ANTHROPIC_API_KEY").ok();
        if api_key.is_none() {
            info!("ANTHROPIC_API_KEY not set - AI service will use database-stored keys");
        }

        let model = env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        if model != DEFAULT_MODEL {
            info!("Using custom Anthropic model: {}", model);
        }

        Self {
            client: Self::create_client(),
            api_key,
            model,
        }
    }

    /// Creates a new AI service instance with a specific API key
    pub fn with_api_key(api_key: String) -> Self {
        let model = env::var("ANTHROPIC_MODEL").unwrap_or_else(|_| DEFAULT_MODEL.to_string());

        Self {
            client: Self::create_client(),
            api_key: Some(api_key),
            model,
        }
    }

    /// Creates a new AI service instance with a specific API key and model
    pub fn with_api_key_and_model(api_key: String, model: String) -> Self {
        Self {
            client: Self::create_client(),
            api_key: Some(api_key),
            model,
        }
    }

    /// Get the model being used by this service
    pub fn model(&self) -> &str {
        &self.model
    }

    /// Makes a structured generation call to Claude
    /// The prompt should request JSON output and the response will be parsed
    pub async fn generate_structured<T: for<'de> Deserialize<'de>>(
        &self,
        prompt: String,
        system_prompt: Option<String>,
    ) -> AIServiceResult<AIResponse<T>> {
        let api_key = self.api_key.as_ref().ok_or(AIServiceError::NoApiKey)?;

        let max_tokens = get_max_tokens_for_model(&self.model);
        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens,
            temperature: DEFAULT_TEMPERATURE,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            system: system_prompt,
            stream: None,
        };

        info!(
            "Making Anthropic API request: model={}, max_tokens={}, timeout=600s",
            request.model, request.max_tokens
        );

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    error!("Anthropic API request timed out after 600 seconds");
                    AIServiceError::ApiError("Request timed out after 600 seconds. The AI service may be overloaded or unavailable.".to_string())
                } else if e.is_connect() {
                    error!("Failed to connect to Anthropic API: {}", e);
                    AIServiceError::ApiError(format!("Connection failed: {}. Please check your internet connection.", e))
                } else {
                    error!("Anthropic API request failed: {}", e);
                    AIServiceError::RequestFailed(e)
                }
            })?;

        info!(
            "Received response from Anthropic API: status={}",
            response.status()
        );

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Anthropic API error: {} - {}", status, error_text);
            return Err(AIServiceError::ApiError(format!(
                "API returned {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| AIServiceError::ParseError(e.to_string()))?;

        // Extract text from the first content block
        let text = anthropic_response
            .content
            .first()
            .ok_or(AIServiceError::InvalidResponse)?
            .text
            .clone();

        // Strip markdown code fences if present (```json ... ``` or ````json ... ````)
        let cleaned_text = text.trim();
        let json_text = if cleaned_text.starts_with("```") {
            // Find the first newline after opening fence
            let start = cleaned_text.find('\n').map(|i| i + 1).unwrap_or(0);
            // Find the closing fence (search from start position to avoid finding opening fence)
            let end = cleaned_text[start..]
                .rfind("```")
                .map(|i| i + start)
                .unwrap_or(cleaned_text.len());
            cleaned_text[start..end].trim()
        } else {
            cleaned_text
        };

        // Parse the JSON response
        info!(
            "Raw JSON response (first 5000 chars): {}",
            &json_text[..json_text.len().min(5000)]
        );
        let data: T = serde_json::from_str(json_text).map_err(|e| {
            error!(
                "JSON parsing failed: {}. JSON snippet: {}",
                e,
                &json_text[..json_text.len().min(500)]
            );
            AIServiceError::ParseError(format!("Failed to parse JSON: {}", e))
        })?;

        Ok(AIResponse {
            data,
            usage: anthropic_response.usage,
        })
    }

    /// Makes a text generation call to Claude
    pub async fn generate_text(
        &self,
        prompt: String,
        system_prompt: Option<String>,
    ) -> AIServiceResult<AIResponse<String>> {
        let api_key = self.api_key.as_ref().ok_or(AIServiceError::NoApiKey)?;

        let request = AnthropicRequest {
            model: DEFAULT_MODEL.to_string(),
            max_tokens: DEFAULT_MAX_TOKENS,
            temperature: DEFAULT_TEMPERATURE,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            system: system_prompt,
            stream: None,
        };

        info!(
            "Making Anthropic API text generation request: model={}",
            request.model
        );

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Anthropic API error: {} - {}", status, error_text);
            return Err(AIServiceError::ApiError(format!(
                "API returned {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| AIServiceError::ParseError(e.to_string()))?;

        // Extract text from the first content block
        let text = anthropic_response
            .content
            .first()
            .ok_or(AIServiceError::InvalidResponse)?
            .text
            .clone();

        Ok(AIResponse {
            data: text,
            usage: anthropic_response.usage,
        })
    }

    /// Makes a streaming text generation call to Claude
    /// Returns a stream of text chunks as they arrive
    pub async fn generate_text_stream(
        &self,
        prompt: String,
        system_prompt: Option<String>,
    ) -> AIServiceResult<impl Stream<Item = Result<String, AIServiceError>>> {
        let api_key = self.api_key.as_ref().ok_or(AIServiceError::NoApiKey)?;

        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: get_max_tokens_for_model(&self.model),
            temperature: DEFAULT_TEMPERATURE,
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt,
            }],
            system: system_prompt,
            stream: Some(true),
        };

        info!(
            "Making Anthropic API streaming text generation request: model={}",
            request.model
        );

        let response = self
            .client
            .post(ANTHROPIC_API_URL)
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            error!("Anthropic API error: {} - {}", status, error_text);
            return Err(AIServiceError::ApiError(format!(
                "API returned {}: {}",
                status, error_text
            )));
        }

        // Create a stream from the response bytes
        let stream = async_stream::stream! {
            use futures::StreamExt;
            let mut byte_stream = response.bytes_stream();
            let mut buffer = String::new();

            while let Some(chunk_result) = byte_stream.next().await {
                match chunk_result {
                    Ok(bytes) => {
                        let chunk_str = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&chunk_str);

                        // Process complete SSE events
                        while let Some(event_end) = buffer.find("\n\n") {
                            let event = buffer[..event_end].to_string();
                            buffer = buffer[event_end + 2..].to_string();

                            // Parse SSE event
                            for line in event.lines() {
                                if let Some(data) = line.strip_prefix("data: ") {
                                    // Parse the JSON event
                                    if let Ok(event_json) = serde_json::from_str::<serde_json::Value>(data) {
                                        // Extract text delta from content_block_delta events
                                        if event_json["type"] == "content_block_delta" {
                                            if let Some(text) = event_json["delta"]["text"].as_str() {
                                                yield Ok(text.to_string());
                                            }
                                        }
                                        // Handle errors
                                        else if event_json["type"] == "error" {
                                            let error_msg = event_json["error"]["message"]
                                                .as_str()
                                                .unwrap_or("Unknown streaming error");
                                            yield Err(AIServiceError::ApiError(error_msg.to_string()));
                                            return;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(AIServiceError::RequestFailed(e));
                        return;
                    }
                }
            }
        };

        Ok(stream)
    }
}

impl Default for AIService {
    fn default() -> Self {
        Self::new()
    }
}
