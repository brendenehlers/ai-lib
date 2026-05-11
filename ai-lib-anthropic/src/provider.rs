use ai_lib_core::{
    self as core,
    capabilities::{domain, provider},
};
use core::errors;

const DEFAULT_BASE_URL: &str = "https://api.anthropic.com";
const MESSAGES_PATH: &str = "/v1/messages";

pub struct AnthropicProvider {
    reqwest: reqwest::Client,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: &str) -> errors::AiLibResult<Self> {
        Self::with_base_url(api_key, DEFAULT_BASE_URL)
    }

    pub fn with_base_url(api_key: &str, base_url: &str) -> errors::AiLibResult<Self> {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            reqwest::header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("anthropic-version"),
            reqwest::header::HeaderValue::from_static("2023-06-01"),
        );
        headers.insert(
            reqwest::header::HeaderName::from_static("x-api-key"),
            reqwest::header::HeaderValue::from_str(api_key)?,
        );
        let reqwest = reqwest::Client::builder().default_headers(headers);
        Ok(AnthropicProvider {
            reqwest: reqwest.build()?,
            base_url: base_url.into(),
        })
    }
}

impl provider::ChatProvider for AnthropicProvider {
    async fn generate_text(
        &self,
        request: domain::GenerateTextRequest,
    ) -> core::errors::AiLibResult<domain::GenerateTextResponse> {
        let anthropic_request: AnthropicGenerateTextRequest = request.into();
        let url = format!("{}{}", self.base_url, MESSAGES_PATH);
        let raw_response = self
            .reqwest
            .post(url)
            .json::<AnthropicGenerateTextRequest>(&anthropic_request)
            .send()
            .await?;

        if !raw_response.status().is_success() {
            let status = raw_response.status();
            let body = raw_response.text().await?;
            return Err(core::errors::AiLibError::HttpStatus { status, body });
        }

        let response = raw_response
            .json::<AnthropicGenerateTextResponse>()
            .await?
            .into();

        Ok(response)
    }
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AnthropicGenerateTextRequest {
    max_tokens: u32,
    messages: Vec<MessageParam>,
    model: String,
    system: Option<SystemPrompt>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct MessageParam {
    content: String,
    role: Role,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "assistant")]
    Assistant,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
enum SystemPrompt {
    Text(String),
    Blocks(Vec<SystemBlock>),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SystemBlock {
    #[serde(rename = "type")]
    kind: String,
    text: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct AnthropicGenerateTextResponse {
    content: Vec<ContentBlock>,
    usage: Usage,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
enum ContentBlock {
    #[serde(rename = "text")]
    TextBlock { text: String },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Usage {
    cache_creation_input_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
}

impl From<domain::GenerateTextRequest> for AnthropicGenerateTextRequest {
    fn from(value: domain::GenerateTextRequest) -> Self {
        AnthropicGenerateTextRequest {
            max_tokens: value.max_tokens.unwrap_or(64000), // minimun supported value between opus, sonnet, and haiku
            messages: value
                .prompt
                .into_iter()
                .map(domain::RequestMessage::into)
                .collect(),
            model: value.model_name,
            system: value.system_prompt.map(|s| SystemPrompt::Text(s)),
        }
    }
}

impl From<domain::RequestMessage> for MessageParam {
    fn from(value: domain::RequestMessage) -> Self {
        MessageParam {
            content: value.text,
            role: value.role.map(domain::Role::into).unwrap_or(Role::User),
        }
    }
}

impl From<domain::Role> for Role {
    fn from(value: domain::Role) -> Self {
        match value {
            domain::Role::User => Role::User,
            domain::Role::Assistant => Role::Assistant,
        }
    }
}

impl From<AnthropicGenerateTextResponse> for domain::GenerateTextResponse {
    fn from(value: AnthropicGenerateTextResponse) -> Self {
        domain::GenerateTextResponse {
            content: domain::ChatResponse {
                messages: value.content.into_iter().map(ContentBlock::into).collect(),
            },
            usage: value.usage.into(),
        }
    }
}

impl From<ContentBlock> for domain::ResponseMessage {
    fn from(value: ContentBlock) -> Self {
        match value {
            ContentBlock::TextBlock { text } => domain::ResponseMessage { text },
        }
    }
}

impl From<Usage> for domain::UsageMetadata {
    fn from(value: Usage) -> Self {
        domain::UsageMetadata {
            input_tokens: value.input_tokens.unwrap_or(0),
            cached_tokens: value.cache_creation_input_tokens.unwrap_or(0)
                + value.cache_read_input_tokens.unwrap_or(0),
            output_tokens: value.output_tokens.unwrap_or(0),
            // anthropic doesn't supply total tokens but docs describe calculation
            // https://platform.claude.com/docs/en/api/messages/create
            total_tokens: value.input_tokens.unwrap_or(0)
                + value.cache_creation_input_tokens.unwrap_or(0)
                + value.cache_read_input_tokens.unwrap_or(0)
                + value.output_tokens.unwrap_or(0),
            reasoning_tokens: None, // anthropic doesn't include reasoning tokens
        }
    }
}
