use crate::{
    capabilities::provider::{self, ChatResponse},
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
    pub fn new(auth: GeminiAuth) -> errors::AiLibResult<GeminiProvider> {
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
        request: provider::ChatRequest,
    ) -> errors::AiLibResult<provider::ChatResponse> {
        let url = GeminiProvider::gemini_url(&request.model);
        let gemini_request = request.into();
        let raw_response = self
            .reqwest
            .post(url)
            .json::<GeminiApiRequest>(&gemini_request)
            .send()
            .await
            .inspect_err(|e| println!("request failed: {}", e))?;

        let chat_response = raw_response.json::<GeminiApiResponse>().await?.into();

        Ok(chat_response)
    }
}

impl From<provider::ChatRequest> for GeminiApiRequest {
    fn from(value: provider::ChatRequest) -> Self {
        GeminiApiRequest {
            contents: value.prompt.into_iter().map(Content::from).collect(),
        }
    }
}

impl From<provider::RequestMessage> for Content {
    fn from(value: provider::RequestMessage) -> Self {
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

impl From<GeminiApiResponse> for provider::ChatResponse {
    fn from(value: GeminiApiResponse) -> Self {
        let first_candidate = value.candidates.into_iter().next().unwrap();
        ChatResponse {
            content: vec![first_candidate.content.into()],
        }
    }
}

impl From<Content> for provider::ResponseMessage {
    fn from(value: Content) -> Self {
        provider::ResponseMessage {
            text: value
                .parts
                .into_iter()
                .map(|p| p.text)
                .collect::<Vec<_>>()
                .join(" "),
        }
    }
}

impl From<provider::Role> for Role {
    fn from(value: provider::Role) -> Self {
        match value {
            provider::Role::User | provider::Role::Assistant => Role::User,
            provider::Role::System => Role::Model,
        }
    }
}

/// https://ai.google.dev/api/generate-content#method:-models.generatecontent
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GeminiApiRequest {
    contents: Vec<Content>,
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
struct Part {
    text: String,
}

/// https://ai.google.dev/api/generate-content#generatecontentresponse
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct GeminiApiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Candidate {
    content: Content,
}
