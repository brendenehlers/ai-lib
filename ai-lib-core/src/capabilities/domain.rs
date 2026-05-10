#[derive(Debug)]
pub struct ChatRequest {
    pub prompt: Vec<RequestMessage>,
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
    System,
}

#[derive(Debug)]
pub struct ChatResponse {
    pub content: Vec<ResponseMessage>,
}

#[derive(Debug)]
pub struct ResponseMessage {
    pub text: String,
}
