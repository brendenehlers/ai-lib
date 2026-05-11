#[derive(Debug)]
pub struct GenerateTextRequest {
    pub prompt: Vec<RequestMessage>,
    pub model_name: String,
    pub max_tokens: Option<u32>,
    pub system_prompt: Option<String>,
}

#[derive(Debug)]
pub struct RequestMessage {
    pub text: String,
    pub role: Option<Role>,
}

#[derive(Debug)]
pub enum Role {
    User,
    Assistant,
}

#[derive(Debug)]
pub struct GenerateTextResponse {
    pub content: ChatResponse,
    pub usage: UsageMetadata,
}

#[derive(Debug)]
pub struct ChatResponse {
    pub messages: Vec<ResponseMessage>,
}

#[derive(Debug)]
pub struct ResponseMessage {
    pub text: String,
}

#[derive(Debug)]
pub struct UsageMetadata {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub reasoning_tokens: Option<u32>,
    pub cached_tokens: u32,
    pub total_tokens: u32,
}
