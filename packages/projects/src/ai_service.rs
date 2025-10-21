// ABOUTME: AI service for making structured generation calls to Anthropic Claude
// ABOUTME: Handles API requests, response parsing, and usage tracking

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use thiserror::Error;
use tracing::{error, info};

const ANTHROPIC_API_URL: &str = "https://api.anthropic.com/v1/messages";
const DEFAULT_MODEL: &str = "claude-3-5-sonnet-20241022";
const DEFAULT_MAX_TOKENS: u32 = 8000;
const DEFAULT_TEMPERATURE: f32 = 0.7;

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
}

impl AIService {
    /// Creates a new AI service instance
    /// API key is fetched from ANTHROPIC_API_KEY environment variable
    pub fn new() -> Self {
        let api_key = env::var("ANTHROPIC_API_KEY").ok();
        if api_key.is_none() {
            info!("ANTHROPIC_API_KEY not set - AI service will use database-stored keys");
        }

        Self {
            client: Client::new(),
            api_key,
        }
    }

    /// Creates a new AI service instance with a specific API key
    pub fn with_api_key(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key: Some(api_key),
        }
    }

    /// Makes a structured generation call to Claude
    /// The prompt should request JSON output and the response will be parsed
    pub async fn generate_structured<T: for<'de> Deserialize<'de>>(
        &self,
        prompt: String,
        system_prompt: Option<String>,
    ) -> AIServiceResult<AIResponse<T>> {
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
        };

        info!(
            "Making Anthropic API request: model={}, max_tokens={}",
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

        // Parse the JSON response
        let data: T = serde_json::from_str(&text)
            .map_err(|e| AIServiceError::ParseError(format!("Failed to parse JSON: {}", e)))?;

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
}

impl Default for AIService {
    fn default() -> Self {
        Self::new()
    }
}
