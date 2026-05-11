use ai_lib_core::{
    capabilities::{domain, provider},
    errors,
};
use reqwest::header;

pub struct GeminiProvider {
    reqwest: reqwest::Client,
}

pub enum GeminiAuth {
    ApiKey(String),
}

impl GeminiProvider {
    pub fn new(auth: GeminiAuth) -> errors::AiLibResult<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        match auth {
            GeminiAuth::ApiKey(api_key) => {
                let api_key = api_key.clone();
                headers.insert(
                    header::HeaderName::from_static("x-goog-api-key"),
                    header::HeaderValue::from_str(api_key.as_str())?,
                );
            }
        }
        let reqwest = reqwest::Client::builder().default_headers(headers);
        Ok(GeminiProvider {
            reqwest: reqwest.build()?,
        })
    }

    fn gemini_url(model: &str) -> String {
        format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent",
            &model
        )
    }
}

impl provider::ChatProvider for GeminiProvider {
    async fn generate_text(
        &self,
        request: domain::GenerateTextRequest,
    ) -> errors::AiLibResult<domain::GenerateTextResponse> {
        let url = GeminiProvider::gemini_url(&request.model_name);
        let gemini_request = request.into();
        let raw_response = self
            .reqwest
            .post(url)
            .json::<GeminiGenerateTextRequest>(&gemini_request)
            .send()
            .await?;

        if !raw_response.status().is_success() {
            let status = raw_response.status();
            let body = raw_response.text().await?;
            return Err(errors::AiLibError::HttpStatus { status, body });
        }

        let chat_response = raw_response.json::<GeminiApiResponse>().await?.into();

        Ok(chat_response)
    }
}

impl From<domain::GenerateTextRequest> for GeminiGenerateTextRequest {
    fn from(value: domain::GenerateTextRequest) -> Self {
        GeminiGenerateTextRequest {
            contents: value.prompt.into_iter().map(Content::from).collect(),
            generation_config: GenerationConfig {
                max_output_tokens: value.max_tokens,
            },
        }
    }
}

impl From<domain::RequestMessage> for Content {
    fn from(value: domain::RequestMessage) -> Self {
        let text = value.text;
        Content {
            parts: vec![Part { text: text }],
            role: match value.role {
                Some(role) => Some(role.into()),
                None => None,
            },
        }
    }
}

impl From<GeminiApiResponse> for domain::GenerateTextResponse {
    fn from(value: GeminiApiResponse) -> Self {
        let first_candidate = value.candidates.into_iter().next().unwrap();
        domain::GenerateTextResponse {
            content: domain::ChatResponse {
                messages: vec![first_candidate.content.into()],
            },
            usage: value.usage_metadata.into(),
        }
    }
}

impl From<Content> for domain::ResponseMessage {
    fn from(value: Content) -> Self {
        domain::ResponseMessage {
            text: value
                .parts
                .into_iter()
                .map(|p| p.text)
                .collect::<Vec<_>>()
                .join(" "),
        }
    }
}

impl From<domain::Role> for Role {
    fn from(value: domain::Role) -> Self {
        match value {
            domain::Role::User => Role::User,
            domain::Role::Assistant => Role::Model,
        }
    }
}

impl From<UsageMetadata> for domain::UsageMetadata {
    fn from(value: UsageMetadata) -> Self {
        domain::UsageMetadata {
            input_tokens: value.prompt_token_count.unwrap_or(0)
                + value.tool_use_prompt_token_count.unwrap_or(0),
            output_tokens: value.candidates_token_count.unwrap_or(0),
            reasoning_tokens: value.thoughts_token_count,
            cached_tokens: value.cached_content_token_count.unwrap_or(0),
            total_tokens: value.total_token_count.unwrap_or(0),
        }
    }
}

/// https://ai.google.dev/api/generate-content#method:-models.generatecontent
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GeminiGenerateTextRequest {
    contents: Vec<Content>,
    #[serde(rename = "generationConfig")]
    generation_config: GenerationConfig,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Content {
    parts: Vec<Part>,
    role: Option<Role>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
enum Role {
    #[serde(rename = "user")]
    User,
    #[serde(rename = "model")]
    Model,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GenerationConfig {
    #[serde(rename = "maxOutputTokens")]
    max_output_tokens: Option<u32>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Part {
    text: String,
}

/// https://ai.google.dev/api/generate-content#generatecontentresponse
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GeminiApiResponse {
    candidates: Vec<Candidate>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: UsageMetadata,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Candidate {
    content: Content,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct UsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u32>,
    #[serde(rename = "cachedContentTokenCount")]
    cached_content_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
    candidates_token_count: Option<u32>,
    #[serde(rename = "toolUsePromptTokenCount")]
    tool_use_prompt_token_count: Option<u32>,
    #[serde(rename = "thoughtsTokenCount")]
    thoughts_token_count: Option<u32>,
    #[serde(rename = "totalTokenCount")]
    total_token_count: Option<u32>,
}
